//! A golden JSON snapshot for the sample-project fixture, checked in
//! (`fixtures/sample-project/expected_output.json`) rather than only
//! compared "two runs match each other." This catches any unintended
//! change to the report's shape or content, not just non-determinism.
//!
//! Uses `analyze_path_without_git_history` rather than `analyze_path`:
//! this whole workspace sits nested inside a larger outer repository, so a
//! snapshot that included git-backed findings would silently drift as that
//! outer repository's own history grows — not a stable "golden" file at
//! all. The golden snapshot is about the fixture's own source, not ambient
//! repository state.
//!
//! To regenerate after an intentional report-shape change, run with
//! `UPDATE_GOLDEN=1` set (see `docs/contributor-guide.md`).

use std::path::Path;

use scent_core::{analyze_path_without_git_history, to_json};

#[test]
fn json_report_matches_the_golden_snapshot() {
    let root = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample-project"
    ));
    let golden_path = root.join("expected_output.json");

    let actual = to_json(&analyze_path_without_git_history(root).unwrap(), root);

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(&golden_path, &actual).unwrap();
        return;
    }

    let expected = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!(
            "no golden file at {}; run once with UPDATE_GOLDEN=1 to create it",
            golden_path.display()
        )
    });
    assert_eq!(
        actual,
        expected,
        "JSON report shape/content changed — if this is intentional, \
         re-run with UPDATE_GOLDEN=1 to regenerate {}",
        golden_path.display()
    );
}
