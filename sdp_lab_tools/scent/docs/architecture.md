# Architecture

## Purpose

SCENT analyzes source code structurally. Its implementation is Rust; C# is the
first input language. The language boundary ends at the parser adapter. Every
later layer operates over language-neutral contracts.

## Data-flow contract

```text
source files
    ↓
discovery
    ↓
Tree-sitter C# concrete syntax tree
    ↓
syntax extraction
    ↓
declaration index
    ↓
symbol resolution
    ↓
semantic IR (facts only)
    ↓
metrics and dependency graph
    ↓
evidence and rules
    ↓
findings, principle risks, and recommendations
    ↓
JSON / SARIF / TUI / quality gate
```

The implementation now reaches through rules and reporting: every box in
this diagram is implemented, including git history (`scent-git`) and a
first slice of principle risks and recommendations (`scent-rules::
{principles,patterns,refactoring}`) — see `docs/status.md` for exactly
which of the 9 design principles are assessed and why the rest are
deferred rather than fabricated. The TUI, sandbox, cache, and C++ remain
planned. See `docs/status.md` for the authoritative, continuously-updated
breakdown — this file describes the stable shape, not the day-to-day status.

## Layer ownership

| Crate | Owns | Must not own |
| --- | --- | --- |
| `scent-domain` | IDs, paths, ranges, diagnostics, resolution contracts, common enums | parsing, metrics, CLI code |
| `scent-ir` | semantic facts and structural validation | Tree-sitter nodes, metrics, findings |
| `scent-parser` | discovery, language grammars, CST traversal, extracted facts, two-pass resolution | smell decisions, metrics, report formatting |
| `scent-graph` | typed dependency edges built from resolved facts | rule/finding logic |
| `scent-metrics` | LOC, cyclomatic complexity, nesting, LCOM4, CBO | finding/severity decisions |
| `scent-git` | shelling out to `git`, commit parsing, entity-resolved changes, the co-change graph | rule/finding logic, CST/language concerns |
| `scent-config` | parsing `smell_detector.toml`, per-rule enabled/severity/threshold overrides, quality-gate overrides | rule logic, CLI |
| `scent-rules` | `Rule` trait, evidence/confidence/severity engine, the rule set, principle risk/pattern/refactoring advisors (read findings, never mutate them) | report formatting, CLI |
| `scent-report` | JSON writer, SARIF, baseline, quality gate | rule logic |
| `scent-core` | pipeline orchestration, the combined JSON/diagnostic report | language-specific CST traversal |
| `scent-cli` | argument parsing, human-readable output | semantic logic |

Dependencies flow from `scent-cli` toward `scent-domain`; lower crates do not
depend on CLI or report surfaces. `scent-core` is the only crate allowed to
depend on everything else, since it orchestrates the whole pipeline.

## Facts versus interpretations

The IR may state that a method calls another method, a class has a field, or a
reference is unresolved. It must never store a conclusion such as “this is a
Long Method.”

```text
Semantic fact → derived metric → evidence → finding → risk → recommendation
```

This separation makes results explainable, testable, and safe to consume in
automation.

## Determinism

SCENT normalizes project paths to repository-relative POSIX form and derives
IDs from semantic identity inputs. Filesystem traversal is sorted before data is
returned. The future report layer must sort every unordered collection and must
exclude timestamps and durations from analytical identity.

Runtime metadata is allowed for progress displays, but never for fingerprints,
baselines, or deterministic JSON/SARIF output.

## C# parser boundary

`scent-parser` uses `tree-sitter-c-sharp` to make a concrete syntax tree. A CST
is temporary parser data, not SCENT's semantic model. `CSharpAdapter` reports
syntax errors as structured diagnostics and `extract_file` converts supported
declarations into `FileIR`, `NamespaceIR`, and `TypeIR` values.

Tree-sitter does not provide full C# compilation semantics. The resolver will
only resolve exact, unambiguous intra-project symbols at first. Framework,
NuGet, and ambiguous references must be represented as `UnresolvedReference`
until a future metadata provider can establish them safely.

## Current semantic model

`ProjectIR` contains files, namespaces, types, methods, fields, and properties.
`MethodIR` carries calls, field accesses, type references, object
instantiations, and local variable declarations, with each call/access
carrying a `CallReceiver` (`SelfOrImplicit` / `Named` / `Other`) recording
what the call/access syntax says its receiver is — the resolver follows
`SelfOrImplicit` (against the enclosing type) and `Named` (against a
same-named local variable, parameter, or field's own resolved type, checked
in that shadowing order); a receiver it cannot account for stays
`Unresolved` with a stated reason. These fields intentionally carry no LOC,
complexity, severity, confidence, code-smell, or recommendation values —
see `docs/LEARNING_GUIDE.md` §3 for a full worked trace of one file through
every stage.

`ValidateProjectIr` checks duplicate IDs and dangling file/type/owner links
across types, methods, fields, and properties.

## Reading further

- `docs/LEARNING_GUIDE.md` — plain-language walkthrough of the whole
  pipeline with a real example, plus a Rust primer for newcomers to the
  language.
- `docs/status.md` — exactly what is implemented right now.
- `docs/implementation_plan.md` (repo root `docs/`) — the milestone plan.
