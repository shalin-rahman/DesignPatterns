//! C# CST construction and parse-error collection.

use scent_domain::{
    Diagnostic, DiagnosticKind, NormalizedPath, SourceLocation, SourcePosition, SourceRange,
};
use tree_sitter::{Node, Parser, Tree};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    pub path: NormalizedPath,
    pub contents: String,
}

#[derive(Debug)]
pub struct CSharpParseResult {
    pub tree: Tree,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct CSharpAdapter {
    parser: Parser,
}

impl CSharpAdapter {
    #[must_use]
    /// Creates an adapter using SCENT's bundled C# grammar.
    ///
    /// # Panics
    ///
    /// Panics only if the statically bundled Tree-sitter grammar is invalid.
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
            .expect("the bundled C# grammar must be valid");
        Self { parser }
    }

    #[must_use]
    /// Parses in-memory UTF-8 C# source and retains syntax errors as diagnostics.
    ///
    /// # Panics
    ///
    /// Panics only if Tree-sitter fails to return a tree for in-memory source.
    pub fn parse(&mut self, source: &SourceFile) -> CSharpParseResult {
        let tree = self
            .parser
            .parse(&source.contents, None)
            .expect("Tree-sitter returns a tree for in-memory UTF-8 source");
        let mut diagnostics = Vec::new();
        collect_parse_diagnostics(tree.root_node(), source, &mut diagnostics);
        CSharpParseResult { tree, diagnostics }
    }
}

impl Default for CSharpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_parse_diagnostics(
    node: Node<'_>,
    source: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.is_error() || node.is_missing() {
        let position = node.start_position();
        let end = node.end_position();
        let range = SourceRange::new(
            SourcePosition {
                line: to_u32(position.row),
                column: to_u32(position.column),
            },
            SourcePosition {
                line: to_u32(end.row),
                column: to_u32(end.column),
            },
        )
        .expect("Tree-sitter nodes cannot have backwards ranges");
        diagnostics.push(Diagnostic::new(
            DiagnosticKind::ParseError,
            if node.is_missing() {
                "missing C# syntax"
            } else {
                "invalid C# syntax"
            },
            Some(SourceLocation::new(source.path.clone(), range)),
        ));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_parse_diagnostics(child, source, diagnostics);
    }
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).expect("source positions must fit the SCENT location format")
}
