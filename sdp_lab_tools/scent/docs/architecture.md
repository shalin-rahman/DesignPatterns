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

The current implementation reaches syntax extraction for namespaces and types.
All later boxes are planned work.

## Layer ownership

| Crate | Owns | Must not own |
| --- | --- | --- |
| `scent-domain` | IDs, paths, ranges, diagnostics, resolution contracts, common enums | parsing, metrics, CLI code |
| `scent-ir` | semantic facts and structural validation | Tree-sitter nodes, metrics, findings |
| `scent-parser` | discovery, language grammars, CST traversal, extracted facts | smell decisions, metrics, report formatting |
| `scent-core` | future orchestration and snapshots | language-specific CST traversal |
| `scent-cli` | future arguments, config loading, human interaction | semantic logic |

Dependencies flow from `scent-cli` toward `scent-domain`; lower crates do not
depend on CLI or report surfaces.

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
`MethodIR` reserves relationships for calls, type references, and object
instantiations. These fields intentionally carry no LOC, complexity, severity,
confidence, code-smell, or recommendation values.

`ValidateProjectIr` currently checks duplicate IDs and dangling file/type-owner
links. More invariants are added with the declaration and resolution stages.
