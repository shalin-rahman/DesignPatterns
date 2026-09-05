use scent_domain::{Language, NormalizedPath, ProjectId, Resolution};
use scent_ir::ProjectIR;
use scent_parser::{extract_file, resolve_project, CSharpAdapter, DeclarationIndex, SourceFile};

fn build_project(sources: &[(&str, &str)]) -> ProjectIR {
    let mut adapter = CSharpAdapter::new();
    let mut project = ProjectIR {
        id: ProjectId::from_identity("csharp:sample"),
        language: Language::CSharp,
        files: vec![],
        namespaces: vec![],
        types: vec![],
        methods: vec![],
        fields: vec![],
        properties: vec![],
    };
    for (path, contents) in sources {
        let source = SourceFile {
            path: NormalizedPath::parse(path).unwrap(),
            contents: (*contents).into(),
        };
        let parsed = adapter.parse(&source);
        let extracted = extract_file(&source, &parsed.tree);
        project.files.push(extracted.file);
        project.namespaces.extend(extracted.namespaces);
        project.types.extend(extracted.types);
        project.methods.extend(extracted.methods);
        project.fields.extend(extracted.fields);
        project.properties.extend(extracted.properties);
    }
    project
}

#[test]
fn resolves_an_exact_intra_project_type_reference() {
    let project = build_project(&[(
        "src/Order.cs",
        "namespace Demo { public class Order {} \
             public class OrderFactory { public void Create() { new Order(); } } }",
    )]);
    let index = DeclarationIndex::build(&project);
    let resolved = resolve_project(project, &index);

    let create = resolved
        .methods
        .iter()
        .find(|item| item.name == "Create")
        .unwrap();
    assert!(matches!(
        create.instantiations[0].target,
        Resolution::Resolved(_)
    ));
}

#[test]
fn leaves_an_overload_ambiguous_call_unresolved() {
    let project = build_project(&[(
        "src/Order.cs",
        "namespace Demo { public class Order { \
             public void Process() { this.Ship(); } \
             public void Ship() {} \
             public void Ship(int retries) {} \
         } }",
    )]);
    let index = DeclarationIndex::build(&project);
    let resolved = resolve_project(project, &index);

    let process = resolved
        .methods
        .iter()
        .find(|item| item.name == "Process")
        .unwrap();
    match &process.calls[0].target {
        Resolution::Unresolved(reference) => {
            assert_eq!(reference.reason, "ambiguous overload on the declaring type");
        }
        Resolution::Resolved(_) => panic!("an overloaded call must stay unresolved"),
    }
}

#[test]
fn leaves_an_unknown_framework_type_unresolved() {
    let project = build_project(&[(
        "src/Order.cs",
        "namespace Demo { public class Order { \
             public void Process() { new System.Guid(); } \
         } }",
    )]);
    let index = DeclarationIndex::build(&project);
    let resolved = resolve_project(project, &index);

    let process = &resolved.methods[0];
    match &process.instantiations[0].target {
        Resolution::Unresolved(reference) => {
            assert_eq!(reference.reason, "no intra-project type with this name");
        }
        Resolution::Resolved(_) => panic!("a framework type must stay unresolved in Phase 1"),
    }
}

#[test]
fn leaves_an_unresolved_member_access_unresolved() {
    let project = build_project(&[(
        "src/Order.cs",
        "namespace Demo { public class Order { \
             public void Process() { var value = this.Missing; } \
         } }",
    )]);
    let index = DeclarationIndex::build(&project);
    let resolved = resolve_project(project, &index);

    let process = &resolved.methods[0];
    match &process.field_accesses[0].target {
        Resolution::Unresolved(reference) => {
            assert_eq!(
                reference.reason,
                "no field or property with this name on the declaring type"
            );
        }
        Resolution::Resolved(_) => panic!("an unknown member must stay unresolved"),
    }
}

#[test]
fn resolves_a_call_routed_through_a_local_variable() {
    let project = build_project(&[(
        "src/Order.cs",
        "namespace Demo { \
             public class Order { \
                 public void Ship() { Warehouse w = new Warehouse(); w.Reserve(); } \
             } \
             public class Warehouse { public void Reserve() {} } \
         }",
    )]);
    let index = DeclarationIndex::build(&project);
    let resolved = resolve_project(project, &index);

    let ship = resolved
        .methods
        .iter()
        .find(|item| item.name == "Ship")
        .unwrap();
    assert!(matches!(ship.calls[0].target, Resolution::Resolved(_)));
}

#[test]
fn a_local_variable_shadows_a_field_of_the_same_name() {
    let project = build_project(&[(
        "src/Order.cs",
        "namespace Demo { \
             public class Order { \
                 private Warehouse warehouse; \
                 public void Ship() { Depot warehouse = new Depot(); warehouse.Reserve(); } \
             } \
             public class Warehouse { public void Reserve() {} } \
             public class Depot { public void Reserve() {} } \
         }",
    )]);
    let index = DeclarationIndex::build(&project);
    let resolved = resolve_project(project, &index);

    let depot = resolved
        .types
        .iter()
        .find(|item| item.name == "Depot")
        .unwrap();
    let ship = resolved
        .methods
        .iter()
        .find(|item| item.name == "Ship")
        .unwrap();
    match &ship.calls[0].target {
        Resolution::Resolved(method_id) => {
            let resolved_method = resolved
                .methods
                .iter()
                .find(|item| &item.id == method_id)
                .unwrap();
            assert_eq!(&resolved_method.owner_type, &depot.id);
        }
        Resolution::Unresolved(_) => panic!("the local variable should shadow the field"),
    }
}

#[test]
fn resolution_does_not_change_entity_ids() {
    let project = build_project(&[(
        "src/Order.cs",
        "namespace Demo { public class Order {} \
             public class OrderFactory { public void Create() { new Order(); } } }",
    )]);
    let type_ids_before: Vec<_> = project.types.iter().map(|item| item.id.clone()).collect();
    let method_ids_before: Vec<_> = project.methods.iter().map(|item| item.id.clone()).collect();
    let index = DeclarationIndex::build(&project);
    let resolved = resolve_project(project, &index);

    let type_ids_after: Vec<_> = resolved.types.iter().map(|item| item.id.clone()).collect();
    let method_ids_after: Vec<_> = resolved
        .methods
        .iter()
        .map(|item| item.id.clone())
        .collect();
    assert_eq!(type_ids_before, type_ids_after);
    assert_eq!(method_ids_before, method_ids_after);
}
