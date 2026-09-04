//! C# project discovery and, later, language adapters.

pub mod csharp;
pub mod discovery;

pub use csharp::{CSharpAdapter, CSharpParseResult, ExtractedFile, SourceFile};
pub use discovery::{
    discover_csharp, DiscoveryError, DiscoveryOptions, DiscoveryResult, ProjectDiscoveryStatus,
};
