//! Speculative Generality (Dispensables). Flags an interface that no
//! resolved `Implements` edge ever points to — an abstraction with no
//! (intra-project) reason to exist yet. Conservative by construction: an
//! interface implemented from outside this project, or not yet resolved,
//! is not flagged, since that would require guessing about code this
//! analyzer cannot see.

use scent_domain::AnalysisScope;
use scent_graph::{DependencyKind, EntityRef};
use scent_ir::TypeKind;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.SPECULATIVE_GENERALITY.thresholds]`); these are the built-in
/// defaults.
pub struct SpeculativeGenerality {
    pub observed_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "SPECULATIVE_GENERALITY";
/// `observed = 1 / (implementor_count + 1)` maps "no implementors" to 1.0
/// and "one implementor" to 0.5; the shared ratio curve then makes zero or
/// one implementor the suspicious end and two or more comfortably safe.
const OBSERVED_THRESHOLD: f64 = 0.5;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for SpeculativeGenerality {
    fn default() -> Self {
        Self {
            observed_threshold: OBSERVED_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for SpeculativeGenerality {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Speculative Generality"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Type
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for type_ir in &ctx.project.types {
            if type_ir.kind != TypeKind::Interface {
                continue;
            }
            let implementor_count = ctx
                .graph
                .incoming(&EntityRef::Type(type_ir.id.clone()))
                .iter()
                .map(|index| &ctx.graph.edges()[*index])
                .filter(|edge| edge.kind == DependencyKind::Implements)
                .count();
            let observed =
                1.0 / (f64::from(u32::try_from(implementor_count).unwrap_or(u32::MAX)) + 1.0);

            let observations = [Observation {
                metric_key: "inverse_implementor_count",
                observed_value: observed,
                threshold: self.observed_threshold,
                weight: 1.0,
                explain: |value, _threshold| {
                    let implementors = (1.0 / value - 1.0).round();
                    format!("implemented by {implementors} type(s) in this project")
                },
            }];
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
