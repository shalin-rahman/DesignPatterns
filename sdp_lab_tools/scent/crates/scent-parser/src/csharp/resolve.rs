//! Pass 2 of two-pass resolution: replaces `Unresolved` facts with
//! `Resolved` ones only where the declaration index has exactly one
//! intra-project candidate. Ambiguous or unknown spellings are left
//! `Unresolved` with an explanatory reason — never guessed.

use std::collections::HashMap;

use scent_domain::{MethodId, Resolution, TypeId, UnresolvedReference};
use scent_ir::{CallReceiver, MemberTarget, ProjectIR, TypeKind};

use super::index::{DeclarationIndex, Lookup};

/// Per-type field/property name -> resolved type, snapshotted once fields
/// and properties are resolved, so a bare-identifier receiver later in this
/// pass (`b.Foo()`) can be looked up as a same-type field/property without
/// re-borrowing `project` while methods are being mutated.
struct MemberTypesByOwner {
    fields: HashMap<(TypeId, String), Resolution<TypeId>>,
    properties: HashMap<(TypeId, String), Resolution<TypeId>>,
}

#[must_use]
pub fn resolve_project(mut project: ProjectIR, index: &DeclarationIndex) -> ProjectIR {
    for type_ir in &mut project.types {
        type_ir.interface_types = type_ir
            .interface_types
            .iter()
            .cloned()
            .map(|entry| resolve_type_ref(entry, index))
            .collect();
        // Milestone-3 follow-up: the first interface_types entry is the
        // actual base class exactly when it resolves to a Class or Record;
        // syntax alone could not tell the two apart at extraction time.
        if let Some(Resolution::Resolved(candidate)) = type_ir.interface_types.first().cloned() {
            if matches!(
                index.type_kind(&candidate),
                Some(TypeKind::Class | TypeKind::Record)
            ) {
                type_ir.base_types.push(type_ir.interface_types.remove(0));
            }
        }
    }
    for field in &mut project.fields {
        field.type_reference = resolve_type_ref(field.type_reference.clone(), index);
    }
    for property in &mut project.properties {
        property.type_reference = resolve_type_ref(property.type_reference.clone(), index);
    }

    let member_types = MemberTypesByOwner {
        fields: project
            .fields
            .iter()
            .map(|field| {
                (
                    (field.owner_type.clone(), field.name.clone()),
                    field.type_reference.clone(),
                )
            })
            .collect(),
        properties: project
            .properties
            .iter()
            .map(|property| {
                (
                    (property.owner_type.clone(), property.name.clone()),
                    property.type_reference.clone(),
                )
            })
            .collect(),
    };

    for method in &mut project.methods {
        let owner = method.owner_type.clone();
        // Snapshotted before the mutable loops below: parameters and locals
        // of this same method are resolved first, so a `Named` receiver can
        // be looked up against them without borrowing `method` twice at once.
        let parameter_types: HashMap<String, Resolution<TypeId>> = method
            .parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.type_reference.clone()))
            .collect();
        for parameter in &mut method.parameters {
            parameter.type_reference = resolve_type_ref(parameter.type_reference.clone(), index);
        }
        method.return_type = method
            .return_type
            .clone()
            .map(|entry| resolve_type_ref(entry, index));
        for local in &mut method.local_variables {
            local.type_reference = resolve_type_ref(local.type_reference.clone(), index);
        }
        // Later declarations win on a name collision, matching how a
        // redeclared local shadows an earlier one in the same scope in real
        // C# (Phase 1 does not track nested block scoping beyond this).
        let local_types: HashMap<String, Resolution<TypeId>> = method
            .local_variables
            .iter()
            .map(|local| (local.name.clone(), local.type_reference.clone()))
            .collect();

        for call in &mut method.calls {
            call.target = resolve_scoped(
                &owner,
                &call.receiver,
                &local_types,
                &parameter_types,
                &member_types,
                index,
                resolve_method_ref,
                call.target.clone(),
            );
        }
        for access in &mut method.field_accesses {
            access.target = resolve_scoped(
                &owner,
                &access.receiver,
                &local_types,
                &parameter_types,
                &member_types,
                index,
                resolve_field_access,
                access.target.clone(),
            );
        }
        for instantiation in &mut method.instantiations {
            instantiation.target = resolve_type_ref(instantiation.target.clone(), index);
        }
    }
    project
}

/// Resolves `current` against the right owner type for `receiver`:
/// - `SelfOrImplicit` resolves against the enclosing type `owner` — C#'s own
///   binding rule for a bare or `this.`-qualified reference.
/// - `Named(name)` resolves against whatever type a local variable,
///   parameter, or field named `name` holds (checked in that order — real
///   C# shadowing: a local shadows a parameter, which shadows a field), if
///   that type itself resolved — a scoping fact, not a guess about program
///   behavior.
/// - `Other` (anything more complex) is left exactly as extraction left it,
///   with a reason explaining resolution was not attempted.
#[allow(clippy::too_many_arguments)]
fn resolve_scoped<T>(
    owner: &TypeId,
    receiver: &CallReceiver,
    local_types: &HashMap<String, Resolution<TypeId>>,
    parameter_types: &HashMap<String, Resolution<TypeId>>,
    member_types: &MemberTypesByOwner,
    index: &DeclarationIndex,
    resolve_against: impl Fn(&TypeId, Resolution<T>, &DeclarationIndex) -> Resolution<T>,
    current: Resolution<T>,
) -> Resolution<T> {
    match receiver {
        CallReceiver::SelfOrImplicit => resolve_against(owner, current, index),
        CallReceiver::Named(name) => {
            let receiver_type = local_types
                .get(name)
                .or_else(|| parameter_types.get(name))
                .or_else(|| member_types.fields.get(&(owner.clone(), name.clone())))
                .or_else(|| member_types.properties.get(&(owner.clone(), name.clone())));
            match receiver_type {
                Some(Resolution::Resolved(receiver_owner)) => {
                    resolve_against(receiver_owner, current, index)
                }
                Some(Resolution::Unresolved(_)) => mark_out_of_scope(
                    current,
                    "receiver's own type did not resolve; cannot resolve through it",
                ),
                None => mark_out_of_scope(
                    current,
                    "receiver is not a known local variable, parameter, or field of the enclosing type",
                ),
            }
        }
        CallReceiver::Other => mark_out_of_scope(
            current,
            "receiver is a complex expression; only `this`, bare, and named-local/parameter/field receivers are resolved in Phase 1",
        ),
    }
}

/// Rewrites an already-`Unresolved` fact's reason without attempting any
/// lookup — used when resolution is intentionally out of scope rather than
/// merely unmatched.
fn mark_out_of_scope<T>(current: Resolution<T>, reason: &str) -> Resolution<T> {
    match current {
        Resolution::Resolved(_) => current,
        Resolution::Unresolved(reference) => Resolution::Unresolved(UnresolvedReference {
            reason: reason.into(),
            ..reference
        }),
    }
}

fn resolve_type_ref(current: Resolution<TypeId>, index: &DeclarationIndex) -> Resolution<TypeId> {
    let Resolution::Unresolved(reference) = current else {
        return current;
    };
    match index.lookup_type(&reference.spelling) {
        Lookup::Found(type_id) => Resolution::Resolved(type_id),
        Lookup::Ambiguous => Resolution::Unresolved(UnresolvedReference {
            reason: "ambiguous intra-project type name".into(),
            ..reference
        }),
        Lookup::Unknown => Resolution::Unresolved(UnresolvedReference {
            reason: "no intra-project type with this name".into(),
            ..reference
        }),
    }
}

fn resolve_method_ref(
    owner: &TypeId,
    current: Resolution<MethodId>,
    index: &DeclarationIndex,
) -> Resolution<MethodId> {
    let Resolution::Unresolved(reference) = current else {
        return current;
    };
    match index.lookup_method(owner, &reference.spelling) {
        Lookup::Found(method_id) => Resolution::Resolved(method_id),
        Lookup::Ambiguous => Resolution::Unresolved(UnresolvedReference {
            reason: "ambiguous overload on the declaring type".into(),
            ..reference
        }),
        Lookup::Unknown => Resolution::Unresolved(UnresolvedReference {
            reason: "no method with this name on the declaring type".into(),
            ..reference
        }),
    }
}

fn resolve_field_access(
    owner: &TypeId,
    current: Resolution<MemberTarget>,
    index: &DeclarationIndex,
) -> Resolution<MemberTarget> {
    let Resolution::Unresolved(reference) = current else {
        return current;
    };
    let field = index.lookup_field(owner, &reference.spelling);
    let property = index.lookup_property(owner, &reference.spelling);
    match (field, property) {
        (Some(field_id), None) => Resolution::Resolved(MemberTarget::Field(field_id)),
        (None, Some(property_id)) => Resolution::Resolved(MemberTarget::Property(property_id)),
        (Some(_), Some(_)) => Resolution::Unresolved(UnresolvedReference {
            reason: "ambiguous between a field and a property of the same name".into(),
            ..reference
        }),
        (None, None) => Resolution::Unresolved(UnresolvedReference {
            reason: "no field or property with this name on the declaring type".into(),
            ..reference
        }),
    }
}
