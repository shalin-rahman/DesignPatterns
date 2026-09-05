// All metric values under test are exact integer counts stored as f64, so
// strict equality is correct here, not a rounding hazard.
#![allow(clippy::float_cmp)]

use scent_domain::{Language, NormalizedPath, ProjectId};
use scent_graph::{build_graph, EntityRef};
use scent_ir::ProjectIR;
use scent_metrics::{build_metric_store, MetricKind};
use scent_parser::{extract_file, resolve_project, CSharpAdapter, DeclarationIndex, SourceFile};

fn analyze(contents: &str) -> (ProjectIR, scent_metrics::MetricStore) {
    let mut adapter = CSharpAdapter::new();
    let source = SourceFile {
        path: NormalizedPath::parse("src/Order.cs").unwrap(),
        contents: contents.into(),
    };
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);
    let method_complexity = extracted.method_complexity.clone();
    let project = ProjectIR {
        id: ProjectId::from_identity("csharp:sample"),
        language: Language::CSharp,
        files: vec![extracted.file],
        namespaces: extracted.namespaces,
        types: extracted.types,
        methods: extracted.methods,
        fields: extracted.fields,
        properties: extracted.properties,
    };
    let index = DeclarationIndex::build(&project);
    let project = resolve_project(project, &index);
    let graph = build_graph(&project);
    let metrics = build_metric_store(&project, &graph, &method_complexity);
    (project, metrics)
}

#[test]
fn cyclomatic_complexity_is_one_plus_decision_points() {
    let (project, metrics) = analyze(include_str!(
        "fixtures/TestSubjects/src/CyclomaticComplexity.cs"
    ));
    let method = &project.methods[0];
    let cc = metrics
        .get(
            &EntityRef::Method(method.id.clone()),
            MetricKind::CyclomaticComplexity,
        )
        .unwrap();
    assert_eq!(cc.value, 3.0);
}

#[test]
fn lcom4_is_one_when_two_methods_share_a_field() {
    let (project, metrics) = analyze(include_str!("fixtures/TestSubjects/src/Lcom4Connected.cs"));
    let type_ir = &project.types[0];
    let lcom4 = metrics
        .get(&EntityRef::Type(type_ir.id.clone()), MetricKind::Lcom4)
        .unwrap();
    assert_eq!(lcom4.value, 1.0);
}

#[test]
fn lcom4_is_two_when_methods_are_unconnected() {
    let (project, metrics) = analyze(include_str!(
        "fixtures/TestSubjects/src/Lcom4Unconnected.cs"
    ));
    let type_ir = &project.types[0];
    let lcom4 = metrics
        .get(&EntityRef::Type(type_ir.id.clone()), MetricKind::Lcom4)
        .unwrap();
    assert_eq!(lcom4.value, 2.0);
}

#[test]
fn cbo_counts_a_resolved_instantiation_of_another_project_type() {
    let (project, metrics) = analyze(include_str!("fixtures/TestSubjects/src/CboCoupling.cs"));
    let factory = project
        .types
        .iter()
        .find(|t| t.name == "OrderFactory")
        .unwrap();
    let cbo = metrics
        .get(&EntityRef::Type(factory.id.clone()), MetricKind::Cbo)
        .unwrap();
    assert_eq!(cbo.value, 1.0);
}

#[test]
fn loc_spans_the_methods_declared_lines() {
    let (project, metrics) = analyze(include_str!("fixtures/TestSubjects/src/LocSpan.cs"));
    let method = &project.methods[0];
    let loc = metrics
        .get(&EntityRef::Method(method.id.clone()), MetricKind::Loc)
        .unwrap();
    assert_eq!(loc.value, 3.0);
}
