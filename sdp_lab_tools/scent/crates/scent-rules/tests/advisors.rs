//! Principle Risk Engine and Pattern Advisor tests. Both read `Finding`s
//! (or, for patterns, `AnalysisContext::switch_shapes`) computed by the
//! same real pipeline the other rule tests use, not fabricated inputs.

use std::fmt::Write as _;

use scent_domain::{Language, NormalizedPath, Principle, ProjectId, SuppressionMap};
use scent_graph::build_graph;
use scent_ir::ProjectIR;
use scent_metrics::build_metric_store;
use scent_parser::{extract_file, resolve_project, CSharpAdapter, DeclarationIndex, SourceFile};
use scent_rules::{
    assess_principle_risks, default_registry, recommend_patterns, AnalysisContext, PatternCandidate,
};

fn analyze(contents: &str) -> (ProjectIR, Vec<scent_rules::Finding>, AnalysisSetup) {
    let mut adapter = CSharpAdapter::new();
    let source = SourceFile {
        path: NormalizedPath::parse("src/Order.cs").unwrap(),
        contents: contents.into(),
    };
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);
    let method_clone_signatures = extracted.method_clone_signatures.clone();
    let method_switches = extracted.method_switches.clone();
    let project = ProjectIR {
        id: ProjectId::from_identity("csharp:advisors-sample"),
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
    let metrics = build_metric_store(&project, &graph, &[]);
    let suppressions = SuppressionMap::default();
    let findings = default_registry().evaluate_all(&AnalysisContext {
        project: &project,
        graph: &graph,
        metrics: &metrics,
        suppressions: &suppressions,
        clone_signatures: &method_clone_signatures,
        switch_shapes: &method_switches,
        history: None,
    });
    (
        project,
        findings,
        AnalysisSetup {
            graph,
            metrics,
            suppressions,
            method_clone_signatures,
            method_switches,
        },
    )
}

struct AnalysisSetup {
    graph: scent_graph::DependencyGraph,
    metrics: scent_metrics::MetricStore,
    suppressions: SuppressionMap,
    method_clone_signatures: Vec<(scent_domain::MethodId, scent_parser::CloneSignature)>,
    method_switches: Vec<(scent_domain::MethodId, Vec<scent_parser::SwitchShape>)>,
}

fn context<'a>(project: &'a ProjectIR, setup: &'a AnalysisSetup) -> AnalysisContext<'a> {
    AnalysisContext {
        project,
        graph: &setup.graph,
        metrics: &setup.metrics,
        suppressions: &setup.suppressions,
        clone_signatures: &setup.method_clone_signatures,
        switch_shapes: &setup.method_switches,
        history: None,
    }
}

#[test]
fn assesses_srp_risk_from_a_large_class_finding() {
    let mut source = String::from("class BigOne {\n");
    for i in 0..20 {
        let _ = writeln!(source, "private int field{i};");
    }
    for i in 0..40 {
        let _ = writeln!(source, "void M{i}()\n{{\n    var x{i} = {i};\n}}");
    }
    source.push('}');
    let (project, findings, setup) = analyze(&source);
    assert!(findings.iter().any(|f| f.rule_id == "LARGE_CLASS"));

    let risks = assess_principle_risks(&context(&project, &setup), &findings);
    assert!(risks.iter().any(|risk| risk.principle == Principle::Srp));
    let srp_risk = risks
        .iter()
        .find(|risk| risk.principle == Principle::Srp)
        .unwrap();
    assert!(srp_risk.explanation.contains("SRP risk"));
    assert!(!srp_risk.explanation.to_lowercase().contains("violated"));
}

#[test]
fn does_not_assess_srp_risk_without_a_supporting_finding() {
    let (project, findings, setup) = analyze("class Order { void Ship() { var x = 1; } }");
    assert!(findings.is_empty());
    let risks = assess_principle_risks(&context(&project, &setup), &findings);
    assert!(risks.is_empty());
}

#[test]
fn assesses_ocp_risk_from_a_switch_statements_finding() {
    let source = "class ShapeRenderer { \
         void Render(int kind) { \
             switch (kind) { \
                 case 1: if (kind > 0) { if (kind > 1) { if (kind > 2) { break; } } } break; \
                 case 2: if (kind > 0) { if (kind > 1) { if (kind > 2) { break; } } } break; \
                 case 3: if (kind > 0) { if (kind > 1) { if (kind > 2) { break; } } } break; \
                 case 4: if (kind > 0) { if (kind > 1) { if (kind > 2) { break; } } } break; \
                 default: break; \
             } \
         } \
     }";
    let (project, findings, setup) = analyze(source);
    assert!(findings.iter().any(|f| f.rule_id == "SWITCH_STATEMENTS"));
    let risks = assess_principle_risks(&context(&project, &setup), &findings);
    assert!(risks.iter().any(|risk| risk.principle == Principle::Ocp));
}

#[test]
fn assesses_dip_risk_for_a_type_that_depends_mostly_on_concrete_classes() {
    let source = "class Order { \
         Concrete1 A() { return new Concrete1(); } \
         Concrete2 B() { return new Concrete2(); } \
         Concrete3 C() { return new Concrete3(); } \
         Concrete4 D() { return new Concrete4(); } \
     } \
     class Concrete1 {} class Concrete2 {} class Concrete3 {} class Concrete4 {}";
    let (project, _, setup) = analyze(source);
    let risks = assess_principle_risks(&context(&project, &setup), &[]);
    assert!(risks.iter().any(|risk| risk.principle == Principle::Dip));
}

#[test]
fn does_not_assess_dip_risk_with_too_few_dependencies() {
    let source = "class Order { Concrete1 A() { return new Concrete1(); } } class Concrete1 {}";
    let (project, _, setup) = analyze(source);
    let risks = assess_principle_risks(&context(&project, &setup), &[]);
    assert!(!risks.iter().any(|risk| risk.principle == Principle::Dip));
}

#[test]
fn assesses_isp_risk_for_a_stubbed_interface_member() {
    let source = "interface IWorker { void DoWork(); void DoOther(); } \
         class Worker : IWorker { \
             void DoWork() { this.Persist(); } \
             void DoOther() { throw new NotImplementedException(); } \
             void Persist() {} \
         }";
    let (project, _, setup) = analyze(source);
    let risks = assess_principle_risks(&context(&project, &setup), &[]);
    assert!(risks.iter().any(|risk| risk.principle == Principle::Isp));
}

#[test]
fn does_not_assess_isp_risk_when_every_interface_member_is_implemented() {
    let source = "interface IWorker { void DoWork(); } \
         class Worker : IWorker { void DoWork() { this.Persist(); } void Persist() {} }";
    let (project, _, setup) = analyze(source);
    let risks = assess_principle_risks(&context(&project, &setup), &[]);
    assert!(!risks.iter().any(|risk| risk.principle == Principle::Isp));
}

#[test]
fn assesses_law_of_demeter_risk_for_deep_chain_access() {
    let source = "class Order { \
         void Ship() { \
             a.b.c.Foo(); \
             x.y.z.Bar(); \
         } \
     }";
    let (project, _, setup) = analyze(source);
    let risks = assess_principle_risks(&context(&project, &setup), &[]);
    assert!(risks
        .iter()
        .any(|risk| risk.principle == Principle::LawOfDemeter));
}

#[test]
fn does_not_assess_law_of_demeter_risk_for_shallow_access() {
    let source = "class Order { void Ship() { this.Validate(); } void Validate() {} }";
    let (project, _, setup) = analyze(source);
    let risks = assess_principle_risks(&context(&project, &setup), &[]);
    assert!(!risks
        .iter()
        .any(|risk| risk.principle == Principle::LawOfDemeter));
}

#[test]
fn recommends_factory_method_when_a_switch_constructs_distinct_types() {
    let source = "class ShapeFactory { \
         Shape Create(int kind) { \
             switch (kind) { \
                 case 1: return new Circle(); \
                 case 2: return new Square(); \
                 default: return new Triangle(); \
             } \
         } \
     } class Shape {} class Circle : Shape {} class Square : Shape {} class Triangle : Shape {}";
    let (project, _, setup) = analyze(source);
    let ctx = AnalysisContext {
        project: &project,
        graph: &setup.graph,
        metrics: &setup.metrics,
        suppressions: &setup.suppressions,
        clone_signatures: &setup.method_clone_signatures,
        switch_shapes: &setup.method_switches,
        history: None,
    };
    let recommendations = recommend_patterns(&ctx);
    assert_eq!(recommendations.len(), 1);
    assert_eq!(
        recommendations[0].candidate,
        PatternCandidate::FactoryMethod
    );
}

#[test]
fn recommends_do_nothing_for_a_switch_with_no_object_creation() {
    let source = "class Order { \
         void Handle(int kind) { \
             switch (kind) { \
                 case 1: break; \
                 case 2: break; \
             } \
         } \
     }";
    let (project, _, setup) = analyze(source);
    let ctx = AnalysisContext {
        project: &project,
        graph: &setup.graph,
        metrics: &setup.metrics,
        suppressions: &setup.suppressions,
        clone_signatures: &setup.method_clone_signatures,
        switch_shapes: &setup.method_switches,
        history: None,
    };
    let recommendations = recommend_patterns(&ctx);
    assert_eq!(recommendations.len(), 1);
    assert_eq!(recommendations[0].candidate, PatternCandidate::DoNothing);
}
