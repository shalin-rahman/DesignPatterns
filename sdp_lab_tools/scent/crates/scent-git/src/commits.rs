//! Shells out to the `git` CLI for commit history rather than depending on
//! `git2`/libgit2 — this only ever needs commit metadata and line-range
//! diffs, not full repository object access, and shelling out keeps the
//! workspace's existing minimal-dependency posture (it already hand-rolls
//! its own JSON writer instead of pulling in `serde`).

use std::path::Path;
use std::process::Command;

use scent_domain::{CommitId, NormalizedPath};

#[derive(Debug)]
pub enum GitError {
    /// `git` itself is missing, or `root` is not inside a Git work tree.
    Unavailable(String),
    CommandFailed(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Commit {
    pub id: CommitId,
    /// Unix seconds, from the commit's own author date (`git log %at`) —
    /// real, not runtime metadata: it's a fact about the commit itself, no
    /// different from a `SourceLocation` being a fact about a file.
    pub timestamp: i64,
    pub changed_files: Vec<NormalizedPath>,
}

/// Runs `git log` over `root` and parses one [`Commit`] per non-merge
/// commit, oldest details preserved as `git` itself orders them. Merge
/// commits are excluded (`--no-merges`): a merge's file list conflates two
/// histories and would misattribute co-changes to commits that did not
/// themselves introduce the change.
///
/// # Errors
///
/// Returns [`GitError::Unavailable`] when `git` cannot run at all or `root`
/// is not a Git work tree, and [`GitError::CommandFailed`] if `git` runs but
/// exits with a non-zero status.
pub fn log_commits(root: &Path) -> Result<Vec<Commit>, GitError> {
    let prefix = repo_prefix(root)?;
    let output = run_git(
        root,
        &[
            "log",
            "--no-merges",
            "--format=%H%x09%at",
            "--name-only",
            "--",
            ".",
        ],
    )?;
    Ok(parse_log(&output, &prefix))
}

/// `root`'s path relative to its repository's top level, with a trailing
/// slash (empty when `root` already is the top level) — from
/// `git rev-parse --show-prefix`. `--name-only` always prints paths
/// relative to the repository root regardless of `-C`, so this prefix is
/// what turns those back into paths relative to `root` — matching the
/// `NormalizedPath`s already recorded on every fact. Pathspec matching
/// (the `-- .` above, and `git show ... -- <file>` in `entities.rs`) is
/// unaffected: pathspecs are resolved relative to `-C`'s directory already.
fn repo_prefix(root: &Path) -> Result<String, GitError> {
    let output = run_git(root, &["rev-parse", "--show-prefix"])?;
    Ok(output.trim().to_owned())
}

fn parse_log(output: &str, prefix: &str) -> Vec<Commit> {
    let mut commits = Vec::new();
    let mut lines = output.lines().peekable();
    while let Some(header) = lines.next() {
        let header = header.trim();
        if header.is_empty() {
            continue;
        }
        let Some((sha, timestamp)) = header.split_once('\t') else {
            continue;
        };
        let Ok(timestamp) = timestamp.parse::<i64>() else {
            continue;
        };
        // `git log --name-only` always prints one blank line between the
        // commit header and its file list, even with an empty `--format`
        // body — that separator is unconditional, not the terminator.
        if lines.peek().is_some_and(|next| next.trim().is_empty()) {
            lines.next();
        }
        // A file's own list runs until a blank line (when it's the last
        // commit) or directly into the next commit's header line — git does
        // not print a trailing blank after the last file, only the one
        // separating the header from the file list. A header is the only
        // line here containing a tab (our own `%H%x09%at` format), which no
        // repository-relative file path can contain, so that's what marks
        // the boundary.
        let mut changed_files = Vec::new();
        while let Some(next) = lines.peek() {
            if next.trim().is_empty() || next.contains('\t') {
                break;
            }
            if let Some(relative) = next.trim().strip_prefix(prefix) {
                if let Ok(path) = NormalizedPath::parse(relative) {
                    changed_files.push(path);
                }
            }
            lines.next();
        }
        commits.push(Commit {
            id: CommitId::from_sha(sha),
            timestamp,
            changed_files,
        });
    }
    commits
}

pub(crate) fn run_git(root: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| GitError::Unavailable(error.to_string()))?;
    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::parse_log;

    #[test]
    fn parses_commits_with_their_changed_files() {
        // Real `git log --name-only` inserts a blank line between the
        // header and the file list (even with an empty `--format` body),
        // but no blank line between the file list and the next header.
        let output = "aaa111\t1700000000\n\nsrc/A.cs\nsrc/B.cs\nbbb222\t1700000100\n\nsrc/A.cs\n";
        let commits = parse_log(output, "");
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].id.as_str(), "aaa111");
        assert_eq!(commits[0].timestamp, 1_700_000_000);
        assert_eq!(commits[0].changed_files.len(), 2);
        assert_eq!(commits[1].changed_files.len(), 1);
    }

    #[test]
    fn a_commit_with_no_files_still_parses() {
        let output = "aaa111\t1700000000\n";
        let commits = parse_log(output, "");
        assert_eq!(commits.len(), 1);
        assert!(commits[0].changed_files.is_empty());
    }

    #[test]
    fn strips_the_repo_root_relative_prefix_when_root_is_a_subdirectory() {
        // `--name-only` always prints paths relative to the repo top level,
        // even when analyzing a nested subdirectory (e.g. this workspace
        // nested inside a larger monorepo) — the prefix must be stripped so
        // the result matches the `NormalizedPath`s recorded on every fact.
        let output = "aaa111\t1700000000\n\nsub/dir/src/A.cs\n";
        let commits = parse_log(output, "sub/dir/");
        assert_eq!(commits.len(), 1);
        assert_eq!(
            commits[0].changed_files,
            vec![super::NormalizedPath::parse("src/A.cs").unwrap()]
        );
    }
}
