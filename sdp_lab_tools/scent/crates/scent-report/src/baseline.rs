//! Baseline comparison. A baseline is the set of fingerprint keys from a
//! prior run; a finding already present in it does not fail a quality gate
//! unless configured otherwise (`docs/prompt.md` §40).

use std::collections::BTreeSet;

use scent_rules::Finding;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Baseline {
    keys: BTreeSet<String>,
}

impl Baseline {
    #[must_use]
    pub fn from_findings(findings: &[Finding]) -> Self {
        Self {
            keys: findings
                .iter()
                .map(|finding| finding.fingerprint.as_key())
                .collect(),
        }
    }

    /// One fingerprint key per line, already sorted — stable, diffable, and
    /// trivial to read back with [`Self::parse`]. No JSON parser is needed
    /// for a flat set of opaque keys.
    #[must_use]
    pub fn serialize(&self) -> String {
        self.keys.iter().cloned().collect::<Vec<_>>().join("\n")
    }

    #[must_use]
    pub fn parse(text: &str) -> Self {
        Self {
            keys: text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect(),
        }
    }

    #[must_use]
    pub fn contains(&self, finding: &Finding) -> bool {
        self.keys.contains(&finding.fingerprint.as_key())
    }

    /// Findings not present in this baseline.
    #[must_use]
    pub fn new_findings<'a>(&self, findings: &'a [Finding]) -> Vec<&'a Finding> {
        findings
            .iter()
            .filter(|finding| !self.contains(finding))
            .collect()
    }
}
