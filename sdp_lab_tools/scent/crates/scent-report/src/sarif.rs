//! Minimal, valid, deterministic SARIF 2.1.0 output. Rules and results are
//! sorted before serialization so identical findings always produce a
//! byte-identical document.

use std::collections::BTreeSet;

use scent_domain::{Severity, SourceLocation};
use scent_rules::Finding;

use crate::findings::severity_str;
use crate::json::Json;

#[must_use]
pub fn to_sarif(findings: &[Finding]) -> String {
    let mut sorted: Vec<&Finding> = findings.iter().collect();
    sorted.sort_by_key(|finding| finding.fingerprint.as_key());

    let mut rule_ids: BTreeSet<(&str, &str)> = BTreeSet::new();
    for finding in &sorted {
        rule_ids.insert((finding.rule_id, finding.rule_name));
    }

    Json::Object(vec![
        (
            "$schema",
            Json::String(
                "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json"
                    .into(),
            ),
        ),
        ("version", Json::String("2.1.0".into())),
        (
            "runs",
            Json::Array(vec![Json::Object(vec![
                (
                    "tool",
                    Json::Object(vec![(
                        "driver",
                        Json::Object(vec![
                            ("name", Json::String("SCENT".into())),
                            (
                                "rules",
                                Json::Array(
                                    rule_ids
                                        .iter()
                                        .map(|(id, name)| {
                                            Json::Object(vec![
                                                ("id", Json::String((*id).to_owned())),
                                                (
                                                    "name",
                                                    Json::String((*name).to_owned()),
                                                ),
                                            ])
                                        })
                                        .collect(),
                                ),
                            ),
                        ]),
                    )]),
                ),
                (
                    "results",
                    Json::Array(sorted.iter().map(|finding| result_json(finding)).collect()),
                ),
            ])]),
        ),
    ])
    .to_json_string()
}

fn result_json(finding: &Finding) -> Json {
    Json::Object(vec![
        ("ruleId", Json::String(finding.rule_id.into())),
        ("level", Json::String(sarif_level(finding.severity).into())),
        (
            "message",
            Json::Object(vec![(
                "text",
                Json::String(format!(
                    "{} ({:.0}% confidence)",
                    finding.rule_name,
                    f64::from(finding.confidence) * 100.0
                )),
            )]),
        ),
        (
            "locations",
            Json::Array(vec![Json::Object(vec![(
                "physicalLocation",
                physical_location_json(&finding.location),
            )])]),
        ),
        (
            "properties",
            Json::Object(vec![
                ("confidence", Json::Float(f64::from(finding.confidence))),
                (
                    "severity",
                    Json::String(severity_str(finding.severity).into()),
                ),
            ]),
        ),
    ])
}

fn physical_location_json(location: &SourceLocation) -> Json {
    Json::Object(vec![
        (
            "artifactLocation",
            Json::Object(vec![(
                "uri",
                Json::String(location.path.as_str().to_owned()),
            )]),
        ),
        (
            "region",
            Json::Object(vec![
                (
                    "startLine",
                    Json::Number(u64::from(location.range.start.line)),
                ),
                (
                    "startColumn",
                    Json::Number(u64::from(location.range.start.column)),
                ),
                ("endLine", Json::Number(u64::from(location.range.end.line))),
                (
                    "endColumn",
                    Json::Number(u64::from(location.range.end.column)),
                ),
            ]),
        ),
    ])
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
    }
}
