# Implementation Status

Status is intentionally specific. “Planned” means no production capability is
claimed, even if a type or placeholder crate exists.

| Area | Status | Evidence |
| --- | --- | --- |
| Rust workspace and CI gate | Implemented | workspace manifests and `.github/workflows/ci.yml` |
| Deterministic typed IDs | Implemented | `scent-domain::id` tests |
| Repository-relative path normalization | Implemented | `scent-domain::path` tests |
| Source locations, diagnostics, resolution contracts | Implemented | `scent-domain` modules |
| Facts-only IR and basic validation | Implemented | `scent-ir::model` and validation tests, including `FieldAccess`/`PropertyId` (closed a gap against `docs/prompt.md` §10 found during review) |
| C# discovery and default exclusions | Implemented | `scent-parser::discovery` fixture |
| C# Tree-sitter parsing and parse diagnostics | Implemented | `CSharpAdapter` tests |
| Namespace and type declaration extraction | Implemented | `csharp.rs` tests |
| Member extraction | Implemented | methods, constructors, fields, properties, parameters (`csharp/extract.rs`) |
| Calls, creations, and field accesses inside method bodies | Implemented | recorded as `Unresolved` facts; see `collect_method_facts` |
| Inheritance/interface-list extraction | Implemented (interfaces only) | `TypeIR.interface_types` holds every `base_list` entry unresolved; `base_types` stays empty until Milestone 4 can split base-class-vs-interface from a resolved `TypeKind` (see `parse_base_list`) |
| Two-pass symbol index/resolution | Planned | exact intra-project resolution only |
| `scent analyze <path>` | Planned | CLI currently prints a bootstrap message |
| Deterministic IR JSON | Planned | no report crate yet |
| Metrics and dependency graph | Planned | no metrics/graph crates yet |
| Findings, evidence, rules, SARIF, baseline | Planned | no rules/report crates yet |
| TUI, Docker sandbox, cache, C++ | Planned | later milestones |

## Immediate next steps

1. Build sorted declaration indexes (`index.rs`) from the now-complete
   extracted facts.
2. Resolve only unambiguous intra-project references (`resolve.rs`), including
   the deferred base-class-vs-interface split for `TypeIR.base_types`.
3. Add a standalone suppression scanner producing `SuppressionMap`.
4. Wire discovery, parsing, extraction, indexing, and resolution into
   `scent-core`, then into `scent analyze <path> [--format json]`.
5. Emit a deterministic semantic-IR JSON snapshot with golden fixtures.

The detailed milestone and acceptance criteria are maintained in
`../../docs/implementation_plan.md`.
