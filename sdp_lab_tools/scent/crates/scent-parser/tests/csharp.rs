use scent_domain::NormalizedPath;
use scent_parser::{csharp::extract_file, CSharpAdapter, SourceFile};

fn source(contents: &str) -> SourceFile {
    SourceFile {
        path: NormalizedPath::parse("src/Order.cs").unwrap(),
        contents: contents.into(),
    }
}

#[test]
fn extracts_namespace_and_type_declarations() {
    let source = source(
        "namespace Demo.Services { public class OrderService {} internal interface IOrders {} }",
    );
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    assert!(parsed.diagnostics.is_empty());
    assert_eq!(extracted.namespaces.len(), 1);
    assert_eq!(extracted.namespaces[0].name, "Demo.Services");
    assert_eq!(
        extracted
            .types
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        ["IOrders", "OrderService"]
    );
}

#[test]
fn reports_malformed_csharp_without_panicking() {
    let source = source("public class {");
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);

    assert!(!parsed.diagnostics.is_empty());
}

#[test]
fn applies_a_file_scoped_namespace_to_following_types() {
    let source = source("namespace Demo.Models; public record Order;");
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    assert!(parsed.diagnostics.is_empty());
    assert_eq!(extracted.types.len(), 1);
    assert!(extracted.types[0].namespace_id.is_some());
}

#[test]
fn extracts_fields_properties_constructor_and_methods() {
    let source = source(
        "namespace Demo { public class Order { \
             private int total; \
             public string Status { get; set; } \
             public Order() {} \
             public void Ship() {} \
         } }",
    );
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    assert!(parsed.diagnostics.is_empty());
    assert_eq!(extracted.fields.len(), 1);
    assert_eq!(extracted.fields[0].name, "total");
    assert_eq!(extracted.fields[0].owner_type, extracted.types[0].id);
    assert_eq!(extracted.properties.len(), 1);
    assert_eq!(extracted.properties[0].name, "Status");
    assert_eq!(
        extracted
            .methods
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        ["Order", "Ship"]
    );
    assert!(extracted
        .methods
        .iter()
        .all(|item| item.owner_type == extracted.types[0].id));
}

#[test]
fn records_a_method_call_to_another_method_as_unresolved() {
    let source = source(
        "namespace Demo { public class Order { \
             public void Process() { this.Validate(); } \
             public void Validate() {} \
         } }",
    );
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    let process = extracted
        .methods
        .iter()
        .find(|item| item.name == "Process")
        .expect("Process method extracted");
    assert_eq!(process.calls.len(), 1);
    match &process.calls[0].target {
        scent_domain::Resolution::Unresolved(reference) => {
            assert_eq!(reference.spelling, "Validate");
        }
        scent_domain::Resolution::Resolved(_) => panic!("call must stay unresolved at Milestone 3"),
    }
}

#[test]
fn records_an_object_creation_as_an_instantiation() {
    let source = source(
        "namespace Demo { public class OrderFactory { \
             public void Create() { new Order(); } \
         } }",
    );
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    let create = &extracted.methods[0];
    assert_eq!(create.instantiations.len(), 1);
    match &create.instantiations[0].target {
        scent_domain::Resolution::Unresolved(reference) => {
            assert_eq!(reference.spelling, "Order");
        }
        scent_domain::Resolution::Resolved(_) => {
            panic!("creation must stay unresolved at Milestone 3")
        }
    }
}

#[test]
fn records_a_field_access_inside_a_method_body() {
    let source = source(
        "namespace Demo { public class Order { \
             public void Process() { var value = this.Total; } \
         } }",
    );
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    let process = &extracted.methods[0];
    assert_eq!(process.field_accesses.len(), 1);
    match &process.field_accesses[0].target {
        scent_domain::Resolution::Unresolved(reference) => {
            assert_eq!(reference.spelling, "Total");
        }
        scent_domain::Resolution::Resolved(_) => {
            panic!("access must stay unresolved at Milestone 3")
        }
    }
    assert!(
        process.calls.is_empty(),
        "a plain field access must not also be recorded as a method call"
    );
}

#[test]
fn extracts_base_class_and_interface_list_entries_as_unresolved_types() {
    // The base/interface split is deferred to Milestone 4: syntax alone
    // cannot tell a base class from an implemented interface, only their
    // resolved TypeKind can. See `parse_base_list`.
    let source = source("class Order : BaseOrder, IOrder, ISellable {}");
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    assert!(extracted.types[0].base_types.is_empty());
    let spellings: Vec<&str> = extracted.types[0]
        .interface_types
        .iter()
        .map(|item| match item {
            scent_domain::Resolution::Unresolved(reference) => reference.spelling.as_str(),
            scent_domain::Resolution::Resolved(_) => panic!("must stay unresolved at Milestone 3"),
        })
        .collect();
    assert_eq!(spellings, ["BaseOrder", "IOrder", "ISellable"]);
}

#[test]
fn reports_a_malformed_member_without_panicking() {
    let source = source(
        "namespace Demo { public class Order { \
             private int @#$; \
             public void Ship() {} \
         } }",
    );
    let mut adapter = CSharpAdapter::new();
    let parsed = adapter.parse(&source);
    let extracted = extract_file(&source, &parsed.tree);

    assert!(!parsed.diagnostics.is_empty());
    assert!(extracted.methods.iter().any(|item| item.name == "Ship"));
}
