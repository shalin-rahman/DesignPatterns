# Contributor Guide

## Before you change code

1. Read `README.md`, `docs/architecture.md`, and `docs/status.md`.
2. Read the matching section in `../docs/implementation_plan.md`.
3. Identify whether the change is a semantic fact, derived metric, evidence
   rule, report surface, or developer UX concern. Put it in that layer only.
4. Add a focused fixture or test before declaring the change complete.

## Development workflow

Run these commands from `scent/` after every code change:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Clippy warnings are errors in CI. Fix their cause; do not disable a lint merely
to pass a gate.

## Adding C# syntax support

1. Add a minimal C# snippet to `crates/scent-parser/tests/` or a named fixture.
2. Parse it through `CSharpAdapter`.
3. Extract only source facts into the IR or an explicit intermediate fact.
4. Preserve malformed syntax as a `Diagnostic`, rather than crashing or
   guessing a relationship.
5. Assert deterministic order and IDs when output includes collections.
6. Keep Tree-sitter `Node` and `Tree` values inside `scent-parser`.

## Adding semantic resolution

Resolution is deliberately two-pass:

1. Index declarations without assuming references are valid.
2. Resolve references only when exactly one intra-project target is valid.

Every other case is `Resolution::Unresolved(UnresolvedReference)`. A resolver
must not select a “most likely” overload, external type, or member.

## Adding metrics or rules

Metrics belong in `scent-metrics` and rules in `scent-rules`. Do not put
either in `scent-ir` or `scent-parser`.

Rules consume read-only facts, metrics, graph data, git history
(`AnalysisContext::history`, `None` when unavailable), configuration, and
suppressions. A rule emits evidence-backed findings; it does not edit source,
write files, or alter the IR. See `docs/LEARNING_GUIDE.md` §7 for the
step-by-step "add a rule" / "add a metric" walkthrough.

## Updating the golden JSON fixture

`scent-core/tests/golden.rs` compares the sample-project's JSON report
against a checked-in file (`fixtures/sample-project/expected_output.json`).
After an intentional change to the report's shape or content, regenerate it:

```powershell
$env:UPDATE_GOLDEN=1; cargo test -p scent-core --test golden; Remove-Item Env:\UPDATE_GOLDEN
```

Review the diff before committing — a passing regeneration only proves the
file now matches current output, not that the change was correct.

## Documentation expectations

Update `docs/status.md` when a milestone becomes implemented. Update
`docs/architecture.md` when an ownership or data-flow boundary changes. Keep
the README's “Current capabilities” and “not available yet” statements honest.
