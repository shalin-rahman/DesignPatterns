//! Orchestrates discovery -> parse -> extract -> index -> resolve into one
//! `ProjectIR`, alongside the diagnostics and suppressions collected along
//! the way.

use std::{fs, io, path::Path};

use scent_config::ScentConfig;
use scent_domain::{
    Diagnostic, Language, MethodId, NormalizedPath, ProjectId, Severity, SuppressionMap,
};
use scent_git::{build_history, GitHistory};
use scent_graph::{build_graph, DependencyGraph};
use scent_ir::ProjectIR;
use scent_metrics::{build_metric_store, MetricStore};
use scent_parser::{
    discover_csharp, extract_file, resolve_project, CSharpAdapter, CloneSignature,
    ComplexitySignals, DeclarationIndex, DiscoveryError, DiscoveryOptions, DiscoveryResult,
    SourceFile, SwitchShape,
};
use scent_rules::{
    assess_principle_risks, build_registry, recommend_patterns, recommend_refactorings,
    AnalysisContext, Finding, PatternRecommendation, PrincipleRisk, RefactoringRecommendation,
};

#[derive(Debug)]
pub enum AnalyzeError {
    Discovery(DiscoveryError),
    ReadSource {
        path: NormalizedPath,
        source: io::Error,
    },
}

impl From<DiscoveryError> for AnalyzeError {
    fn from(value: DiscoveryError) -> Self {
        Self::Discovery(value)
    }
}

#[derive(Debug)]
pub struct AnalysisReport {
    pub discovery: DiscoveryResult,
    pub project: ProjectIR,
    pub diagnostics: Vec<Diagnostic>,
    pub suppressions: SuppressionMap,
    pub graph: DependencyGraph,
    pub metrics: MetricStore,
    pub findings: Vec<Finding>,
    /// `None` when `root` is not a Git work tree (or `git` itself is
    /// unavailable) — the git-backed rules already degrade gracefully in
    /// that case, so this is not surfaced as an error.
    pub git_history: Option<GitHistory>,
    pub principle_risks: Vec<PrincipleRisk>,
    pub pattern_recommendations: Vec<PatternRecommendation>,
    pub refactoring_recommendations: Vec<RefactoringRecommendation>,
}

/// Runs the whole Phase 1 pipeline over a C# project rooted at `root`.
///
/// # Errors
///
/// Returns an error when discovery fails or a discovered source file cannot
/// be read from disk.
pub fn analyze_path(root: &Path) -> Result<AnalysisReport, AnalyzeError> {
    analyze_path_with_progress(root, |_| {})
}

/// Same pipeline as [`analyze_path`], but calls `on_progress` with a short,
/// human-readable line as each stage starts, so a caller (the CLI) can show
/// the user the pipeline is actually working rather than going silent until
/// the final report.
///
/// # Errors
///
/// Returns an error when discovery fails or a discovered source file cannot
/// be read from disk.
pub fn analyze_path_with_progress(
    root: &Path,
    on_progress: impl FnMut(&str),
) -> Result<AnalysisReport, AnalyzeError> {
    analyze_path_inner(root, on_progress, true)
}

/// Same pipeline as [`analyze_path`], but skips git-history analysis
/// entirely (`AnalysisReport::git_history` is always `None`, and the two
/// git-backed rules produce no findings) — for analyzing a checkout that
/// isn't a Git work tree at all, or when history simply isn't wanted for
/// this run.
///
/// # Errors
///
/// Returns an error when discovery fails or a discovered source file cannot
/// be read from disk.
pub fn analyze_path_without_git_history(root: &Path) -> Result<AnalysisReport, AnalyzeError> {
    analyze_path_inner(root, |_| {}, false)
}

fn analyze_path_inner(
    root: &Path,
    mut on_progress: impl FnMut(&str),
    check_git_history: bool,
) -> Result<AnalysisReport, AnalyzeError> {
    on_progress(&format!(
        "Discovering C# project files under {}",
        root.display()
    ));
    let discovery = discover_csharp(root, &DiscoveryOptions::default())?;
    on_progress(&format!(
        "Discovered {} solution file(s), {} project file(s), {} source file(s), {} excluded",
        discovery.solution_files.len(),
        discovery.project_files.len(),
        discovery.source_files.len(),
        discovery.excluded_source_files.len(),
    ));

    let ParsedFiles {
        project,
        diagnostics,
        suppressions,
        method_complexity,
        method_clone_signatures,
        method_switches,
    } = parse_all_files(root, &discovery, &mut on_progress)?;

    on_progress("Building declaration index");
    let index = DeclarationIndex::build(&project);
    on_progress("Resolving intra-project references");
    let project = resolve_project(project, &index);
    on_progress("Building the dependency graph");
    let graph = build_graph(&project);
    on_progress("Calculating metrics (LOC, CC, nesting, LCOM4, CBO)");
    let metrics = build_metric_store(&project, &graph, &method_complexity);
    let git_history = if check_git_history {
        on_progress("Reading Git history (co-change graph)");
        build_history(root, &project)
    } else {
        None
    };
    on_progress("Loading configuration (smell_detector.toml)");
    let config = ScentConfig::load(root);
    on_progress("Evaluating rules");
    let ctx = AnalysisContext {
        project: &project,
        graph: &graph,
        metrics: &metrics,
        suppressions: &suppressions,
        clone_signatures: &method_clone_signatures,
        switch_shapes: &method_switches,
        history: git_history.as_ref(),
    };
    let mut findings = build_registry(&config).evaluate_all(&ctx);
    for finding in &mut findings {
        if let Some(severity) = config.severity_override(finding.rule_id) {
            if let Some(parsed) = parse_severity(severity) {
                finding.severity = parsed;
            }
        }
    }
    on_progress("Assessing principle risks, pattern and refactoring recommendations");
    let principle_risks = assess_principle_risks(&ctx, &findings);
    let pattern_recommendations = recommend_patterns(&ctx);
    let refactoring_recommendations = recommend_refactorings(&findings);
    on_progress("Analysis complete");

    Ok(AnalysisReport {
        discovery,
        project,
        diagnostics,
        suppressions,
        graph,
        metrics,
        findings,
        git_history,
        principle_risks,
        pattern_recommendations,
        refactoring_recommendations,
    })
}

struct ParsedFiles {
    project: ProjectIR,
    diagnostics: Vec<Diagnostic>,
    suppressions: SuppressionMap,
    method_complexity: Vec<(MethodId, ComplexitySignals)>,
    method_clone_signatures: Vec<(MethodId, CloneSignature)>,
    method_switches: Vec<(MethodId, Vec<SwitchShape>)>,
}

/// Parses and extracts every discovered source file into one `ProjectIR`
/// plus its parallel per-method signal lists, in deterministic
/// (sorted-path) order.
fn parse_all_files(
    root: &Path,
    discovery: &DiscoveryResult,
    on_progress: &mut impl FnMut(&str),
) -> Result<ParsedFiles, AnalyzeError> {
    let mut adapter = CSharpAdapter::new();
    let mut project = ProjectIR {
        id: project_id(discovery),
        language: Language::CSharp,
        files: vec![],
        namespaces: vec![],
        types: vec![],
        methods: vec![],
        fields: vec![],
        properties: vec![],
    };
    let mut diagnostics = Vec::new();
    let mut suppressions = SuppressionMap::default();
    let mut method_complexity: Vec<(MethodId, ComplexitySignals)> = Vec::new();
    let mut method_clone_signatures: Vec<(MethodId, CloneSignature)> = Vec::new();
    let mut method_switches: Vec<(MethodId, Vec<SwitchShape>)> = Vec::new();

    let mut source_files = discovery.source_files.clone();
    source_files.sort();
    for (index, path) in source_files.iter().enumerate() {
        on_progress(&format!(
            "Parsing [{}/{}] {path}",
            index + 1,
            source_files.len()
        ));
        let contents = fs::read_to_string(root.join(path.as_str())).map_err(|source| {
            AnalyzeError::ReadSource {
                path: path.clone(),
                source,
            }
        })?;
        suppressions = suppressions.merge(SuppressionMap::scan(path, &contents));

        let source = SourceFile {
            path: path.clone(),
            contents,
        };
        let parsed = adapter.parse(&source);
        diagnostics.extend(parsed.diagnostics);
        let extracted = extract_file(&source, &parsed.tree);
        project.files.push(extracted.file);
        project.namespaces.extend(extracted.namespaces);
        project.types.extend(extracted.types);
        project.methods.extend(extracted.methods);
        project.fields.extend(extracted.fields);
        project.properties.extend(extracted.properties);
        method_complexity.extend(extracted.method_complexity);
        method_clone_signatures.extend(extracted.method_clone_signatures);
        method_switches.extend(extracted.method_switches);
    }

    Ok(ParsedFiles {
        project,
        diagnostics,
        suppressions,
        method_complexity,
        method_clone_signatures,
        method_switches,
    })
}

/// Matches `smell_detector.toml`'s `severity = "..."` string against
/// `Severity`'s SARIF-style spelling (`docs/prompt.md` §36's own example
/// uses `"warning"`, which SARIF/most tools spell `"medium"` in this
/// engine's four-level scale — accepted as an alias so the example config
/// in the spec works as written). An unrecognized spelling leaves the
/// finding's own evidence-computed severity untouched rather than guessing.
fn parse_severity(value: &str) -> Option<Severity> {
    match value.to_ascii_lowercase().as_str() {
        "low" => Some(Severity::Low),
        "medium" | "warning" => Some(Severity::Medium),
        "high" => Some(Severity::High),
        "critical" | "error" => Some(Severity::Critical),
        _ => None,
    }
}

/// Derived only from repository-relative paths, never the absolute
/// filesystem root, so identical repository content produces the same
/// `ProjectId` on any machine.
fn project_id(discovery: &DiscoveryResult) -> ProjectId {
    let mut identity_parts: Vec<&str> = discovery
        .project_files
        .iter()
        .map(NormalizedPath::as_str)
        .chain(discovery.solution_files.iter().map(NormalizedPath::as_str))
        .chain(discovery.source_files.iter().map(NormalizedPath::as_str))
        .collect();
    identity_parts.sort_unstable();
    ProjectId::from_identity(&format!(
        "scent:v1:csharp:project:{}",
        identity_parts.join(",")
    ))
}
