//! LLM- and CI-friendly JSON for `Finding`s (`docs/prompt.md` §32).

use scent_domain::{Severity, SourceLocation};
use scent_rules::{EvidenceItem, Finding};

use crate::json::Json;

#[must_use]
pub fn findings_to_json_string(findings: &[Finding]) -> String {
    Json::Array(findings.iter().map(finding_json).collect()).to_json_string()
}

#[must_use]
pub fn finding_json(finding: &Finding) -> Json {
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
