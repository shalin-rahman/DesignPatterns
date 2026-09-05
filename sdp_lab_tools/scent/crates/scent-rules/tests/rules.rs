use std::fmt::Write as _;

use scent_domain::{Language, NormalizedPath, ProjectId, Severity, SuppressionMap};
use scent_graph::build_graph;
use scent_ir::ProjectIR;
use scent_metrics::build_metric_store;
use scent_parser::{extract_file, resolve_project, CSharpAdapter, DeclarationIndex, SourceFile};
use scent_rules::{default_registry, AnalysisContext};

fn evaluate(contents: &str) -> Vec<scent_rules::Finding> {
    let mut adapter = CSharpAdapter::new();
    let source = SourceFile {
        path: NormalizedPath::parse("src/Order.cs").unwrap(),
        contents: contents.into(),
    };
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);
    let method_complexity = extracted.method_complexity.clone();
    let method_clone_signatures = extracted.method_clone_signatures.clone();
    let method_switches = extracted.method_switches.clone();
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
    let suppressions =
        SuppressionMap::scan(&NormalizedPath::parse("src/Order.cs").unwrap(), contents);

    default_registry().evaluate_all(&AnalysisContext {
        project: &project,
        graph: &graph,
        metrics: &metrics,
        suppressions: &suppressions,
        clone_signatures: &method_clone_signatures,
        switch_shapes: &method_switches,
        history: None,
    })
}

fn long_method_body() -> String {
    let mut body = String::from("class Order { void Process() {");
    for i in 0..30 {
        let _ = writeln!(body, "if (a == {i}) {{ var x = {i}; }}");
    }
    body.push_str("} }");
    body
}

#[test]
fn flags_a_method_with_high_complexity_and_length() {
    let findings = evaluate(&long_method_body());
    assert!(findings.iter().any(|f| f.rule_id == "LONG_METHOD"));
    let finding = findings
        .iter()
        .find(|f| f.rule_id == "LONG_METHOD")
        .unwrap();
    assert!(finding.confidence >= 0.5);
    assert_eq!(finding.severity, Severity::Critical);
}

#[test]
fn does_not_flag_a_small_method() {
    let findings = evaluate("class Order { void Ship() { var x = 1; } }");
    assert!(findings.is_empty());
}

#[test]
fn flags_a_long_parameter_list() {
    let findings = evaluate("class Order { void M(int a, int b, int c, int d, int e, int f) {} }");
    assert!(findings.iter().any(|f| f.rule_id == "LONG_PARAMETER_LIST"));
}

#[test]
fn a_suppressed_finding_is_not_reported() {
    let source = format!(
        "// scent:disable LONG_METHOD\n{}\n// scent:enable LONG_METHOD",
        long_method_body()
    );
    let findings = evaluate(&source);
    assert!(!findings.iter().any(|f| f.rule_id == "LONG_METHOD"));
}

#[test]
fn flags_a_large_class() {
    let mut source = String::from("class BigOne {\n");
    for i in 0..20 {
        let _ = writeln!(source, "private int field{i};");
    }
    for i in 0..40 {
        let _ = writeln!(source, "void M{i}()\n{{\n    var x{i} = {i};\n}}");
    }
    source.push('}');
    let findings = evaluate(&source);
    assert!(findings.iter().any(|f| f.rule_id == "LARGE_CLASS"));
}

#[test]
fn flags_a_recurring_parameter_group_as_a_data_clump() {
    let source = "class Order { \
         void A(int startDate, int endDate, int timezone) {} \
         void B(int startDate, int endDate, int timezone) {} \
         void C(int startDate, int endDate, int timezone) {} \
     }";
    let findings = evaluate(source);
    let clump_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "DATA_CLUMPS")
        .collect();
    assert_eq!(
        clump_findings.len(),
        3,
        "one finding per method in the clump"
    );
}

#[test]
fn a_one_off_parameter_group_is_not_a_data_clump() {
    let findings = evaluate("class Order { void A(int startDate, int endDate, int timezone) {} }");
    assert!(!findings.iter().any(|f| f.rule_id == "DATA_CLUMPS"));
}

#[test]
fn flags_a_method_with_many_primitive_parameters() {
    let findings = evaluate("class Order { void M(int a, string b, bool c, double d) {} }");
    assert!(findings.iter().any(|f| f.rule_id == "PRIMITIVE_OBSESSION"));
}

#[test]
fn does_not_flag_domain_typed_parameters_as_primitive_obsession() {
    let findings = evaluate(
        "class Order { void M(Customer a, Address b) {} } class Customer {} class Address {}",
    );
    assert!(!findings.iter().any(|f| f.rule_id == "PRIMITIVE_OBSESSION"));
}

#[test]
fn flags_two_types_with_heavy_bidirectional_coupling() {
    let mut source = String::from("class A { private B b; ");
    for i in 0..6 {
        let _ = writeln!(source, "void CallB{i}() {{ b.Foo(); }}");
    }
    source.push_str("} class B { private A a; void Foo() { a.Bar(); } void Bar() {} }");
    let findings = evaluate(&source);
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "INAPPROPRIATE_INTIMACY"));
}

#[test]
fn flags_an_override_that_throws_not_implemented() {
    let source = "class Base { void Save() {} } \
         class Derived : Base { void Save() { throw new NotImplementedException(); } }";
    let findings = evaluate(source);
    assert!(findings.iter().any(|f| f.rule_id == "REFUSED_BEQUEST"));
}

#[test]
fn does_not_flag_a_real_override_as_refused_bequest() {
    let source = "class Base { void Save() {} } \
         class Derived : Base { void Save() { this.Persist(); } void Persist() {} }";
    let findings = evaluate(source);
    assert!(!findings.iter().any(|f| f.rule_id == "REFUSED_BEQUEST"));
}

#[test]
fn flags_an_interface_with_no_implementors() {
    let findings = evaluate("interface IUnused { }");
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "SPECULATIVE_GENERALITY"));
}

#[test]
fn flags_a_method_that_mostly_uses_another_types_members() {
    let source = "class Order { \
             private Warehouse warehouse; \
             void Ship() { \
                 warehouse.Reserve(); \
                 warehouse.Pack(); \
                 warehouse.Dispatch(); \
                 warehouse.Notify(); \
             } \
         } \
         class Warehouse { \
             void Reserve() {} void Pack() {} void Dispatch() {} void Notify() {} \
         }";
    let findings = evaluate(source);
    assert!(findings.iter().any(|f| f.rule_id == "FEATURE_ENVY"));
}

#[test]
fn flags_a_method_that_mostly_uses_another_types_members_via_a_local_variable() {
    let source = "class Order { \
             void Ship() { \
                 Warehouse warehouse = new Warehouse(); \
                 warehouse.Reserve(); \
                 warehouse.Pack(); \
                 warehouse.Dispatch(); \
                 warehouse.Notify(); \
             } \
         } \
         class Warehouse { \
             void Reserve() {} void Pack() {} void Dispatch() {} void Notify() {} \
         }";
    let findings = evaluate(source);
    assert!(
        findings.iter().any(|f| f.rule_id == "FEATURE_ENVY"),
        "a call routed through a local variable must resolve just like one routed through a field"
    );
}

#[test]
fn does_not_flag_a_method_that_mostly_uses_its_own_members() {
    let source = "class Order { \
             private Warehouse warehouse; \
             void Ship() { \
                 this.Validate(); \
                 this.Log(); \
                 this.Persist(); \
                 warehouse.Notify(); \
             } \
             void Validate() {} void Log() {} void Persist() {} \
         } \
         class Warehouse { void Notify() {} }";
    let findings = evaluate(source);
    assert!(!findings.iter().any(|f| f.rule_id == "FEATURE_ENVY"));
}

#[test]
fn flags_two_methods_with_the_same_structure_after_renaming() {
    let source = "class Order {\n\
             void A() {\n\
                 int total = 1;\n\
                 total = total + 1;\n\
                 if (total > 0) { total = 0; }\n\
                 return;\n\
             }\n\
             void B() {\n\
                 int amount = 99;\n\
                 amount = amount + 1;\n\
                 if (amount > 0) { amount = 0; }\n\
                 return;\n\
             }\n\
         }";
    let findings = evaluate(source);
    let clone_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "DUPLICATED_CODE")
        .collect();
    assert_eq!(clone_findings.len(), 2, "one finding per cloned method");
}

#[test]
fn does_not_flag_structurally_different_methods_as_duplicated() {
    let source = "class Order {\n\
             void A() {\n\
                 int total = 1;\n\
                 total = total + 1;\n\
                 if (total > 0) { total = 0; }\n\
                 return;\n\
             }\n\
             void B() {\n\
                 int amount = 1;\n\
                 amount = amount - 1;\n\
                 while (amount > 0) { amount = 0; }\n\
                 return;\n\
             }\n\
         }";
    let findings = evaluate(source);
    assert!(!findings.iter().any(|f| f.rule_id == "DUPLICATED_CODE"));
}

#[test]
fn flags_a_switch_with_many_complex_cases() {
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
    let findings = evaluate(source);
    assert!(findings.iter().any(|f| f.rule_id == "SWITCH_STATEMENTS"));
}

#[test]
fn does_not_flag_a_small_switch() {
    let source = "class ShapeRenderer { \
         void Render(int kind) { \
             switch (kind) { \
                 case 1: break; \
                 case 2: break; \
             } \
         } \
     }";
    let findings = evaluate(source);
    assert!(!findings.iter().any(|f| f.rule_id == "SWITCH_STATEMENTS"));
}

#[test]
fn does_not_flag_an_interface_with_several_implementors() {
    let source = "interface IShape {} \
         class Circle : IShape {} \
         class Square : IShape {} \
         class Triangle : IShape {}";
    let findings = evaluate(source);
    assert!(!findings
        .iter()
        .any(|f| f.rule_id == "SPECULATIVE_GENERALITY"));
}
