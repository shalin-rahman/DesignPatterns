//! Inappropriate Intimacy (Couplers). Flags a pair of types that call each
//! other's methods and read each other's fields heavily in both
//! directions — real bidirectional coupling read straight from the
//! dependency graph, not inferred from naming or proximity.

use std::collections::{BTreeMap, HashMap};

use scent_domain::{AnalysisScope, FieldId, MethodId, TypeId};
use scent_graph::{DependencyKind, EntityRef};

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.INAPPROPRIATE_INTIMACY.thresholds]`); these are the built-in
/// defaults.
pub struct InappropriateIntimacy {
    /// Total Calls/ReadsField edges crossing between the two types, counted
    /// in both directions.
    pub cross_edge_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "INAPPROPRIATE_INTIMACY";
const CROSS_EDGE_THRESHOLD: f64 = 5.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for InappropriateIntimacy {
    fn default() -> Self {
        Self {
            cross_edge_threshold: CROSS_EDGE_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for InappropriateIntimacy {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Inappropriate Intimacy"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Type
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
        let owner_type = |entity: &EntityRef| -> Option<TypeId> {
            match entity {
                EntityRef::Method(id) => owner_of_method.get(id).map(|t| (*t).clone()),
                EntityRef::Field(id) => owner_of_field.get(id).map(|t| (*t).clone()),
                EntityRef::Type(_) | EntityRef::Property(_) => None,
            }
        };

        let mut cross_counts: BTreeMap<(TypeId, TypeId), u32> = BTreeMap::new();
        for edge in ctx.graph.edges() {
            if !matches!(
                edge.kind,
                DependencyKind::Calls | DependencyKind::ReadsField
            ) {
                continue;
            }
            let (Some(from_type), Some(to_type)) = (owner_type(&edge.from), owner_type(&edge.to))
            else {
                continue;
            };
            if from_type == to_type {
                continue;
            }
            let key = if from_type < to_type {
                (from_type, to_type)
            } else {
                (to_type, from_type)
            };
            *cross_counts.entry(key).or_insert(0) += 1;
        }

        let mut findings = Vec::new();
        for ((left, right), count) in cross_counts {
            let observations = [Observation {
                metric_key: "cross_type_edge_count",
                observed_value: f64::from(count),
                threshold: self.cross_edge_threshold,
                weight: 1.0,
                explain: |value, threshold| {
                    format!("{value} call/field-read edges cross between the two types (threshold {threshold})")
                },
            }];
            let (evidence, confidence) = build_evidence(&observations);
            if confidence < self.min_confidence {
                continue;
            }
            let severity = severity_from_observations(&observations);

            let Some(left_type) = ctx.project.types.iter().find(|t| t.id == left) else {
                continue;
            };
            let entity_id = format!("{}<->{}", left.as_str(), right.as_str());
            findings.push(Finding {
                fingerprint: FindingFingerprint::new(
                    RULE_ID,
                    &entity_id,
                    left_type.location.path.as_str(),
                ),
                rule_id: RULE_ID,
                rule_name: self.name(),
                scope: AnalysisScope::Type,
                severity,
                confidence,
                location: left_type.location.clone(),
                entity_id,
                evidence,
            });
        }
        findings
    }
}
