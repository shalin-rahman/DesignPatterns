//! Pattern Advisor (`docs/prompt.md` §29). Pattern recommendations are kept
//! independent of smell detection: this reads the same `SwitchShape` facts
//! the Switch Statements rule uses, not that rule's findings, and only ever
//! names Factory Method when there is real supporting evidence
//! (`distinct_created_types`) for it.
//!
//! Strategy/State candidacy needs a "does behavior depend on internal
//! state" signal this codebase does not compute yet, so neither is offered
//! — `"Do Nothing"` is returned instead, which the spec states explicitly
//! must be a valid recommendation, not a fallback failure.

use scent_domain::MethodId;

use crate::registry::AnalysisContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternCandidate {
    FactoryMethod,
    DoNothing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatternRecommendation {
    pub method_id: MethodId,
    /// Which of the method's switch statements this recommendation is
    /// about, in source-visitation order — mirrors `SWITCH_STATEMENTS`'
    /// own `entity_id` scheme (`{method_id}#switch{ordinal}`).
    pub switch_ordinal: usize,
    pub candidate: PatternCandidate,
    pub explanation: String,
}

const MIN_DISTINCT_CREATED_TYPES: u32 = 2;

/// One recommendation per switch statement found anywhere in the project,
/// regardless of whether `SWITCH_STATEMENTS` itself fired for it — the
/// Pattern Advisor answers a different question (which pattern, if any)
/// than the smell rule (is this switch a problem at all).
#[must_use]
pub fn recommend_patterns(ctx: &AnalysisContext) -> Vec<PatternRecommendation> {
    let mut recommendations = Vec::new();
    for (method_id, shapes) in ctx.switch_shapes {
        for (ordinal, shape) in shapes.iter().enumerate() {
            let (candidate, explanation) = if shape.distinct_created_types
                >= MIN_DISTINCT_CREATED_TYPES
            {
                (
                    PatternCandidate::FactoryMethod,
                    format!(
                        "this switch constructs {} distinct types across its cases; centralizing that construction is a Factory Method candidate",
                        shape.distinct_created_types
                    ),
                )
            } else {
                (
                    PatternCandidate::DoNothing,
                    "no object-creation or behavioral-state evidence supports a specific pattern here".to_owned(),
                )
            };
            recommendations.push(PatternRecommendation {
                method_id: method_id.clone(),
                switch_ordinal: ordinal,
                candidate,
                explanation,
            });
        }
    }
    recommendations
}
