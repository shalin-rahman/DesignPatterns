//! Exercises the whole git history pipeline (`log_commits` -> `resolve_changes`
//! -> `CoChangeGraph`) against a real, disposable Git repository rather than
//! mocked `git` output, so the `--name-only`/`--unified=0` parsing is
//! verified against what `git` actually prints.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use scent_domain::{Language, NormalizedPath, ProjectId};
use scent_git::{build_history, log_commits};
use scent_ir::ProjectIR;
use scent_parser::{extract_file, resolve_project, CSharpAdapter, DeclarationIndex, SourceFile};

struct TempRepo {
    path: PathBuf,
}

impl TempRepo {
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("scent-git-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        let repo = Self { path };
        repo.git(&["init", "--quiet"]);
        repo.git(&["config", "user.email", "test@example.com"]);
        repo.git(&["config", "user.name", "Test"]);
        repo
    }

    fn git(&self, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .status()
            .expect("git must be installed to run this test");
        assert!(status.success(), "git {args:?} failed");
    }

    fn write(&self, relative: &str, contents: &str) {
        let full_path = self.path.join(relative);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, contents).unwrap();
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "."]);
        self.git(&["commit", "--quiet", "-m", message]);
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn build_project(repo_root: &Path, path: &str) -> ProjectIR {
    let contents = fs::read_to_string(repo_root.join(path)).unwrap();
    let mut adapter = CSharpAdapter::new();
    let source = SourceFile {
        path: NormalizedPath::parse(path).unwrap(),
        contents,
    };
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);
    let project = ProjectIR {
        id: ProjectId::from_identity("csharp:git-history-sample"),
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

#[test]
fn logs_every_commit_with_its_changed_files() {
    let repo = TempRepo::new("log");
    repo.write("src/Order.cs", "class Order {}");
    repo.commit("add Order");
    repo.write("src/Order.cs", "class Order { void Ship() {} }");
    repo.commit("add Ship");

    let commits = log_commits(&repo.path).unwrap();
    assert_eq!(commits.len(), 2);
    assert!(
        commits
            .iter()
            .all(|commit| commit.changed_files
                == vec![NormalizedPath::parse("src/Order.cs").unwrap()])
    );
}

#[test]
fn builds_a_cochange_edge_for_methods_repeatedly_touched_together() {
    let repo = TempRepo::new("cochange");
    let source_v1 = "class Order {\n\
         void Ship()\n{\n    var x = 1;\n}\n\
         void Cancel()\n{\n    var y = 1;\n}\n\
     }\n";
    repo.write("src/Order.cs", source_v1);
    repo.commit("initial");

    for revision in 2..=4 {
        let source = format!(
            "class Order {{\n\
                 void Ship()\n{{\n    var x = {revision};\n}}\n\
                 void Cancel()\n{{\n    var y = {revision};\n}}\n\
             }}\n"
        );
        repo.write("src/Order.cs", &source);
        repo.commit(&format!("touch both methods, revision {revision}"));
    }

    let project = build_project(&repo.path, "src/Order.cs");
    let history = build_history(&repo.path, &project).expect("a git repo must produce history");
    assert_eq!(history.commits.len(), 4);

    let ship = project
        .methods
        .iter()
        .find(|method| method.name == "Ship")
        .unwrap();
    let cancel = project
        .methods
        .iter()
        .find(|method| method.name == "Cancel")
        .unwrap();
    let ship_ref = scent_graph::EntityRef::Method(ship.id.clone());
    let cancel_ref = scent_graph::EntityRef::Method(cancel.id.clone());

    let edge = history
        .cochange
        .edges()
        .iter()
        .find(|edge| {
            (edge.a == ship_ref && edge.b == cancel_ref)
                || (edge.a == cancel_ref && edge.b == ship_ref)
        })
        .expect("Ship and Cancel changed together in every commit");
    assert!(
        edge.commit_count() >= 3,
        "both methods changed together across every revision commit"
    );
}
