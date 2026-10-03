# scent-llm

> Full reference (every command/flag, configuration precedence, what each
> smell means and how to fix it, sandbox details, troubleshooting):
> [docs/GUIDE.md](docs/GUIDE.md)

A CLI tool that:

1. Generates code by calling a configurable LLM backend (local Ollama or cloud Groq/Gemini/OpenRouter).
2. Statically detects five "LLM code smells" in Python source via AST analysis:
   `NSO` (No Structured Output), `UMM` (Unbounded Max Metrics), `TNES` (Temperature Not
   Explicitly Set), `NMVP` (No Model Version Pinning), `NSM` (No System Message).
3. Gives an actionable refactoring suggestion for every finding.
4. Renders a textual project tree and a sequence-diagram view of its own pipeline.
5. Dry-runs a source file in an isolated sandbox (Docker if available, subprocess otherwise)
   across Python, JS/TS, Go, Ruby, Java, C#, C, and C++.

No API key is hardcoded anywhere. Groq requires `GROQ_API_KEY`, Gemini requires
`GEMINI_API_KEY`, OpenRouter requires `OPENROUTER_API_KEY`, and Ollama needs no
key and is the default provider.

## Install

```
cd sdp_lab_tools/scent-llm
python -m pip install -e .
```

Requires Python 3.10+ (tested on 3.14). Gemini support installs the official
`google-genai` SDK; `tomli` is also installed on Python 3.10; `python-dotenv`
loads the optional `.env` file.

## Configuration

Resolution order: CLI flag > environment variable (including one loaded from a
`.env` file in the cwd) > `scent_llm.toml` in the cwd > built-in default.

Copy `.env.example` to `.env` and fill in the keys for the providers you use.
`.env` is gitignored, so real keys never get committed. A real shell/CI
environment variable always overrides the value from `.env`.

```
cp .env.example .env
```

| Setting | Env var | Default |
|---|---|---|
| provider | `SCENT_LLM_PROVIDER` | `ollama` |
| model | `SCENT_LLM_MODEL` | provider-specific |
| Ollama host | `OLLAMA_HOST` | `http://localhost:11434` |
| Groq API key | `GROQ_API_KEY` | none (required only for `--provider groq`) |
| Gemini API key | `GEMINI_API_KEY` | none (required only for `--provider gemini`) |
| OpenRouter API key | `OPENROUTER_API_KEY` | none (required only for `--provider openrouter`) |
| temperature | `SCENT_LLM_TEMPERATURE` | `0.2` |
| max tokens | `SCENT_LLM_MAX_TOKENS` | `2048` |

`scent_llm.toml` example:

```toml
[llm]
provider = "groq"
model = "llama-3.3-70b-versatile"
temperature = 0.1
```

Gemini can be selected without changing source code:

```powershell
$env:GEMINI_API_KEY = "your-key"
scent-llm generate "write a bounded retry helper" --provider gemini --analyze
```

OpenRouter works the same way, using an OpenAI-compatible chat completions API:

```powershell
$env:OPENROUTER_API_KEY = "your-key"
scent-llm generate "write a bounded retry helper" --provider openrouter --analyze
```

## Usage

Every command prints numbered progress steps to stderr (`[1/2] Discovering...`
-> `done (0.1s)`) while it works; add `-q` / `--quiet` to any subcommand to
suppress them.

```
# Generate code with the configured LLM, then check the result for smells
scent-llm generate "write a function that summarizes a list of strings" --analyze

# Scan an existing file or directory
scent-llm analyze path/to/project --json

# Dry-run a file in an isolated sandbox
scent-llm sandbox path/to/script.py --language python

# Render this tool's own pipeline as a sequence diagram, or a project tree
scent-llm diagram sequence
scent-llm diagram tree --path .
```

## Design notes

- `analysis/ast_analyzer.py` finds LLM call sites purely by matching known SDK call-shape
  suffixes (`.chat.completions.create`, `.messages.create`, `ollama.chat`, etc.) against
  `ast.unparse()` of the call target — no third-party SDK needs to be installed to analyze
  code that uses it.
- `smells/detectors.py` holds one pure function per smell: `LLMCallSite -> Finding | None`.
  Adding a new smell means adding one function and registering it, nothing else changes.
- `sandbox/dry_run.py` prefers Docker (`--network none`, memory/pid caps) when present, and
  falls back to a plain subprocess with a wall-clock timeout otherwise. The subprocess path
  is a developer convenience, not a security boundary — treat untrusted code accordingly.
- `examples/smelly_example.py` has one function with all five smells and one clean function,
  used as a smoke test (`tests/test_detectors.py`).

## Run tests

```
python -m unittest discover -s tests
```
