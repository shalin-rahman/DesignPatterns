//! SCENT command-line entry point.

use std::{env, fs, path::PathBuf, process::ExitCode};

use scent_config::ScentConfig;
use scent_core::{analyze_path_with_progress, to_json, AnalysisReport};
use scent_report::{evaluate_gate, to_sarif, Baseline, QualityGateConfig};
use scent_rules::default_registry;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("analyze") => run_analyze(&args.collect::<Vec<_>>()),
        Some("rules") => run_rules(),
        Some("gate") => run_gate(&args.collect::<Vec<_>>()),
        _ => {
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn run_analyze(rest: &[String]) -> ExitCode {
    let Some(path) = rest.first() else {
        print_usage();
        return ExitCode::FAILURE;
    };
    let format = flag_value(rest, "--format").unwrap_or("human");

    match analyze_with_progress(path) {
        Ok(report) => {
            match format {
                "json" => println!("{}", to_json(&report)),
                "sarif" => println!("{}", to_sarif(&report.findings)),
                _ => print_summary(&report),
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("scent analyze failed: {error:?}");
            ExitCode::FAILURE
        }
    }
}

fn run_rules() -> ExitCode {
    println!("Registered rules");
    for rule in default_registry().all() {
        println!("  {:<24} {}", rule.id(), rule.name());
    }
    ExitCode::SUCCESS
}

fn run_gate(rest: &[String]) -> ExitCode {
    let Some(path) = rest.first() else {
        print_usage();
        return ExitCode::FAILURE;
    };
    // `smell_detector.toml`'s `[quality_gate]` section is the base; CLI
    // flags are the more immediate, explicit intent and override it.
    let mut config = QualityGateConfig::default();
    if let Some(overrides) = ScentConfig::load(std::path::Path::new(path)).quality_gate {
        if let Some(value) = overrides.max_critical_findings {
            config.max_critical_findings = value;
        }
        if let Some(value) = overrides.max_high_findings {
            config.max_high_findings = value;
        }
        if let Some(value) = overrides.fail_on_new_violations {
            config.fail_on_new_violations = value;
        }
    }
    if let Some(value) = flag_value(rest, "--max-critical").and_then(|v| v.parse().ok()) {
        config.max_critical_findings = value;
    }
    if let Some(value) = flag_value(rest, "--max-high").and_then(|v| v.parse().ok()) {
        config.max_high_findings = value;
    }
    let baseline = flag_value(rest, "--baseline").map(|baseline_path| {
        fs::read_to_string(baseline_path)
            .map_or_else(|_| Baseline::default(), |text| Baseline::parse(&text))
    });

    match analyze_with_progress(path) {
        Ok(report) => {
            let result = evaluate_gate(&report.findings, baseline.as_ref(), &config);
            if result.passed {
                println!("scent gate: PASSED ({} finding(s))", report.findings.len());
                ExitCode::SUCCESS
            } else {
                println!("scent gate: FAILED");
                for reason in &result.reasons {
                    println!("  - {reason}");
                }
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("scent gate failed: {error:?}");
            ExitCode::FAILURE
        }
    }
}

fn analyze_with_progress(path: &str) -> Result<AnalysisReport, scent_core::AnalyzeError> {
    // Progress goes to stderr so stdout stays a clean, pipeable report (JSON,
    // SARIF, or human summary) with no interleaved status noise.
    analyze_path_with_progress(&PathBuf::from(path), |message| {
        eprintln!("[scent] {message}");
    })
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  scent analyze <path> [--format human|json|sarif]");
    eprintln!("  scent rules");
    eprintln!("  scent gate <path> [--max-critical N] [--max-high N] [--baseline FILE]");
}

fn print_summary(report: &AnalysisReport) {
    println!("SCENT");
    println!();
    println!("Discovery");
    println!("  Solutions: {}", report.discovery.solution_files.len());
    println!("  Projects:  {}", report.discovery.project_files.len());
    println!("  Sources:   {}", report.discovery.source_files.len());
    println!(
        "  Excluded:  {}",
        report.discovery.excluded_source_files.len()
    );
    println!();
    println!("Semantic IR");
    println!("  Namespaces: {}", report.project.namespaces.len());
    println!("  Types:      {}", report.project.types.len());
    println!("  Methods:    {}", report.project.methods.len());
    println!("  Fields:     {}", report.project.fields.len());
    println!("  Properties: {}", report.project.properties.len());
    println!();
    println!("Metrics");
    println!(
        "  Methods measured: {}",
        report
            .metrics
            .iter()
            .filter(|((_, kind), _)| *kind == scent_metrics::MetricKind::CyclomaticComplexity)
            .count()
    );
    println!(
        "  Types measured:   {}",
        report
            .metrics
            .iter()
            .filter(|((_, kind), _)| *kind == scent_metrics::MetricKind::Lcom4)
            .count()
    );
    println!("  Dependency edges: {}", report.graph.edges().len());
    println!();
    println!("Findings: {}", report.findings.len());
    for finding in &report.findings {
        println!(
            "  [{:?}] {} — {} (confidence {:.0}%)",
            finding.severity,
            finding.rule_name,
            finding.entity_id,
            f64::from(finding.confidence) * 100.0
        );
    }
    println!();
    println!("Principle risks: {}", report.principle_risks.len());
    for risk in &report.principle_risks {
        // `explanation` already states its own risk level ("Medium KISS
        // risk: ..."), so it is printed alone rather than alongside a
        // second, redundant `{:?}` of the same value.
        println!("  {}", risk.explanation);
    }
    println!();
    println!(
        "Pattern recommendations: {}",
        report.pattern_recommendations.len()
    );
    for recommendation in &report.pattern_recommendations {
        println!(
            "  {:?} — {}",
            recommendation.candidate, recommendation.explanation
        );
    }
    println!();
    println!(
        "Refactoring recommendations: {}",
        report.refactoring_recommendations.len()
    );
    for recommendation in &report.refactoring_recommendations {
        println!(
            "  {} -> {:?}",
            recommendation.entity_id, recommendation.refactoring
        );
    }
    println!();
    println!("Diagnostics: {}", report.diagnostics.len());
}
