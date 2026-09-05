//! Git history analysis (`docs/prompt.md` §14-15): commit diffing,
//! entity-resolved changes, and the resulting co-change graph. Static syntax
//! alone cannot detect Divergent Change or Shotgun Surgery — both need real
//! commit history, read here and nowhere else in the workspace.

pub mod cochange;
pub mod commits;
pub mod entities;

pub use cochange::{CoChangeEdge, CoChangeGraph};
pub use commits::{log_commits, Commit, GitError};
pub use entities::{resolve_changes, ChangeType, EntityChange};

use std::path::Path;

use scent_ir::ProjectIR;

/// Everything the git-backed rules need: the raw commit list (for
/// timestamps/evidence text) and the co-change graph built from it.
#[derive(Clone, Debug, Default)]
pub struct GitHistory {
    pub commits: Vec<Commit>,
    pub cochange: CoChangeGraph,
}

/// Builds the full git history view for `root`, or `None` when `root` is
/// not a Git work tree (or `git` itself is unavailable) — the git-backed
/// rules simply produce no findings in that case rather than treating a
/// missing repository as an error.
#[must_use]
pub fn build_history(root: &Path, project: &ProjectIR) -> Option<GitHistory> {
    let commits = log_commits(root).ok()?;
    let changes = resolve_changes(root, project, &commits).ok()?;
    let cochange = CoChangeGraph::build(&changes, &commits);
    Some(GitHistory { commits, cochange })
}
