//! SCENT command-line entry point.

use std::{env, fs, path::PathBuf, process::ExitCode};

use scent_config::ScentConfig;
use scent_core::{analyze_path_with_progress, to_json, AnalysisReport};
use scent_report::{evaluate_gate, to_sarif, Baseline, QualityGateConfig};
use scent_rules::default_registry;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("-h" | "--help") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("analyze") => run_analyze(&args.collect::<Vec<_>>()),
        Some("rules") => run_rules(),
        Some("gate") => run_gate(&args.collect::<Vec<_>>()),
        Some(other) => {
            eprintln!("scent: unrecognized command '{other}'");
            if let Some(suggestion) = closest_command(other) {
                eprintln!("       did you mean '{suggestion}'?");
            }
            print_usage();
            ExitCode::FAILURE
        }
    }
}

const KNOWN_COMMANDS: &[&str] = &["analyze", "rules", "gate", "--help"];

// Suggests the closest known command for a typo, the same idea as
// git/cargo's "did you mean" — only offered within a small edit distance
// so an unrelated word doesn't produce a misleading suggestion.
fn closest_command(input: &str) -> Option<&'static str> {
    KNOWN_COMMANDS
        .iter()
        .map(|&candidate| (candidate, levenshtein(input, candidate)))
        .filter(|&(_, distance)| distance <= 2)
        .min_by_key(|&(_, distance)| distance)
        .map(|(candidate, _)| candidate)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();

    for (i, &ca) in a.iter().enumerate() {
        let mut prev_diagonal = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let temp = row[j + 1];
            row[j + 1] = if ca == cb {
                prev_diagonal
            } else {
                1 + prev_diagonal.min(row[j]).min(row[j + 1])
            };
            prev_diagonal = temp;
        }
    }

    row[b.len()]
}

fn run_analyze(rest: &[String]) -> ExitCode {
    if has_flag(rest, "-h") || has_flag(rest, "--help") {
        print_analyze_help();
        return ExitCode::SUCCESS;
    }
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
                "table" => print_table(path, &report),
                _ => print_summary(path, &report),
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("scent analyze failed: {error:?}");
            ExitCode::FAILURE
        }
    }
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn run_rules() -> ExitCode {
    println!("Registered rules");
    for rule in default_registry().all() {
        println!("  {:<24} {}", rule.id(), rule.name());
    }
    ExitCode::SUCCESS
}

fn run_gate(rest: &[String]) -> ExitCode {
    if has_flag(rest, "-h") || has_flag(rest, "--help") {
        print_gate_help();
        return ExitCode::SUCCESS;
    }
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
    eprintln!("  scent analyze <path> [--format human|json|sarif|table]");
    eprintln!("  scent rules");
    eprintln!("  scent gate <path> [--max-critical N] [--max-high N] [--baseline FILE]");
    eprintln!("  scent --help                (show full help)");
    eprintln!("  scent analyze --help        (show analyze's flags)");
}

fn print_help() {
    println!("scent — static smell/design analysis for C# projects");
    println!();
    println!("USAGE:");
    println!("  scent <COMMAND> [ARGS]");
    println!();
    println!("COMMANDS:");
    println!("  analyze <path>   Analyze a C# solution/project directory and report findings");
    println!("  rules            List every registered rule id and name");
    println!("  gate <path>      Analyze, then pass/fail against a quality gate");
    println!();
    println!("<path> can be any directory on disk — inside or outside this repo — that");
    println!("contains a .sln, .csproj, or .cs files. scent only reads it; it never runs");
    println!("MSBuild or dotnet.");
    println!();
    println!("Run 'scent analyze --help' or 'scent gate --help' for a command's own flags.");
    println!("See docs/CLI_WORKFLOW_GUIDE.md for real examples of every command.");
}

fn print_analyze_help() {
    println!("scent analyze <path> [OPTIONS]");
    println!();
    println!("Analyze a C# solution/project directory. <path> can point anywhere on");
    println!("disk, including a directory outside this repository.");
    println!();
    println!("OPTIONS:");
    println!("  --format human   Readable summary with findings, risks, recommendations (default)");
    println!("  --format json    Full machine-readable report — see docs/JSON_OUTPUT_REFERENCE.md");
    println!("  --format sarif   SARIF output for editors/CI (e.g. GitHub code scanning)");
    println!("  --format table   Findings and principle risks as aligned columns");
    println!("  -h, --help       Show this message");
    println!();
    println!("A `smell_detector.toml` file in <path> overrides rule thresholds/severity");
    println!("and quality-gate limits — see docs/architecture.md.");
}

fn print_gate_help() {
    println!("scent gate <path> [OPTIONS]");
    println!();
    println!("Analyze <path>, then pass or fail against a quality gate. Exits 0 on");
    println!("PASSED, 1 on FAILED — safe to use as a CI step.");
    println!();
    println!("OPTIONS:");
    println!("  --max-critical N   Fail if more than N Critical findings (default from");
    println!("                     smell_detector.toml, else built-in default)");
    println!("  --max-high N       Fail if more than N High findings");
    println!("  --baseline FILE    Ignore findings already present in this baseline file");
    println!("  -h, --help         Show this message");
}

fn print_summary(path: &str, report: &AnalysisReport) {
    println!("SCENT");
    println!("Project: {path}");
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

/// Aligned-column view of the same findings and principle risks the human
/// summary prints as free text — for pasting into a terminal or a plain
/// text file where columns need to line up.
fn print_table(path: &str, report: &AnalysisReport) {
    println!("Project: {path}");
    println!();

    println!("FINDINGS ({})", report.findings.len());
    if report.findings.is_empty() {
        println!("  (none)");
    } else {
        println!("  {:<10} {:<24} {:<8} ENTITY", "SEVERITY", "RULE", "CONF %");
        for finding in &report.findings {
            println!(
                "  {:<10} {:<24} {:<8.0} {}",
                format!("{:?}", finding.severity),
                finding.rule_name,
                f64::from(finding.confidence) * 100.0,
                finding.entity_id
            );
        }
    }
    println!();

    println!("PRINCIPLE RISKS ({})", report.principle_risks.len());
    if report.principle_risks.is_empty() {
        println!("  (none)");
    } else {
        println!(
            "  {:<8} {:<14} {:<8} EXPLANATION",
            "RISK", "PRINCIPLE", "CONF %"
        );
        for risk in &report.principle_risks {
            println!(
                "  {:<8} {:<14} {:<8.0} {}",
                format!("{:?}", risk.risk),
                format!("{:?}", risk.principle),
                f64::from(risk.confidence) * 100.0,
                risk.explanation
            );
        }
    }
}
