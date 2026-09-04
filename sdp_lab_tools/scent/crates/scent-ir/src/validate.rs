//! Invariant checks for extracted semantic facts.

use std::collections::BTreeSet;

use scent_domain::{FieldId, FileId, MethodId, PropertyId, TypeId};

use crate::ProjectIR;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectIrValidationError {
    DuplicateFile(FileId),
    DuplicateType(TypeId),
    DuplicateMethod(MethodId),
    DuplicateField(FieldId),
    DuplicateProperty(PropertyId),
    UnknownFileForType(TypeId),
    UnknownOwnerForMethod(MethodId),
    UnknownOwnerForField(FieldId),
    UnknownOwnerForProperty(PropertyId),
}

pub trait ValidateProjectIr {
    /// Validates only structural facts; it does not calculate derived metrics.
    ///
    /// # Errors
    ///
    /// Returns the first duplicate ID or dangling parent relationship.
    fn validate(&self) -> Result<(), ProjectIrValidationError>;
}

impl ValidateProjectIr for ProjectIR {
    fn validate(&self) -> Result<(), ProjectIrValidationError> {
        let file_ids = collect_ids(
            &self.files,
            |file| file.id.clone(),
            ProjectIrValidationError::DuplicateFile,
        )?;
        let type_ids = collect_ids(
            &self.types,
            |item| item.id.clone(),
            ProjectIrValidationError::DuplicateType,
        )?;
        collect_ids(
            &self.methods,
            |item| item.id.clone(),
            ProjectIrValidationError::DuplicateMethod,
        )?;
        collect_ids(
            &self.fields,
            |item| item.id.clone(),
            ProjectIrValidationError::DuplicateField,
        )?;
        collect_ids(
            &self.properties,
            |item| item.id.clone(),
            ProjectIrValidationError::DuplicateProperty,
        )?;

        for item in &self.types {
            if !file_ids.contains(&item.file_id) {
                return Err(ProjectIrValidationError::UnknownFileForType(
                    item.id.clone(),
                ));
            }
        }
        for item in &self.methods {
            if !type_ids.contains(&item.owner_type) {
                return Err(ProjectIrValidationError::UnknownOwnerForMethod(
                    item.id.clone(),
                ));
            }
        }
        for item in &self.fields {
            if !type_ids.contains(&item.owner_type) {
                return Err(ProjectIrValidationError::UnknownOwnerForField(
                    item.id.clone(),
                ));
            }
        }
        for item in &self.properties {
            if !type_ids.contains(&item.owner_type) {
                return Err(ProjectIrValidationError::UnknownOwnerForProperty(
                    item.id.clone(),
                ));
            }
        }
        Ok(())
    }
}

fn collect_ids<T, Id, GetId, Error>(
    items: &[T],
    get_id: GetId,
    make_error: impl Fn(Id) -> Error,
) -> Result<BTreeSet<Id>, Error>
where
    Id: Clone + Ord,
    GetId: Fn(&T) -> Id,
{
    let mut ids = BTreeSet::new();
    for item in items {
        let id = get_id(item);
        if !ids.insert(id.clone()) {
            return Err(make_error(id));
        }
    }
    Ok(ids)
}
