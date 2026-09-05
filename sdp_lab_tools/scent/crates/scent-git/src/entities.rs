//! Maps a changed line range in a commit's diff to the `ProjectIR` entity
//! (type or method) whose `SourceLocation` contains it — reusing the
//! locations already recorded on every fact, no new resolution logic.
//!
//! Scope, stated plainly: this maps changes against the *current* checkout's
//! entity locations, not a historical reconstruction of where an entity was
//! at the time of an older commit. A pure deletion (a hunk with no new-file
//! lines) has nothing left in the current tree to attribute it to, so it is
//! not represented as an `EntityChange` at all — recording a guess about
//! which entity a deleted line "belonged to" would violate the project's
//! own never-guess rule (`docs/prompt.md` §9).

use std::path::Path;

use scent_domain::{CommitId, NormalizedPath};
use scent_graph::EntityRef;
use scent_ir::ProjectIR;

use crate::commits::{run_git, Commit, GitError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeType {
    /// A hunk whose old side was empty: these lines are new to the file.
    Added,
    /// A hunk with content on both sides: existing lines were changed.
    Modified,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityChange {
    pub commit_id: CommitId,
    pub entity: EntityRef,
    pub change_type: ChangeType,
}

/// One `@@ -old_start,old_len +new_start,new_len @@` hunk header, keeping
/// only the new-file side: that is what the current checkout's entity
/// locations can still be checked against.
struct Hunk {
    new_start: u32,
    new_len: u32,
    old_len: u32,
}

/// Computes every [`EntityChange`] across `commits` by diffing each
/// commit's changed files against its parent and mapping new-file line
/// ranges onto `project`'s current type/method locations.
///
/// # Errors
///
/// Returns [`GitError`] if `git show` fails for a commit/file pair.
pub fn resolve_changes(
    root: &Path,
    project: &ProjectIR,
    commits: &[Commit],
) -> Result<Vec<EntityChange>, GitError> {
    let mut changes = Vec::new();
    for commit in commits {
        for file in &commit.changed_files {
            let hunks = diff_hunks(root, &commit.id, file)?;
            for hunk in hunks {
                if hunk.new_len == 0 {
                    continue;
                }
                let change_type = if hunk.old_len == 0 {
                    ChangeType::Added
                } else {
                    ChangeType::Modified
                };
                let mut entities = entities_in_range(project, file, hunk.new_start, hunk.new_len);
                entities.sort();
                entities.dedup();
                for entity in entities {
                    changes.push(EntityChange {
                        commit_id: commit.id.clone(),
                        entity,
                        change_type,
                    });
                }
            }
        }
    }
    Ok(changes)
}

fn diff_hunks(
    root: &Path,
    commit: &CommitId,
    file: &NormalizedPath,
) -> Result<Vec<Hunk>, GitError> {
    // `git show` diffs a commit against its parent (or the empty tree for a
    // root commit) on its own, so this works uniformly for both cases.
    let output = run_git(
        root,
        &[
            "show",
            "--unified=0",
            "--format=",
            commit.as_str(),
            "--",
            file.as_str(),
        ],
    )?;
    Ok(output.lines().filter_map(parse_hunk_header).collect())
}

fn parse_hunk_header(line: &str) -> Option<Hunk> {
    let line = line.strip_prefix("@@ -")?;
    let (old_part, rest) = line.split_once(" +")?;
    let (new_part, _rest) = rest.split_once(" @@")?;
    let (_old_start, old_len) = parse_range(old_part);
    let (new_start, new_len) = parse_range(new_part);
    Some(Hunk {
        new_start,
        new_len,
        old_len,
    })
}

/// A hunk range is `start` or `start,len` (an omitted length means 1).
fn parse_range(range: &str) -> (u32, u32) {
    range.split_once(',').map_or_else(
        || (range.parse().unwrap_or(0), 1),
        |(start, len)| (start.parse().unwrap_or(0), len.parse().unwrap_or(0)),
    )
}

/// `new_start`/`new_len` are 1-based, inclusive git line numbers; entity
/// locations are 0-based (Tree-sitter rows), so this converts before
/// checking for overlap.
fn entities_in_range(
    project: &ProjectIR,
    file: &NormalizedPath,
    new_start: u32,
    new_len: u32,
) -> Vec<EntityRef> {
    if new_start == 0 || new_len == 0 {
        return vec![];
    }
    let range_start = new_start - 1;
    let range_end = range_start + new_len - 1;
    let mut found = Vec::new();
    for type_ir in &project.types {
        if type_ir.location.path == *file
            && overlaps(
                type_ir.location.range.start.line,
                type_ir.location.range.end.line,
                range_start,
                range_end,
            )
        {
            found.push(EntityRef::Type(type_ir.id.clone()));
        }
    }
    for method in &project.methods {
        if method.location.path == *file
            && overlaps(
                method.location.range.start.line,
                method.location.range.end.line,
                range_start,
                range_end,
            )
        {
            found.push(EntityRef::Method(method.id.clone()));
        }
    }
    found
}

fn overlaps(a_start: u32, a_end: u32, b_start: u32, b_end: u32) -> bool {
    a_start <= b_end && b_start <= a_end
}

#[cfg(test)]
mod tests {
    use super::parse_hunk_header;

    #[test]
    fn parses_a_hunk_with_explicit_lengths() {
        let hunk = parse_hunk_header("@@ -10,2 +12,4 @@ void Ship()").unwrap();
        assert_eq!(hunk.old_len, 2);
        assert_eq!(hunk.new_start, 12);
        assert_eq!(hunk.new_len, 4);
    }

    #[test]
    fn parses_a_hunk_with_implicit_single_line_lengths() {
        let hunk = parse_hunk_header("@@ -5 +5 @@").unwrap();
        assert_eq!(hunk.old_len, 1);
        assert_eq!(hunk.new_start, 5);
        assert_eq!(hunk.new_len, 1);
    }

    #[test]
    fn parses_a_pure_deletion_hunk() {
        let hunk = parse_hunk_header("@@ -10,3 +9,0 @@").unwrap();
        assert_eq!(hunk.new_len, 0);
    }
}
