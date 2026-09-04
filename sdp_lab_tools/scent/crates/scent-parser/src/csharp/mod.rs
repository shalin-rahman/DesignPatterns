//! Tree-sitter-backed C# parsing and syntax extraction.

mod extract;
mod parse;

pub use extract::{extract_file, ExtractedFile};
pub use parse::{CSharpAdapter, CSharpParseResult, SourceFile};
