//! Objective decision-point and nesting counts, derived straight from the
//! syntax tree. These are still raw counts, not a metric formula or an
//! interpretation — `scent-metrics` turns them into cyclomatic complexity
//! (`1 + decision_points`) and a nesting-depth value. Kept separate from
//! `MethodIR` so the facts-only IR never carries a derived number.

use tree_sitter::Node;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ComplexitySignals {
    pub decision_points: u32,
    pub max_nesting_depth: u32,
}

/// Counts decision points recognized for C#: `if`, `for`, `foreach`,
/// `while`, `do`, `switch` sections, `catch`, the conditional (`?:`)
/// operator, and logical `&&`/`||` branching, per the definition in
/// `docs/prompt.md` §12. Nesting depth counts `if`/`for`/`foreach`/`while`/
/// `do`/`switch`/`catch` as one nesting level each.
#[must_use]
pub fn measure(body: Node<'_>) -> ComplexitySignals {
    let mut signals = ComplexitySignals::default();
    walk(body, 0, &mut signals);
    signals
}

fn walk(node: Node<'_>, depth: u32, signals: &mut ComplexitySignals) {
    // `switch_statement` itself is not a decision point — its `switch_section`
    // children are the actual branches — but it still opens one nesting level
    // for whatever its sections contain.
    let nests = matches!(
        node.kind(),
        "if_statement"
            | "for_statement"
            | "foreach_statement"
            | "while_statement"
            | "do_statement"
            | "switch_statement"
            | "catch_clause"
    );
    let is_decision_point = matches!(
        node.kind(),
        "if_statement"
            | "for_statement"
            | "foreach_statement"
            | "while_statement"
            | "do_statement"
            | "catch_clause"
            | "switch_section"
            | "conditional_expression"
    );
    if is_decision_point {
        signals.decision_points += 1;
    }
    if node.kind() == "binary_expression" {
        let is_logical = node
            .child_by_field_name("operator")
            .is_some_and(|operator| matches!(operator.kind(), "&&" | "||"));
        if is_logical {
            signals.decision_points += 1;
        }
    }

    let next_depth = if nests { depth + 1 } else { depth };
    signals.max_nesting_depth = signals.max_nesting_depth.max(next_depth);

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        walk(child, next_depth, signals);
    }
}

#[cfg(test)]
mod tests {
    use tree_sitter::Parser;

    use super::measure;

    fn body_signals(source: &str) -> super::ComplexitySignals {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();
        let mut cursor = tree.root_node().walk();
        let body = find(tree.root_node(), &mut cursor, "block").unwrap();
        measure(body)
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
    fn counts_a_flat_method_as_a_single_decision_point_free_body() {
        let signals = body_signals("class C { void M() { var x = 1; } }");
        assert_eq!(signals.decision_points, 0);
        assert_eq!(signals.max_nesting_depth, 0);
    }

    #[test]
    fn counts_an_if_statement_as_one_decision_point_and_one_nesting_level() {
        let signals = body_signals("class C { void M() { if (true) { var x = 1; } } }");
        assert_eq!(signals.decision_points, 1);
        assert_eq!(signals.max_nesting_depth, 1);
    }

    #[test]
    fn counts_nested_control_flow_depth() {
        let signals =
            body_signals("class C { void M() { if (true) { for (;;) { var x = 1; } } } }");
        assert_eq!(signals.decision_points, 2);
        assert_eq!(signals.max_nesting_depth, 2);
    }

    #[test]
    fn counts_logical_operators_and_the_conditional_operator() {
        let signals = body_signals("class C { void M() { var x = a && b || c ? 1 : 2; } }");
        assert_eq!(signals.decision_points, 3);
    }

    #[test]
    fn counts_each_switch_section() {
        let signals = body_signals(
            "class C { void M() { switch (x) { case 1: break; case 2: break; default: break; } } }",
        );
        assert_eq!(signals.decision_points, 3);
    }
}
