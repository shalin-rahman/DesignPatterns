//! Deterministic structural metrics derived from an already-resolved
//! `ProjectIR` plus its dependency graph. Metrics are derived data, never
//! facts: nothing here is written back onto the IR, and every value here
//! is reproducible from (project, graph, complexity signals) alone.
//!
//! `Cbo` only ever counts couplings to types the resolver actually matched
//! intra-project (§4/§12): a framework or primitive type stays `Unresolved`
//! and therefore never produces a graph edge, so it is excluded from CBO
//! automatically rather than through a separate configured exclusion list.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use scent_domain::{FieldId, MethodId, PropertyId, Resolution, TypeId};
use scent_graph::{DependencyGraph, EntityRef};
use scent_ir::{MemberTarget, MethodIR, ProjectIR};
use scent_parser::ComplexitySignals;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetricKind {
    Loc,
    CyclomaticComplexity,
    NestingDepth,
    Lcom4,
    Cbo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetricValue {
    pub value: f64,
    pub confidence: f32,
    pub provenance: &'static str,
}

#[derive(Debug, Default)]
pub struct MetricStore {
    values: BTreeMap<(EntityRef, MetricKind), MetricValue>,
}

impl MetricStore {
    #[must_use]
    pub fn get(&self, entity: &EntityRef, kind: MetricKind) -> Option<&MetricValue> {
        self.values.get(&(entity.clone(), kind))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&(EntityRef, MetricKind), &MetricValue)> {
        self.values.iter()
    }

    fn set(
        &mut self,
        entity: EntityRef,
        kind: MetricKind,
        value: f64,
        confidence: f32,
        provenance: &'static str,
    ) {
        self.values.insert(
            (entity, kind),
            MetricValue {
                value,
                confidence,
                provenance,
            },
        );
    }
}

/// Runs every Phase 1 calculator and returns the combined store.
#[must_use]
pub fn build_metric_store(
    project: &ProjectIR,
    graph: &DependencyGraph,
    method_complexity: &[(MethodId, ComplexitySignals)],
) -> MetricStore {
    let mut store = MetricStore::default();
    calculate_loc(project, &mut store);
    calculate_complexity(method_complexity, &mut store);
    calculate_lcom4(project, &mut store);
    calculate_cbo(project, graph, &mut store);
    store
}

/// Physical lines spanned by a declaration's own location, inclusive of its
/// first and last line.
fn calculate_loc(project: &ProjectIR, store: &mut MetricStore) {
    for method in &project.methods {
        store.set(
            EntityRef::Method(method.id.clone()),
            MetricKind::Loc,
            f64::from(physical_lines(
                method.location.range.start.line,
                method.location.range.end.line,
            )),
            1.0,
            "loc:physical_lines",
        );
    }
    for type_ir in &project.types {
        store.set(
            EntityRef::Type(type_ir.id.clone()),
            MetricKind::Loc,
            f64::from(physical_lines(
                type_ir.location.range.start.line,
                type_ir.location.range.end.line,
            )),
            1.0,
            "loc:physical_lines",
        );
    }
}

fn physical_lines(start_line: u32, end_line: u32) -> u32 {
    end_line - start_line + 1
}

/// `CC = 1 + decision_points`; nesting depth is the raw signal as measured.
/// Both come straight from `scent-parser`'s objective node-kind counts.
fn calculate_complexity(
    method_complexity: &[(MethodId, ComplexitySignals)],
    store: &mut MetricStore,
) {
    for (method_id, signals) in method_complexity {
        store.set(
            EntityRef::Method(method_id.clone()),
            MetricKind::CyclomaticComplexity,
            f64::from(1 + signals.decision_points),
            1.0,
            "cc:decision_points_plus_one",
        );
        store.set(
            EntityRef::Method(method_id.clone()),
            MetricKind::NestingDepth,
            f64::from(signals.max_nesting_depth),
            1.0,
            "nesting:max_depth",
        );
    }
}

/// Connects two methods of the same type when they call each other or
/// access the same field (never a property — the resolver's `MemberTarget`
/// already keeps that distinction, so this stays exact rather than folding
/// properties in as an approximation). LCOM4 is the resulting number of
/// connected components; a cohesive type has one.
fn calculate_lcom4(project: &ProjectIR, store: &mut MetricStore) {
    for type_ir in &project.types {
        let methods: Vec<&MethodIR> = project
            .methods
            .iter()
            .filter(|method| method.owner_type == type_ir.id)
            .collect();
        if methods.is_empty() {
            continue;
        }
        let index_of: HashMap<&MethodId, usize> = methods
            .iter()
            .enumerate()
            .map(|(index, m)| (&m.id, index))
            .collect();
        let mut dsu = DisjointSet::new(methods.len());

        for (index, method) in methods.iter().enumerate() {
            for call in &method.calls {
                if let Resolution::Resolved(target) = &call.target {
                    if let Some(&other) = index_of.get(target) {
                        dsu.union(index, other);
                    }
                }
            }
        }

        let mut field_to_methods: HashMap<&FieldId, Vec<usize>> = HashMap::new();
        for (index, method) in methods.iter().enumerate() {
            for access in &method.field_accesses {
                if let Resolution::Resolved(MemberTarget::Field(field_id)) = &access.target {
                    field_to_methods.entry(field_id).or_default().push(index);
                }
            }
        }
        for indices in field_to_methods.values() {
            for pair in indices.windows(2) {
                dsu.union(pair[0], pair[1]);
            }
        }

        store.set(
            EntityRef::Type(type_ir.id.clone()),
            MetricKind::Lcom4,
            f64::from(dsu.count_components()),
            1.0,
            "lcom4:connected_components",
        );
    }
}

/// Distinct other types coupled to a type through the dependency graph,
/// counted symmetrically (both efferent and afferent couplings), matching
/// the classical CBO definition.
fn calculate_cbo(project: &ProjectIR, graph: &DependencyGraph, store: &mut MetricStore) {
    let owner_of_method: HashMap<&MethodId, &TypeId> = project
        .methods
        .iter()
        .map(|m| (&m.id, &m.owner_type))
        .collect();
    let owner_of_field: HashMap<&FieldId, &TypeId> = project
        .fields
        .iter()
        .map(|f| (&f.id, &f.owner_type))
        .collect();
    let owner_of_property: HashMap<&PropertyId, &TypeId> = project
        .properties
        .iter()
        .map(|p| (&p.id, &p.owner_type))
        .collect();
    let owner_type = |entity: &EntityRef| -> Option<TypeId> {
        match entity {
            EntityRef::Type(id) => Some(id.clone()),
            EntityRef::Method(id) => owner_of_method.get(id).map(|t| (*t).clone()),
            EntityRef::Field(id) => owner_of_field.get(id).map(|t| (*t).clone()),
            EntityRef::Property(id) => owner_of_property.get(id).map(|t| (*t).clone()),
        }
    };

    let mut coupled: HashMap<TypeId, BTreeSet<TypeId>> = HashMap::new();
    for edge in graph.edges() {
        let (Some(from_type), Some(to_type)) = (owner_type(&edge.from), owner_type(&edge.to))
        else {
            continue;
        };
        if from_type == to_type {
            continue;
        }
        coupled
            .entry(from_type.clone())
            .or_default()
            .insert(to_type.clone());
        coupled.entry(to_type).or_default().insert(from_type);
    }

    for type_ir in &project.types {
        let count = coupled.get(&type_ir.id).map_or(0, BTreeSet::len);
        store.set(
            EntityRef::Type(type_ir.id.clone()),
            MetricKind::Cbo,
            f64::from(u32::try_from(count).unwrap_or(u32::MAX)),
            1.0,
            "cbo:distinct_coupled_types",
        );
    }
}

struct DisjointSet {
    parent: Vec<usize>,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, node: usize) -> usize {
        if self.parent[node] != node {
            self.parent[node] = self.find(self.parent[node]);
        }
        self.parent[node]
    }

    fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root != right_root {
            self.parent[left_root] = right_root;
        }
    }

    fn count_components(&mut self) -> u32 {
        let roots: BTreeSet<usize> = (0..self.parent.len()).map(|node| self.find(node)).collect();
        u32::try_from(roots.len()).unwrap_or(u32::MAX)
    }
}
