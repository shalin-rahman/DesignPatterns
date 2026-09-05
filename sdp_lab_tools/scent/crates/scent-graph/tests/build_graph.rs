use scent_domain::{Language, NormalizedPath, ProjectId};
use scent_graph::{build_graph, DependencyKind, EntityRef};
use scent_ir::ProjectIR;
use scent_parser::{extract_file, resolve_project, CSharpAdapter, DeclarationIndex, SourceFile};

fn build_project(contents: &str) -> ProjectIR {
    let mut adapter = CSharpAdapter::new();
    let source = SourceFile {
        path: NormalizedPath::parse("src/Order.cs").unwrap(),
        contents: contents.into(),
    };
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);
    let project = ProjectIR {
        id: ProjectId::from_identity("csharp:sample"),
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
fn emits_a_calls_edge_for_a_resolved_self_type_call() {
    let project = build_project(include_str!("fixtures/TestSubjects/src/CallsEdge.cs"));
    let graph = build_graph(&project);
    let process = project
        .methods
        .iter()
        .find(|m| m.name == "Process")
        .unwrap();
    let validate = project
        .methods
        .iter()
        .find(|m| m.name == "Validate")
        .unwrap();

    let edges = graph.edges_between(
        &EntityRef::Method(process.id.clone()),
        &EntityRef::Method(validate.id.clone()),
    );
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].kind, DependencyKind::Calls);
}

#[test]
fn emits_no_edge_for_an_unresolved_framework_type() {
    let project = build_project(include_str!(
        "fixtures/TestSubjects/src/UnresolvedFrameworkType.cs"
    ));
    let graph = build_graph(&project);
    let process = &project.methods[0];
    assert!(graph
        .outgoing(&EntityRef::Method(process.id.clone()))
        .is_empty());
}

#[test]
fn emits_an_implements_edge_for_a_resolved_interface() {
    let project = build_project(include_str!("fixtures/TestSubjects/src/ImplementsEdge.cs"));
    let graph = build_graph(&project);
    let order = project.types.iter().find(|t| t.name == "Order").unwrap();
    let iorder = project.types.iter().find(|t| t.name == "IOrder").unwrap();

    let edges = graph.edges_between(
        &EntityRef::Type(order.id.clone()),
        &EntityRef::Type(iorder.id.clone()),
    );
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].kind, DependencyKind::Implements);
}
