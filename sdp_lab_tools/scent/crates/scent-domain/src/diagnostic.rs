//! Structured diagnostics that preserve non-fatal analysis failures.

use crate::SourceLocation;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticKind {
    ParseError,
    ResolutionError,
    UnsupportedSyntax,
    ConfigurationError,
    GitError,
    DockerError,
    InternalError,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub location: Option<SourceLocation>,
}

impl Diagnostic {
    #[must_use]
    pub fn new(
        kind: DiagnosticKind,
        message: impl Into<String>,
        location: Option<SourceLocation>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            location,
        }
    }
}
