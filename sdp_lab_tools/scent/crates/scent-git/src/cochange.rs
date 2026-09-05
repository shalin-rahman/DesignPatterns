//! Builds a co-change graph from resolved [`EntityChange`]s: which entities
//! repeatedly change together, with the historical evidence (commit count,
//! commit IDs, time window) needed to explain a finding rather than just
//! assert it (`docs/prompt.md` §15).

use std::collections::HashMap;

use scent_domain::CommitId;
use scent_graph::EntityRef;

use crate::commits::Commit;
use crate::entities::EntityChange;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoChangeEdge {
    pub a: EntityRef,
    pub b: EntityRef,
    pub commit_ids: Vec<CommitId>,
    pub first_seen: i64,
    pub last_seen: i64,
}

impl CoChangeEdge {
    #[must_use]
    pub fn commit_count(&self) -> usize {
        self.commit_ids.len()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CoChangeGraph {
    edges: Vec<CoChangeEdge>,
    /// Every commit (id, timestamp) that touched a given entity, oldest
    /// first — the raw history a Divergent Change check reads to explain
    /// its own evidence without recomputing it from scratch.
    entity_commits: HashMap<EntityRef, Vec<(CommitId, i64)>>,
}

impl CoChangeGraph {
    #[must_use]
    pub fn build(changes: &[EntityChange], commits: &[Commit]) -> Self {
        let timestamps: HashMap<&CommitId, i64> = commits
            .iter()
            .map(|commit| (&commit.id, commit.timestamp))
            .collect();

        let mut entities_by_commit: HashMap<&CommitId, Vec<&EntityRef>> = HashMap::new();
        let mut entity_commits: HashMap<EntityRef, Vec<(CommitId, i64)>> = HashMap::new();
        for change in changes {
            entities_by_commit
                .entry(&change.commit_id)
                .or_default()
                .push(&change.entity);
            let timestamp = timestamps.get(&change.commit_id).copied().unwrap_or(0);
            entity_commits
                .entry(change.entity.clone())
                .or_default()
                .push((change.commit_id.clone(), timestamp));
        }
        for history in entity_commits.values_mut() {
            history.sort_by_key(|(_, timestamp)| *timestamp);
            history.dedup_by(|left, right| left.0 == right.0);
        }

        let mut pair_evidence: HashMap<(EntityRef, EntityRef), Vec<CommitId>> = HashMap::new();
        for (commit_id, mut entities) in entities_by_commit {
            entities.sort();
            entities.dedup();
            for i in 0..entities.len() {
                for j in (i + 1)..entities.len() {
                    let key = ordered_pair(entities[i].clone(), entities[j].clone());
                    pair_evidence
                        .entry(key)
                        .or_default()
                        .push(commit_id.clone());
                }
            }
        }

        let edges = pair_evidence
            .into_iter()
            .map(|((a, b), commit_ids)| {
                let times: Vec<i64> = commit_ids
                    .iter()
                    .map(|id| timestamps.get(&id).copied().unwrap_or(0))
                    .collect();
                let first_seen = times.iter().copied().min().unwrap_or(0);
                let last_seen = times.iter().copied().max().unwrap_or(0);
                CoChangeEdge {
                    a,
                    b,
                    commit_ids,
                    first_seen,
                    last_seen,
                }
            })
            .collect();

        Self {
            edges,
            entity_commits,
        }
    }

    #[must_use]
    pub fn edges(&self) -> &[CoChangeEdge] {
        &self.edges
    }

    /// Every commit (in chronological order) that changed `entity`,
    /// deduplicated — the total change history a Divergent Change check
    /// reads to look for distinct change contexts.
    #[must_use]
    pub fn changes_for(&self, entity: &EntityRef) -> &[(CommitId, i64)] {
        self.entity_commits.get(entity).map_or(&[], Vec::as_slice)
    }

    /// Edges touching `entity`, each paired with whichever side is the
    /// *other* entity.
    #[must_use]
    pub fn partners_of(&self, entity: &EntityRef) -> Vec<(&EntityRef, &CoChangeEdge)> {
        self.edges
            .iter()
            .filter_map(|edge| {
                if edge.a == *entity {
                    Some((&edge.b, edge))
                } else if edge.b == *entity {
                    Some((&edge.a, edge))
                } else {
                    None
                }
            })
            .collect()
    }
}

fn ordered_pair(a: EntityRef, b: EntityRef) -> (EntityRef, EntityRef) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use scent_domain::{FieldId, MethodId};

    use super::CoChangeGraph;
    use crate::commits::Commit;
    use crate::entities::{ChangeType, EntityChange};
    use scent_domain::CommitId;
    use scent_graph::EntityRef;

    fn method(name: &str) -> EntityRef {
        EntityRef::Method(MethodId::from_identity(name))
    }

    #[test]
    fn builds_an_edge_for_entities_that_change_in_the_same_commit() {
        let commits = vec![
            Commit {
                id: CommitId::from_sha("c1"),
                timestamp: 100,
                changed_files: vec![],
            },
            Commit {
                id: CommitId::from_sha("c2"),
                timestamp: 200,
                changed_files: vec![],
            },
        ];
        let changes = vec![
            EntityChange {
                commit_id: CommitId::from_sha("c1"),
                entity: method("A"),
                change_type: ChangeType::Modified,
            },
            EntityChange {
                commit_id: CommitId::from_sha("c1"),
                entity: method("B"),
                change_type: ChangeType::Modified,
            },
            EntityChange {
                commit_id: CommitId::from_sha("c2"),
                entity: method("A"),
                change_type: ChangeType::Modified,
            },
            EntityChange {
                commit_id: CommitId::from_sha("c2"),
                entity: method("B"),
                change_type: ChangeType::Modified,
            },
        ];
        let graph = CoChangeGraph::build(&changes, &commits);
        assert_eq!(graph.edges().len(), 1);
        assert_eq!(graph.edges()[0].commit_count(), 2);
        assert_eq!(graph.changes_for(&method("A")).len(), 2);
    }

    #[test]
    fn entities_touched_in_different_commits_only_do_not_cochange() {
        let commits = vec![
            Commit {
                id: CommitId::from_sha("c1"),
                timestamp: 100,
                changed_files: vec![],
            },
            Commit {
                id: CommitId::from_sha("c2"),
                timestamp: 200,
                changed_files: vec![],
            },
        ];
        let changes = vec![
            EntityChange {
                commit_id: CommitId::from_sha("c1"),
                entity: method("A"),
                change_type: ChangeType::Modified,
            },
            EntityChange {
                commit_id: CommitId::from_sha("c2"),
                entity: EntityRef::Field(FieldId::from_identity("B")),
                change_type: ChangeType::Modified,
            },
        ];
        let graph = CoChangeGraph::build(&changes, &commits);
        assert!(graph.edges().is_empty());
    }
}
