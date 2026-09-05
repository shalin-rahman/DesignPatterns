//! The finding model. A `Finding` is an interpretation built on top of
//! metrics and graph facts — never a fact itself.

use scent_domain::{AnalysisScope, Severity, SourceLocation, StableId};

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceItem {
    pub metric_key: &'static str,
    pub observed_value: f64,
    pub normalized_score: f64,
    pub weight: f64,
    pub contribution: f64,
    pub explanation: String,
    pub location: Option<SourceLocation>,
}

/// Deterministic identity for a finding, independent of runtime IDs.
/// `location_hash` is derived from `(rule_id, entity_id)`, not a line
/// number, so edits elsewhere in the file never invalidate a baseline
/// entry for this finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindingFingerprint {
    pub rule_id: String,
    pub entity_id: String,
    pub normalized_path: String,
    pub location_hash: String,
}

impl FindingFingerprint {
    #[must_use]
    pub fn new(rule_id: &str, entity_id: &str, normalized_path: &str) -> Self {
        let location_hash =
            StableId::from_identity(&format!("scent:v1:fingerprint:{rule_id}:{entity_id}"))
                .as_str()
                .to_owned();
        Self {
            rule_id: rule_id.to_owned(),
            entity_id: entity_id.to_owned(),
            normalized_path: normalized_path.to_owned(),
            location_hash,
        }
    }

    #[must_use]
    pub fn as_key(&self) -> String {
        format!("{}:{}", self.rule_id, self.location_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub fingerprint: FindingFingerprint,
    pub rule_id: &'static str,
    pub rule_name: &'static str,
    pub scope: AnalysisScope,
    pub severity: Severity,
    pub confidence: f32,
    pub location: SourceLocation,
    pub entity_id: String,
    pub evidence: Vec<EvidenceItem>,
}
