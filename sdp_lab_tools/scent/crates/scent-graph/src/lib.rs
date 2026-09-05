//! A typed, indexed dependency graph built from an already-resolved
//! `ProjectIR`. Only `Resolved` facts ever produce an edge — an edge to an
//! unknown target would be a guess, not a fact, so unresolved references
//! (including every framework/primitive type in Phase 1) simply produce no
//! edge at all.

use std::collections::HashMap;

use scent_domain::{EdgeId, FieldId, MethodId, PropertyId, Resolution, SourceLocation, TypeId};
use scent_ir::{MemberTarget, ProjectIR};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EntityRef {
    Type(TypeId),
    Method(MethodId),
    Field(FieldId),
    Property(PropertyId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DependencyKind {
    Inherits,
    Implements,
    Calls,
    /// Property-member reads are not yet distinguished from field reads and
    /// are not emitted at all (rather than mislabeled); see module docs.
    ReadsField,
    WritesField,
    UsesType,
    Creates,
    Returns,
    AcceptsParameter,
    Throws,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEdge {
    pub id: EdgeId,
    pub kind: DependencyKind,
    pub from: EntityRef,
    pub to: EntityRef,
    pub location: SourceLocation,
}

#[derive(Debug, Default)]
pub struct DependencyGraph {
    edges: Vec<DependencyEdge>,
    outgoing: HashMap<EntityRef, Vec<usize>>,
    incoming: HashMap<EntityRef, Vec<usize>>,
}

impl DependencyGraph {
    #[must_use]
    pub fn edges(&self) -> &[DependencyEdge] {
        &self.edges
    }

    #[must_use]
    pub fn outgoing(&self, entity: &EntityRef) -> &[usize] {
        self.outgoing.get(entity).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn incoming(&self, entity: &EntityRef) -> &[usize] {
        self.incoming.get(entity).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn edges_between(&self, from: &EntityRef, to: &EntityRef) -> Vec<&DependencyEdge> {
        self.outgoing(from)
            .iter()
            .map(|index| &self.edges[*index])
            .filter(|edge| &edge.to == to)
            .collect()
    }

    #[must_use]
    pub fn dependencies(&self, entity: &EntityRef) -> Vec<&EntityRef> {
        let mut targets: Vec<&EntityRef> = self
            .outgoing(entity)
            .iter()
            .map(|index| &self.edges[*index].to)
            .collect();
        targets.sort_unstable();
        targets.dedup();
        targets
    }

    #[must_use]
    pub fn dependents(&self, entity: &EntityRef) -> Vec<&EntityRef> {
        let mut sources: Vec<&EntityRef> = self
            .incoming(entity)
            .iter()
            .map(|index| &self.edges[*index].from)
            .collect();
        sources.sort_unstable();
        sources.dedup();
        sources
    }

    fn push(
        &mut self,
        kind: DependencyKind,
        from: EntityRef,
        to: EntityRef,
        location: SourceLocation,
    ) {
        let identity = format!(
            "scent:v1:edge:{kind:?}:{from:?}:{to:?}:{}:{}:{}",
            location.path, location.range.start.line, location.range.start.column
        );
        let index = self.edges.len();
        self.edges.push(DependencyEdge {
            id: EdgeId::from_identity(&identity),
            kind,
            from: from.clone(),
            to: to.clone(),
            location,
        });
        self.outgoing.entry(from).or_default().push(index);
        self.incoming.entry(to).or_default().push(index);
    }
}

/// Builds the dependency graph from a resolved project. Requires
/// `resolve_project` to have already run; running it against an
/// unresolved `ProjectIR` simply yields an empty graph, since nothing is
/// `Resolved` yet.
#[must_use]
pub fn build_graph(project: &ProjectIR) -> DependencyGraph {
    let mut graph = DependencyGraph::default();
    push_type_edges(project, &mut graph);
    push_method_edges(project, &mut graph);
    push_member_type_edges(project, &mut graph);
    graph
}

fn push_type_edges(project: &ProjectIR, graph: &mut DependencyGraph) {
    for type_ir in &project.types {
        let owner = EntityRef::Type(type_ir.id.clone());
        for base in &type_ir.base_types {
            if let Resolution::Resolved(target) = base {
                graph.push(
                    DependencyKind::Inherits,
                    owner.clone(),
                    EntityRef::Type(target.clone()),
                    type_ir.location.clone(),
                );
            }
        }
        for interface in &type_ir.interface_types {
            if let Resolution::Resolved(target) = interface {
                graph.push(
                    DependencyKind::Implements,
                    owner.clone(),
                    EntityRef::Type(target.clone()),
                    type_ir.location.clone(),
                );
            }
        }
    }
}

fn push_method_edges(project: &ProjectIR, graph: &mut DependencyGraph) {
    for method in &project.methods {
        let owner = EntityRef::Method(method.id.clone());
        for call in &method.calls {
            if let Resolution::Resolved(target) = &call.target {
                graph.push(
                    DependencyKind::Calls,
                    owner.clone(),
                    EntityRef::Method(target.clone()),
                    call.location.clone(),
                );
            }
        }
        for access in &method.field_accesses {
            if let Resolution::Resolved(MemberTarget::Field(target)) = &access.target {
                graph.push(
                    DependencyKind::ReadsField,
                    owner.clone(),
                    EntityRef::Field(target.clone()),
                    access.location.clone(),
                );
            }
        }
        for instantiation in &method.instantiations {
            if let Resolution::Resolved(target) = &instantiation.target {
                graph.push(
                    DependencyKind::Creates,
                    owner.clone(),
                    EntityRef::Type(target.clone()),
                    instantiation.location.clone(),
                );
            }
        }
        if let Some(Resolution::Resolved(target)) = &method.return_type {
            graph.push(
                DependencyKind::Returns,
                owner.clone(),
                EntityRef::Type(target.clone()),
                method.location.clone(),
            );
        }
        for parameter in &method.parameters {
            if let Resolution::Resolved(target) = &parameter.type_reference {
                graph.push(
                    DependencyKind::AcceptsParameter,
                    owner.clone(),
                    EntityRef::Type(target.clone()),
                    parameter.location.clone(),
                );
            }
        }
        for reference in &method.type_references {
            if let Resolution::Resolved(target) = &reference.target {
                graph.push(
                    DependencyKind::UsesType,
                    owner.clone(),
                    EntityRef::Type(target.clone()),
                    reference.location.clone(),
                );
            }
        }
    }
}

fn push_member_type_edges(project: &ProjectIR, graph: &mut DependencyGraph) {
    for field in &project.fields {
        if let Resolution::Resolved(target) = &field.type_reference {
            graph.push(
                DependencyKind::UsesType,
                EntityRef::Field(field.id.clone()),
                EntityRef::Type(target.clone()),
                field.location.clone(),
            );
        }
    }
    for property in &project.properties {
        if let Resolution::Resolved(target) = &property.type_reference {
            graph.push(
                DependencyKind::UsesType,
                EntityRef::Property(property.id.clone()),
                EntityRef::Type(target.clone()),
                property.location.clone(),
            );
        }
    }
}
