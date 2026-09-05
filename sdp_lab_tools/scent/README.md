# SCENT

**SCENT** is an evidence-based static-analysis platform for C# projects.
It is written in Rust, but it analyzes C# source code. Rust is the
implementation language for the analyzer; it does not change, compile, or
otherwise require a target C# project to use Rust.

```text
C# source → Tree-sitter C# syntax tree → semantic facts → metrics → findings → reports
```

The codebase runs the full pipeline end to end today: discovery through
Tree-sitter parsing, semantic extraction, two-pass resolution, a typed
dependency graph, metrics, all 11 Refactoring.Guru rules, git-history rules,
a first slice of principle/pattern/refactoring advisors, and JSON/SARIF/
quality-gate reporting. See "Analyzing a project" below to run it.

## Current capabilities

- Discover `.sln`, `.slnx`, `.csproj`, and `.cs` files deterministically.
- Exclude `bin`, `obj`, `generated`, `*.g.cs`, and `*.Designer.cs` by default.
- Parse C# source, extract facts (namespaces, types, members, calls, field
  accesses, instantiations, local variables), and resolve intra-project
  references — never guessing framework/ambiguous ones.
- Build a typed dependency graph and calculate LOC, cyclomatic complexity,
  nesting depth, LCOM4, and CBO.
- Run all 11 Refactoring.Guru rules (Long Method, Large Class, Long
  Parameter List, Data Clumps, Primitive Obsession, Inappropriate Intimacy,
  Refused Bequest, Speculative Generality, Feature Envy, Duplicated Code,
  Switch Statements) plus the 2 git-history rules (Divergent Change,
  Shotgun Surgery, when `git` is available).
- Assess all 9 design-principle risks (SRP, OCP, LSP, ISP, DIP, DRY, KISS,
  YAGNI, Law of Demeter), recommend Factory Method/"Do Nothing" for switch
  statements, and map findings to a standard refactoring name.
- Load per-rule enabled/severity/threshold overrides and quality-gate
  defaults from an optional `smell_detector.toml` in the analyzed project.
- Enforce `// scent:disable RULE_ID` / `// scent:enable RULE_ID`
  suppressions before reporting.
- Output deterministic JSON, SARIF, or a human-readable summary; run as a
  pass/fail quality gate with an optional baseline.
- Generate deterministic SHA-256 identifiers and repository-relative POSIX
  paths throughout.

Read [docs/status.md](docs/status.md) for the precise, continuously-updated
implementation status (including what's still planned), and
[docs/architecture.md](docs/architecture.md) for the design.
[docs/LEARNING_GUIDE.md](docs/LEARNING_GUIDE.md) is a plain-language
walkthrough of the whole pipeline plus a Rust (and C# comparison) primer.
[docs/CLI_WORKFLOW_GUIDE.md](docs/CLI_WORKFLOW_GUIDE.md) shows every command
with real, captured example output and a step-by-step trace of what each
pipeline stage does.

## Requirements

- Rust stable, with `rustfmt` and Clippy. The repository pins the toolchain
  through `rust-toolchain.toml`.
- A supported host C/C++ toolchain, because Tree-sitter's grammar is compiled
  as part of the Rust build. On Windows, install Visual Studio Build Tools with
  the C++ build tools workload.

No .NET SDK is required — SCENT parses C# source text directly with
Tree-sitter; it does not compile or execute the target project. `git` on
`PATH` is optional: the 2 history-based rules simply produce no findings
when the analyzed path isn't a Git work tree or `git` is unavailable.

## Build and verify

From this directory:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Analyzing a project

Point it at the root of a C# solution or project (the directory containing
its `.sln`/`.csproj` files, or any ancestor of them):

```powershell
cargo run -p scent-cli -- analyze <path> [--format human|json|sarif]
cargo run -p scent-cli -- rules
cargo run -p scent-cli -- gate <path> [--max-critical N] [--max-high N] [--baseline FILE]
```

`analyze` defaults to a human-readable summary; `--format json` is the full
deterministic report (facts, metrics, findings, principle risks, pattern
and refactoring recommendations); `--format sarif` is for CI/IDE tools that
consume SARIF. `gate` exits non-zero when the findings exceed the
configured thresholds (or introduce new violations against a baseline),
for use in a CI pipeline. Progress lines are printed to stderr as each
pipeline stage runs, so stdout stays a clean, pipeable report.

## Repository layout

```text
crates/
├── scent-cli/       command-line entry point
├── scent-core/      pipeline orchestration, the combined JSON/diagnostic report
├── scent-domain/    stable IDs, paths, locations, diagnostics, and contracts
├── scent-git/       git history: commit parsing, entity-resolved changes, co-change graph
├── scent-graph/     typed dependency graph built from resolved facts
├── scent-ir/        facts-only semantic intermediate representation
├── scent-metrics/   LOC, cyclomatic complexity, nesting, LCOM4, CBO
├── scent-parser/    C# discovery, Tree-sitter parsing, syntax extraction, resolution
├── scent-report/    JSON writer, SARIF, baseline, quality gate
└── scent-rules/     the rule engine, evidence/confidence/severity, and the advisors
docs/
├── architecture.md     layer boundaries and data flow
├── LEARNING_GUIDE.md   plain-language pipeline walkthrough + Rust/C# primer
├── contributor-guide.md
└── status.md            implemented vs planned work, updated continuously
```

## Principles

- Facts are not interpretations: IR contains what source says, not a judgment
  that code is good or bad.
- Unresolved symbols must stay explicit; SCENT must not guess.
- Analytical identities and output ordering must be deterministic.
- Confidence and severity are independent values backed by evidence.
- C# is the first target; the IR must remain language-neutral so C++ can follow.

## Contributing

Start with [docs/contributor-guide.md](docs/contributor-guide.md). Each change
must keep the three verification commands above green and add a fixture or unit
test for new syntax and semantic behavior.

## Roadmap

Remaining work: config loading currently overrides one threshold per rule,
not every field on every rule; then the TUI, Docker sandbox, incremental
cache, and C++ support. See [docs/status.md](docs/status.md) for the exact,
current breakdown and `../docs/implementation_plan.md` for the full
sequenced plan.
