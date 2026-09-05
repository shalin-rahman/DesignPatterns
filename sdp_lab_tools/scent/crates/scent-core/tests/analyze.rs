use std::path::Path;

use scent_core::{analyze_path, to_json};
use scent_domain::{NormalizedPath, Resolution};

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample-project"
    ))
}

#[test]
fn analyzes_a_project_end_to_end() {
    let report = analyze_path(fixture_root()).unwrap();

    assert_eq!(report.project.types.len(), 2);
    assert_eq!(report.project.methods.len(), 3);
    assert!(report.diagnostics.is_empty());

    let create = report
        .project
        .methods
        .iter()
        .find(|item| item.name == "Create")
        .unwrap();
    assert!(matches!(
        create.instantiations[0].target,
        Resolution::Resolved(_)
    ));

    let ship = report
        .project
        .methods
        .iter()
        .find(|item| item.name == "Ship")
        .unwrap();
    assert!(matches!(ship.calls[0].target, Resolution::Resolved(_)));
}

#[test]
fn scans_suppressions_from_source_comments() {
    let report = analyze_path(fixture_root()).unwrap();
    let path = NormalizedPath::parse("src/Order.cs").unwrap();
    assert!(report.suppressions.is_suppressed(&path, "LONG_METHOD", 13));
    assert!(!report.suppressions.is_suppressed(&path, "LONG_METHOD", 3));
}

#[test]
fn json_report_is_byte_identical_across_runs() {
    let first = to_json(&analyze_path(fixture_root()).unwrap(), fixture_root());
    let second = to_json(&analyze_path(fixture_root()).unwrap(), fixture_root());
    assert_eq!(first, second);
    assert!(first.starts_with("{\"project_id\":"));
}
