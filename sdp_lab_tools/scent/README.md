# SCENT

**SCENT** is an evidence-based static-analysis platform for C# projects.
It is written in Rust, but it analyzes C# source code. Rust is the
implementation language for the analyzer; it does not change, compile, or
otherwise require a target C# project to use Rust.

```text
C# source → Tree-sitter C# syntax tree → semantic facts → metrics → findings → reports
```

The current codebase is an early, tested foundation. It discovers C# projects,
parses C# with Tree-sitter, collects syntax diagnostics, and extracts namespace
and type declarations. It does not yet produce code-smell findings.

## Current capabilities

- Discover `.sln`, `.slnx`, `.csproj`, and `.cs` files deterministically.
- Exclude `bin`, `obj`, `generated`, `*.g.cs`, and `*.Designer.cs` by default.
- Parse C# source using the Tree-sitter C# grammar.
- Retain malformed-source diagnostics rather than terminating an analysis.
- Extract file-scoped and block namespaces plus class, interface, struct, enum,
  record, and nested type declarations.
- Generate deterministic SHA-256 identifiers and repository-relative POSIX
  paths.

Read [docs/status.md](docs/status.md) for the precise implementation status,
and [docs/architecture.md](docs/architecture.md) for the design.

## Requirements

- Rust stable, with `rustfmt` and Clippy. The repository pins the toolchain
  through `rust-toolchain.toml`.
- A supported host C/C++ toolchain, because Tree-sitter's grammar is compiled
  as part of the Rust build. On Windows, install Visual Studio Build Tools with
  the C++ build tools workload.

No .NET SDK is required for the implemented discovery and syntax stages. Later
semantic-resolution and sandbox stages may use project metadata or .NET build
tools, but they are not implemented yet.

## Build and verify

From this directory:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

To run the current CLI placeholder:

```powershell
cargo run -p scent-cli
```

`scent analyze <path>` is planned but is not available yet. Do not treat the
placeholder CLI as an analyzer result.

## Repository layout

```text
crates/
├── scent-cli/       command-line entry point
├── scent-core/      future pipeline orchestration
├── scent-domain/    stable IDs, paths, locations, diagnostics, and contracts
├── scent-ir/        facts-only semantic intermediate representation
└── scent-parser/    C# discovery, Tree-sitter parsing, and syntax extraction
docs/
├── architecture.md  layer boundaries and data flow
├── contributor-guide.md
└── status.md        implemented vs planned work
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

The next milestones are member extraction, two-pass intra-project resolution,
the `scent analyze` pipeline, deterministic IR JSON, metrics/graphs, findings,
and eventually reporting, TUI, sandboxing, caching, and C++ support. The full
sequenced plan is in `../docs/implementation_plan.md`.
