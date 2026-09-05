//! Loads `smell_detector.toml` (`docs/prompt.md` §36): per-rule
//! enabled/severity/threshold overrides and quality-gate defaults. A
//! project without this file, or with one that fails to parse, is not an
//! error — it just gets every rule at its built-in defaults, since a
//! config file is optional supporting data, not a required input.

mod toml;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub use toml::{parse as parse_toml, TomlDocument, TomlValue};

pub const CONFIG_FILE_NAME: &str = "smell_detector.toml";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuleConfig {
    pub enabled: bool,
    pub severity: Option<String>,
    pub thresholds: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QualityGateOverrides {
    pub max_critical_findings: Option<u32>,
    pub max_high_findings: Option<u32>,
    pub fail_on_new_violations: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScentConfig {
    /// Keyed by rule id (e.g. `"LONG_METHOD"`). A rule id absent here has
    /// no overrides at all — every rule is enabled by default even though
    /// `RuleConfig::default().enabled` is `false`, because absence from
    /// this map (not an explicit `enabled = false`) is what "no override"
    /// means; see [`ScentConfig::rule_enabled`]/[`ScentConfig::thresholds_for`].
    pub rules: BTreeMap<String, RuleConfig>,
    pub quality_gate: Option<QualityGateOverrides>,
}

impl ScentConfig {
    /// Looks for `smell_detector.toml` directly under `root`. Never
    /// returns an error: a missing or unparseable file just means no
    /// overrides.
    #[must_use]
    pub fn load(root: &Path) -> Self {
        let Ok(text) = fs::read_to_string(root.join(CONFIG_FILE_NAME)) else {
            return Self::default();
        };
        Self::from_toml(&parse_toml(&text))
    }

    #[must_use]
    pub fn from_toml(doc: &TomlDocument) -> Self {
        let mut rules: BTreeMap<String, RuleConfig> = BTreeMap::new();
        for (rest, value) in doc.entries_under("rules.") {
            let Some((rule_id, field)) = rest.split_once('.') else {
                continue;
            };
            let entry = rules
                .entry(rule_id.to_owned())
                .or_insert_with(|| RuleConfig {
                    enabled: true,
                    severity: None,
                    thresholds: BTreeMap::new(),
                });
            if field == "enabled" {
                if let Some(enabled) = value.as_bool() {
                    entry.enabled = enabled;
                }
            } else if field == "severity" {
                if let Some(severity) = value.as_str() {
                    entry.severity = Some(severity.to_owned());
                }
            } else if let Some(metric_key) = field.strip_prefix("thresholds.") {
                if let Some(threshold) = value.as_f64() {
                    entry.thresholds.insert(metric_key.to_owned(), threshold);
                }
            }
        }

        let quality_gate = if doc.entries_under("quality_gate.").next().is_some() {
            Some(QualityGateOverrides {
                max_critical_findings: doc
                    .get("quality_gate.max_critical_findings")
                    .and_then(TomlValue::as_f64)
                    .map(clamp_to_u32),
                max_high_findings: doc
                    .get("quality_gate.max_high_findings")
                    .and_then(TomlValue::as_f64)
                    .map(clamp_to_u32),
                fail_on_new_violations: doc
                    .get("quality_gate.fail_on_new_violations")
                    .and_then(TomlValue::as_bool),
            })
        } else {
            None
        };

        Self {
            rules,
            quality_gate,
        }
    }

    /// `true` unless this rule id has an explicit `enabled = false` entry.
    #[must_use]
    pub fn rule_enabled(&self, rule_id: &str) -> bool {
        self.rules.get(rule_id).is_none_or(|rule| rule.enabled)
    }

    /// The configured override for `rule_id`'s `metric_key` threshold, if
    /// any — falls back to the rule's own built-in default when absent.
    #[must_use]
    pub fn threshold(&self, rule_id: &str, metric_key: &str, default: f64) -> f64 {
        self.rules
            .get(rule_id)
            .and_then(|rule| rule.thresholds.get(metric_key))
            .copied()
            .unwrap_or(default)
    }

    #[must_use]
    pub fn severity_override(&self, rule_id: &str) -> Option<&str> {
        self.rules
            .get(rule_id)
            .and_then(|rule| rule.severity.as_deref())
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn clamp_to_u32(value: f64) -> u32 {
    if value <= 0.0 {
        0
    } else if value >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        value as u32
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // exact literal values round-trip through this small integer/float parser
mod tests {
    use super::{parse_toml, ScentConfig};

    #[test]
    fn a_missing_file_produces_every_default() {
        let config = ScentConfig::load(std::path::Path::new(
            "this-directory-does-not-exist-in-any-checkout",
        ));
        assert!(config.rule_enabled("LONG_METHOD"));
        assert_eq!(config.threshold("LONG_METHOD", "loc", 30.0), 30.0);
    }

    #[test]
    fn a_threshold_override_replaces_the_default() {
        let doc = parse_toml("[rules.LONG_METHOD.thresholds]\nloc = 50\n");
        let config = ScentConfig::from_toml(&doc);
        assert_eq!(config.threshold("LONG_METHOD", "loc", 30.0), 50.0);
        assert_eq!(
            config.threshold("LONG_METHOD", "cyclomatic_complexity", 10.0),
            10.0
        );
    }

    #[test]
    fn a_disabled_rule_reports_disabled() {
        let doc = parse_toml("[rules.LARGE_CLASS]\nenabled = false\n");
        let config = ScentConfig::from_toml(&doc);
        assert!(!config.rule_enabled("LARGE_CLASS"));
        assert!(config.rule_enabled("LONG_METHOD"));
    }

    #[test]
    fn a_severity_override_is_readable() {
        let doc = parse_toml("[rules.LONG_METHOD]\nseverity = \"critical\"\n");
        let config = ScentConfig::from_toml(&doc);
        assert_eq!(config.severity_override("LONG_METHOD"), Some("critical"));
    }

    #[test]
    fn quality_gate_overrides_are_parsed() {
        let doc = parse_toml(
            "[quality_gate]\nmax_critical_findings = 0\nmax_high_findings = 10\nfail_on_new_violations = true\n",
        );
        let config = ScentConfig::from_toml(&doc);
        let gate = config.quality_gate.unwrap();
        assert_eq!(gate.max_critical_findings, Some(0));
        assert_eq!(gate.max_high_findings, Some(10));
        assert_eq!(gate.fail_on_new_violations, Some(true));
    }
}
