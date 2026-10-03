# DesignPatterns and Scent Lab

This repository holds course material and working tools on four related topics:
design patterns, code smells in C#, code smells in LLM-calling code, and the
quality of prompts people write to chatbots. Each topic has its own folder.
Start from the index below.

## Contents

- [Projects by topic](#projects-by-topic)
- [Document index](#document-index)
- [Quick start: SCENT](#quick-start-scent)
- [Quick start: scent-llm](#quick-start-scent-llm)
- [Quick start: wild-chat-analysis](#quick-start-wild-chat-analysis)
- [SpecDetect4LLM research tool](#specdetect4llm-research-tool)
- [Learning materials](#learning-materials)
- [Repository map](#repository-map)

## Projects by topic

### 1. Design patterns

Small Java examples that show a problem first and then the pattern that fixes it.

| Folder | What it shows | Tech |
|---|---|---|
| [`classical_design_patterns/singletone`](classical_design_patterns/singletone/) | Singleton: `Admin_Problem.java` (many instances) and `Admin_Solve.java` (one shared instance) | Java |
| [`DesignPatternFactory/DesignPatternCourse`](DesignPatternFactory/DesignPatternCourse/) | Factory patterns: [`fm_problem`](DesignPatternFactory/DesignPatternCourse/src/fm_problem/) (the problem), [`simple_factory`](DesignPatternFactory/DesignPatternCourse/src/simple_factory/) and [`factory_method`](DesignPatternFactory/DesignPatternCourse/src/factory_method/) | Java (VS Code project) |

Theory: [Design Patterns lecture](sdp_lab_tools/materials/lectutre_1_Design_Patterns.pdf).

### 2. Code smells and static analysis (C#)

| Folder | Purpose | Tech |
|---|---|---|
| [`sdp_lab_tools/scent`](sdp_lab_tools/scent/) | Analyze C# projects for structural code smells, metrics, design-principle risks and quality-gate violations | Rust + Tree-sitter |
| [`sdp_lab_tools/code-smells-demo`](sdp_lab_tools/code-smells-demo/) | Small C# app that has the lecture's smells on purpose, for demos and refactoring practice | C#/.NET |

Theory: [Code Smells lecture](sdp_lab_tools/materials/Code_Smells_Lecture.pdf).

### 3. Code smells in LLM-calling code

| Folder | Purpose | Tech |
|---|---|---|
| [`sdp_lab_tools/scent-llm`](sdp_lab_tools/scent-llm/) | Generate code through configurable LLM providers and detect five LLM code smells in Python API calls | Python |
| SpecDetect4LLM (separate checkout, not in git) | The research detector that scent-llm is compared against. See the [setup guide](sdp_lab_tools/specDetect4LLM_setuo_guide.md) | Python |

Theory: the two LLM code smell papers and the LLM lecture under [Learning materials](#learning-materials).

### 4. Prompt quality and LLM apps

| Folder | Purpose | Tech |
|---|---|---|
| [`wild-chat-analysis`](wild-chat-analysis/) | Finds prompt smells (vague requests, unclear references, conflicting rules and so on) in real user prompts from the WildChat dataset, using an LLM as the labeler. Includes a short paper draft in [`paper/`](wild-chat-analysis/paper/) | Python |
| [`sdp_lab_tools/LLM_Chat/LLM_Powered_ChatBot_System`](sdp_lab_tools/LLM_Chat/LLM_Powered_ChatBot_System/) | A course group project (other authors): a chatbot web app that calls an LLM through OpenRouter | Next.js + Spring Boot |

Theory: [Prompt Smells slides](sdp_lab_tools/LLM_Chat/Prompt_Smells_1.pdf).

## Document index

Every written guide in the repo, grouped by project.

| Project | Document | Read it when you want to |
|---|---|---|
| SCENT | [README](sdp_lab_tools/scent/README.md) | analyze a C# project |
| SCENT | [Learning guide](sdp_lab_tools/scent/docs/LEARNING_GUIDE.md) | learn the whole pipeline step by step |
| SCENT | [CLI workflow guide](sdp_lab_tools/scent/docs/CLI_WORKFLOW_GUIDE.md) | copy exact commands and see real output |
| SCENT | [JSON output reference](sdp_lab_tools/scent/docs/JSON_OUTPUT_REFERENCE.md) | read report fields, locations, evidence and fingerprints |
| SCENT | [Architecture](sdp_lab_tools/scent/docs/architecture.md) | understand the crates and how they depend on each other |
| SCENT | [Contributor guide](sdp_lab_tools/scent/docs/contributor-guide.md) | change the code and run the checks |
| SCENT | [Status](sdp_lab_tools/scent/docs/status.md) | see what is built and what is still planned |
| SCENT | [Shell completions](sdp_lab_tools/scent/completions/README.md) | add tab completion to your shell |
| SCENT | [Implementation plan](sdp_lab_tools/docs/implementation_plan.md) | see the original build plan |
| SCENT | [System prompt](sdp_lab_tools/docs/prompt.md) | see the spec the tool was built from |
| code-smells-demo | [Refactoring guide](sdp_lab_tools/code-smells-demo/REFACTORING_GUIDE.md) | practise fixing each lecture smell, from Long Method to Inappropriate Intimacy |
| scent-llm | [README](sdp_lab_tools/scent-llm/README.md) | install it and run a first analysis |
| scent-llm | [Full guide](sdp_lab_tools/scent-llm/docs/GUIDE.md) | set up providers, read the five smells and how to fix each, sandbox and troubleshooting |
| SpecDetect4LLM | [Windows setup guide](sdp_lab_tools/specDetect4LLM_setuo_guide.md) | install and run the research detector |
| wild-chat-analysis | [README](wild-chat-analysis/README.md) | set up the API key, run the analysis, read the output format |
| wild-chat-analysis | [Paper draft](wild-chat-analysis/paper/main.tex) | read or build the 2-page report (LaTeX, IEEE format) |
| LLM ChatBot | [README](sdp_lab_tools/LLM_Chat/LLM_Powered_ChatBot_System/README.md) | run the group chatbot project |

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

## Quick start: wild-chat-analysis

From [`wild-chat-analysis/`](wild-chat-analysis/):

```powershell
python -m venv .venv
.venv\Scripts\activate
python -m pip install -r requirements.txt
copy .env.example .env      # then put your API key in .env

python main.py --dry-run --max-records 200        # no API calls
python main.py --max-prompts 20                   # small real run
python -m pytest -q                               # tests, no API key needed
```

Any OpenAI-compatible API works; the README shows Groq. A stopped run picks up
where it left off when you run the same command again. See the
[wild-chat-analysis README](wild-chat-analysis/README.md) for every flag, rate
limits and the output format.

## SpecDetect4LLM research tool

The separate SpecDetect4LLM research checkout has a Windows setup guide at
[`sdp_lab_tools/specDetect4LLM_setuo_guide.md`](sdp_lab_tools/specDetect4LLM_setuo_guide.md).
It covers Python 3.11 setup, the detector and web application, Docker usage,
tests, repository layout, prevalence analysis, and the `R25`–`R29`
LLM-integration smell rules. Use it to compare the research implementation
with the smaller [`scent-llm`](sdp_lab_tools/scent-llm/) reference tool.

## Learning materials

The [`sdp_lab_tools/materials/`](sdp_lab_tools/materials/) directory contains
the source material used by the tools and exercises:

- [`Code Smells lecture`](sdp_lab_tools/materials/Code_Smells_Lecture.pdf)
- [`Design Patterns lecture`](sdp_lab_tools/materials/lectutre_1_Design_Patterns.pdf)
- [`LLM Code Smells taxonomy and detection paper`](sdp_lab_tools/materials/LLM%20Code%20Smells-%20A%20Taxonomy%20and%20Detection%20Approach.pdf)
- [`Specification and Detection of LLM Code Smells paper`](sdp_lab_tools/materials/Specification%20and%20Detection%20of%20LLM%20Code%20Smells%20paper.pdf)
- [`LLM Code Smells and Design Patterns lecture`](sdp_lab_tools/materials/LLM_Code_Smells_Design_Patterns_Lecture_v2.pdf)

Prompt smells material sits with the chatbot project:
[`Prompt Smells slides`](sdp_lab_tools/LLM_Chat/Prompt_Smells_1.pdf).

The LLM tool implements the original five-smell catalog. The newer taxonomy
paper expands that catalog; the difference and current implementation limits
are documented in the [scent-llm guide](sdp_lab_tools/scent-llm/docs/GUIDE.md).

## Repository map

```text
.
├── classical_design_patterns/
│   └── singletone/            Java Singleton example (problem and fix)
├── DesignPatternFactory/
│   └── DesignPatternCourse/   Java Simple Factory and Factory Method examples
├── sdp_lab_tools/
│   ├── scent/                 Rust C# static analyzer
│   ├── code-smells-demo/      C# app with smells on purpose
│   ├── scent-llm/             Python LLM generator and smell analyzer
│   ├── SpecDetect4LLM_ICSE/   research detector checkout (git-ignored)
│   ├── LLM_Chat/              group chatbot project and prompt smells slides
│   ├── materials/             lectures and papers (PDF)
│   ├── docs/                  SCENT plan and spec
│   └── specDetect4LLM_setuo_guide.md
├── wild-chat-analysis/        prompt smell study on WildChat, with paper draft
└── README.md
```

Project-specific contribution instructions and verification commands live in:

- [`scent contributor guide`](sdp_lab_tools/scent/docs/contributor-guide.md)
- [`scent architecture`](sdp_lab_tools/scent/docs/architecture.md)
- [`scent-llm full guide`](sdp_lab_tools/scent-llm/docs/GUIDE.md)
- [`wild-chat-analysis README`](wild-chat-analysis/README.md)
