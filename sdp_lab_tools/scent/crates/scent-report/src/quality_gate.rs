//! Quality gate: pass/fail against configured finding-count limits, plus an
//! optional baseline so pre-existing findings never fail the build on their
//! own (`docs/prompt.md` §40).

use scent_domain::Severity;
use scent_rules::Finding;

use crate::baseline::Baseline;

#[derive(Clone, Debug)]
pub struct QualityGateConfig {
    pub max_critical_findings: u32,
    pub max_high_findings: u32,
    pub fail_on_new_violations: bool,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            max_critical_findings: 0,
            max_high_findings: 10,
            fail_on_new_violations: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GateResult {
    pub passed: bool,
    pub reasons: Vec<String>,
}

#[must_use]
pub fn evaluate_gate(
    findings: &[Finding],
    baseline: Option<&Baseline>,
    config: &QualityGateConfig,
) -> GateResult {
    let mut reasons = Vec::new();
    let critical = count_u32(findings, Severity::Critical);
    let high = count_u32(findings, Severity::High);

    if critical > config.max_critical_findings {
        reasons.push(format!(
            "{critical} critical finding(s) exceed the allowed {}",
            config.max_critical_findings
        ));
    }
    if high > config.max_high_findings {
        reasons.push(format!(
            "{high} high finding(s) exceed the allowed {}",
            config.max_high_findings
        ));
    }
    if config.fail_on_new_violations {
        if let Some(baseline) = baseline {
            let new_count = baseline.new_findings(findings).len();
            if new_count > 0 {
                reasons.push(format!(
                    "{new_count} new finding(s) not present in baseline"
                ));
            }
        }
    }

    GateResult {
        passed: reasons.is_empty(),
        reasons,
    }
}

fn count_u32(findings: &[Finding], severity: Severity) -> u32 {
    let count = findings
        .iter()
        .filter(|finding| finding.severity == severity)
        .count();
    u32::try_from(count).unwrap_or(u32::MAX)
}
