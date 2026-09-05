//! Principle Risk Engine (`docs/prompt.md` §25). Aggregates existing
//! findings and facts into a risk assessment per design principle, using
//! the spec's own hedged wording ("High X risk" / "evidence suggests...")
//! rather than "X violated" — no single rule here is a formally
//! deterministic proof of a principle violation.
//!
//! `docs/prompt.md` never gives OCP/ISP/DIP/Law of Demeter a dedicated
//! evidence section the way DRY/KISS/YAGNI each get one, so each is
//! grounded here in whatever real, already-extracted (or cheaply, honestly
//! addable) signal actually supports it — never a proxy that could
//! systematically misattribute:
//! - **OCP** reuses the `SWITCH_STATEMENTS` finding: a large
//!   type-discriminating switch is the textbook "must modify to extend"
//!   shape (`prompt.md`'s own Switch Statements evidence already names
//!   "type-code discrimination").
//! - **DIP** reads the dependency graph directly: a type whose methods'
//!   `Creates`/`AcceptsParameter`/`Returns`/`UsesType` edges skew heavily
//!   toward concrete classes rather than interfaces is DIP risk.
//! - **ISP** reuses `RefusedBequest`'s exact "looks trivially unimplemented"
//!   check, applied to `Implements` edges instead of `Inherits` — a
//!   same-named interface member an implementor stubs out or throws from is
//!   the textbook "this interface is too fat for me" shape.
//! - **Law of Demeter** scans real `receiver_chain_depth` facts (a pure
//!   syntactic count, no resolution needed) for chains of depth 2+ —
//!   "talking to a friend of a friend."

use std::collections::BTreeMap;

use scent_domain::{Principle, RiskLevel};
use scent_graph::{DependencyKind, EntityRef};
use scent_ir::TypeKind;

use crate::model::Finding;
use crate::registry::AnalysisContext;
use crate::rules::refused_bequest::looks_refused;

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceRef {
    pub rule_id: &'static str,
    pub entity_id: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrincipleRisk {
    pub principle: Principle,
    pub risk: RiskLevel,
    pub confidence: f32,
    pub evidence: Vec<EvidenceRef>,
    pub explanation: String,
}

const FINDING_BASED_PRINCIPLES: [Principle; 6] = [
    Principle::Srp,
    Principle::Lsp,
    Principle::Dry,
    Principle::Kiss,
    Principle::Yagni,
    Principle::Ocp,
];

/// Every rule id whose finding is real, existing evidence for `principle`.
/// An empty slice means `principle` is assessed some other way (or not at
/// all) rather than from findings directly.
fn contributing_rule_ids(principle: Principle) -> &'static [&'static str] {
    match principle {
        Principle::Srp => &["LARGE_CLASS", "FEATURE_ENVY"],
        Principle::Lsp => &["REFUSED_BEQUEST"],
        Principle::Dry => &["DUPLICATED_CODE", "DATA_CLUMPS"],
        Principle::Kiss => &["LONG_METHOD", "LARGE_CLASS", "SWITCH_STATEMENTS"],
        Principle::Yagni => &["SPECULATIVE_GENERALITY"],
        Principle::Ocp => &["SWITCH_STATEMENTS"],
        Principle::Isp | Principle::Dip | Principle::LawOfDemeter => &[],
    }
}

fn principle_code(principle: Principle) -> &'static str {
    match principle {
        Principle::Srp => "SRP",
        Principle::Ocp => "OCP",
        Principle::Lsp => "LSP",
        Principle::Isp => "ISP",
        Principle::Dip => "DIP",
        Principle::Dry => "DRY",
        Principle::Kiss => "KISS",
        Principle::Yagni => "YAGNI",
        Principle::LawOfDemeter => "Law of Demeter",
    }
}

fn risk_word(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Low => "Low",
        RiskLevel::Medium => "Medium",
        RiskLevel::High => "High",
    }
}

fn risk_level_from_confidence(confidence: f32) -> RiskLevel {
    if confidence >= 0.75 {
        RiskLevel::High
    } else if confidence >= 0.5 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

fn explanation_for(principle: Principle, risk: RiskLevel) -> String {
    format!(
        "{} {} risk: evidence suggests {}",
        risk_word(risk),
        principle_code(principle),
        principle_hint(principle)
    )
}

fn principle_hint(principle: Principle) -> &'static str {
    match principle {
        Principle::Srp => "this entity may carry more than one responsibility",
        Principle::Lsp => "a subtype does not fully honor its base type's contract",
        Principle::Dry => "duplicated logic or a repeated structural pattern",
        Principle::Kiss => "unnecessary complexity relative to what the entity needs to do",
        Principle::Yagni => "speculative generality with no current use",
        Principle::Ocp => {
            "a type-discriminating switch that must be modified, not extended, for a new case"
        }
        Principle::Isp => "an implementor cannot honor this interface's full member set",
        Principle::Dip => "this type depends mostly on concrete classes rather than abstractions",
        Principle::LawOfDemeter => "code reaches through an object to a friend of a friend",
    }
}

/// One [`PrincipleRisk`] per entity per assessed principle with real
/// supporting evidence. Finding-based principles (see
/// [`FINDING_BASED_PRINCIPLES`]) group `findings` by entity; DIP, ISP, and
/// Law of Demeter read `ctx` directly, since none of them has a natural
/// single-`Finding` shape to reuse.
#[must_use]
pub fn assess_principle_risks(ctx: &AnalysisContext, findings: &[Finding]) -> Vec<PrincipleRisk> {
    let mut risks = Vec::new();
    for principle in FINDING_BASED_PRINCIPLES {
        risks.extend(assess_from_findings(principle, findings));
    }
    risks.extend(assess_dip(ctx));
    risks.extend(assess_isp(ctx));
    risks.extend(assess_law_of_demeter(ctx));
    risks
}

fn assess_from_findings(principle: Principle, findings: &[Finding]) -> Vec<PrincipleRisk> {
    let rule_ids = contributing_rule_ids(principle);
    if rule_ids.is_empty() {
        return vec![];
    }
    // BTreeMap keeps entity order deterministic without a separate sort
    // pass — findings arrive already fingerprint-sorted, but grouping by
    // entity_id would otherwise scramble that order.
    let mut by_entity: BTreeMap<&str, Vec<&Finding>> = BTreeMap::new();
    for finding in findings {
        if rule_ids.contains(&finding.rule_id) {
            by_entity
                .entry(finding.entity_id.as_str())
                .or_default()
                .push(finding);
        }
    }
    by_entity
        .into_values()
        .map(|matches| {
            // One strong finding is evidence enough, not diluted by
            // averaging in a weaker one.
            let confidence = matches
                .iter()
                .map(|finding| finding.confidence)
                .fold(0.0_f32, f32::max);
            let risk = risk_level_from_confidence(confidence);
            let evidence = matches
                .iter()
                .map(|finding| EvidenceRef {
                    rule_id: finding.rule_id,
                    entity_id: finding.entity_id.clone(),
                    summary: format!(
                        "{} finding at {:.2} confidence",
                        finding.rule_name, finding.confidence
                    ),
                })
                .collect();
            PrincipleRisk {
                principle,
                risk,
                confidence,
                evidence,
                explanation: explanation_for(principle, risk),
            }
        })
        .collect()
}

const MIN_DIP_DEPENDENCIES: f32 = 4.0;
const MIN_CONCRETE_RATIO: f32 = 0.75;

/// For each type, looks at its methods' outgoing
/// `Creates`/`AcceptsParameter`/`Returns`/`UsesType` edges and compares how
/// many point at a concrete `TypeKind::Class`/`Struct`/`Record` versus
/// `TypeKind::Interface`. A type with few dependencies overall is not
/// flagged regardless of ratio — one concrete dependency out of one is not
/// meaningful evidence of anything.
fn assess_dip(ctx: &AnalysisContext) -> Vec<PrincipleRisk> {
    let type_kind_of: BTreeMap<&str, TypeKind> = ctx
        .project
        .types
        .iter()
        .map(|type_ir| (type_ir.id.as_str(), type_ir.kind))
        .collect();
    let owner_of_method: BTreeMap<&str, &str> = ctx
        .project
        .methods
        .iter()
        .map(|method| (method.id.as_str(), method.owner_type.as_str()))
        .collect();

    let mut concrete_counts: BTreeMap<&str, u32> = BTreeMap::new();
    let mut total_counts: BTreeMap<&str, u32> = BTreeMap::new();
    for edge in ctx.graph.edges() {
        if !matches!(
            edge.kind,
            DependencyKind::Creates
                | DependencyKind::AcceptsParameter
                | DependencyKind::Returns
                | DependencyKind::UsesType
        ) {
            continue;
        }
        let EntityRef::Method(method_id) = &edge.from else {
            continue;
        };
        let EntityRef::Type(target_type_id) = &edge.to else {
            continue;
        };
        let Some(&owner_type) = owner_of_method.get(method_id.as_str()) else {
            continue;
        };
        let Some(&target_kind) = type_kind_of.get(target_type_id.as_str()) else {
            continue;
        };
        *total_counts.entry(owner_type).or_insert(0) += 1;
        if !matches!(target_kind, TypeKind::Interface) {
            *concrete_counts.entry(owner_type).or_insert(0) += 1;
        }
    }

    let mut risks = Vec::new();
    for (owner_type, total) in total_counts {
        #[allow(clippy::cast_precision_loss)]
        let total_f32 = total as f32;
        if total_f32 < MIN_DIP_DEPENDENCIES {
            continue;
        }
        #[allow(clippy::cast_precision_loss)]
        let concrete_ratio = f32::from(
            u16::try_from(concrete_counts.get(owner_type).copied().unwrap_or(0))
                .unwrap_or(u16::MAX),
        ) / total_f32;
        if concrete_ratio < MIN_CONCRETE_RATIO {
            continue;
        }
        let confidence = concrete_ratio;
        let risk = risk_level_from_confidence(confidence);
        risks.push(PrincipleRisk {
            principle: Principle::Dip,
            risk,
            confidence,
            evidence: vec![EvidenceRef {
                rule_id: "DIP_CONCRETE_DEPENDENCY_RATIO",
                entity_id: owner_type.to_owned(),
                summary: format!(
                    "{:.0}% of {total} resolved type dependencies are concrete classes, not interfaces",
                    concrete_ratio * 100.0
                ),
            }],
            explanation: explanation_for(Principle::Dip, risk),
        });
    }
    risks
}

const MIN_CONFIDENCE_ISP: f32 = 0.5;

/// The same "looks trivially unimplemented" check `RefusedBequest` uses for
/// base-class overrides, applied to `Implements` edges instead of
/// `Inherits`: a same-named interface member an implementor stubs out or
/// throws from is real evidence the interface is too fat for that
/// implementor.
fn assess_isp(ctx: &AnalysisContext) -> Vec<PrincipleRisk> {
    let mut risks = Vec::new();
    for type_ir in &ctx.project.types {
        let interface_ids: Vec<_> = ctx
            .graph
            .outgoing(&EntityRef::Type(type_ir.id.clone()))
            .iter()
            .map(|index| &ctx.graph.edges()[*index])
            .filter(|edge| edge.kind == DependencyKind::Implements)
            .filter_map(|edge| match &edge.to {
                EntityRef::Type(id) => Some(id.clone()),
                _ => None,
            })
            .collect();
        if interface_ids.is_empty() {
            continue;
        }
        let interface_method_names: std::collections::HashSet<&str> = ctx
            .project
            .methods
            .iter()
            .filter(|method| interface_ids.contains(&method.owner_type))
            .map(|method| method.name.as_str())
            .collect();
        let refused_count = ctx
            .project
            .methods
            .iter()
            .filter(|method| method.owner_type == type_ir.id)
            .filter(|method| interface_method_names.contains(method.name.as_str()))
            .filter(|method| looks_refused(ctx, method))
            .count();
        if refused_count == 0 {
            continue;
        }
        #[allow(clippy::cast_precision_loss)]
        let confidence = (refused_count as f32 / (refused_count as f32 + 1.0)).clamp(0.0, 1.0);
        if confidence < MIN_CONFIDENCE_ISP {
            continue;
        }
        let risk = risk_level_from_confidence(confidence);
        risks.push(PrincipleRisk {
            principle: Principle::Isp,
            risk,
            confidence,
            evidence: vec![EvidenceRef {
                rule_id: "ISP_UNIMPLEMENTED_INTERFACE_MEMBER",
                entity_id: type_ir.id.as_str().to_owned(),
                summary: format!(
                    "{refused_count} interface member(s) look trivially unimplemented here"
                ),
            }],
            explanation: explanation_for(Principle::Isp, risk),
        });
    }
    risks
}

const MIN_DEEP_CHAIN_COUNT: f32 = 2.0;

/// Scans every method's calls/field-accesses for a receiver chain of depth
/// 2 or more — "talking to a friend of a friend" — a pure syntactic fact
/// (`receiver_chain_depth`), no resolution needed.
fn assess_law_of_demeter(ctx: &AnalysisContext) -> Vec<PrincipleRisk> {
    let mut risks = Vec::new();
    for method in &ctx.project.methods {
        let deep_chain_count = method
            .calls
            .iter()
            .filter(|call| call.receiver_chain_depth >= 2)
            .count()
            + method
                .field_accesses
                .iter()
                .filter(|access| access.receiver_chain_depth >= 2)
                .count();
        if deep_chain_count == 0 {
            continue;
        }
        #[allow(clippy::cast_precision_loss)]
        let observed = deep_chain_count as f32;
        if observed < MIN_DEEP_CHAIN_COUNT {
            continue;
        }
        let confidence = (observed / (observed + MIN_DEEP_CHAIN_COUNT)).clamp(0.0, 1.0);
        let risk = risk_level_from_confidence(confidence);
        risks.push(PrincipleRisk {
            principle: Principle::LawOfDemeter,
            risk,
            confidence,
            evidence: vec![EvidenceRef {
                rule_id: "LAW_OF_DEMETER_DEEP_CHAIN_ACCESS",
                entity_id: method.id.as_str().to_owned(),
                summary: format!(
                    "{deep_chain_count} call(s)/access(es) reach through a chain of depth 2 or more"
                ),
            }],
            explanation: explanation_for(Principle::LawOfDemeter, risk),
        });
    }
    risks
}
