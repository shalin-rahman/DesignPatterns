//! Pipeline orchestration: discovery -> parse -> extract -> index -> resolve,
//! plus a deterministic JSON report of the result.

pub mod pipeline;
pub mod report;

pub use pipeline::{
    analyze_path, analyze_path_with_progress, analyze_path_without_git_history, AnalysisReport,
    AnalyzeError,
};
pub use report::to_json;
