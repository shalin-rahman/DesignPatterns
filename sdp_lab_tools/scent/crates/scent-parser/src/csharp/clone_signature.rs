//! A structural clone signature for one method body: its token stream with
//! identifiers and literals normalized away, then hashed. Two methods with
//! the same signature are structurally identical after renaming variables
//! and changing literal values — the common "copy-pasted, then tweaked"
//! duplicate (`docs/prompt.md` §24).
//!
//! Scope, stated plainly: this detects only *exact* structural duplicates
//! at whole-method granularity. It does not score partial/fuzzy similarity
//! across differently-sized snippets (the spec's 0.85 default similarity),
//! and it does not detect duplicated sub-method-sized blocks. Both are
//! real future work — approximating them here would mean fabricating a
//! similarity score this analyzer cannot actually justify.

use scent_domain::StableId;
use tree_sitter::Node;

use super::SourceFile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneSignature {
    pub statement_count: u32,
    pub hash: String,
}

/// Every `*_statement` node kind in the grammar counts as one statement —
/// a naming convention `tree-sitter-c-sharp` uses consistently, not a
/// guess (verified against its `node-types.json`).
fn is_statement(kind: &str) -> bool {
    kind.ends_with("_statement")
}

const LITERAL_KINDS: [&str; 8] = [
    "integer_literal",
    "real_literal",
    "string_literal",
    "verbatim_string_literal",
    "raw_string_literal",
    "character_literal",
    "boolean_literal",
    "null_literal",
];

#[must_use]
pub fn measure(body: Node<'_>, source: &SourceFile) -> CloneSignature {
    let mut tokens = Vec::new();
    let mut statement_count = 0;
    collect_tokens(body, source, &mut tokens, &mut statement_count);
    CloneSignature {
        statement_count,
        hash: StableId::from_identity(&tokens.join("\u{1}")).to_string(),
    }
}

fn collect_tokens(
    node: Node<'_>,
    source: &SourceFile,
    tokens: &mut Vec<String>,
    statement_count: &mut u32,
) {
    if is_statement(node.kind()) {
        *statement_count += 1;
    }
    if node.child_count() == 0 {
        let token = if node.kind() == "identifier" {
            "\u{0}ID".to_owned()
        } else if LITERAL_KINDS.contains(&node.kind()) {
            "\u{0}LIT".to_owned()
        } else {
            node.utf8_text(source.contents.as_bytes())
                .unwrap_or_default()
                .to_owned()
        };
        tokens.push(token);
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_tokens(child, source, tokens, statement_count);
    }
}

#[cfg(test)]
mod tests {
    use tree_sitter::Parser;

    use super::measure;
    use crate::csharp::SourceFile;
    use scent_domain::NormalizedPath;

    fn signature(source_text: &str) -> super::CloneSignature {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
            .unwrap();
        let source = SourceFile {
            path: NormalizedPath::parse("src/Order.cs").unwrap(),
            contents: source_text.into(),
        };
        let tree = parser.parse(&source.contents, None).unwrap();
        let mut cursor = tree.root_node().walk();
        let body = find(tree.root_node(), &mut cursor, "block").unwrap();
        measure(body, &source)
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
    fn renaming_identifiers_and_literals_does_not_change_the_signature() {
        let a = signature("class C { void M() { int total = 1; return; } }");
        let b = signature("class C { void M() { int amount = 42; return; } }");
        assert_eq!(a.hash, b.hash);
        assert_eq!(a.statement_count, 2);
    }

    #[test]
    fn a_different_operator_changes_the_signature() {
        let a = signature("class C { void M() { var x = a + b; } }");
        let b = signature("class C { void M() { var x = a - b; } }");
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn a_different_statement_shape_changes_the_signature() {
        let a = signature("class C { void M() { if (a) { return; } } }");
        let b = signature("class C { void M() { while (a) { return; } } }");
        assert_ne!(a.hash, b.hash);
    }
}
