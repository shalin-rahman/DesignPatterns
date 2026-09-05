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

#[test]
fn flags_a_method_with_high_complexity_and_length() {
    let findings = evaluate(include_str!("fixtures/TestSubjects/src/LongMethod.cs"));
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
    let findings = evaluate(include_str!("fixtures/TestSubjects/src/SmallMethod.cs"));
    assert!(findings.is_empty());
}

#[test]
fn flags_a_long_parameter_list() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/LongParameterList.cs"
    ));
    assert!(findings.iter().any(|f| f.rule_id == "LONG_PARAMETER_LIST"));
}

#[test]
fn a_suppressed_finding_is_not_reported() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/LongMethodSuppressed.cs"
    ));
    assert!(!findings.iter().any(|f| f.rule_id == "LONG_METHOD"));
}

#[test]
fn flags_a_large_class() {
    let findings = evaluate(include_str!("fixtures/TestSubjects/src/LargeClass.cs"));
    assert!(findings.iter().any(|f| f.rule_id == "LARGE_CLASS"));
}

#[test]
fn flags_a_recurring_parameter_group_as_a_data_clump() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/DataClumpsRecurring.cs"
    ));
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
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/DataClumpsOneOff.cs"
    ));
    assert!(!findings.iter().any(|f| f.rule_id == "DATA_CLUMPS"));
}

#[test]
fn flags_a_method_with_many_primitive_parameters() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/PrimitiveObsession.cs"
    ));
    assert!(findings.iter().any(|f| f.rule_id == "PRIMITIVE_OBSESSION"));
}

#[test]
fn does_not_flag_domain_typed_parameters_as_primitive_obsession() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/PrimitiveObsessionDomainTyped.cs"
    ));
    assert!(!findings.iter().any(|f| f.rule_id == "PRIMITIVE_OBSESSION"));
}

#[test]
fn flags_two_types_with_heavy_bidirectional_coupling() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/InappropriateIntimacy.cs"
    ));
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "INAPPROPRIATE_INTIMACY"));
}

#[test]
fn flags_an_override_that_throws_not_implemented() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/RefusedBequestThrows.cs"
    ));
    assert!(findings.iter().any(|f| f.rule_id == "REFUSED_BEQUEST"));
}

#[test]
fn does_not_flag_a_real_override_as_refused_bequest() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/RefusedBequestRealOverride.cs"
    ));
    assert!(!findings.iter().any(|f| f.rule_id == "REFUSED_BEQUEST"));
}

#[test]
fn flags_an_interface_with_no_implementors() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/SpeculativeGeneralityUnused.cs"
    ));
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "SPECULATIVE_GENERALITY"));
}

#[test]
fn flags_a_method_that_mostly_uses_another_types_members() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/FeatureEnvyForeignMembers.cs"
    ));
    assert!(findings.iter().any(|f| f.rule_id == "FEATURE_ENVY"));
}

#[test]
fn flags_a_method_that_mostly_uses_another_types_members_via_a_local_variable() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/FeatureEnvyForeignMembersViaLocal.cs"
    ));
    assert!(
        findings.iter().any(|f| f.rule_id == "FEATURE_ENVY"),
        "a call routed through a local variable must resolve just like one routed through a field"
    );
}

#[test]
fn does_not_flag_a_method_that_mostly_uses_its_own_members() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/FeatureEnvyOwnMembers.cs"
    ));
    assert!(!findings.iter().any(|f| f.rule_id == "FEATURE_ENVY"));
}

#[test]
fn flags_two_methods_with_the_same_structure_after_renaming() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/DuplicatedCodeSameStructure.cs"
    ));
    let clone_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "DUPLICATED_CODE")
        .collect();
    assert_eq!(clone_findings.len(), 2, "one finding per cloned method");
}

#[test]
fn does_not_flag_structurally_different_methods_as_duplicated() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/DuplicatedCodeDifferentStructure.cs"
    ));
    assert!(!findings.iter().any(|f| f.rule_id == "DUPLICATED_CODE"));
}

#[test]
fn flags_a_switch_with_many_complex_cases() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/SwitchStatementsComplex.cs"
    ));
    assert!(findings.iter().any(|f| f.rule_id == "SWITCH_STATEMENTS"));
}

#[test]
fn does_not_flag_a_small_switch() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/SwitchStatementsSmall.cs"
    ));
    assert!(!findings.iter().any(|f| f.rule_id == "SWITCH_STATEMENTS"));
}

#[test]
fn does_not_flag_an_interface_with_several_implementors() {
    let findings = evaluate(include_str!(
        "fixtures/TestSubjects/src/SpeculativeGeneralityMultipleImplementors.cs"
    ));
    assert!(!findings
        .iter()
        .any(|f| f.rule_id == "SPECULATIVE_GENERALITY"));
}
