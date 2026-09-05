//! Per-switch shape signals (`docs/prompt.md` §24 "Switch Statements": case
//! count, branch complexity, object creation). Like [`super::ComplexitySignals`]
//! and [`super::CloneSignature`], this is a raw signal derived straight from
//! syntax, not a metric formula or a smell decision — `scent-rules` turns it
//! into evidence.

use std::collections::HashSet;

use scent_domain::{NormalizedPath, SourceLocation};
use tree_sitter::Node;

use super::complexity;
use super::SourceFile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchShape {
    pub location: SourceLocation,
    pub case_count: u32,
    pub has_default: bool,
    /// Decision points inside the densest single `switch_section`'s
    /// statements (via [`complexity::measure`] applied to that section),
    /// not the whole switch — a proxy for "branch complexity" per §24.
    pub max_branch_complexity: u32,
    /// Distinct type spellings constructed via `new` across every section,
    /// a proxy for "object creation" / type-code discrimination per §24.
    pub distinct_created_types: u32,
}

/// `switch_node` is a `switch_statement`: fields `value` (the discriminant)
/// and `body` (a `switch_body` containing `switch_section` children, one per
/// group of fallthrough `case`/`default` labels — verified against
/// `tree-sitter-c-sharp`'s `node-types.json`).
#[must_use]
pub fn measure(switch_node: Node<'_>, source: &SourceFile) -> SwitchShape {
    let mut case_count = 0u32;
    let mut has_default = false;
    let mut max_branch_complexity = 0u32;
    let mut created_types: HashSet<String> = HashSet::new();

    if let Some(body) = switch_node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for section in body
            .named_children(&mut cursor)
            .filter(|child| child.kind() == "switch_section")
        {
            case_count += 1;
            if section_has_default_label(section) {
                has_default = true;
            }
            let branch_complexity = section_body_complexity(section);
            max_branch_complexity = max_branch_complexity.max(branch_complexity);
            collect_created_types(section, source, &mut created_types);
        }
    }

    SwitchShape {
        location: location(switch_node, &source.path),
        case_count,
        has_default,
        max_branch_complexity,
        distinct_created_types: u32::try_from(created_types.len()).unwrap_or(u32::MAX),
    }
}

/// A `default:` label is an unnamed `"default"` token directly under the
/// section (siblings of its case-label expressions/patterns and body
/// statements) — there is no dedicated `default_switch_label` node in this
/// grammar version.
fn section_has_default_label(section: Node<'_>) -> bool {
    let mut cursor = section.walk();
    let found = section
        .children(&mut cursor)
        .any(|child| child.kind() == "default");
    found
}

/// Sums decision points across a section's own body statements only —
/// skipping the section node itself (already reflected in `case_count`) and
/// its case-label expression/pattern children, which are not statements.
/// Every `*_statement` node kind in the grammar counts as one statement, the
/// same convention `clone_signature::is_statement` uses (verified against
/// `node-types.json`).
fn section_body_complexity(section: Node<'_>) -> u32 {
    let mut cursor = section.walk();
    section
        .named_children(&mut cursor)
        .filter(|child| child.kind().ends_with("_statement"))
        .map(|statement| complexity::measure(statement).decision_points)
        .sum()
}

fn collect_created_types(node: Node<'_>, source: &SourceFile, found: &mut HashSet<String>) {
    if node.kind() == "object_creation_expression" {
        if let Some(type_node) = node.child_by_field_name("type") {
            if let Some(spelling) = text(type_node, source) {
                found.insert(spelling);
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_created_types(child, source, found);
    }
}

fn text(node: Node<'_>, source: &SourceFile) -> Option<String> {
    node.utf8_text(source.contents.as_bytes())
        .ok()
        .map(str::to_owned)
}

fn location(node: Node<'_>, path: &NormalizedPath) -> SourceLocation {
    let start = node.start_position();
    let end = node.end_position();
    let range = scent_domain::SourceRange::new(
        scent_domain::SourcePosition {
            line: to_u32(start.row),
            column: to_u32(start.column),
        },
        scent_domain::SourcePosition {
            line: to_u32(end.row),
            column: to_u32(end.column),
        },
    )
    .expect("Tree-sitter nodes cannot have backwards ranges");
    SourceLocation::new(path.clone(), range)
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).expect("source positions must fit the SCENT location format")
}

#[cfg(test)]
mod tests {
    use tree_sitter::Parser;

    use super::measure;
    use crate::csharp::SourceFile;
    use scent_domain::NormalizedPath;

    fn switch_shape(source_text: &str) -> super::SwitchShape {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source_text, None).unwrap();
        let mut cursor = tree.root_node().walk();
        let switch_node = find(tree.root_node(), &mut cursor, "switch_statement").unwrap();
        let source = SourceFile {
            path: NormalizedPath::parse("src/Sample.cs").unwrap(),
            contents: source_text.into(),
        };
        measure(switch_node, &source)
    }

    fn find<'a>(
        node: tree_sitter::Node<'a>,
        cursor: &mut tree_sitter::TreeCursor<'a>,
        kind: &str,
    ) -> Option<tree_sitter::Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }
        for child in node.named_children(cursor) {
            let mut child_cursor = child.walk();
            if let Some(found) = find(child, &mut child_cursor, kind) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn counts_sections_and_detects_default() {
        let shape = switch_shape(
            "class C { void M(int x) { switch (x) { case 1: break; case 2: break; default: break; } } }",
        );
        assert_eq!(shape.case_count, 3);
        assert!(shape.has_default);
        assert_eq!(shape.distinct_created_types, 0);
    }

    #[test]
    fn counts_distinct_created_types_across_sections() {
        let shape = switch_shape(
            "class C { void M(int x) { switch (x) { \
                 case 1: var a = new Circle(); break; \
                 case 2: var b = new Square(); break; \
                 case 3: var c = new Circle(); break; \
             } } }",
        );
        assert_eq!(shape.case_count, 3);
        assert_eq!(shape.distinct_created_types, 2);
    }

    #[test]
    fn measures_branch_complexity_from_the_densest_section() {
        let shape = switch_shape(
            "class C { void M(int x) { switch (x) { \
                 case 1: if (x > 0) { if (x > 1) { break; } } break; \
                 case 2: break; \
             } } }",
        );
        assert_eq!(shape.max_branch_complexity, 2);
    }
}
