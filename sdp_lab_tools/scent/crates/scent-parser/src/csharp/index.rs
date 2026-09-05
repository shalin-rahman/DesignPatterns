//! Pass 1 of two-pass resolution: declaration indexes built from already
//! extracted facts. Building the index never guesses anything; it only
//! records what declarations exist and under what names they can be found.

use std::collections::HashMap;

use scent_domain::{FieldId, MethodId, PropertyId, TypeId};
use scent_ir::{ProjectIR, TypeKind};

/// Declaration lookup tables for one project's extracted facts.
///
/// Type lookups are keyed two ways: by fully qualified name (`namespace.
/// Outer.Inner`) for spellings that already carry a namespace, and by the
/// type's own simple name (`Inner`) for bare references. Member lookups are
/// scoped to an owning type, matching Phase 1's "exact, intra-project"
/// resolution scope: a member reference is resolved only against the type
/// that declares it, never through unrelated types that merely happen to
/// share a member name.
#[derive(Debug, Default)]
pub struct DeclarationIndex {
    types_by_qualified_name: HashMap<String, Vec<TypeId>>,
    types_by_simple_name: HashMap<String, Vec<TypeId>>,
    type_kinds: HashMap<TypeId, TypeKind>,
    methods_by_owner_and_name: HashMap<(TypeId, String), Vec<MethodId>>,
    fields_by_owner_and_name: HashMap<(TypeId, String), FieldId>,
    properties_by_owner_and_name: HashMap<(TypeId, String), PropertyId>,
}

/// The outcome of looking up a single spelling in the index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lookup<T> {
    Found(T),
    Ambiguous,
    Unknown,
}

impl DeclarationIndex {
    #[must_use]
    pub fn build(project: &ProjectIR) -> Self {
        let mut namespace_names: HashMap<&str, &str> = HashMap::new();
        for namespace in &project.namespaces {
            namespace_names.insert(namespace.id.as_str(), namespace.name.as_str());
        }

        let mut index = Self::default();
        for type_ir in &project.types {
            index.type_kinds.insert(type_ir.id.clone(), type_ir.kind);

            let simple_name = type_ir
                .name
                .rsplit('.')
                .next()
                .unwrap_or(type_ir.name.as_str());
            index
                .types_by_simple_name
                .entry(simple_name.to_owned())
                .or_default()
                .push(type_ir.id.clone());

            let qualified_name = type_ir
                .namespace_id
                .as_deref()
                .and_then(|id| namespace_names.get(id))
                .map_or_else(
                    || type_ir.name.clone(),
                    |namespace_name| format!("{namespace_name}.{}", type_ir.name),
                );
            index
                .types_by_qualified_name
                .entry(qualified_name)
                .or_default()
                .push(type_ir.id.clone());
        }
        for method in &project.methods {
            index
                .methods_by_owner_and_name
                .entry((method.owner_type.clone(), method.name.clone()))
                .or_default()
                .push(method.id.clone());
        }
        for field in &project.fields {
            index.fields_by_owner_and_name.insert(
                (field.owner_type.clone(), field.name.clone()),
                field.id.clone(),
            );
        }
        for property in &project.properties {
            index.properties_by_owner_and_name.insert(
                (property.owner_type.clone(), property.name.clone()),
                property.id.clone(),
            );
        }
        index
    }

    #[must_use]
    pub fn type_kind(&self, type_id: &TypeId) -> Option<TypeKind> {
        self.type_kinds.get(type_id).copied()
    }

    /// Resolves a type spelling exactly as it appeared in source. A
    /// dotted spelling is tried as a fully qualified name first; a bare
    /// spelling is matched against every type's simple name.
    #[must_use]
    pub fn lookup_type(&self, spelling: &str) -> Lookup<TypeId> {
        if spelling.contains('.') {
            if let Some(matches) = self.types_by_qualified_name.get(spelling) {
                return one_match(matches);
            }
        }
        self.types_by_simple_name
            .get(spelling)
            .map_or(Lookup::Unknown, |matches| one_match(matches))
    }

    #[must_use]
    pub fn lookup_method(&self, owner: &TypeId, spelling: &str) -> Lookup<MethodId> {
        self.methods_by_owner_and_name
            .get(&(owner.clone(), spelling.to_owned()))
            .map_or(Lookup::Unknown, |matches| one_match(matches))
    }

    #[must_use]
    pub fn lookup_field(&self, owner: &TypeId, spelling: &str) -> Option<FieldId> {
        self.fields_by_owner_and_name
            .get(&(owner.clone(), spelling.to_owned()))
            .cloned()
    }

    #[must_use]
    pub fn lookup_property(&self, owner: &TypeId, spelling: &str) -> Option<PropertyId> {
        self.properties_by_owner_and_name
            .get(&(owner.clone(), spelling.to_owned()))
            .cloned()
    }
}

fn one_match<T: Clone>(matches: &[T]) -> Lookup<T> {
    match matches {
        [] => Lookup::Unknown,
        [single] => Lookup::Found(single.clone()),
        _ => Lookup::Ambiguous,
    }
}
