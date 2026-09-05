//! Switch Statements (Bloaters). Flags an individual `switch` whose case
//! count and branch complexity suggest it may be better expressed with
//! polymorphism (`docs/prompt.md` §24). This rule reports only the smell —
//! it never names a candidate pattern (Strategy/State/Factory Method); that
//! decision belongs to the separate Pattern Advisor, kept independent of
//! smell detection per §29.

use scent_domain::AnalysisScope;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.SWITCH_STATEMENTS.thresholds]`); these are the built-in
/// defaults.
pub struct SwitchStatements {
    pub min_case_count: f64,
    pub branch_complexity_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "SWITCH_STATEMENTS";
const MIN_CASE_COUNT: f64 = 4.0;
const BRANCH_COMPLEXITY_THRESHOLD: f64 = 3.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for SwitchStatements {
    fn default() -> Self {
        Self {
            min_case_count: MIN_CASE_COUNT,
            branch_complexity_threshold: BRANCH_COMPLEXITY_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for SwitchStatements {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Switch Statements"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Method
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (method_id, shapes) in ctx.switch_shapes {
            let Some(method) = ctx.project.methods.iter().find(|m| &m.id == method_id) else {
                continue;
            };
            // Ordinal position within the method, not a line number, is the
            // stable part of this finding's identity — a method's switch
            // statements are recorded in source-visitation order and don't
            // reorder themselves between unrelated edits.
            for (ordinal, shape) in shapes.iter().enumerate() {
                let observations = [
                    Observation {
                        metric_key: "case_count",
                        observed_value: f64::from(shape.case_count),
                        threshold: self.min_case_count,
                        weight: 0.6,
                        explain: |value, threshold| {
                            format!("{value} cases (threshold {threshold})")
                        },
                    },
                    Observation {
                        metric_key: "max_branch_complexity",
                        observed_value: f64::from(shape.max_branch_complexity),
                        threshold: self.branch_complexity_threshold,
                        weight: 0.4,
                        explain: |value, threshold| {
                            format!(
                                "densest case has {value} decision points (threshold {threshold})"
                            )
                        },
                    },
                ];
                let (evidence, confidence) = build_evidence(&observations);
                if confidence < self.min_confidence {
                    continue;
                }
                let severity = severity_from_observations(&observations);
                let entity_id = format!("{}#switch{ordinal}", method_id.as_str());
                findings.push(Finding {
                    fingerprint: FindingFingerprint::new(
                        RULE_ID,
                        &entity_id,
                        method.location.path.as_str(),
                    ),
                    rule_id: RULE_ID,
                    rule_name: self.name(),
                    scope: AnalysisScope::Method,
                    severity,
                    confidence,
                    location: shape.location.clone(),
                    entity_id,
                    evidence,
                });
            }
        }
        findings
    }
}
