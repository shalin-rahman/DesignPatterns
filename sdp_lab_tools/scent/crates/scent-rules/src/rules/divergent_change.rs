//! Divergent Change (Change Preventers). Flags an entity repeatedly changed
//! across many commits alongside many *different* other entities — a proxy
//! for "distinct change contexts" per `docs/prompt.md` §14/§24: an entity
//! tightly coupled to one consistent partner is not divergent, but one that
//! rarely shares the same partner twice likely serves several unrelated
//! responsibilities. Produces no findings without git history.

use scent_domain::AnalysisScope;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};
use crate::rules::shotgun_surgery::entity_reference;

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.DIVERGENT_CHANGE.thresholds]`); these are the built-in
/// defaults.
pub struct DivergentChange {
    pub min_total_commits: f64,
    pub min_distinct_contexts: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "DIVERGENT_CHANGE";
const MIN_TOTAL_COMMITS: f64 = 3.0;
const MIN_DISTINCT_CONTEXTS: f64 = 2.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for DivergentChange {
    fn default() -> Self {
        Self {
            min_total_commits: MIN_TOTAL_COMMITS,
            min_distinct_contexts: MIN_DISTINCT_CONTEXTS,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for DivergentChange {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Divergent Change"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Project
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let Some(history) = ctx.history else {
            return vec![];
        };
        let mut findings = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for edge in history.cochange.edges() {
            for entity in [&edge.a, &edge.b] {
                if !seen.insert(entity.clone()) {
                    continue;
                }
                let total_commits = history.cochange.changes_for(entity).len();
                let total_commits_f64 = f64::from(u32::try_from(total_commits).unwrap_or(u32::MAX));
                if total_commits_f64 < self.min_total_commits {
                    continue;
                }
                let partners = history.cochange.partners_of(entity);
                // A partner present in *every* one of this entity's commits
                // is a consistent collaborator, not a distinct context; only
                // a partner missing from at least one commit counts as
                // evidence the entity changes for more than one reason.
                let distinct_contexts = partners
                    .iter()
                    .filter(|(_, partner_edge)| partner_edge.commit_count() < total_commits)
                    .count();
                let distinct_contexts_f64 =
                    f64::from(u32::try_from(distinct_contexts).unwrap_or(u32::MAX));

                let observations = [
                    Observation {
                        metric_key: "total_commits",
                        observed_value: total_commits_f64,
                        threshold: self.min_total_commits,
                        weight: 0.5,
                        explain: |value, threshold| {
                            format!("changed in {value} commits (threshold {threshold})")
                        },
                    },
                    Observation {
                        metric_key: "distinct_change_contexts",
                        observed_value: distinct_contexts_f64,
                        threshold: self.min_distinct_contexts,
                        weight: 0.5,
                        explain: |value, threshold| {
                            format!(
                                "{value} co-changing partners do not appear in every commit (threshold {threshold}), suggesting distinct change contexts"
                            )
                        },
                    },
                ];
                let (evidence, confidence) = build_evidence(&observations);
                if confidence < self.min_confidence {
                    continue;
                }
                let severity = severity_from_observations(&observations);
                let Some((entity_id, location)) = entity_reference(ctx, entity) else {
                    continue;
                };
                findings.push(Finding {
                    fingerprint: FindingFingerprint::new(
                        RULE_ID,
                        &entity_id,
                        location.path.as_str(),
                    ),
                    rule_id: RULE_ID,
                    rule_name: self.name(),
                    scope: AnalysisScope::Project,
                    severity,
                    confidence,
                    location,
                    entity_id,
                    evidence,
                });
            }
        }
        findings
    }
}
