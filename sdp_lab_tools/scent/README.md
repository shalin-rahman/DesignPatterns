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
pipeline stage does. [docs/JSON_OUTPUT_REFERENCE.md](docs/JSON_OUTPUT_REFERENCE.md)
explains, in plain words, what every field in `--format json` means
(severity vs. risk, confidence, evidence, and so on).

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
its `.sln`/`.csproj` files, or any ancestor of them). This directory does
**not** need to be inside this repository — any path on disk works, since
SCENT only reads C# source text; it never compiles or runs the target
project:

```powershell
cargo run -p scent-cli -- --help
cargo run -p scent-cli -- analyze <path> [--format human|json|sarif|table]
cargo run -p scent-cli -- analyze --help
cargo run -p scent-cli -- rules
cargo run -p scent-cli -- gate <path> [--max-critical N] [--max-high N] [--baseline FILE]
cargo run -p scent-cli -- gate --help
```

`cargo run -p scent-cli --` works from inside this repository. To run it as
a plain `scent` command from anywhere (any shell, any directory), install
it once:

```powershell
cargo install --path crates/scent-cli
```

This builds a release binary and copies it to `~/.cargo/bin/scent.exe`
(Cargo's standard install location, already on `PATH` if you installed Rust
via `rustup`). After that, every command above works with `scent` in place
of `cargo run -p scent-cli --`, for example:

```powershell
scent analyze <path> --format table
```

Re-run `cargo install --path crates/scent-cli` after pulling code changes
to update the installed binary; `cargo uninstall scent-cli` removes it.

| Command | What it does |
|---|---|
| `--help` (top level, or after `analyze`/`gate`) | Prints usage and every flag for that command. Exits 0. |
| `analyze <path>` | Runs the whole pipeline and prints a report. Defaults to `--format human`. |
| `analyze <path> --format json` | The full deterministic report — every fact, metric, finding, principle risk, and recommendation. See [docs/JSON_OUTPUT_REFERENCE.md](docs/JSON_OUTPUT_REFERENCE.md) for what every field means. |
| `analyze <path> --format sarif` | [SARIF 2.1.0](https://sarifweb.azurewebsites.net/) output for CI/IDE tools (GitHub code scanning, VS Code's SARIF viewer). |
| `analyze <path> --format table` | Findings and principle risks as aligned text columns instead of free-form sentences or JSON. |
| `rules` | Lists every registered rule id and name. |
| `gate <path>` | Runs the analysis, then passes/fails against a quality gate (`--max-critical`, `--max-high`, or `[quality_gate]` in `smell_detector.toml`). Exits non-zero on failure — the shape a CI step needs. |

Progress lines are printed to stderr as each pipeline stage runs, so stdout
stays a clean, pipeable report. See
[docs/CLI_WORKFLOW_GUIDE.md](docs/CLI_WORKFLOW_GUIDE.md) for a real,
captured run of every command above, and what each pipeline stage does
internally.

Shell tab-completion (bash and PowerShell) is available under
[`completions/`](completions/) — see [completions/README.md](completions/README.md)
to install it. Because SCENT's argument parser is hand-rolled rather than
built on a framework like `clap`, these scripts complete command and flag
*names* only; they cannot suggest file paths from inside your shell
smarter than your shell already does.

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
├── architecture.md          layer boundaries and data flow
├── LEARNING_GUIDE.md        plain-language pipeline walkthrough + Rust/C# primer
├── CLI_WORKFLOW_GUIDE.md    every command, real captured output, pipeline trace
├── JSON_OUTPUT_REFERENCE.md what every field in --format json means, in plain words
├── contributor-guide.md
└── status.md                implemented vs planned work, updated continuously
completions/
├── scent.bash               bash tab-completion for command/flag names
├── scent.ps1                PowerShell tab-completion for command/flag names
└── README.md                how to install either one
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
