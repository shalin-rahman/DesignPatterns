//! LLM- and CI-friendly JSON for `Finding`s (`docs/prompt.md` §32).

use std::{fs, path::Path};

use scent_domain::{Severity, SourceLocation};
use scent_rules::{EvidenceItem, Finding};

use crate::json::Json;

/// How many source lines a snippet includes at most, from its start line.
/// Long Method findings can span dozens of lines; capping keeps the report
/// readable and bounds its size, at the cost of not showing the whole body.
const MAX_SNIPPET_LINES: usize = 20;

/// One finding's source snippet: the real lines from disk at the time the
/// report was generated, each paired with its 1-based line number.
pub struct Snippet {
    pub lines: Vec<(u32, String)>,
    /// How many lines past `MAX_SNIPPET_LINES` were left out, 0 if none.
    pub omitted: usize,
}

/// Reads the lines a finding's location covers from `root.join(location.path)`.
/// Returns `None` if the file can't be read (moved/deleted since analysis,
/// or a location built in a test with no real file behind it) or the
/// recorded line range no longer fits the file — never panics on a
/// filesystem/IO failure.
#[must_use]
pub fn read_snippet(root: &Path, location: &SourceLocation) -> Option<Snippet> {
    let contents = fs::read_to_string(root.join(location.path.as_str())).ok()?;
    let all_lines: Vec<&str> = contents.lines().collect();
    let start = location.range.start.line as usize;
    if start >= all_lines.len() {
        return None;
    }
    let end = (location.range.end.line as usize).min(all_lines.len() - 1);
    let capped_end = (start + MAX_SNIPPET_LINES - 1).min(end);
    let omitted = end - capped_end;

    // Paired with a running u32 line number (1-based; `start.line` is
    // already the 0-based u32 the parser recorded) rather than
    // `index as u32`, so no usize-to-u32 cast is needed for a value the
    // compiler can't see is small.
    let lines = (location.range.start.line + 1..)
        .zip(start..=capped_end)
        .map(|(line_number, index)| (line_number, all_lines[index].to_owned()))
        .collect();
    Some(Snippet { lines, omitted })
}

fn snippet_json(root: &Path, location: &SourceLocation) -> Json {
    let Some(snippet) = read_snippet(root, location) else {
        return Json::Null;
    };
    Json::Object(vec![
        (
            "lines",
            Json::Array(
                snippet
                    .lines
                    .into_iter()
                    .map(|(line, text)| {
                        Json::Object(vec![
                            ("line", Json::Number(u64::from(line))),
                            ("text", Json::String(text)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "omitted_lines",
            Json::Number(u64::try_from(snippet.omitted).unwrap_or(u64::MAX)),
        ),
    ])
}

#[must_use]
pub fn findings_to_json_string(findings: &[Finding], root: &Path) -> String {
    Json::Array(
        findings
            .iter()
            .map(|finding| finding_json(finding, root))
            .collect(),
    )
    .to_json_string()
}

#[must_use]
pub fn finding_json(finding: &Finding, root: &Path) -> Json {
    Json::Object(vec![
        ("rule", Json::String(finding.rule_id.into())),
        ("rule_name", Json::String(finding.rule_name.into())),
        ("entity", Json::String(finding.entity_id.clone())),
        ("fingerprint", Json::String(finding.fingerprint.as_key())),
        (
            "severity",
            Json::String(severity_str(finding.severity).into()),
        ),
        ("confidence", Json::Float(f64::from(finding.confidence))),
        ("location", location_json(&finding.location)),
        ("snippet", snippet_json(root, &finding.location)),
        (
            "evidence",
            Json::Array(finding.evidence.iter().map(evidence_json).collect()),
        ),
    ])
}

fn evidence_json(evidence: &EvidenceItem) -> Json {
    Json::Object(vec![
        ("metric", Json::String(evidence.metric_key.into())),
        ("observed_value", Json::Float(evidence.observed_value)),
        ("normalized_score", Json::Float(evidence.normalized_score)),
        ("weight", Json::Float(evidence.weight)),
        ("contribution", Json::Float(evidence.contribution)),
        ("explanation", Json::String(evidence.explanation.clone())),
    ])
}

pub(crate) fn location_json(location: &SourceLocation) -> Json {
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

pub(crate) fn severity_str(severity: Severity) -> &'static str {
    match severity {
        Severity::Low => "low",
        Severity::Medium => "medium",
        Severity::High => "high",
        Severity::Critical => "critical",
    }
}
