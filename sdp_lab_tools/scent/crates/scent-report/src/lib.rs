//! Report formats built on top of `scent-rules::Finding`: JSON, SARIF,
//! baseline comparison, and the quality gate. The JSON writer here is also
//! reused by `scent-core` for the Phase 1 IR/diagnostic report, so the
//! workspace has exactly one deterministic JSON serializer.

pub mod baseline;
pub mod findings;
pub mod json;
pub mod quality_gate;
pub mod sarif;

pub use baseline::Baseline;
pub use findings::{finding_json, findings_to_json_string, read_snippet, Snippet};
pub use json::Json;
pub use quality_gate::{evaluate_gate, GateResult, QualityGateConfig};
pub use sarif::to_sarif;
