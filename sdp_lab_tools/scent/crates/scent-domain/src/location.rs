//! Source locations using zero-based, UTF-8 byte columns.

use crate::NormalizedPath;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceRangeError {
    EndBeforeStart,
}

impl SourceRange {
    /// Creates a range after verifying that it does not run backwards.
    ///
    /// # Errors
    ///
    /// Returns `EndBeforeStart` when the final position precedes the initial
    /// position.
    pub fn new(start: SourcePosition, end: SourcePosition) -> Result<Self, SourceRangeError> {
        if end < start {
            return Err(SourceRangeError::EndBeforeStart);
        }
        Ok(Self { start, end })
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceLocation {
    pub path: NormalizedPath,
    pub range: SourceRange,
}

impl SourceLocation {
    #[must_use]
    pub fn new(path: NormalizedPath, range: SourceRange) -> Self {
        Self { path, range }
    }
}

#[cfg(test)]
mod tests {
    use super::{SourcePosition, SourceRange, SourceRangeError};

    #[test]
    fn source_range_rejects_backward_locations() {
        let start = SourcePosition { line: 3, column: 2 };
        let end = SourcePosition { line: 2, column: 8 };
        assert_eq!(
            SourceRange::new(start, end),
            Err(SourceRangeError::EndBeforeStart)
        );
    }
}
