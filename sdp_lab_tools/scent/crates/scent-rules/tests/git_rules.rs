//! `DivergentChange`/`ShotgunSurgery` read `AnalysisContext::history`, which
//! `scent-git` normally builds from a real repository (see
//! `scent-git/tests/history.rs` for that end-to-end path). Here the same
//! `GitHistory` shape is built by hand from a resolved `ProjectIR`'s real
//! method IDs, so these tests stay focused on the rules' own evidence logic.

use scent_domain::{CommitId, Language, NormalizedPath, ProjectId, SuppressionMap};
use scent_git::{ChangeType, EntityChange};
use scent_git::{CoChangeGraph, Commit, GitHistory};
use scent_graph::{build_graph, EntityRef};
use scent_ir::ProjectIR;
use scent_metrics::build_metric_store;
use scent_parser::{extract_file, resolve_project, CSharpAdapter, DeclarationIndex, SourceFile};
use scent_rules::{default_registry, AnalysisContext};

const SOURCE: &str = "class Order { \
     void Ship() { var x = 1; } \
     void Cancel() { var y = 1; } \
     void Validate() { var z = 1; } \
 }";

fn build_project() -> ProjectIR {
    let mut adapter = CSharpAdapter::new();
    let source = SourceFile {
        path: NormalizedPath::parse("src/Order.cs").unwrap(),
        contents: SOURCE.into(),
    };
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);
    let project = ProjectIR {
        id: ProjectId::from_identity("csharp:git-rules-sample"),
        language: Language::CSharp,
        files: vec![extracted.file],
        namespaces: extracted.namespaces,
        types: extracted.types,
        methods: extracted.methods,
        fields: extracted.fields,
        properties: extracted.properties,
    };
    let index = DeclarationIndex::build(&project);
    resolve_project(project, &index)
}

fn commits(count: usize) -> Vec<Commit> {
    (0..count)
        .map(|i| Commit {
            id: CommitId::from_sha(&format!("c{i}")),
            timestamp: i64::try_from(i).unwrap_or(0) * 100,
            changed_files: vec![],
        })
        .collect()
}

fn evaluate(project: &ProjectIR, history: &GitHistory) -> Vec<scent_rules::Finding> {
    let graph = build_graph(project);
    let metrics = build_metric_store(project, &graph, &[]);
    let suppressions = SuppressionMap::default();
    default_registry().evaluate_all(&AnalysisContext {
        project,
        graph: &graph,
        metrics: &metrics,
        suppressions: &suppressions,
        clone_signatures: &[],
        switch_shapes: &[],
        history: Some(history),
    })
}

#[test]
fn produces_no_git_findings_without_history() {
    let project = build_project();
    let graph = build_graph(&project);
    let metrics = build_metric_store(&project, &graph, &[]);
    let suppressions = SuppressionMap::default();
    let findings = default_registry().evaluate_all(&AnalysisContext {
        project: &project,
        graph: &graph,
        metrics: &metrics,
        suppressions: &suppressions,
        clone_signatures: &[],
        switch_shapes: &[],
        history: None,
    });
    assert!(!findings.iter().any(|f| f.rule_id == "SHOTGUN_SURGERY"));
    assert!(!findings.iter().any(|f| f.rule_id == "DIVERGENT_CHANGE"));
}

#[test]
fn flags_shotgun_surgery_for_methods_that_repeatedly_cochange() {
    let project = build_project();
    let ship = project.methods.iter().find(|m| m.name == "Ship").unwrap();
    let cancel = project.methods.iter().find(|m| m.name == "Cancel").unwrap();
    let ship_ref = EntityRef::Method(ship.id.clone());
    let cancel_ref = EntityRef::Method(cancel.id.clone());

    let commits = commits(4);
    let changes: Vec<EntityChange> = commits
        .iter()
        .flat_map(|commit| {
            [ship_ref.clone(), cancel_ref.clone()]
                .into_iter()
                .map(|entity| EntityChange {
                    commit_id: commit.id.clone(),
                    entity,
                    change_type: ChangeType::Modified,
                })
        })
        .collect();
    let history = GitHistory {
        cochange: CoChangeGraph::build(&changes, &commits),
        commits,
    };

    let findings = evaluate(&project, &history);
    let shotgun: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "SHOTGUN_SURGERY")
        .collect();
    assert_eq!(
        shotgun.len(),
        2,
        "one finding per entity in the co-changing pair"
    );
}

#[test]
fn does_not_flag_shotgun_surgery_below_the_commit_threshold() {
    let project = build_project();
    let ship = project.methods.iter().find(|m| m.name == "Ship").unwrap();
    let cancel = project.methods.iter().find(|m| m.name == "Cancel").unwrap();
    let ship_ref = EntityRef::Method(ship.id.clone());
    let cancel_ref = EntityRef::Method(cancel.id.clone());

    let commits = commits(2);
    let changes: Vec<EntityChange> = commits
        .iter()
        .flat_map(|commit| {
            [ship_ref.clone(), cancel_ref.clone()]
                .into_iter()
                .map(|entity| EntityChange {
                    commit_id: commit.id.clone(),
                    entity,
                    change_type: ChangeType::Modified,
                })
        })
        .collect();
    let history = GitHistory {
        cochange: CoChangeGraph::build(&changes, &commits),
        commits,
    };

    let findings = evaluate(&project, &history);
    assert!(!findings.iter().any(|f| f.rule_id == "SHOTGUN_SURGERY"));
}

#[test]
fn flags_divergent_change_for_an_entity_with_many_distinct_partners() {
    let project = build_project();
    let ship = project.methods.iter().find(|m| m.name == "Ship").unwrap();
    let cancel = project.methods.iter().find(|m| m.name == "Cancel").unwrap();
    let validate = project
        .methods
        .iter()
        .find(|m| m.name == "Validate")
        .unwrap();
    let ship_ref = EntityRef::Method(ship.id.clone());
    let cancel_ref = EntityRef::Method(cancel.id.clone());
    let validate_ref = EntityRef::Method(validate.id.clone());

    // Ship changes in every commit; Cancel and Validate each share only
    // some of those commits with Ship, and never with each other — Ship's
    // changes correlate with two different, mostly-disjoint partners.
    let commits = commits(4);
    let mut changes = Vec::new();
    for commit in &commits {
        changes.push(EntityChange {
            commit_id: commit.id.clone(),
            entity: ship_ref.clone(),
            change_type: ChangeType::Modified,
        });
    }
    for commit in &commits[0..2] {
        changes.push(EntityChange {
            commit_id: commit.id.clone(),
            entity: cancel_ref.clone(),
            change_type: ChangeType::Modified,
        });
    }
    for commit in &commits[2..4] {
        changes.push(EntityChange {
            commit_id: commit.id.clone(),
            entity: validate_ref.clone(),
            change_type: ChangeType::Modified,
        });
    }
    let history = GitHistory {
        cochange: CoChangeGraph::build(&changes, &commits),
        commits,
    };

    let findings = evaluate(&project, &history);
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "DIVERGENT_CHANGE" && f.entity_id == ship.id.as_str()));
}

#[test]
fn does_not_flag_divergent_change_for_a_tightly_coupled_pair() {
    let project = build_project();
    let ship = project.methods.iter().find(|m| m.name == "Ship").unwrap();
    let cancel = project.methods.iter().find(|m| m.name == "Cancel").unwrap();
    let ship_ref = EntityRef::Method(ship.id.clone());
    let cancel_ref = EntityRef::Method(cancel.id.clone());

    let commits = commits(4);
    let changes: Vec<EntityChange> = commits
        .iter()
        .flat_map(|commit| {
            [ship_ref.clone(), cancel_ref.clone()]
                .into_iter()
                .map(|entity| EntityChange {
                    commit_id: commit.id.clone(),
                    entity,
                    change_type: ChangeType::Modified,
                })
        })
        .collect();
    let history = GitHistory {
        cochange: CoChangeGraph::build(&changes, &commits),
        commits,
    };

    let findings = evaluate(&project, &history);
    assert!(!findings.iter().any(|f| f.rule_id == "DIVERGENT_CHANGE"));
}
