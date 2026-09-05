//! Large Class (Bloaters): size (LOC, method count, field count) combined
//! with cohesion (LCOM4) and coupling (CBO) evidence.

use scent_domain::AnalysisScope;
use scent_graph::EntityRef;
use scent_metrics::MetricKind;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.LARGE_CLASS.thresholds]`); these are the built-in defaults.
pub struct LargeClass {
    pub loc_threshold: f64,
    pub method_count_threshold: f64,
    pub field_count_threshold: f64,
    pub lcom4_threshold: f64,
    pub cbo_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "LARGE_CLASS";
const LOC_THRESHOLD: f64 = 200.0;
const METHOD_COUNT_THRESHOLD: f64 = 20.0;
const FIELD_COUNT_THRESHOLD: f64 = 15.0;
const LCOM4_THRESHOLD: f64 = 1.0;
const CBO_THRESHOLD: f64 = 10.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for LargeClass {
    fn default() -> Self {
        Self {
            loc_threshold: LOC_THRESHOLD,
            method_count_threshold: METHOD_COUNT_THRESHOLD,
            field_count_threshold: FIELD_COUNT_THRESHOLD,
            lcom4_threshold: LCOM4_THRESHOLD,
            cbo_threshold: CBO_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for LargeClass {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Large Class"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Type
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for type_ir in &ctx.project.types {
            let entity = EntityRef::Type(type_ir.id.clone());
            let loc = ctx
                .metrics
                .get(&entity, MetricKind::Loc)
                .map_or(0.0, |v| v.value);
            let lcom4 = ctx
                .metrics
                .get(&entity, MetricKind::Lcom4)
                .map_or(0.0, |v| v.value);
            let cbo = ctx
                .metrics
                .get(&entity, MetricKind::Cbo)
                .map_or(0.0, |v| v.value);
            let method_count = f64::from(
                u32::try_from(
                    ctx.project
                        .methods
                        .iter()
                        .filter(|method| method.owner_type == type_ir.id)
                        .count(),
                )
                .unwrap_or(u32::MAX),
            );
            let field_count = f64::from(
                u32::try_from(
                    ctx.project
                        .fields
                        .iter()
                        .filter(|field| field.owner_type == type_ir.id)
                        .count(),
                )
                .unwrap_or(u32::MAX),
            );

            let observations = [
                Observation {
                    metric_key: "loc",
                    observed_value: loc,
                    threshold: self.loc_threshold,
                    weight: 0.3,
                    explain: |value, threshold| {
                        format!("{value} physical lines (threshold {threshold})")
                    },
                },
                Observation {
                    metric_key: "method_count",
                    observed_value: method_count,
                    threshold: self.method_count_threshold,
                    weight: 0.25,
                    explain: |value, threshold| format!("{value} methods (threshold {threshold})"),
                },
                Observation {
                    metric_key: "field_count",
                    observed_value: field_count,
                    threshold: self.field_count_threshold,
                    weight: 0.15,
                    explain: |value, threshold| format!("{value} fields (threshold {threshold})"),
                },
                Observation {
                    metric_key: "lcom4",
                    observed_value: lcom4,
                    threshold: self.lcom4_threshold,
                    weight: 0.15,
                    explain: |value, threshold| {
                        format!("LCOM4 {value} connected component(s) (cohesive is {threshold})")
                    },
                },
                Observation {
                    metric_key: "cbo",
                    observed_value: cbo,
                    threshold: self.cbo_threshold,
                    weight: 0.15,
                    explain: |value, threshold| {
                        format!("coupled to {value} other type(s) (threshold {threshold})")
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
                    type_ir.id.as_str(),
                    type_ir.location.path.as_str(),
                ),
                rule_id: RULE_ID,
                rule_name: self.name(),
                scope: AnalysisScope::Type,
                severity: severity_from_observations(&observations),
                confidence,
                location: type_ir.location.clone(),
                entity_id: type_ir.id.as_str().to_owned(),
                evidence,
            });
        }
        findings
    }
}
