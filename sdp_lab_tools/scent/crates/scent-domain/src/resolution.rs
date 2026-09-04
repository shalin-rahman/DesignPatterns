//! Explicit semantic resolution results.

use crate::SourceLocation;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Resolution<T> {
    Resolved(T),
    Unresolved(UnresolvedReference),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UnresolvedReferenceKind {
    Type,
    Method,
    Field,
    Property,
    Namespace,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnresolvedReference {
    pub kind: UnresolvedReferenceKind,
    pub spelling: String,
    pub location: SourceLocation,
    pub reason: String,
}

impl<T> Resolution<T> {
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }
}

#[cfg(test)]
mod tests {
    use super::Resolution;

    #[test]
    fn reports_resolved_state_without_interpreting_unresolved_values() {
        assert!(Resolution::<u8>::Resolved(1).is_resolved());
    }
}
