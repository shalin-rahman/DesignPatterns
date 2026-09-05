//! Tree-sitter-backed C# parsing and syntax extraction.

mod clone_signature;
mod complexity;
mod extract;
mod index;
mod parse;
mod resolve;
mod switch_shape;

pub use clone_signature::CloneSignature;
pub use complexity::ComplexitySignals;
pub use extract::{extract_file, ExtractedFile};
pub use index::{DeclarationIndex, Lookup};
pub use parse::{CSharpAdapter, CSharpParseResult, SourceFile};
pub use resolve::resolve_project;
pub use switch_shape::SwitchShape;
