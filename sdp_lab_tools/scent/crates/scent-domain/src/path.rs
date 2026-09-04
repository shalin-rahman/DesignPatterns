//! Repository-relative, POSIX-normalized paths.

use core::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NormalizedPath(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NormalizedPathError {
    Empty,
    Absolute,
    ParentTraversal,
}

impl NormalizedPath {
    /// Parses a non-empty, repository-relative path into POSIX form.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty path, an absolute path, or a path that
    /// attempts parent-directory traversal.
    pub fn parse(path: &str) -> Result<Self, NormalizedPathError> {
        let normalized = path.replace('\\', "/");
        if normalized.is_empty() {
            return Err(NormalizedPathError::Empty);
        }
        if normalized.starts_with('/')
            || normalized.starts_with("//")
            || normalized.as_bytes().get(1) == Some(&b':')
        {
            return Err(NormalizedPathError::Absolute);
        }

        let parts: Vec<_> = normalized
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect();
        if parts.contains(&"..") {
            return Err(NormalizedPathError::ParentTraversal);
        }
        if parts.is_empty() {
            return Err(NormalizedPathError::Empty);
        }
        Ok(Self(parts.join("/")))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NormalizedPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for NormalizedPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "path must not be empty",
            Self::Absolute => "path must be repository-relative",
            Self::ParentTraversal => "path must not traverse above the repository root",
        })
    }
}

impl std::error::Error for NormalizedPathError {}

#[cfg(test)]
mod tests {
    use super::{NormalizedPath, NormalizedPathError};

    #[test]
    fn equivalent_separators_normalize_to_posix() {
        assert_eq!(
            NormalizedPath::parse("src\\services/Order.cs")
                .unwrap()
                .as_str(),
            "src/services/Order.cs"
        );
    }

    #[test]
    fn rejects_machine_specific_paths() {
        assert_eq!(
            NormalizedPath::parse("C:/repo/Order.cs"),
            Err(NormalizedPathError::Absolute)
        );
        assert_eq!(
            NormalizedPath::parse("/repo/Order.cs"),
            Err(NormalizedPathError::Absolute)
        );
    }
}
