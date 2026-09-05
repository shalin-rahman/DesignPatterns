//! Refused Bequest (OO Abusers). A derived type is never classified as
//! refusing its parent purely from the inheritance edge — that would be
//! guessing from structure alone (`docs/prompt.md` §24). Evidence instead
//! comes from a same-named method on the derived type that looks trivially
//! unimplemented: no decision points and either an empty body or an
//! explicit `NotImplementedException`/`NotSupportedException` throw.

use scent_domain::AnalysisScope;
use scent_graph::{DependencyKind, EntityRef};
use scent_metrics::MetricKind;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.REFUSED_BEQUEST.thresholds]`); these are the built-in defaults.
pub struct RefusedBequest {
    pub refused_override_threshold: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "REFUSED_BEQUEST";
const REFUSED_OVERRIDE_THRESHOLD: f64 = 1.0;
const MIN_CONFIDENCE: f32 = 0.5;
const REFUSAL_EXCEPTION_SPELLINGS: [&str; 2] = ["NotImplementedException", "NotSupportedException"];

impl Default for RefusedBequest {
    fn default() -> Self {
        Self {
            refused_override_threshold: REFUSED_OVERRIDE_THRESHOLD,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for RefusedBequest {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Refused Bequest"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Type
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for type_ir in &ctx.project.types {
            let base_ids: Vec<_> = ctx
                .graph
                .outgoing(&EntityRef::Type(type_ir.id.clone()))
                .iter()
                .map(|index| &ctx.graph.edges()[*index])
                .filter(|edge| edge.kind == DependencyKind::Inherits)
                .filter_map(|edge| match &edge.to {
                    EntityRef::Type(id) => Some(id.clone()),
                    _ => None,
                })
                .collect();
            if base_ids.is_empty() {
                continue;
            }

            let base_method_names: std::collections::HashSet<&str> = ctx
                .project
                .methods
                .iter()
                .filter(|method| base_ids.contains(&method.owner_type))
                .map(|method| method.name.as_str())
                .collect();

            let refused_count = ctx
                .project
                .methods
                .iter()
                .filter(|method| method.owner_type == type_ir.id)
                .filter(|method| base_method_names.contains(method.name.as_str()))
                .filter(|method| looks_refused(ctx, method))
                .count();
            if refused_count == 0 {
                continue;
            }

            let observations = [Observation {
                metric_key: "refused_override_count",
                observed_value: f64::from(u32::try_from(refused_count).unwrap_or(u32::MAX)),
                threshold: self.refused_override_threshold,
                weight: 1.0,
                explain: |value, threshold| {
                    format!("{value} same-named base method(s) look unimplemented here (threshold {threshold})")
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

/// Shared with `principles::assess_isp_risks`, which runs the same
/// "looks trivially unimplemented" check against interface members instead
/// of base-class members.
pub(crate) fn looks_refused(ctx: &AnalysisContext, method: &scent_ir::MethodIR) -> bool {
    let throws_refusal_exception = method.instantiations.iter().any(|instantiation| {
        matches!(
            &instantiation.target,
            scent_domain::Resolution::Unresolved(reference)
                if REFUSAL_EXCEPTION_SPELLINGS.contains(&reference.spelling.as_str())
        )
    });
    if throws_refusal_exception {
        return true;
    }
    let trivial_body = ctx
        .metrics
        .get(
            &EntityRef::Method(method.id.clone()),
            MetricKind::CyclomaticComplexity,
        )
        .is_some_and(|value| value.value <= 1.0);
    trivial_body && method.calls.is_empty() && method.field_accesses.is_empty()
}
