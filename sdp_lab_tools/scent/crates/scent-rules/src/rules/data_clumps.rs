//! Data Clumps (Bloaters). Flags parameter groups that recur, unchanged,
//! across several methods — a sign the group should be its own type.
//!
//! Simplification, stated up front rather than hidden: this groups methods
//! by their *exact* full parameter signature (every parameter's name and
//! type spelling, in order), not by arbitrary recurring subsets. Detecting
//! partial/reordered clumps would need combinatorial subset search across
//! every method's parameter list, which is real future scope, not silently
//! approximated here.

use std::collections::HashMap;

use scent_domain::AnalysisScope;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.DATA_CLUMPS.thresholds]`); these are the built-in defaults.
pub struct DataClumps {
    /// A clump needs at least this many parameters to be worth a shared type.
    pub min_clump_size: usize,
    /// A clump needs to recur across at least this many methods.
    pub occurrence_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "DATA_CLUMPS";
const MIN_CLUMP_SIZE: usize = 3;
const OCCURRENCE_THRESHOLD: f64 = 3.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for DataClumps {
    fn default() -> Self {
        Self {
            min_clump_size: MIN_CLUMP_SIZE,
            occurrence_threshold: OCCURRENCE_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for DataClumps {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Data Clumps"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Method
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, method) in ctx.project.methods.iter().enumerate() {
            if method.parameters.len() < self.min_clump_size {
                continue;
            }
            let signature = method
                .parameters
                .iter()
                .map(|parameter| {
                    format!("{}:{}", parameter.name, spelling(&parameter.type_reference))
                })
                .collect::<Vec<_>>()
                .join(",");
            groups.entry(signature).or_default().push(index);
        }

        let mut findings = Vec::new();
        for method_indices in groups.values() {
            let occurrence_count =
                f64::from(u32::try_from(method_indices.len()).unwrap_or(u32::MAX));
            let observations = [Observation {
                metric_key: "occurrence_count",
                observed_value: occurrence_count,
                threshold: self.occurrence_threshold,
                weight: 1.0,
                explain: |value, threshold| {
                    format!("this exact parameter group recurs in {value} methods (threshold {threshold})")
                },
            }];
            let (evidence, confidence) = build_evidence(&observations);
            if confidence < self.min_confidence {
                continue;
            }
            let severity = severity_from_observations(&observations);

            for &index in method_indices {
                let method = &ctx.project.methods[index];
                findings.push(Finding {
                    fingerprint: FindingFingerprint::new(
                        RULE_ID,
                        method.id.as_str(),
                        method.location.path.as_str(),
                    ),
                    rule_id: RULE_ID,
                    rule_name: self.name(),
                    scope: AnalysisScope::Method,
                    severity,
                    confidence,
                    location: method.location.clone(),
                    entity_id: method.id.as_str().to_owned(),
                    evidence: evidence.clone(),
                });
            }
        }
        findings
    }
}

fn spelling(type_reference: &scent_domain::Resolution<scent_domain::TypeId>) -> String {
    match type_reference {
        scent_domain::Resolution::Resolved(id) => id.as_str().to_owned(),
        scent_domain::Resolution::Unresolved(reference) => reference.spelling.clone(),
    }
}
