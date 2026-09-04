//! Conversion of C# declaration syntax into language-neutral semantic facts.

use scent_domain::{
    FieldId, FileId, MethodId, NormalizedPath, PropertyId, Resolution, SourceLocation,
    SourcePosition, SourceRange, TypeId, UnresolvedReference, UnresolvedReferenceKind, Visibility,
};
use scent_ir::{
    FieldAccess, FieldIR, FileIR, Instantiation, MethodCall, MethodIR, NamespaceIR, ParameterIR,
    PropertyIR, TypeIR, TypeKind, TypeReference,
};
use tree_sitter::{Node, Tree};

use super::SourceFile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractedFile {
    pub file: FileIR,
    pub namespaces: Vec<NamespaceIR>,
    pub types: Vec<TypeIR>,
    pub methods: Vec<MethodIR>,
    pub fields: Vec<FieldIR>,
    pub properties: Vec<PropertyIR>,
}

#[must_use]
pub fn extract_file(source: &SourceFile, tree: &Tree) -> ExtractedFile {
    let file_id = FileId::from_identity(&format!("scent:v1:csharp:file:{}", source.path));
    let mut extracted = ExtractedFile {
        file: FileIR {
            id: file_id.clone(),
            path: source.path.clone(),
            namespace_ids: vec![],
            type_ids: vec![],
        },
        namespaces: vec![],
        types: vec![],
        methods: vec![],
        fields: vec![],
        properties: vec![],
    };
    let default_namespace = file_scoped_namespace(tree.root_node(), source);
    visit_node(
        tree.root_node(),
        source,
        &file_id,
        default_namespace.as_deref(),
        None,
        None,
        &mut extracted,
    );
    extracted
        .namespaces
        .sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
    extracted
        .types
        .sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
    extracted.methods.sort_by(|left, right| {
        left.owner_type
            .cmp(&right.owner_type)
            .then(left.name.cmp(&right.name))
            .then(left.id.cmp(&right.id))
    });
    extracted.fields.sort_by(|left, right| {
        left.owner_type
            .cmp(&right.owner_type)
            .then(left.name.cmp(&right.name))
            .then(left.id.cmp(&right.id))
    });
    extracted.properties.sort_by(|left, right| {
        left.owner_type
            .cmp(&right.owner_type)
            .then(left.name.cmp(&right.name))
            .then(left.id.cmp(&right.id))
    });
    extracted.file.namespace_ids = extracted
        .namespaces
        .iter()
        .map(|item| item.id.clone())
        .collect();
    extracted.file.type_ids = extracted.types.iter().map(|item| item.id.clone()).collect();
    extracted
}

fn file_scoped_namespace(root: Node<'_>, source: &SourceFile) -> Option<String> {
    let mut cursor = root.walk();
    let namespace = root.children(&mut cursor).find_map(|child| {
        (child.kind() == "file_scoped_namespace_declaration")
            .then(|| child.child_by_field_name("name"))
            .flatten()
            .and_then(|name| text(name, source))
    });
    namespace
}

/// `owner` identifies the innermost enclosing type: its qualified name (used
/// to build stable member identities) and its [`TypeId`] (used for member
/// `owner_type` linkage). Both travel together because they are always set
/// or cleared as a pair when entering or leaving a type declaration.
struct Owner<'a> {
    qualified_name: &'a str,
    type_id: &'a TypeId,
}

fn visit_node(
    node: Node<'_>,
    source: &SourceFile,
    file_id: &FileId,
    namespace: Option<&str>,
    parent_type: Option<&str>,
    owner: Option<&Owner<'_>>,
    extracted: &mut ExtractedFile,
) {
    if node.kind() == "namespace_declaration" {
        let namespace_name = node
            .child_by_field_name("name")
            .and_then(|name| text(name, source));
        if let Some(namespace_name) = namespace_name {
            add_namespace(&namespace_name, node, source, file_id, extracted);
            visit_named_children(
                node,
                source,
                file_id,
                Some(&namespace_name),
                parent_type,
                owner,
                extracted,
            );
        }
        return;
    }
    if node.kind() == "file_scoped_namespace_declaration" {
        return;
    }
    if let Some(kind) = type_kind(node.kind()) {
        let Some(name) = node
            .child_by_field_name("name")
            .and_then(|item| text(item, source))
        else {
            return;
        };
        let qualified_name =
            parent_type.map_or_else(|| name.clone(), |parent| format!("{parent}.{name}"));
        let type_id = add_type(
            &qualified_name,
            kind,
            node,
            source,
            file_id,
            namespace,
            extracted,
        );
        let nested_owner = Owner {
            qualified_name: &qualified_name,
            type_id: &type_id,
        };
        visit_named_children(
            node,
            source,
            file_id,
            namespace,
            Some(&qualified_name),
            Some(&nested_owner),
            extracted,
        );
        return;
    }
    if let Some(owner) = owner {
        match node.kind() {
            "field_declaration" => {
                add_field(node, source, namespace, owner, extracted);
                return;
            }
            "property_declaration" => {
                add_property(node, source, namespace, owner, extracted);
                return;
            }
            "method_declaration" => {
                add_method(node, source, namespace, owner, extracted);
                return;
            }
            "constructor_declaration" => {
                add_constructor(node, source, namespace, owner, extracted);
                return;
            }
            _ => {}
        }
    }
    visit_named_children(
        node,
        source,
        file_id,
        namespace,
        parent_type,
        owner,
        extracted,
    );
}

fn visit_named_children(
    node: Node<'_>,
    source: &SourceFile,
    file_id: &FileId,
    namespace: Option<&str>,
    parent_type: Option<&str>,
    owner: Option<&Owner<'_>>,
    extracted: &mut ExtractedFile,
) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_node(
            child,
            source,
            file_id,
            namespace,
            parent_type,
            owner,
            extracted,
        );
    }
}

fn add_namespace(
    name: &str,
    node: Node<'_>,
    source: &SourceFile,
    file_id: &FileId,
    extracted: &mut ExtractedFile,
) {
    let id = format!("{}:namespace:{name}", file_id.as_str());
    if extracted.namespaces.iter().any(|item| item.id == id) {
        return;
    }
    extracted.namespaces.push(NamespaceIR {
        id,
        name: name.into(),
        file_id: file_id.clone(),
        location: location(node, &source.path),
    });
}

fn add_type(
    name: &str,
    kind: TypeKind,
    node: Node<'_>,
    source: &SourceFile,
    file_id: &FileId,
    namespace: Option<&str>,
    extracted: &mut ExtractedFile,
) -> TypeId {
    let namespace_id = namespace.map(|item| format!("{}:namespace:{item}", file_id.as_str()));
    let identity = format!(
        "scent:v1:csharp:{}:{}:{kind:?}:{name}",
        source.path,
        namespace.unwrap_or_default()
    );
    let type_id = TypeId::from_identity(&identity);
    extracted.types.push(TypeIR {
        id: type_id.clone(),
        file_id: file_id.clone(),
        namespace_id,
        name: name.into(),
        kind,
        location: location(node, &source.path),
        visibility: visibility(node, source),
        // A `base_list` entry cannot be classified as base-class-vs-interface
        // from C# syntax alone (only convention says "base class comes
        // first", which is a guess, not a fact). Every entry is recorded as
        // an unresolved interface reference here; Milestone 4's resolver,
        // once it knows a resolved entry's `TypeKind`, moves a resolved
        // `Class`/`Record` occupying the first position into `base_types`.
        base_types: vec![],
        interface_types: parse_base_list(node, source),
    });
    type_id
}

fn parse_base_list(type_node: Node<'_>, source: &SourceFile) -> Vec<Resolution<TypeId>> {
    let Some(base_list) = child_of_kind(type_node, "base_list") else {
        return vec![];
    };
    let mut cursor = base_list.walk();
    base_list
        .named_children(&mut cursor)
        .filter(|child| {
            !matches!(
                child.kind(),
                "argument_list" | "primary_constructor_base_type"
            )
        })
        .map(|child| unresolved_type(child, source))
        .collect()
}

fn add_field(
    field_decl: Node<'_>,
    source: &SourceFile,
    namespace: Option<&str>,
    owner: &Owner<'_>,
    extracted: &mut ExtractedFile,
) {
    let Some(variable_declaration) = child_of_kind(field_decl, "variable_declaration") else {
        return;
    };
    let Some(type_node) = variable_declaration.child_by_field_name("type") else {
        return;
    };
    let type_reference = unresolved_type(type_node, source);
    let field_visibility = visibility(field_decl, source);
    let mut cursor = variable_declaration.walk();
    for declarator in variable_declaration
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "variable_declarator")
    {
        let Some(name) = declarator
            .child_by_field_name("name")
            .and_then(|item| text(item, source))
        else {
            continue;
        };
        let identity = format!(
            "scent:v1:csharp:{}:{}:field:{}.{name}",
            source.path,
            namespace.unwrap_or_default(),
            owner.qualified_name,
        );
        extracted.fields.push(FieldIR {
            id: FieldId::from_identity(&identity),
            owner_type: owner.type_id.clone(),
            name,
            location: location(declarator, &source.path),
            visibility: field_visibility,
            type_reference: type_reference.clone(),
        });
    }
}

fn add_property(
    property_decl: Node<'_>,
    source: &SourceFile,
    namespace: Option<&str>,
    owner: &Owner<'_>,
    extracted: &mut ExtractedFile,
) {
    let Some(name) = property_decl
        .child_by_field_name("name")
        .and_then(|item| text(item, source))
    else {
        return;
    };
    let type_reference = property_decl.child_by_field_name("type").map_or_else(
        || {
            unresolved(
                UnresolvedReferenceKind::Type,
                String::new(),
                location(property_decl, &source.path),
            )
        },
        |type_node| unresolved_type(type_node, source),
    );
    let identity = format!(
        "scent:v1:csharp:{}:{}:property:{}.{name}",
        source.path,
        namespace.unwrap_or_default(),
        owner.qualified_name,
    );
    extracted.properties.push(PropertyIR {
        id: PropertyId::from_identity(&identity),
        owner_type: owner.type_id.clone(),
        name,
        location: location(property_decl, &source.path),
        visibility: visibility(property_decl, source),
        type_reference,
    });
}

fn add_method(
    node: Node<'_>,
    source: &SourceFile,
    namespace: Option<&str>,
    owner: &Owner<'_>,
    extracted: &mut ExtractedFile,
) {
    let Some(name) = node
        .child_by_field_name("name")
        .and_then(|item| text(item, source))
    else {
        return;
    };
    let Some(parameters_node) = node.child_by_field_name("parameters") else {
        return;
    };
    let return_type = node
        .child_by_field_name("returns")
        .map(|returns_node| unresolved_type(returns_node, source));
    let method = build_method_ir(
        node,
        &name,
        parameters_node,
        return_type,
        "method",
        source,
        namespace,
        owner,
    );
    extracted.methods.push(method);
}

fn add_constructor(
    node: Node<'_>,
    source: &SourceFile,
    namespace: Option<&str>,
    owner: &Owner<'_>,
    extracted: &mut ExtractedFile,
) {
    let Some(name) = node
        .child_by_field_name("name")
        .and_then(|item| text(item, source))
    else {
        return;
    };
    let Some(parameters_node) = node.child_by_field_name("parameters") else {
        return;
    };
    let method = build_method_ir(
        node,
        &name,
        parameters_node,
        None,
        "constructor",
        source,
        namespace,
        owner,
    );
    extracted.methods.push(method);
}

#[allow(clippy::too_many_arguments)]
fn build_method_ir(
    decl_node: Node<'_>,
    name: &str,
    parameters_node: Node<'_>,
    return_type: Option<Resolution<TypeId>>,
    kind_label: &str,
    source: &SourceFile,
    namespace: Option<&str>,
    owner: &Owner<'_>,
) -> MethodIR {
    let parameters = parse_parameters(parameters_node, source);
    // The raw parameter-list source text (rather than resolved types, which
    // don't exist yet) is enough to give overloads distinct, stable
    // identities even before overload resolution runs.
    let signature = text(parameters_node, source).unwrap_or_default();
    let identity = format!(
        "scent:v1:csharp:{}:{}:{kind_label}:{}.{name}{signature}",
        source.path,
        namespace.unwrap_or_default(),
        owner.qualified_name,
    );
    let mut facts = MethodFacts::default();
    if let Some(body) = decl_node.child_by_field_name("body") {
        collect_method_facts(body, source, &mut facts);
    }
    MethodIR {
        id: MethodId::from_identity(&identity),
        owner_type: owner.type_id.clone(),
        name: name.into(),
        location: location(decl_node, &source.path),
        visibility: visibility(decl_node, source),
        parameters,
        return_type,
        calls: facts.calls,
        field_accesses: facts.field_accesses,
        type_references: facts.type_references,
        instantiations: facts.instantiations,
    }
}

fn parse_parameters(parameter_list: Node<'_>, source: &SourceFile) -> Vec<ParameterIR> {
    let mut cursor = parameter_list.walk();
    parameter_list
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "parameter")
        .filter_map(|parameter| {
            let name = parameter
                .child_by_field_name("name")
                .and_then(|item| text(item, source))?;
            let type_reference = parameter.child_by_field_name("type").map_or_else(
                || {
                    unresolved(
                        UnresolvedReferenceKind::Type,
                        String::new(),
                        location(parameter, &source.path),
                    )
                },
                |type_node| unresolved_type(type_node, source),
            );
            Some(ParameterIR {
                name,
                location: location(parameter, &source.path),
                type_reference,
            })
        })
        .collect()
}

/// Facts collected while walking a single method or constructor body. Kept
/// separate from [`MethodIR`] itself so the walk can be built up field by
/// field before the enclosing method's identity/location/visibility are
/// known.
#[derive(Default)]
struct MethodFacts {
    calls: Vec<MethodCall>,
    field_accesses: Vec<FieldAccess>,
    type_references: Vec<TypeReference>,
    instantiations: Vec<Instantiation>,
}

/// Walks a method/constructor body attaching call, creation, and
/// member-access facts to `facts`. Local functions and local classes are
/// deliberately not descended into specially here; C# does not commonly nest
/// type/namespace declarations inside a method body for Phase 1's fixture
/// scope, and doing so is left to a later slice.
fn collect_method_facts(node: Node<'_>, source: &SourceFile, facts: &mut MethodFacts) {
    match node.kind() {
        "invocation_expression" => {
            if let Some(function_node) = node.child_by_field_name("function") {
                let (spelling, receiver) = match function_node.kind() {
                    "member_access_expression" => {
                        let spelling = function_node
                            .child_by_field_name("name")
                            .and_then(|item| text(item, source))
                            .unwrap_or_default();
                        (spelling, function_node.child_by_field_name("expression"))
                    }
                    _ => (text(function_node, source).unwrap_or_default(), None),
                };
                facts.calls.push(MethodCall {
                    target: unresolved(
                        UnresolvedReferenceKind::Method,
                        spelling,
                        location(function_node, &source.path),
                    ),
                    location: location(node, &source.path),
                });
                // The receiver of a call (e.g. `this` in `this.Other()`) is
                // walked as an ordinary sub-expression so a receiver that is
                // itself a field access or nested call is still recorded —
                // but the member-access naming the called method is not
                // double-counted as a field access.
                if let Some(receiver) = receiver {
                    collect_method_facts(receiver, source, facts);
                }
            }
            if let Some(arguments) = node.child_by_field_name("arguments") {
                collect_children(arguments, source, facts);
            }
        }
        "object_creation_expression" => {
            if let Some(type_node) = node.child_by_field_name("type") {
                facts.instantiations.push(Instantiation {
                    target: unresolved_type(type_node, source),
                    location: location(node, &source.path),
                });
            }
            if let Some(arguments) = node.child_by_field_name("arguments") {
                collect_children(arguments, source, facts);
            }
        }
        "member_access_expression" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                facts.field_accesses.push(FieldAccess {
                    target: unresolved(
                        UnresolvedReferenceKind::Field,
                        text(name_node, source).unwrap_or_default(),
                        location(name_node, &source.path),
                    ),
                    location: location(node, &source.path),
                });
            }
            if let Some(receiver) = node.child_by_field_name("expression") {
                collect_method_facts(receiver, source, facts);
            }
        }
        _ => collect_children(node, source, facts),
    }
}

fn collect_children(node: Node<'_>, source: &SourceFile, facts: &mut MethodFacts) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_method_facts(child, source, facts);
    }
}

fn child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let found = node
        .children(&mut cursor)
        .find(|child| child.kind() == kind);
    found
}

fn unresolved_type(node: Node<'_>, source: &SourceFile) -> Resolution<TypeId> {
    unresolved(
        UnresolvedReferenceKind::Type,
        text(node, source).unwrap_or_default(),
        location(node, &source.path),
    )
}

fn unresolved<T>(
    kind: UnresolvedReferenceKind,
    spelling: String,
    location: SourceLocation,
) -> Resolution<T> {
    Resolution::Unresolved(UnresolvedReference {
        kind,
        spelling,
        location,
        reason: "not yet resolved".into(),
    })
}

fn type_kind(kind: &str) -> Option<TypeKind> {
    match kind {
        "class_declaration" => Some(TypeKind::Class),
        "interface_declaration" => Some(TypeKind::Interface),
        "struct_declaration" => Some(TypeKind::Struct),
        "enum_declaration" => Some(TypeKind::Enum),
        "record_declaration" => Some(TypeKind::Record),
        _ => None,
    }
}

fn visibility(node: Node<'_>, source: &SourceFile) -> Visibility {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "modifier" {
            match text(child, source).as_deref() {
                Some("public") => return Visibility::Public,
                Some("protected") => return Visibility::Protected,
                Some("internal") => return Visibility::Internal,
                Some("private") => return Visibility::Private,
                _ => {}
            }
        }
    }
    Visibility::Unknown
}

fn text(node: Node<'_>, source: &SourceFile) -> Option<String> {
    node.utf8_text(source.contents.as_bytes())
        .ok()
        .map(str::to_owned)
}

fn location(node: Node<'_>, path: &NormalizedPath) -> SourceLocation {
    let start = node.start_position();
    let end = node.end_position();
    let range = SourceRange::new(
        SourcePosition {
            line: to_u32(start.row),
            column: to_u32(start.column),
        },
        SourcePosition {
            line: to_u32(end.row),
            column: to_u32(end.column),
        },
    )
    .expect("Tree-sitter nodes cannot have backwards ranges");
    SourceLocation::new(path.clone(), range)
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).expect("source positions must fit the SCENT location format")
}
