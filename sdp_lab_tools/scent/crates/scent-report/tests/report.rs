use scent_domain::{
    NormalizedPath, Resolution, Severity, SourceLocation, SourcePosition, SourceRange,
};
use scent_report::{evaluate_gate, findings_to_json_string, to_sarif, Baseline, QualityGateConfig};
use scent_rules::{EvidenceItem, Finding, FindingFingerprint};

fn location() -> SourceLocation {
    let path = NormalizedPath::parse("src/Order.cs").unwrap();
    let start = SourcePosition { line: 3, column: 0 };
    let end = SourcePosition {
        line: 10,
        column: 1,
    };
    SourceLocation::new(path, SourceRange::new(start, end).unwrap())
}

fn sample_finding(rule_id: &'static str, severity: Severity) -> Finding {
    Finding {
        fingerprint: FindingFingerprint::new(rule_id, "entity-1", "src/Order.cs"),
        rule_id,
        rule_name: "Sample Rule",
        scope: scent_domain::AnalysisScope::Method,
        severity,
        confidence: 0.8,
        location: location(),
        entity_id: "entity-1".into(),
        evidence: vec![EvidenceItem {
            metric_key: "loc",
            observed_value: 42.0,
            normalized_score: 0.8,
            weight: 1.0,
            contribution: 0.8,
            explanation: "42 lines".into(),
            location: None,
        }],
    }
}

#[test]
fn findings_json_round_trips_deterministically() {
    let findings = vec![sample_finding("LONG_METHOD", Severity::High)];
    let first = findings_to_json_string(&findings);
    let second = findings_to_json_string(&findings);
    assert_eq!(first, second);
    assert!(first.contains("\"rule\":\"LONG_METHOD\""));
    assert!(first.contains("\"severity\":\"high\""));
}

#[test]
fn sarif_output_is_valid_shaped_json_with_a_result_per_finding() {
    let findings = vec![
        sample_finding("LONG_METHOD", Severity::Critical),
        sample_finding("LARGE_CLASS", Severity::Medium),
    ];
    let sarif = to_sarif(&findings);
    assert!(sarif.contains("\"version\":\"2.1.0\""));
    assert!(sarif.contains("\"ruleId\":\"LARGE_CLASS\""));
    assert!(sarif.contains("\"level\":\"error\""));
    assert!(sarif.contains("\"level\":\"warning\""));
}

#[test]
fn baseline_round_trips_and_hides_pre_existing_findings_from_the_gate() {
    let existing = sample_finding("LONG_METHOD", Severity::Critical);
    let baseline = Baseline::from_findings(std::slice::from_ref(&existing));
    let restored = Baseline::parse(&baseline.serialize());
    assert!(restored.contains(&existing));

    let config = QualityGateConfig {
        max_critical_findings: 0,
        max_high_findings: 0,
        fail_on_new_violations: true,
    };
    let result = evaluate_gate(std::slice::from_ref(&existing), Some(&restored), &config);
    assert!(!result.passed, "still fails the raw critical-count limit");

    let lenient = QualityGateConfig {
        max_critical_findings: 10,
        max_high_findings: 10,
        fail_on_new_violations: true,
    };
    let result = evaluate_gate(std::slice::from_ref(&existing), Some(&restored), &lenient);
    assert!(
        result.passed,
        "a baselined finding must not fail on its own"
    );
}

#[test]
fn a_new_finding_not_in_the_baseline_fails_the_gate() {
    let baseline = Baseline::default();
    let finding = sample_finding("LONG_METHOD", Severity::Low);
    let config = QualityGateConfig {
        max_critical_findings: 10,
        max_high_findings: 10,
        fail_on_new_violations: true,
    };
    let result = evaluate_gate(std::slice::from_ref(&finding), Some(&baseline), &config);
    assert!(!result.passed);
}

#[test]
fn unresolved_evidence_target_is_not_required_for_a_finding_to_serialize() {
    // Regression guard: Finding/EvidenceItem serialization must not assume
    // every field is populated from a Resolved fact.
    let _ = Resolution::<()>::Unresolved(scent_domain::UnresolvedReference {
        kind: scent_domain::UnresolvedReferenceKind::Type,
        spelling: "int".into(),
        location: location(),
        reason: "not yet resolved".into(),
    });
    let findings = vec![sample_finding("LONG_METHOD", Severity::Low)];
    assert!(!findings_to_json_string(&findings).is_empty());
}
