//! Deterministic identifiers derived from normalized semantic identity.

use core::fmt;
use sha2::{Digest, Sha256};

/// A 256-bit identifier serialized as lower-case hexadecimal.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableId(String);

impl StableId {
    /// Creates an ID from a canonical identity string.
    #[must_use]
    pub fn from_identity(identity: &str) -> Self {
        let digest = Sha256::digest(identity.as_bytes());
        Self(format!("{digest:x}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StableId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(StableId);

        impl $name {
            #[must_use]
            pub fn from_identity(identity: &str) -> Self {
                Self(StableId::from_identity(identity))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

typed_id!(ProjectId);
typed_id!(FileId);
typed_id!(EntityId);
typed_id!(TypeId);
typed_id!(MethodId);
typed_id!(FieldId);
typed_id!(PropertyId);
typed_id!(FindingId);
typed_id!(EdgeId);

/// A Git commit SHA. Unlike the other typed IDs, this wraps the real commit
/// hash rather than re-hashing it through [`StableId::from_identity`] — Git
/// already assigns commits a deterministic, content-addressed identity, so
/// hashing it again would only obscure the real commit hash for no benefit.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommitId(String);

impl CommitId {
    #[must_use]
    pub fn from_sha(sha: &str) -> Self {
        Self(sha.to_owned())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::EntityId;

    #[test]
    fn identity_hash_is_stable() {
        let identity = "scent:v1:csharp:sample:Example.Service:method:Run()";
        assert_eq!(
            EntityId::from_identity(identity),
            EntityId::from_identity(identity)
        );
    }

    #[test]
    fn different_identities_are_distinct() {
        assert_ne!(
            EntityId::from_identity("one"),
            EntityId::from_identity("two")
        );
    }
}
