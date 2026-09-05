//! Shared evidence normalization, confidence aggregation, and severity
//! calculation. A rule's job is to collect raw observations; this module
//! turns them into a normalized score, a weighted confidence, and an
//! independent severity — so no rule hand-rolls a binary threshold.

use scent_domain::Severity;

use crate::model::EvidenceItem;

/// One raw observation before normalization: what was measured, against
/// what threshold, and how much it should weigh in the final confidence.
pub struct Observation {
    pub metric_key: &'static str,
    pub observed_value: f64,
    pub threshold: f64,
    pub weight: f64,
    pub explain: fn(f64, f64) -> String,
}

/// Maps `observed/threshold` to `(0, 1)`: 0 at zero, 0.5 exactly at the
/// threshold, approaching 1 as the observation grows far past it. This is
/// the normalization curve every rule shares, so "evidence strength" means
/// the same thing across rules.
#[must_use]
pub fn normalize_ratio(observed: f64, threshold: f64) -> f64 {
    if threshold <= 0.0 {
        return 0.0;
    }
    let ratio = observed / threshold;
    ratio / (1.0 + ratio)
}

/// Builds evidence items and an aggregate confidence (the weights'
/// weighted sum of normalized scores) from a set of observations. Weights
/// need not sum to exactly 1.0, but rules should keep them close to that so
/// confidence stays within `[0, 1]`.
#[must_use]
pub fn build_evidence(observations: &[Observation]) -> (Vec<EvidenceItem>, f32) {
    let mut evidence = Vec::with_capacity(observations.len());
    let mut confidence = 0.0_f64;
    for observation in observations {
        let normalized = normalize_ratio(observation.observed_value, observation.threshold);
        let contribution = normalized * observation.weight;
        confidence += contribution;
        evidence.push(EvidenceItem {
            metric_key: observation.metric_key,
            observed_value: observation.observed_value,
            normalized_score: normalized,
            weight: observation.weight,
            contribution,
            explanation: (observation.explain)(observation.observed_value, observation.threshold),
            location: None,
        });
    }
    // Confidence is modeled as f32 (§21); the f64 accumulator above just
    // avoids compounding rounding error across several weighted terms.
    #[allow(clippy::cast_possible_truncation)]
    let confidence = confidence.clamp(0.0, 1.0) as f32;
    (evidence, confidence)
}

/// Severity reflects potential impact, not confidence in the finding: it is
/// the worst (highest) ratio among the observations, banded into four
/// levels. A method at 3x its LOC threshold is more severe regardless of
/// how confident the finding is.
#[must_use]
pub fn severity_from_observations(observations: &[Observation]) -> Severity {
    let worst_ratio = observations
        .iter()
        .map(|observation| {
            if observation.threshold <= 0.0 {
                0.0
            } else {
                observation.observed_value / observation.threshold
            }
        })
        .fold(0.0_f64, f64::max);

    if worst_ratio >= 3.0 {
        Severity::Critical
    } else if worst_ratio >= 2.0 {
        Severity::High
    } else if worst_ratio >= 1.0 {
        Severity::Medium
    } else {
        Severity::Low
    }
}
