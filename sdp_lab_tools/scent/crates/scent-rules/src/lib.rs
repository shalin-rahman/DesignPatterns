//! Evidence-based rule evaluation. A rule reads `AnalysisContext` and
//! produces `Finding`s; it never mutates the IR, graph, metrics, or source.

pub mod evidence;
pub mod model;
pub mod patterns;
pub mod principles;
pub mod refactoring;
pub mod registry;
pub mod rules;

pub use evidence::{build_evidence, normalize_ratio, severity_from_observations, Observation};
pub use model::{EvidenceItem, Finding, FindingFingerprint};
pub use patterns::{recommend_patterns, PatternCandidate, PatternRecommendation};
pub use principles::{assess_principle_risks, EvidenceRef, PrincipleRisk};
pub use refactoring::{recommend_refactorings, Refactoring, RefactoringRecommendation};
pub use registry::{AnalysisContext, Rule, RuleRegistry};
pub use rules::{
    DataClumps, DivergentChange, DuplicatedCode, FeatureEnvy, InappropriateIntimacy, LargeClass,
    LongMethod, LongParameterList, PrimitiveObsession, RefusedBequest, ShotgunSurgery,
    SpeculativeGenerality, SwitchStatements,
};

use scent_config::ScentConfig;

/// The full Phase 1 rule set at its built-in default thresholds — every
/// rule enabled, no `smell_detector.toml` overrides applied. Most of the
/// ~15 existing tests in this workspace call this directly; production
/// code should prefer [`build_registry`], which honors project config.
#[must_use]
pub fn default_registry() -> RuleRegistry {
    build_registry(&ScentConfig::default())
}

/// Builds the rule set from `config`: a rule with `enabled = false` in
/// `smell_detector.toml` is skipped entirely (never even constructed), and
/// every other rule reads its own threshold overrides by rule id — falling
/// back to the built-in default (its own `Default` impl) for any threshold
/// the config doesn't mention. The Git-history rules (`DivergentChange`,
/// `ShotgunSurgery`) are configured the same way as every other rule; they
/// already produce no findings without git history regardless.
#[must_use]
pub fn build_registry(config: &ScentConfig) -> RuleRegistry {
    let mut registry = RuleRegistry::new();
    register_bloaters(config, &mut registry);
    register_couplers(config, &mut registry);
    register_dispensables_and_history(config, &mut registry);
    registry
}

fn register_bloaters(config: &ScentConfig, registry: &mut RuleRegistry) {
    if config.rule_enabled("LONG_PARAMETER_LIST") {
        let defaults = LongParameterList::default();
        registry.register(Box::new(LongParameterList {
            parameter_count_threshold: config.threshold(
                "LONG_PARAMETER_LIST",
                "parameter_count",
                defaults.parameter_count_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("LONG_METHOD") {
        let defaults = LongMethod::default();
        registry.register(Box::new(LongMethod {
            loc_threshold: config.threshold("LONG_METHOD", "loc", defaults.loc_threshold),
            cc_threshold: config.threshold(
                "LONG_METHOD",
                "cyclomatic_complexity",
                defaults.cc_threshold,
            ),
            nesting_threshold: config.threshold(
                "LONG_METHOD",
                "nesting_depth",
                defaults.nesting_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("LARGE_CLASS") {
        let defaults = LargeClass::default();
        registry.register(Box::new(LargeClass {
            loc_threshold: config.threshold("LARGE_CLASS", "loc", defaults.loc_threshold),
            method_count_threshold: config.threshold(
                "LARGE_CLASS",
                "method_count",
                defaults.method_count_threshold,
            ),
            field_count_threshold: config.threshold(
                "LARGE_CLASS",
                "field_count",
                defaults.field_count_threshold,
            ),
            lcom4_threshold: config.threshold("LARGE_CLASS", "lcom4", defaults.lcom4_threshold),
            cbo_threshold: config.threshold("LARGE_CLASS", "cbo", defaults.cbo_threshold),
            ..defaults
        }));
    }
    if config.rule_enabled("DATA_CLUMPS") {
        let defaults = DataClumps::default();
        registry.register(Box::new(DataClumps {
            occurrence_threshold: config.threshold(
                "DATA_CLUMPS",
                "occurrence_count",
                defaults.occurrence_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("PRIMITIVE_OBSESSION") {
        let defaults = PrimitiveObsession::default();
        registry.register(Box::new(PrimitiveObsession {
            primitive_parameter_threshold: config.threshold(
                "PRIMITIVE_OBSESSION",
                "primitive_parameter_count",
                defaults.primitive_parameter_threshold,
            ),
            ..defaults
        }));
    }
}

fn register_couplers(config: &ScentConfig, registry: &mut RuleRegistry) {
    if config.rule_enabled("INAPPROPRIATE_INTIMACY") {
        let defaults = InappropriateIntimacy::default();
        registry.register(Box::new(InappropriateIntimacy {
            cross_edge_threshold: config.threshold(
                "INAPPROPRIATE_INTIMACY",
                "cross_type_edge_count",
                defaults.cross_edge_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("REFUSED_BEQUEST") {
        let defaults = RefusedBequest::default();
        registry.register(Box::new(RefusedBequest {
            refused_override_threshold: config.threshold(
                "REFUSED_BEQUEST",
                "refused_override_count",
                defaults.refused_override_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("SPECULATIVE_GENERALITY") {
        let defaults = SpeculativeGenerality::default();
        registry.register(Box::new(SpeculativeGenerality {
            observed_threshold: config.threshold(
                "SPECULATIVE_GENERALITY",
                "inverse_implementor_count",
                defaults.observed_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("FEATURE_ENVY") {
        let defaults = FeatureEnvy::default();
        registry.register(Box::new(FeatureEnvy {
            foreign_ratio_threshold: config.threshold(
                "FEATURE_ENVY",
                "foreign_access_ratio",
                defaults.foreign_ratio_threshold,
            ),
            ..defaults
        }));
    }
}

fn register_dispensables_and_history(config: &ScentConfig, registry: &mut RuleRegistry) {
    if config.rule_enabled("DUPLICATED_CODE") {
        let defaults = DuplicatedCode::default();
        registry.register(Box::new(DuplicatedCode {
            min_statements: config.threshold(
                "DUPLICATED_CODE",
                "min_statements",
                defaults.min_statements,
            ),
            min_loc: clamp_to_u32(config.threshold(
                "DUPLICATED_CODE",
                "min_loc",
                f64::from(defaults.min_loc),
            )),
            occurrence_threshold: config.threshold(
                "DUPLICATED_CODE",
                "occurrence_count",
                defaults.occurrence_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("SWITCH_STATEMENTS") {
        let defaults = SwitchStatements::default();
        registry.register(Box::new(SwitchStatements {
            min_case_count: config.threshold(
                "SWITCH_STATEMENTS",
                "case_count",
                defaults.min_case_count,
            ),
            branch_complexity_threshold: config.threshold(
                "SWITCH_STATEMENTS",
                "max_branch_complexity",
                defaults.branch_complexity_threshold,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("DIVERGENT_CHANGE") {
        let defaults = DivergentChange::default();
        registry.register(Box::new(DivergentChange {
            min_total_commits: config.threshold(
                "DIVERGENT_CHANGE",
                "total_commits",
                defaults.min_total_commits,
            ),
            min_distinct_contexts: config.threshold(
                "DIVERGENT_CHANGE",
                "distinct_change_contexts",
                defaults.min_distinct_contexts,
            ),
            ..defaults
        }));
    }
    if config.rule_enabled("SHOTGUN_SURGERY") {
        let defaults = ShotgunSurgery::default();
        registry.register(Box::new(ShotgunSurgery {
            min_commits: config.threshold(
                "SHOTGUN_SURGERY",
                "cochange_commit_count",
                defaults.min_commits,
            ),
            ..defaults
        }));
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn clamp_to_u32(value: f64) -> u32 {
    if value <= 0.0 {
        0
    } else if value >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        value as u32
    }
}
