//! Language-neutral semantic facts. Derived analysis does not belong here.

pub mod model;
pub mod validate;

pub use model::{
    FieldAccess, FieldIR, FileIR, Instantiation, MemberTarget, MethodCall, MethodIR, NamespaceIR,
    ParameterIR, ProjectIR, PropertyIR, TypeIR, TypeKind, TypeReference,
};
pub use validate::{ProjectIrValidationError, ValidateProjectIr};

#[cfg(test)]
mod tests {
    use scent_domain::{
        FileId, Language, NormalizedPath, ProjectId, PropertyId, Resolution, SourceLocation,
        SourcePosition, SourceRange, TypeId, Visibility,
    };

    use crate::{
        FileIR, ProjectIR, ProjectIrValidationError, PropertyIR, TypeIR, TypeKind,
        ValidateProjectIr,
    };

    fn location() -> SourceLocation {
        let path = NormalizedPath::parse("src/Order.cs").unwrap();
        let point = SourcePosition { line: 0, column: 0 };
        SourceLocation::new(path, SourceRange::new(point, point).unwrap())
    }

    fn valid_project() -> ProjectIR {
        let file_id = FileId::from_identity("csharp:src/Order.cs");
        ProjectIR {
            id: ProjectId::from_identity("csharp:sample"),
            language: Language::CSharp,
            files: vec![FileIR {
                id: file_id.clone(),
                path: NormalizedPath::parse("src/Order.cs").unwrap(),
                namespace_ids: vec![],
                type_ids: vec![TypeId::from_identity("Order")],
            }],
            namespaces: vec![],
            types: vec![TypeIR {
                id: TypeId::from_identity("Order"),
                file_id,
                namespace_id: None,
                name: "Order".into(),
                kind: TypeKind::Class,
                location: location(),
                visibility: Visibility::Public,
                base_types: vec![],
                interface_types: vec![],
            }],
            methods: vec![],
            fields: vec![],
            properties: vec![],
        }
    }

    #[test]
    fn accepts_a_structurally_valid_project() {
        assert_eq!(valid_project().validate(), Ok(()));
    }

    #[test]
    fn rejects_type_that_references_an_unknown_file() {
        let mut project = valid_project();
        project.types[0].file_id = FileId::from_identity("missing");
        assert!(matches!(
            project.validate(),
            Err(ProjectIrValidationError::UnknownFileForType(_))
        ));
    }

    #[test]
    fn rejects_property_that_references_an_unknown_owner_type() {
        let mut project = valid_project();
        project.properties.push(PropertyIR {
            id: PropertyId::from_identity("Order.Total"),
            owner_type: TypeId::from_identity("missing"),
            name: "Total".into(),
            location: location(),
            visibility: Visibility::Public,
            type_reference: Resolution::Unresolved(scent_domain::UnresolvedReference {
                kind: scent_domain::UnresolvedReferenceKind::Type,
                spelling: "decimal".into(),
                location: location(),
                reason: "not yet resolved".into(),
            }),
        });
        assert!(matches!(
            project.validate(),
            Err(ProjectIrValidationError::UnknownOwnerForProperty(_))
        ));
    }
}
