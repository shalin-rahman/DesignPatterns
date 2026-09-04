# SCENT v1.1 — Verified Implementation Plan

## Purpose and scope

This is the executable handoff plan for SCENT. It reconciles the architectural
contract in `prompt.md` with `execution_plan.txt`. The contract takes priority
when they differ.

Implementation root: `sdp_lab_tools/scent/`.

The first release analyzes C# source only. It provides deterministic,
evidence-based structural findings; it does not identify code authorship or
make an LLM call.

## Non-negotiable decisions

1. Semantic facts, metrics, evidence, findings, risks, and recommendations are
   separate models and separate pipeline stages.
2. All externally visible analytical data is deterministic: repository-relative
   POSIX paths, stable IDs/fingerprints, canonical ordering, and no runtime
   metadata in analytical identity.
3. Phase 1 resolves only exact, intra-project C# symbols. Framework, NuGet, and
   ambiguous symbols remain explicit `Unresolved` references. No heuristic
   guessing is permitted.
4. Rule thresholds calibrate normalized evidence; a threshold alone must never
   emit a binary finding.
5. Findings are ordered by deterministic fingerprint. Runtime IDs are never
   needed for deterministic report ordering or baseline identity.
6. `location_hash` is a normalized syntax/context anchor, not a line number, so
   edits above a finding do not invalidate a baseline.
7. Rules are pure readers of `AnalysisContext`; they neither parse suppressions
   nor mutate the IR, graph, metrics, source, or filesystem.

## Corrected delivery sequence

### Milestone 0 — Bootstrap

Create the Cargo workspace and the initial crate layout:

```text
scent/
├── Cargo.toml
├── rust-toolchain.toml
├── crates/{scent-cli,scent-core,scent-domain,scent-ir,scent-parser}/
├── fixtures/
├── tests/
├── docs/
├── examples/
├── .github/workflows/ci.yml
└── smell_detector.toml
```

Add formatting, Clippy, tests, error conventions, and CI. Do not create future
crates until a milestone requires them.

Acceptance:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

### Milestone 1 — Domain contracts

Implement `scent-domain` before parser work.

Files:

```text
src/id.rs              deterministic ProjectId, FileId, EntityId, TypeId, MethodId, FieldId
src/path.rs            NormalizedPath
src/location.rs        SourceLocation and ranges
src/diagnostic.rs      structured non-fatal diagnostics
src/resolution.rs      Resolution<T> and UnresolvedReference
src/analysis.rs        Language, visibility, scope, severity, configuration contracts
```

IDs are SHA-256 hashes over normalized semantic identity inputs. Path
normalization rejects absolute paths and converts separators to `/`.

Tests: stable hash vectors; different semantic signatures produce different
IDs; equivalent Windows/POSIX relative paths normalize identically; absolute
paths are rejected; all serialization output is stable.

### Milestone 2 — Semantic IR before parsing

Implement the minimal facts-only `scent-ir` schema before the C# adapter.

Files: `project.rs`, `file.rs`, `namespace.rs`, `types.rs`, `members.rs`,
`facts.rs`, and `validate.rs`.

`ProjectIR` contains sorted files, namespaces, types, and members. Methods
contain calls, field/property accesses, type references, and instantiations;
they do not contain metrics, smells, confidence, or recommendations.

Tests: construct a small project fixture, serialize it twice, and assert
byte-identical canonical JSON plus validation of parent/entity references.

Review note: a later audit found this milestone's first pass omitted
`MethodIR.field_accesses: Vec<FieldAccess>`, required by `prompt.md` §10
alongside `calls`/`type_references`/`instantiations`, and left `PropertyIR`
without an `id` field (inconsistent with `FieldIR`/`MethodIR`). Both were
closed as part of Milestone 3's implementation rather than reopening this
milestone, since they only became exercisable once member extraction existed
to populate them. `FieldAccess.target` resolves to a `MemberTarget` (`Field`
or `Property`) rather than a bare `FieldId`, because C# syntax alone cannot
tell a field access from a property access — that requires the owner type's
resolved member index, which is Milestone 4's job.

### Milestone 3 — Discovery and C# syntax extraction

Implement `scent-parser` as two separable layers:

```text
discovery.rs     find .sln/.slnx/.csproj/.cs and apply exclusions
csharp/parse.rs  Tree-sitter parse and diagnostics
csharp/extract.rs declarations and unresolved syntax facts
```

Discovery reports `Discovered`, `PartiallyResolved`, or `Unresolved`; it does
not attempt full MSBuild evaluation. The extractor supports namespaces, types,
fields, properties, constructors, methods, parameters, inheritance,
interfaces, type references, creations, calls, and member accesses required by
Phase 1.

Tests: discovery exclusions; basic, nested, interface, generic, malformed, and
generated-like fixtures. Parsing errors become diagnostics and do not prevent
other files from being analyzed.

Review note: `TypeIR.base_types` intentionally stays empty for every type at
extraction time. A `base_list` entry cannot be classified as a base class
versus an implemented interface from C# syntax alone — only convention ("base
class comes first, if present") suggests it, and a convention is a guess, not
a fact. Every `base_list` entry is instead recorded into `interface_types` as
an `Unresolved` reference; Milestone 4's resolver, once it knows a resolved
entry's `TypeKind`, moves a resolved `Class`/`Record` occupying the first
position into `base_types`. This keeps the "never guess a semantic
relationship" rule (`prompt.md` §9) intact rather than trading it for a
plausible-looking shortcut.

### Milestone 4 — Two-pass resolution

Add `index.rs` and `resolve.rs` in `scent-parser`.

Pass 1 creates namespace/type/member indexes from extracted declarations.
Pass 2 resolves only unambiguous intra-project references. Every failure is
retained as an `UnresolvedReference` containing kind, spelling, source location,
and reason. Resolver output is sorted by entity identity.

Tests: exact type/method/field resolution; overload ambiguity; unknown
framework type; unresolved member; unrelated source changes leave stable entity
IDs unchanged.

Follow-up carried from Milestone 3: once an `interface_types` entry resolves
and its `TypeKind` is known, a resolved `Class`/`Record` in the first position
moves to `base_types`; everything else (interfaces, structs/interfaces which
cannot have a base class) stays in `interface_types`.

### Milestone 5 — Suppressions, CLI, and Phase-1 report

`scent-core` orchestrates discovery → parse → extract → index → resolve → IR.
Add a standalone suppression scanner that creates `SuppressionMap` from
`scent:disable` / `scent:enable` comments. `scent-cli` implements only:

```text
scent analyze <path> [--format json]
```

The JSON report is a canonical IR/diagnostic report. It contains no timestamp
or duration. The CLI may display runtime data in human output only.

Tests: end-to-end golden JSON snapshots; repeated runs are byte-identical;
comments are scanned once; malformed files return diagnostics and a non-crash
result.

Phase-1 exit gate: all five foundation crates compile; `scent analyze` reports
entities and unresolved references for every golden fixture; the three
Bootstrap commands pass.

### Milestone 6 — Metrics, graph, and clone infrastructure

Add `scent-graph` and `scent-metrics`.

Implement typed, indexed dependency edges; LOC definitions; cyclomatic
complexity; nesting depth; LCOM4; CBO; AST normalization; and clone subtree
hashing. Store metrics separately in `MetricStore`. Sort all graph adjacency
lists and serialized values.

Tests: hand-calculated metric fixtures, deterministic graph snapshots, graph
index query coverage, and normalized clone candidates.

### Milestone 7 — Finding platform and reporting

Add `scent-rules` and `scent-report`.

Implement `AnalysisContext`, `Rule`, `RuleRegistry`, `EvidenceItem`, evidence
normalization/aggregation, independent severity calculation, finding
fingerprints, JSON, SARIF, baseline, and quality-gate contracts. Add report
schema fixtures and validate SARIF output.

Acceptance: identical inputs produce byte-identical JSON and SARIF reports;
baseline comparison is stable across unrelated line shifts.

### Milestone 8 — Rules, history, and intelligence

Implement rules in this dependency order:

1. Long Parameter List, Long Method, Large Class
2. Duplicated Code, Feature Envy, Switch Statements, Data Clumps
3. Primitive Obsession, Inappropriate Intimacy, Refused Bequest, Speculative Generality
4. Git-backed Divergent Change and Shotgun Surgery
5. Principle risks, refactoring advisor, and contextual pattern advisor

Each rule gets evidence-focused positive, negative, and suppression fixtures.
Git history is required only for the two history rules and uses entity-resolved
changes rather than line counts.

### Milestone 9 — Product surfaces and scale

Build the TUI against `AnalysisSnapshot`, then Docker execution, then the
content-addressable/incremental cache, followed by C++ discovery/adapter.
Each is optional until all preceding milestones are stable.

## Traceability and handoff checklist

| Contract requirement | Delivery milestone | Proof |
| --- | --- | --- |
| Facts separate from interpretations | 1–5 | IR validation and no derived fields in IR |
| Deterministic identity/output | 1–7 | stable ID, snapshot, and report tests |
| Explicit unresolved references | 4–5 | resolver fixture diagnostics |
| C# Phase-1 CLI | 5 | end-to-end `scent analyze` fixture |
| Metrics and typed graph | 6 | calculated metrics and graph snapshots |
| Explainable findings and SARIF | 7 | evidence/schema/SARIF tests |
| Refactoring.Guru and Git smells | 8 | rule and history fixtures |
| Principles, patterns, refactoring | 8 | recommendation tests |
| TUI, sandbox, cache, C++ | 9 | component-specific integration tests |

## Operating rule for the next implementer

Work one milestone at a time. Before opening the next milestone, run the
current milestone's acceptance checks and keep golden fixtures deterministic.
If an implementation requires semantic data not supported by the current
adapter, retain an unresolved fact and add a diagnostic—never infer it.
