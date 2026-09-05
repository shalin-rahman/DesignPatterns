//! Primitive Obsession (Bloaters). Flags methods whose parameter list leans
//! heavily on C#'s built-in primitive types rather than a domain type.

use scent_domain::{AnalysisScope, Resolution};

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.PRIMITIVE_OBSESSION.thresholds]`); these are the built-in
/// defaults.
pub struct PrimitiveObsession {
    pub primitive_parameter_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "PRIMITIVE_OBSESSION";
const PRIMITIVE_PARAMETER_THRESHOLD: f64 = 3.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for PrimitiveObsession {
    fn default() -> Self {
        Self {
            primitive_parameter_threshold: PRIMITIVE_PARAMETER_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

/// C#'s built-in value/reference type keywords (`predefined_type` in the
/// grammar) plus `string`. A parameter spelled exactly one of these is
/// unambiguously primitive — no inference needed.
const PRIMITIVE_SPELLINGS: [&str; 16] = [
    "bool", "byte", "sbyte", "char", "decimal", "double", "float", "int", "uint", "long", "ulong",
    "short", "ushort", "string", "object", "void",
];

impl Rule for PrimitiveObsession {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Primitive Obsession"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Method
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for method in &ctx.project.methods {
            let primitive_count = f64::from(
                u32::try_from(
                    method
                        .parameters
                        .iter()
                        .filter(|parameter| is_primitive(&parameter.type_reference))
                        .count(),
                )
                .unwrap_or(u32::MAX),
            );
            if primitive_count == 0.0 {
                continue;
            }

            let observations = [Observation {
                metric_key: "primitive_parameter_count",
                observed_value: primitive_count,
                threshold: self.primitive_parameter_threshold,
                weight: 1.0,
                explain: |value, threshold| {
                    format!("{value} primitive-typed parameter(s) (threshold {threshold})")
                },
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

fn is_primitive(type_reference: &Resolution<scent_domain::TypeId>) -> bool {
    match type_reference {
        Resolution::Resolved(_) => false,
        Resolution::Unresolved(reference) => {
            PRIMITIVE_SPELLINGS.contains(&reference.spelling.as_str())
        }
    }
}
