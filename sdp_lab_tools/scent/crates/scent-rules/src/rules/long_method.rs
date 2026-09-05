//! Long Method (Bloaters): LOC, cyclomatic complexity, and nesting depth
//! together, not any single metric alone.

use scent_domain::AnalysisScope;
use scent_graph::EntityRef;
use scent_metrics::MetricKind;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.LONG_METHOD.thresholds]`); these are the built-in defaults.
pub struct LongMethod {
    pub loc_threshold: f64,
    pub cc_threshold: f64,
    pub nesting_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "LONG_METHOD";
const LOC_THRESHOLD: f64 = 30.0;
const CC_THRESHOLD: f64 = 10.0;
const NESTING_THRESHOLD: f64 = 4.0;
/// A finding needs at least this much combined evidence to be worth
/// reporting; no single metric alone can reach this on its own with the
/// weights below, so this is inherently a multi-variable gate.
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for LongMethod {
    fn default() -> Self {
        Self {
            loc_threshold: LOC_THRESHOLD,
            cc_threshold: CC_THRESHOLD,
            nesting_threshold: NESTING_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for LongMethod {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Long Method"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Method
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for method in &ctx.project.methods {
            let entity = EntityRef::Method(method.id.clone());
            let loc = ctx
                .metrics
                .get(&entity, MetricKind::Loc)
                .map_or(0.0, |v| v.value);
            let cc = ctx
                .metrics
                .get(&entity, MetricKind::CyclomaticComplexity)
                .map_or(0.0, |v| v.value);
            let nesting = ctx
                .metrics
                .get(&entity, MetricKind::NestingDepth)
                .map_or(0.0, |v| v.value);

            let observations = [
                Observation {
                    metric_key: "loc",
                    observed_value: loc,
                    threshold: self.loc_threshold,
                    weight: 0.4,
                    explain: |value, threshold| {
                        format!("{value} physical lines (threshold {threshold})")
                    },
                },
                Observation {
                    metric_key: "cyclomatic_complexity",
                    observed_value: cc,
                    threshold: self.cc_threshold,
                    weight: 0.4,
                    explain: |value, threshold| {
                        format!("cyclomatic complexity {value} (threshold {threshold})")
                    },
                },
                Observation {
                    metric_key: "nesting_depth",
                    observed_value: nesting,
                    threshold: self.nesting_threshold,
                    weight: 0.2,
                    explain: |value, threshold| {
                        format!("max nesting depth {value} (threshold {threshold})")
                    },
                },
            ];
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
