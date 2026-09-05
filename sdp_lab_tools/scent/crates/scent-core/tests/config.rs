//! Confirms `smell_detector.toml` actually changes analysis behavior end
//! to end through `analyze_path` — not just that `scent-config` can parse
//! the file (that's covered in `scent-config`'s own unit tests).

use std::path::Path;

use scent_core::analyze_path;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

#[test]
fn a_threshold_override_makes_a_small_method_get_flagged() {
    let report = analyze_path(&fixture("config-project")).unwrap();
    assert!(
        report.findings.iter().any(|f| f.rule_id == "LONG_METHOD"),
        "smell_detector.toml lowers LONG_METHOD's thresholds low enough that \
         this tiny method should now be flagged"
    );
}

#[test]
fn a_disabled_rule_produces_no_findings_for_it() {
    let report = analyze_path(&fixture("config-disabled-project")).unwrap();
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.rule_id == "LONG_PARAMETER_LIST"),
        "smell_detector.toml disables LONG_PARAMETER_LIST for this project"
    );
}
