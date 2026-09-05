//! C# project discovery and, later, language adapters.

pub mod csharp;
pub mod discovery;

pub use csharp::{
    extract_file, resolve_project, CSharpAdapter, CSharpParseResult, CloneSignature,
    ComplexitySignals, DeclarationIndex, ExtractedFile, Lookup, SourceFile, SwitchShape,
};
pub use discovery::{
    discover_csharp, DiscoveryError, DiscoveryOptions, DiscoveryResult, ProjectDiscoveryStatus,
};
