//! Stable, language-neutral contracts shared by every SCENT crate.

pub mod analysis;
pub mod diagnostic;
pub mod id;
pub mod location;
pub mod path;
pub mod resolution;

pub use analysis::{AnalysisScope, Language, RiskLevel, Severity, Visibility};
pub use diagnostic::{Diagnostic, DiagnosticKind};
pub use id::{EntityId, FieldId, FileId, FindingId, MethodId, ProjectId, PropertyId, TypeId};
pub use location::{SourceLocation, SourcePosition, SourceRange, SourceRangeError};
pub use path::{NormalizedPath, NormalizedPathError};
pub use resolution::{Resolution, UnresolvedReference, UnresolvedReferenceKind};
