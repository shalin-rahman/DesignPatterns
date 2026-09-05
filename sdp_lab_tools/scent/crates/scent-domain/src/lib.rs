//! Stable, language-neutral contracts shared by every SCENT crate.

pub mod analysis;
pub mod diagnostic;
pub mod id;
pub mod location;
pub mod path;
pub mod resolution;
pub mod suppression;

pub use analysis::{AnalysisScope, Language, Principle, RiskLevel, Severity, Visibility};
pub use diagnostic::{Diagnostic, DiagnosticKind};
pub use id::{
    CommitId, EdgeId, EntityId, FieldId, FileId, FindingId, MethodId, ProjectId, PropertyId,
    StableId, TypeId,
};
pub use location::{SourceLocation, SourcePosition, SourceRange, SourceRangeError};
pub use path::{NormalizedPath, NormalizedPathError};
pub use resolution::{Resolution, UnresolvedReference, UnresolvedReferenceKind};
pub use suppression::{SuppressedRange, SuppressionMap};
