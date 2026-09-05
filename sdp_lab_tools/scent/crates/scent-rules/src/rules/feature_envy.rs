//! Feature Envy (Couplers). Compares how much a method accesses members of
//! its own type versus members of another, resolved, project type. Only
//! resolved accesses count on either side of the ratio — an access this
//! analyzer could not resolve (a local variable's members, a chained or
//! complex receiver) is excluded rather than guessed onto either side.

use std::collections::HashMap;

use scent_domain::{FieldId, MethodId, Resolution, TypeId};

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.FEATURE_ENVY.thresholds]`); these are the built-in defaults.
pub struct FeatureEnvy {
    /// Below this many resolved accesses total, the ratio is too noisy to
    /// mean anything (a method with one resolved foreign call is not
    /// "envious").
    pub min_resolved_accesses: usize,
    pub foreign_ratio_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "FEATURE_ENVY";
const MIN_RESOLVED_ACCESSES: usize = 3;
const FOREIGN_RATIO_THRESHOLD: f64 = 0.5;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for FeatureEnvy {
    fn default() -> Self {
        Self {
            min_resolved_accesses: MIN_RESOLVED_ACCESSES,
            foreign_ratio_threshold: FOREIGN_RATIO_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for FeatureEnvy {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Feature Envy"
    }

    fn scope(&self) -> scent_domain::AnalysisScope {
        scent_domain::AnalysisScope::Method
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let owner_of_method: HashMap<&MethodId, &TypeId> = ctx
            .project
            .methods
            .iter()
            .map(|method| (&method.id, &method.owner_type))
            .collect();
        let owner_of_field: HashMap<&FieldId, &TypeId> = ctx
            .project
            .fields
            .iter()
            .map(|field| (&field.id, &field.owner_type))
            .collect();

        let mut findings = Vec::new();
        for method in &ctx.project.methods {
            let mut local = 0usize;
            let mut foreign = 0usize;

            for call in &method.calls {
                if let Resolution::Resolved(target) = &call.target {
                    match owner_of_method.get(target) {
                        Some(owner) if **owner == method.owner_type => local += 1,
                        Some(_) => foreign += 1,
                        None => {}
                    }
                }
            }
            for access in &method.field_accesses {
                if let Resolution::Resolved(scent_ir::MemberTarget::Field(target)) = &access.target
                {
                    match owner_of_field.get(target) {
                        Some(owner) if **owner == method.owner_type => local += 1,
                        Some(_) => foreign += 1,
                        None => {}
                    }
                }
            }

            let total = local + foreign;
            if total < self.min_resolved_accesses {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let foreign_ratio = foreign as f64 / total as f64;

            let observations = [Observation {
                metric_key: "foreign_access_ratio",
                observed_value: foreign_ratio,
                threshold: self.foreign_ratio_threshold,
                weight: 1.0,
                explain: |value, _threshold| {
                    format!(
                        "{:.0}% of this method's resolved accesses land on another type",
                        value * 100.0
                    )
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
                scope: scent_domain::AnalysisScope::Method,
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
