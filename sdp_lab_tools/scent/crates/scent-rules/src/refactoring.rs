//! Refactoring Advisor (`docs/prompt.md` §30). A static, documented mapping
//! from a rule's smell to one of the spec's own named high-level structural
//! refactorings — never a per-instance guess, and never source rewriting:
//! "the advisor should produce high-level structural proposals... do not
//! automatically rewrite production code in early versions."

use crate::model::Finding;

/// Exactly the refactorings `docs/prompt.md` §30 names as examples — this
/// advisor does not introduce new refactoring names beyond that list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refactoring {
    ExtractMethod,
    ExtractClass,
    IntroduceParameterObject,
    MoveMethod,
    MoveField,
    ReplaceConditionalWithPolymorphism,
    ReplaceInheritanceWithDelegation,
    RemoveSpeculativeGenerality,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefactoringRecommendation {
    pub entity_id: String,
    pub rule_id: &'static str,
    pub refactoring: Refactoring,
}

/// `None` means "no confident mapping yet" — a rule missing from this
/// table is skipped rather than guessed. Every rule in
/// `crate::default_registry()` currently has an entry (see
/// `covers_every_registered_rule` in this module's tests); a new rule
/// should add one here too, or that test starts failing on purpose.
fn refactoring_for_rule(rule_id: &str) -> Option<Refactoring> {
    match rule_id {
        "LONG_PARAMETER_LIST" | "DATA_CLUMPS" | "PRIMITIVE_OBSESSION" => {
            Some(Refactoring::IntroduceParameterObject)
        }
        "LONG_METHOD" | "DUPLICATED_CODE" => Some(Refactoring::ExtractMethod),
        "LARGE_CLASS" | "DIVERGENT_CHANGE" => Some(Refactoring::ExtractClass),
        "INAPPROPRIATE_INTIMACY" | "FEATURE_ENVY" => Some(Refactoring::MoveMethod),
        "SHOTGUN_SURGERY" => Some(Refactoring::MoveField),
        "SWITCH_STATEMENTS" => Some(Refactoring::ReplaceConditionalWithPolymorphism),
        "REFUSED_BEQUEST" => Some(Refactoring::ReplaceInheritanceWithDelegation),
        "SPECULATIVE_GENERALITY" => Some(Refactoring::RemoveSpeculativeGenerality),
        _ => None,
    }
}

/// One recommendation per finding with a known mapping.
#[must_use]
pub fn recommend_refactorings(findings: &[Finding]) -> Vec<RefactoringRecommendation> {
    findings
        .iter()
        .filter_map(|finding| {
            refactoring_for_rule(finding.rule_id).map(|refactoring| RefactoringRecommendation {
                entity_id: finding.entity_id.clone(),
                rule_id: finding.rule_id,
                refactoring,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::refactoring_for_rule;
    use crate::default_registry;

    #[test]
    fn covers_every_registered_rule() {
        let registry = default_registry();
        let missing: Vec<&str> = registry
            .all()
            .iter()
            .map(|rule| rule.id())
            .filter(|rule_id| refactoring_for_rule(rule_id).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "these registered rules have no refactoring mapping: {missing:?}"
        );
    }
}
