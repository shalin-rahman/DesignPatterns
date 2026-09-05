//! Duplicated Code (Dispensables). Flags methods that are structurally
//! identical, after normalizing identifiers and literals, to at least one
//! other method in the project (`docs/prompt.md` §24).
//!
//! Scope, stated plainly (see `scent_parser::CloneSignature`): this finds
//! *exact* structural duplicates at whole-method granularity, not
//! partial/fuzzy overlaps below 100% structural match and not duplicated
//! sub-method-sized blocks. Tiny methods are excluded by a minimum
//! statement count so a two-line getter is never reported as a clone.

use std::collections::HashMap;

use scent_domain::AnalysisScope;
use scent_graph::EntityRef;
use scent_metrics::MetricKind;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.DUPLICATED_CODE.thresholds]`); these are the built-in defaults
/// (matching `docs/prompt.md` §24's stated minimum: 5 statements, 6 LOC).
pub struct DuplicatedCode {
    pub min_statements: f64,
    pub min_loc: u32,
    pub occurrence_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "DUPLICATED_CODE";
const MIN_STATEMENTS: f64 = 5.0;
const MIN_LOC: u32 = 6;
const OCCURRENCE_THRESHOLD: f64 = 2.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for DuplicatedCode {
    fn default() -> Self {
        Self {
            min_statements: MIN_STATEMENTS,
            min_loc: MIN_LOC,
            occurrence_threshold: OCCURRENCE_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for DuplicatedCode {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Duplicated Code"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Method
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
        for (index, (method_id, signature)) in ctx.clone_signatures.iter().enumerate() {
            if f64::from(signature.statement_count) < self.min_statements {
                continue;
            }
            let loc = ctx
                .metrics
                .get(&EntityRef::Method(method_id.clone()), MetricKind::Loc)
                .map_or(0.0, |value| value.value);
            if loc < f64::from(self.min_loc) {
                continue;
            }
            groups
                .entry(signature.hash.as_str())
                .or_default()
                .push(index);
        }

        let mut findings = Vec::new();
        for indices in groups.values() {
            if indices.len() < 2 {
                continue;
            }
            let occurrence_count = f64::from(u32::try_from(indices.len()).unwrap_or(u32::MAX));
            let statement_count = f64::from(ctx.clone_signatures[indices[0]].1.statement_count);

            let observations = [
                Observation {
                    metric_key: "occurrence_count",
                    observed_value: occurrence_count,
                    threshold: self.occurrence_threshold,
                    weight: 0.5,
                    explain: |value, threshold| {
                        format!("this exact structure appears in {value} methods (threshold {threshold})")
                    },
                },
                Observation {
                    metric_key: "statement_count",
                    observed_value: statement_count,
                    threshold: self.min_statements,
                    weight: 0.5,
                    explain: |value, threshold| {
                        format!("{value} statements duplicated (minimum considered {threshold})")
                    },
                },
            ];
            let (evidence, confidence) = build_evidence(&observations);
            if confidence < self.min_confidence {
                continue;
            }
            let severity = severity_from_observations(&observations);

            for &index in indices {
                let (method_id, _) = &ctx.clone_signatures[index];
                let Some(method) = ctx.project.methods.iter().find(|m| &m.id == method_id) else {
                    continue;
                };
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
