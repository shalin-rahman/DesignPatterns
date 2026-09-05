//! Deterministic JSON for the Phase 1 IR/diagnostic/metrics/findings
//! report. Canonical key order here is exactly insertion order, which is
//! already the sorted order the extraction/resolution passes produce, so no
//! extra sort step is needed at serialization time. No runtime metadata
//! (timestamps, durations) participates in this output. Uses the same
//! `Json` writer `scent-report` uses for findings/SARIF, so the workspace
//! has one deterministic JSON serializer, not several.

use std::path::Path;

use scent_domain::{Diagnostic, DiagnosticKind, Resolution, SourceLocation, Visibility};
use scent_graph::EntityRef;
use scent_ir::{
    CallReceiver, FieldIR, MemberTarget, MethodIR, NamespaceIR, ProjectIR, PropertyIR, TypeIR,
    TypeKind,
};
use scent_metrics::{MetricKind, MetricValue};
use scent_report::{finding_json, Json};
use scent_rules::{
    PatternCandidate, PatternRecommendation, PrincipleRisk, Refactoring, RefactoringRecommendation,
};

use crate::AnalysisReport;

/// Renders the analysis result as a canonical JSON report. Two calls over
/// the same input, from the same `root`, produce byte-identical output.
/// `root` is the analyzed project directory (the same path passed to
/// `analyze_path`) — each finding's `snippet` is read from the real file
/// under it, so the report stays accurate to what's on disk right now.
#[must_use]
pub fn to_json(report: &AnalysisReport, root: &Path) -> String {
    let mut out = String::new();
    Json::Object(vec![
        (
            "project_id",
            Json::String(report.project.id.as_str().to_owned()),
        ),
        (
            "language",
            Json::String(format!("{:?}", report.project.language)),
        ),
        ("files", Json::Array(project_files(&report.project))),
        (
            "namespaces",
            Json::Array(
                report
                    .project
                    .namespaces
                    .iter()
                    .map(namespace_json)
                    .collect(),
            ),
        ),
        (
            "types",
            Json::Array(report.project.types.iter().map(type_json).collect()),
        ),
        (
            "methods",
            Json::Array(report.project.methods.iter().map(method_json).collect()),
        ),
        (
            "fields",
            Json::Array(report.project.fields.iter().map(field_json).collect()),
        ),
        (
            "properties",
            Json::Array(
                report
                    .project
                    .properties
                    .iter()
                    .map(property_json)
                    .collect(),
            ),
        ),
        (
            "diagnostics",
            Json::Array(report.diagnostics.iter().map(diagnostic_json).collect()),
        ),
        ("metrics", Json::Array(metrics_json(&report.metrics))),
        (
            "findings",
            Json::Array(
                report
                    .findings
                    .iter()
                    .map(|finding| finding_json(finding, root))
                    .collect(),
            ),
        ),
        (
            "principle_risks",
            Json::Array(
                report
                    .principle_risks
                    .iter()
                    .map(principle_risk_json)
                    .collect(),
            ),
        ),
        (
            "pattern_recommendations",
            Json::Array(
                report
                    .pattern_recommendations
                    .iter()
                    .map(pattern_recommendation_json)
                    .collect(),
            ),
        ),
        (
            "refactoring_recommendations",
            Json::Array(
                report
                    .refactoring_recommendations
                    .iter()
                    .map(refactoring_recommendation_json)
                    .collect(),
            ),
        ),
    ])
    .render(&mut out);
    out
}

fn principle_risk_json(risk: &PrincipleRisk) -> Json {
    Json::Object(vec![
        ("principle", Json::String(format!("{:?}", risk.principle))),
        ("risk", Json::String(format!("{:?}", risk.risk))),
        ("confidence", Json::Float(f64::from(risk.confidence))),
        ("explanation", Json::String(risk.explanation.clone())),
        (
            "evidence",
            Json::Array(
                risk.evidence
                    .iter()
                    .map(|item| {
                        Json::Object(vec![
                            ("rule", Json::String(item.rule_id.into())),
                            ("entity", Json::String(item.entity_id.clone())),
                            ("summary", Json::String(item.summary.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn pattern_recommendation_json(recommendation: &PatternRecommendation) -> Json {
    let candidate = match recommendation.candidate {
        PatternCandidate::FactoryMethod => "factory_method",
        PatternCandidate::DoNothing => "do_nothing",
    };
    Json::Object(vec![
        (
            "method_id",
            Json::String(recommendation.method_id.as_str().to_owned()),
        ),
        (
            "switch_ordinal",
            Json::Number(u64::try_from(recommendation.switch_ordinal).unwrap_or(u64::MAX)),
        ),
        ("candidate", Json::String(candidate.into())),
        (
            "explanation",
            Json::String(recommendation.explanation.clone()),
        ),
    ])
}

fn refactoring_recommendation_json(recommendation: &RefactoringRecommendation) -> Json {
    let refactoring = match recommendation.refactoring {
        Refactoring::ExtractMethod => "extract_method",
        Refactoring::ExtractClass => "extract_class",
        Refactoring::IntroduceParameterObject => "introduce_parameter_object",
        Refactoring::MoveMethod => "move_method",
        Refactoring::MoveField => "move_field",
        Refactoring::ReplaceConditionalWithPolymorphism => "replace_conditional_with_polymorphism",
        Refactoring::ReplaceInheritanceWithDelegation => "replace_inheritance_with_delegation",
        Refactoring::RemoveSpeculativeGenerality => "remove_speculative_generality",
    };
    Json::Object(vec![
        ("entity", Json::String(recommendation.entity_id.clone())),
        ("rule", Json::String(recommendation.rule_id.into())),
        ("refactoring", Json::String(refactoring.into())),
    ])
}

fn metrics_json(metrics: &scent_metrics::MetricStore) -> Vec<Json> {
    metrics
        .iter()
        .map(|((entity, kind), value)| {
            Json::Object(vec![
                ("entity", entity_ref_json(entity)),
                ("metric", Json::String(metric_kind_str(*kind).into())),
                ("value", metric_value_json(value)),
            ])
        })
        .collect()
}

fn entity_ref_json(entity: &EntityRef) -> Json {
    let (kind, id) = match entity {
        EntityRef::Type(id) => ("type", id.as_str()),
        EntityRef::Method(id) => ("method", id.as_str()),
        EntityRef::Field(id) => ("field", id.as_str()),
        EntityRef::Property(id) => ("property", id.as_str()),
    };
    Json::Object(vec![
        ("kind", Json::String(kind.into())),
        ("id", Json::String(id.to_owned())),
    ])
}

fn metric_value_json(value: &MetricValue) -> Json {
    Json::Object(vec![
        ("value", Json::Float(value.value)),
        ("confidence", Json::Float(f64::from(value.confidence))),
        ("provenance", Json::String(value.provenance.into())),
    ])
}

fn metric_kind_str(kind: MetricKind) -> &'static str {
    match kind {
        MetricKind::Loc => "loc",
        MetricKind::CyclomaticComplexity => "cyclomatic_complexity",
        MetricKind::NestingDepth => "nesting_depth",
        MetricKind::Lcom4 => "lcom4",
        MetricKind::Cbo => "cbo",
    }
}

fn project_files(project: &ProjectIR) -> Vec<Json> {
    project
        .files
        .iter()
        .map(|file| {
            Json::Object(vec![
                ("id", Json::String(file.id.as_str().to_owned())),
                ("path", Json::String(file.path.as_str().to_owned())),
                (
                    "namespace_ids",
                    Json::Array(
                        file.namespace_ids
                            .iter()
                            .map(|id| Json::String(id.clone()))
                            .collect(),
                    ),
                ),
                (
                    "type_ids",
                    Json::Array(
                        file.type_ids
                            .iter()
                            .map(|id| Json::String(id.as_str().to_owned()))
                            .collect(),
                    ),
                ),
            ])
        })
        .collect()
}

fn namespace_json(namespace: &NamespaceIR) -> Json {
    Json::Object(vec![
        ("id", Json::String(namespace.id.clone())),
        ("name", Json::String(namespace.name.clone())),
        (
            "file_id",
            Json::String(namespace.file_id.as_str().to_owned()),
        ),
        ("location", location_json(&namespace.location)),
    ])
}

fn type_json(type_ir: &TypeIR) -> Json {
    Json::Object(vec![
        ("id", Json::String(type_ir.id.as_str().to_owned())),
        ("file_id", Json::String(type_ir.file_id.as_str().to_owned())),
        (
            "namespace_id",
            type_ir
                .namespace_id
                .clone()
                .map_or(Json::Null, Json::String),
        ),
        ("name", Json::String(type_ir.name.clone())),
        ("kind", Json::String(type_kind_str(type_ir.kind).into())),
        ("location", location_json(&type_ir.location)),
        (
            "visibility",
            Json::String(visibility_str(type_ir.visibility).into()),
        ),
        (
            "base_types",
            Json::Array(
                type_ir
                    .base_types
                    .iter()
                    .map(type_resolution_json)
                    .collect(),
            ),
        ),
        (
            "interface_types",
            Json::Array(
                type_ir
                    .interface_types
                    .iter()
                    .map(type_resolution_json)
                    .collect(),
            ),
        ),
    ])
}

fn method_json(method: &MethodIR) -> Json {
    Json::Object(vec![
        ("id", Json::String(method.id.as_str().to_owned())),
        (
            "owner_type",
            Json::String(method.owner_type.as_str().to_owned()),
        ),
        ("name", Json::String(method.name.clone())),
        ("location", location_json(&method.location)),
        (
            "visibility",
            Json::String(visibility_str(method.visibility).into()),
        ),
        ("parameters", Json::Array(parameters_json(method))),
        (
            "return_type",
            method
                .return_type
                .as_ref()
                .map_or(Json::Null, type_resolution_json),
        ),
        ("calls", Json::Array(calls_json(method))),
        ("field_accesses", Json::Array(field_accesses_json(method))),
        ("type_references", Json::Array(type_references_json(method))),
        ("instantiations", Json::Array(instantiations_json(method))),
        ("local_variables", Json::Array(local_variables_json(method))),
    ])
}

fn parameters_json(method: &MethodIR) -> Vec<Json> {
    method
        .parameters
        .iter()
        .map(|parameter| {
            Json::Object(vec![
                ("name", Json::String(parameter.name.clone())),
                ("location", location_json(&parameter.location)),
                (
                    "type_reference",
                    resolution_json(&parameter.type_reference, |id| {
                        Json::String(id.as_str().to_owned())
                    }),
                ),
            ])
        })
        .collect()
}

fn calls_json(method: &MethodIR) -> Vec<Json> {
    method
        .calls
        .iter()
        .map(|call| {
            Json::Object(vec![
                (
                    "target",
                    resolution_json(&call.target, |id| Json::String(id.as_str().to_owned())),
                ),
                ("receiver", receiver_json(&call.receiver)),
                (
                    "receiver_chain_depth",
                    Json::Number(u64::from(call.receiver_chain_depth)),
                ),
                ("location", location_json(&call.location)),
            ])
        })
        .collect()
}

fn field_accesses_json(method: &MethodIR) -> Vec<Json> {
    method
        .field_accesses
        .iter()
        .map(|access| {
            Json::Object(vec![
                (
                    "target",
                    resolution_json(&access.target, member_target_json),
                ),
                ("receiver", receiver_json(&access.receiver)),
                (
                    "receiver_chain_depth",
                    Json::Number(u64::from(access.receiver_chain_depth)),
                ),
                ("location", location_json(&access.location)),
            ])
        })
        .collect()
}

fn type_references_json(method: &MethodIR) -> Vec<Json> {
    method
        .type_references
        .iter()
        .map(|reference| {
            Json::Object(vec![
                ("target", type_resolution_json(&reference.target)),
                ("location", location_json(&reference.location)),
            ])
        })
        .collect()
}

fn instantiations_json(method: &MethodIR) -> Vec<Json> {
    method
        .instantiations
        .iter()
        .map(|instantiation| {
            Json::Object(vec![
                ("target", type_resolution_json(&instantiation.target)),
                ("location", location_json(&instantiation.location)),
            ])
        })
        .collect()
}

fn local_variables_json(method: &MethodIR) -> Vec<Json> {
    method
        .local_variables
        .iter()
        .map(|local| {
            Json::Object(vec![
                ("name", Json::String(local.name.clone())),
                ("location", location_json(&local.location)),
                (
                    "type_reference",
                    resolution_json(&local.type_reference, |id| {
                        Json::String(id.as_str().to_owned())
                    }),
                ),
            ])
        })
        .collect()
}

fn field_json(field: &FieldIR) -> Json {
    Json::Object(vec![
        ("id", Json::String(field.id.as_str().to_owned())),
        (
            "owner_type",
            Json::String(field.owner_type.as_str().to_owned()),
        ),
        ("name", Json::String(field.name.clone())),
        ("location", location_json(&field.location)),
        (
            "visibility",
            Json::String(visibility_str(field.visibility).into()),
        ),
        (
            "type_reference",
            type_resolution_json(&field.type_reference),
        ),
    ])
}

fn property_json(property: &PropertyIR) -> Json {
    Json::Object(vec![
        ("id", Json::String(property.id.as_str().to_owned())),
        (
            "owner_type",
            Json::String(property.owner_type.as_str().to_owned()),
        ),
        ("name", Json::String(property.name.clone())),
        ("location", location_json(&property.location)),
        (
            "visibility",
            Json::String(visibility_str(property.visibility).into()),
        ),
        (
            "type_reference",
            type_resolution_json(&property.type_reference),
        ),
    ])
}

fn diagnostic_json(diagnostic: &Diagnostic) -> Json {
    Json::Object(vec![
        (
            "kind",
            Json::String(diagnostic_kind_str(diagnostic.kind).into()),
        ),
        ("message", Json::String(diagnostic.message.clone())),
        (
            "location",
            diagnostic
                .location
                .as_ref()
                .map_or(Json::Null, location_json),
        ),
    ])
}

fn location_json(location: &SourceLocation) -> Json {
    Json::Object(vec![
        ("path", Json::String(location.path.as_str().to_owned())),
        (
            "start_line",
            Json::Number(u64::from(location.range.start.line)),
        ),
        (
            "start_column",
            Json::Number(u64::from(location.range.start.column)),
        ),
        ("end_line", Json::Number(u64::from(location.range.end.line))),
        (
            "end_column",
            Json::Number(u64::from(location.range.end.column)),
        ),
    ])
}

fn type_resolution_json(resolution: &Resolution<scent_domain::TypeId>) -> Json {
    resolution_json(resolution, |id| Json::String(id.as_str().to_owned()))
}

fn resolution_json<T>(resolution: &Resolution<T>, resolved: impl FnOnce(&T) -> Json) -> Json {
    match resolution {
        Resolution::Resolved(value) => Json::Object(vec![
            ("status", Json::String("resolved".into())),
            ("value", resolved(value)),
        ]),
        Resolution::Unresolved(reference) => Json::Object(vec![
            ("status", Json::String("unresolved".into())),
            (
                "reference",
                Json::Object(vec![
                    ("kind", Json::String(format!("{:?}", reference.kind))),
                    ("spelling", Json::String(reference.spelling.clone())),
                    ("location", location_json(&reference.location)),
                    ("reason", Json::String(reference.reason.clone())),
                ]),
            ),
        ]),
    }
}

fn member_target_json(target: &MemberTarget) -> Json {
    match target {
        MemberTarget::Field(id) => Json::Object(vec![
            ("kind", Json::String("field".into())),
            ("id", Json::String(id.as_str().to_owned())),
        ]),
        MemberTarget::Property(id) => Json::Object(vec![
            ("kind", Json::String("property".into())),
            ("id", Json::String(id.as_str().to_owned())),
        ]),
    }
}

fn type_kind_str(kind: TypeKind) -> &'static str {
    match kind {
        TypeKind::Class => "class",
        TypeKind::Interface => "interface",
        TypeKind::Struct => "struct",
        TypeKind::Enum => "enum",
        TypeKind::Record => "record",
    }
}

fn visibility_str(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "public",
        Visibility::Protected => "protected",
        Visibility::Internal => "internal",
        Visibility::Private => "private",
        Visibility::Unknown => "unknown",
    }
}

fn receiver_json(receiver: &CallReceiver) -> Json {
    match receiver {
        CallReceiver::SelfOrImplicit => Json::String("self_or_implicit".into()),
        CallReceiver::Named(name) => Json::String(format!("named:{name}")),
        CallReceiver::Other => Json::String("other".into()),
    }
}

fn diagnostic_kind_str(kind: DiagnosticKind) -> &'static str {
    match kind {
        DiagnosticKind::ParseError => "parse_error",
        DiagnosticKind::ResolutionError => "resolution_error",
        DiagnosticKind::UnsupportedSyntax => "unsupported_syntax",
        DiagnosticKind::ConfigurationError => "configuration_error",
        DiagnosticKind::GitError => "git_error",
        DiagnosticKind::DockerError => "docker_error",
        DiagnosticKind::InternalError => "internal_error",
    }
}
