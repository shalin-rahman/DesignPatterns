//! Read-only rule context, the `Rule` contract, and the registry that runs
//! every registered rule and applies suppressions afterward.

use scent_domain::{AnalysisScope, MethodId, SuppressionMap};
use scent_git::GitHistory;
use scent_graph::DependencyGraph;
use scent_ir::ProjectIR;
use scent_metrics::MetricStore;
use scent_parser::{CloneSignature, SwitchShape};

use crate::model::Finding;

/// Read-only inputs available to every rule. Rules must not mutate any of
/// these — they only ever read facts, metrics, and graph edges and produce
/// findings.
pub struct AnalysisContext<'a> {
    pub project: &'a ProjectIR,
    pub graph: &'a DependencyGraph,
    pub metrics: &'a MetricStore,
    pub suppressions: &'a SuppressionMap,
    pub clone_signatures: &'a [(MethodId, CloneSignature)],
    pub switch_shapes: &'a [(MethodId, Vec<SwitchShape>)],
    /// `None` when `root` was not a Git work tree (or `git` itself was
    /// unavailable) — the git-backed rules simply produce no findings in
    /// that case rather than treating a missing repository as an error.
    pub history: Option<&'a GitHistory>,
}

pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn scope(&self) -> AnalysisScope;
    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding>;
}

#[derive(Default)]
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    #[must_use]
    pub fn get(&self, rule_id: &str) -> Option<&dyn Rule> {
        self.rules
            .iter()
            .find(|rule| rule.id() == rule_id)
            .map(AsRef::as_ref)
    }

    #[must_use]
    pub fn all(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Evaluates every registered rule, drops any finding the
    /// [`SuppressionMap`] covers, and returns the rest sorted by
    /// deterministic fingerprint. Suppression is applied here rather than
    /// inside each rule so a rule never needs to parse comments itself.
    #[must_use]
    pub fn evaluate_all(&self, ctx: &AnalysisContext) -> Vec<Finding> {
        let mut findings: Vec<Finding> = self
            .rules
            .iter()
            .flat_map(|rule| rule.evaluate(ctx))
            .filter(|finding| {
                !ctx.suppressions.is_suppressed(
                    &finding.location.path,
                    finding.rule_id,
                    finding.location.range.start.line,
                )
            })
            .collect();
        findings.sort_by_key(|finding| finding.fingerprint.as_key());
        findings
    }
}
