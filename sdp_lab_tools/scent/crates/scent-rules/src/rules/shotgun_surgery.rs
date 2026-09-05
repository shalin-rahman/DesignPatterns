//! Shotgun Surgery (Change Preventers). Flags a pair of entities that
//! repeatedly change together across distinct commits — a real, git-backed
//! signal static syntax alone cannot see (`docs/prompt.md` §14/§24).
//! Produces no findings without git history (`ctx.history` is `None`), and
//! never flags from a single commit — [`MIN_COMMITS`] enforces the spec's
//! own caution: "do not flag Shotgun Surgery because of one unusual commit."

use scent_domain::AnalysisScope;
use scent_graph::EntityRef;

use crate::evidence::{build_evidence, severity_from_observations, Observation};
use crate::model::{Finding, FindingFingerprint};
use crate::registry::{AnalysisContext, Rule};

/// Thresholds are configurable via `smell_detector.toml`
/// (`[rules.SHOTGUN_SURGERY.thresholds]`); these are the built-in
/// defaults.
pub struct ShotgunSurgery {
    pub min_commits: f64,
    pub min_confidence: f32,
}

const RULE_ID: &str = "SHOTGUN_SURGERY";
const MIN_COMMITS: f64 = 3.0;
const MIN_CONFIDENCE: f32 = 0.5;

impl Default for ShotgunSurgery {
    fn default() -> Self {
        Self {
            min_commits: MIN_COMMITS,
            min_confidence: MIN_CONFIDENCE,
        }
    }
}

impl Rule for ShotgunSurgery {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn name(&self) -> &'static str {
        "Shotgun Surgery"
    }

    fn scope(&self) -> AnalysisScope {
        AnalysisScope::Project
    }

    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let Some(history) = ctx.history else {
            return vec![];
        };
        let mut findings = Vec::new();
        for edge in history.cochange.edges() {
            let commit_count = f64::from(u32::try_from(edge.commit_count()).unwrap_or(u32::MAX));
            let observations = [Observation {
                metric_key: "cochange_commit_count",
                observed_value: commit_count,
                threshold: self.min_commits,
                weight: 1.0,
                explain: |value, threshold| {
                    format!("changed together in {value} commits (threshold {threshold})")
                },
            }];
            let (evidence, confidence) = build_evidence(&observations);
            if confidence < self.min_confidence {
                continue;
            }
            let severity = severity_from_observations(&observations);
            for entity in [&edge.a, &edge.b] {
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
                    evidence: evidence.clone(),
                });
            }
        }
        findings
    }
}

/// Every entity kind this rule can encounter carries its own
/// `SourceLocation` fact already; this just looks it up by ID rather than
/// inventing a new lookup path.
pub(crate) fn entity_reference(
    ctx: &AnalysisContext,
    entity: &EntityRef,
) -> Option<(String, scent_domain::SourceLocation)> {
    match entity {
        EntityRef::Type(id) => ctx
            .project
            .types
            .iter()
            .find(|item| &item.id == id)
            .map(|item| (id.as_str().to_owned(), item.location.clone())),
        EntityRef::Method(id) => ctx
            .project
            .methods
            .iter()
            .find(|item| &item.id == id)
            .map(|item| (id.as_str().to_owned(), item.location.clone())),
        EntityRef::Field(id) => ctx
            .project
            .fields
            .iter()
            .find(|item| &item.id == id)
            .map(|item| (id.as_str().to_owned(), item.location.clone())),
        EntityRef::Property(id) => ctx
            .project
            .properties
            .iter()
            .find(|item| &item.id == id)
            .map(|item| (id.as_str().to_owned(), item.location.clone())),
    }
}
