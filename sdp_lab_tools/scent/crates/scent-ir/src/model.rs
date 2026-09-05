//! The facts-only semantic intermediate representation.

use scent_domain::{
    FieldId, FileId, Language, MethodId, NormalizedPath, PropertyId, Resolution, SourceLocation,
    TypeId, Visibility,
};

use scent_domain::ProjectId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectIR {
    pub id: ProjectId,
    pub language: Language,
    pub files: Vec<FileIR>,
    pub namespaces: Vec<NamespaceIR>,
    pub types: Vec<TypeIR>,
    pub methods: Vec<MethodIR>,
    pub fields: Vec<FieldIR>,
    pub properties: Vec<PropertyIR>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileIR {
    pub id: FileId,
    pub path: NormalizedPath,
    pub namespace_ids: Vec<String>,
    pub type_ids: Vec<TypeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceIR {
    pub id: String,
    pub name: String,
    pub file_id: FileId,
    pub location: SourceLocation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeKind {
    Class,
    Interface,
    Struct,
    Enum,
    Record,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeIR {
    pub id: TypeId,
    pub file_id: FileId,
    pub namespace_id: Option<String>,
    pub name: String,
    pub kind: TypeKind,
    pub location: SourceLocation,
    pub visibility: Visibility,
    pub base_types: Vec<Resolution<TypeId>>,
    pub interface_types: Vec<Resolution<TypeId>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodIR {
    pub id: MethodId,
    pub owner_type: TypeId,
    pub name: String,
    pub location: SourceLocation,
    pub visibility: Visibility,
    pub parameters: Vec<ParameterIR>,
    pub return_type: Option<Resolution<TypeId>>,
    pub calls: Vec<MethodCall>,
    pub field_accesses: Vec<FieldAccess>,
    pub type_references: Vec<TypeReference>,
    pub instantiations: Vec<Instantiation>,
    pub local_variables: Vec<LocalVariable>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterIR {
    pub name: String,
    pub location: SourceLocation,
    pub type_reference: Resolution<TypeId>,
}

/// A local variable declared inside a method body (`var w = ...;`,
/// `Warehouse w = ...;`). Recorded as a real fact — what the source
/// declares — so the resolver can look a bare-identifier receiver up as a
/// local in scope, the same way it already does for parameters and fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalVariable {
    pub name: String,
    pub location: SourceLocation,
    pub type_reference: Resolution<TypeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldIR {
    pub id: FieldId,
    pub owner_type: TypeId,
    pub name: String,
    pub location: SourceLocation,
    pub visibility: Visibility,
    pub type_reference: Resolution<TypeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyIR {
    pub id: PropertyId,
    pub owner_type: TypeId,
    pub name: String,
    pub location: SourceLocation,
    pub visibility: Visibility,
    pub type_reference: Resolution<TypeId>,
}

/// Whether a call or member access had a receiver other than `this`. C#'s
/// own binding rules resolve a bare call/access or an explicit `this.` one
/// against the enclosing type (or its base types) — a real language fact,
/// not a guess. A bare-identifier receiver (`b.Foo()`) is recorded as
/// `Named` so the resolver can look the name up as a parameter or field in
/// scope (again a scoping fact, not a guess) and resolve against *that*
/// type instead. Anything more complex (`a.b.Foo()`, `base.Foo()`, a cast,
/// a nested call) stays `Other`: Phase 1 does not track local-variable
/// types, so guessing further would mean fabricating a receiver type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallReceiver {
    SelfOrImplicit,
    Named(String),
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodCall {
    pub target: Resolution<MethodId>,
    pub receiver: CallReceiver,
    /// How many `.` hops the receiver expression itself contains, counted
    /// purely from syntax (no resolution needed): 0 for bare/`this`, 1 for
    /// a single identifier (`warehouse.Reserve()`), 2+ for a real chain
    /// (`a.b.Foo()`). Real evidence for Law of Demeter risk — "talking to a
    /// friend of a friend" is exactly a chain of depth 2 or more.
    pub receiver_chain_depth: u32,
    pub location: SourceLocation,
}

/// A field-vs-property distinction cannot be made from C# syntax alone; it
/// requires the owner type's resolved member index, so a [`FieldAccess`]
/// stays [`Resolution::Unresolved`] until the Milestone 4 resolver commits to
/// one of these variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberTarget {
    Field(FieldId),
    Property(PropertyId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldAccess {
    pub target: Resolution<MemberTarget>,
    pub receiver: CallReceiver,
    /// See [`MethodCall::receiver_chain_depth`].
    pub receiver_chain_depth: u32,
    pub location: SourceLocation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeReference {
    pub target: Resolution<TypeId>,
    pub location: SourceLocation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instantiation {
    pub target: Resolution<TypeId>,
    pub location: SourceLocation,
}
