# DesignPatterns and Scent Lab

This repository contains teaching material and working tools for studying
design patterns, code smells, static analysis, and LLM-integrating software.
The main executable projects are:

| Project | Purpose | Primary technology |
|---|---|---|
| [`sdp_lab_tools/scent`](sdp_lab_tools/scent/) | Analyze C# projects for structural code smells, metrics, design-principle risks, and quality-gate violations | Rust + Tree-sitter |
| [`sdp_lab_tools/scent-llm`](sdp_lab_tools/scent-llm/) | Generate code through configurable LLM providers and detect five LLM code smells in Python calls | Python |
| [`sdp_lab_tools/code-smells-demo`](sdp_lab_tools/code-smells-demo/) | Small intentionally smelly C# application for demonstrations and refactoring practice | C#/.NET |

## Choose a starting point

- Want to analyze a C# project? Start with
  [`scent/README.md`](sdp_lab_tools/scent/README.md).
- Want to learn the complete SCENT pipeline? Read the
  [`SCENT learning guide`](sdp_lab_tools/scent/docs/LEARNING_GUIDE.md).
- Want exact CLI commands and captured examples? Read the
  [`SCENT CLI workflow guide`](sdp_lab_tools/scent/docs/CLI_WORKFLOW_GUIDE.md).
- Want to generate code or analyze LLM API calls? Start with
  [`scent-llm/README.md`](sdp_lab_tools/scent-llm/README.md), then use the
  [full scent-llm guide](sdp_lab_tools/scent-llm/docs/GUIDE.md).
- Want a hands-on refactoring exercise? Open the
  [`code-smells-demo` refactoring guide](sdp_lab_tools/code-smells-demo/REFACTORING_GUIDE.md).

## Quick start: SCENT

SCENT parses C# source directly; it does not compile or execute the analyzed
project. From [`sdp_lab_tools/scent/`](sdp_lab_tools/scent/):

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

cargo run -p scent-cli -- analyze <path-to-csharp-project> --format table
cargo run -p scent-cli -- analyze <path-to-csharp-project> --format json
cargo run -p scent-cli -- gate <path-to-csharp-project> --max-critical 0 --max-high 0
```

For repeated use, install the CLI:

```powershell
cargo install --path crates/scent-cli
scent analyze <path-to-csharp-project> --format table
```

Configuration is read from the analyzed project when present:

- [`scent/smell_detector.toml`](sdp_lab_tools/scent/smell_detector.toml) shows
  project-level defaults.
- [`SCENT status`](sdp_lab_tools/scent/docs/status.md) distinguishes
  implemented behavior from planned work.
- [`JSON output reference`](sdp_lab_tools/scent/docs/JSON_OUTPUT_REFERENCE.md)
  documents report fields, locations, evidence, and fingerprints.

## Quick start: scent-llm

From [`sdp_lab_tools/scent-llm/`](sdp_lab_tools/scent-llm/):

```powershell
python -m pip install -e .

# No network required:
python -m scent_llm.cli analyze examples\smelly_example.py --json
python -m scent_llm.cli sandbox examples\smelly_example.py --language python --no-docker

# Local provider:
ollama pull llama3.1:8b
ollama serve
scent-llm generate "write a Python CSV validator" --out validator.py --analyze
```

Cloud providers are opt-in and use environment variables for secrets:

```powershell
$env:GROQ_API_KEY = "your-key"
scent-llm generate "write a bounded retry helper" --provider groq --analyze

$env:GEMINI_API_KEY = "your-key"
scent-llm generate "write a Python JSON validator" `
  --provider gemini --model gemini-3-flash-preview --analyze
```

Configuration precedence is:

```text
CLI flag > environment variable > scent_llm.toml in the current directory > built-in default
```

See the [scent-llm configuration and implementation guide](sdp_lab_tools/scent-llm/docs/GUIDE.md)
for provider setup, Gemini details, Hugging Face future integration, smell
definitions, sandbox isolation, troubleshooting, and tests.

## Learning materials

The [`sdp_lab_tools/materials/`](sdp_lab_tools/materials/) directory contains
the source material used by the tools and exercises:

- [`Code Smells lecture`](sdp_lab_tools/materials/Code_Smells_Lecture.pdf)
- [`Design Patterns lecture`](sdp_lab_tools/materials/lectutre_1_Design_Patterns.pdf)
- [`LLM Code Smells taxonomy and detection paper`](sdp_lab_tools/materials/LLM%20Code%20Smells-%20A%20Taxonomy%20and%20Detection%20Approach.pdf)
- [`Specification and Detection of LLM Code Smells paper`](sdp_lab_tools/materials/Specification%20and%20Detection%20of%20LLM%20Code%20Smells%20paper.pdf)
- [`LLM Code Smells and Design Patterns lecture`](sdp_lab_tools/materials/LLM_Code_Smells_Design_Patterns_Lecture_v2.pdf)

The LLM tool implements the original five-smell catalog. The newer taxonomy
paper expands that catalog; the difference and current implementation limits
are documented in the [scent-llm guide](sdp_lab_tools/scent-llm/docs/GUIDE.md).

## Repository map

```text
.
├── sdp_lab_tools/
│   ├── scent/          Rust C# static analyzer
│   ├── scent-llm/      Python LLM generator and smell analyzer
│   ├── code-smells-demo/
│   ├── materials/
│   └── docs/
└── README.md
```

Project-specific contribution instructions and verification commands live in:

- [`scent contributor guide`](sdp_lab_tools/scent/docs/contributor-guide.md)
- [`scent architecture`](sdp_lab_tools/scent/docs/architecture.md)
- [`scent-llm full guide`](sdp_lab_tools/scent-llm/docs/GUIDE.md)

