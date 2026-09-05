//! Long Parameter List (Bloaters). Parameter count alone still runs through
//! the same normalized evidence curve as every other rule, rather than a
//! bare `count > threshold` check.

use scent_domain::AnalysisScope;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.LONG_PARAMETER_LIST.thresholds]`); these are the built-in
/// defaults when a project has no override.
pub struct LongParameterList {
    pub parameter_count_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "LONG_PARAMETER_LIST";
const PARAMETER_COUNT_THRESHOLD: f64 = 5.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for LongParameterList {
    fn default() -> Self {
        Self {
            parameter_count_threshold: PARAMETER_COUNT_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for LongParameterList {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Long Parameter List"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Method
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for method in &ctx.project.methods {
            let parameter_count =
                f64::from(u32::try_from(method.parameters.len()).unwrap_or(u32::MAX));
            let observations = [Observation {
                metric_key: "parameter_count",
                observed_value: parameter_count,
                threshold: self.parameter_count_threshold,
                weight: 1.0,
                explain: |value, threshold| format!("{value} parameters (threshold {threshold})"),
            }];
            let (evidence, confidence) = build_evidence(&observations);
            if confidence < self.min_confidence {
                continue;
            }

            findings.push(Finding {
                fingerprint: FindingFingerprint::new(
                    RULE_ID,
                    method.id.as_str(),
                    method.location.path.as_str(),
                ),
                rule_id: RULE_ID,
                rule_name: self.name(),
                scope: AnalysisScope::Method,
                severity: severity_from_observations(&observations),
                confidence,
                location: method.location.clone(),
                entity_id: method.id.as_str().to_owned(),
                evidence,
            });
        }
        findings
    }
}
