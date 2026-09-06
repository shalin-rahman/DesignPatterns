# scent-llm guide

This is the full reference: how to run it, what each command does, how
configuration is resolved, and what each detected smell means. The
[README](../README.md) stays short; this file has the detail.

## 1. What this tool is

`scent-llm` does two things:

1. Asks a configured LLM (local Ollama or cloud Groq) to write code for a
   task you describe.
2. Statically checks Python source — generated or hand-written — for five
   "LLM code smells": patterns in code that *calls* an LLM API that make
   the call unreliable, unbounded in cost, or non-reproducible.

It also renders two textual diagrams of the tool itself, and can dry-run
a source file (any of several languages) in an isolated sandbox.

Everything talks to the network only when you explicitly run `generate`
against `groq`, `gemini`, or a remote Ollama host. `analyze`, `sandbox`, and `diagram`
never make a network call.

## 2. Install

```
cd sdp_lab_tools/scent-llm
python -m pip install -e .
```

Requires Python 3.10+. The project uses the official `google-genai` SDK for
Gemini; on Python 3.10, `tomli` is also installed for reading
`scent_llm.toml`. The Ollama and Groq transports otherwise use the standard
library.

After installing, the `scent-llm` command is on your PATH. You can also
run it without installing via `python -m scent_llm.cli ...` from inside
`sdp_lab_tools/scent-llm/`.

## 3. Running each command

Every command prints numbered progress steps to **stderr** while it works
(`[1/2] Discovering...` -> `[1/2] done (0.1s)`), so you always see it's
alive during a slow network call or a big directory scan. Pass `-q` /
`--quiet` to any subcommand to suppress those lines — useful when piping
`generate`'s stdout or `analyze --json`'s stdout into another tool.

### 3.1 `generate` — ask the LLM for code

```
scent-llm generate "write a function that dedupes a list preserving order"
```

Steps shown: load configuration -> request code from the provider -> write
output -> (optional) analyze it. Flags:

| Flag | Meaning |
|---|---|
| `--language LANG` | target language for the generated code (default `python`) |
| `--provider {ollama,groq,gemini}` | override the configured provider for this run |
| `--model NAME` | override the configured model for this run |
| `--temperature FLOAT` | override temperature for this run |
| `--max-tokens N` | override the output length bound for this run |
| `--out PATH` | write the generated code to a file instead of stdout |
| `--analyze` | immediately run smell analysis on the generated code (Python only) |
| `-q`, `--quiet` | suppress progress lines |

Example, generating with Groq and checking the result:

```
GROQ_API_KEY=sk-... scent-llm generate "a retry wrapper for an HTTP call" \
  --provider groq --out retry.py --analyze
```

If neither Ollama is running locally nor a cloud provider key is set, the command
prints a specific, actionable error (not a stack trace) and exits 1.

### 3.2 `analyze` — find LLM code smells

```
scent-llm analyze path/to/file_or_directory
```

Steps shown: discover `.py` files under the path -> parse each one and run
the five detectors. Flags:

| Flag | Meaning |
|---|---|
| `--json` | machine-readable output (see schema below) instead of the text report |
| `-q`, `--quiet` | suppress progress lines |

Text output is a report block per finding: which smell, its severity,
the exact file:line:col, the offending line of code, why it matters, and
a concrete fix — see §5 for what each smell means and §6 for an example
of each fix.

JSON output shape:

```json
{
  "root": "...",
  "files_scanned": 3,
  "call_sites_found": 2,
  "counts_by_smell": {"NSO": 1, "UMM": 1},
  "findings": [
    {"smell": "NSO", "title": "No Structured Output", "file": "...",
     "line": 8, "col": 15, "severity": "medium", "detail": "..."}
  ]
}
```

Exit code is always 0 for `analyze` (findings are reported, not treated as
a hard failure) — wire it into a quality gate yourself by checking
`counts_by_smell` or `findings` in the JSON if you want a non-zero exit on
any finding.

### 3.3 `sandbox` — dry-run a file safely

```
scent-llm sandbox path/to/script.py --language python
```

Steps shown: read the file -> run it in the sandbox. Flags:

| Flag | Meaning |
|---|---|
| `--language LANG` | required; one of the languages in §7 |
| `--timeout N` | wall-clock seconds before the run is killed (default 10) |
| `--no-docker` | skip Docker even if installed, and use the plain-subprocess path |
| `-q`, `--quiet` | suppress progress lines |

Output line format: `[isolation] exit=N timed_out=bool`, followed by
`stdout`/`stderr` blocks. Exit code mirrors the sandboxed program's exit
code (or 1 if the sandbox itself couldn't run it).

### 3.4 `diagram` — textual architecture views

```
scent-llm diagram sequence          # this tool's own generate/analyze pipeline
scent-llm diagram tree --path .     # ASCII project tree of a directory
```

No network access, no progress steps (both are instant). Output uses
plain ASCII connectors (`|--`, `` `-- ``) rather than Unicode box-drawing
characters, so it renders correctly on a default-codepage Windows console
as well as any UTF-8 terminal.

## 4. Configuration

Resolution order, highest wins: **CLI flag > environment variable >
`scent_llm.toml` in the current directory > built-in default.**

### Where configuration belongs

Use the narrowest location that matches the lifetime of the setting:

| Location | Put here | Do not put here |
|---|---|---|
| CLI flags | One-off experiments, temporary model/temperature changes | Secrets |
| Environment variables | API keys, machine- or session-specific endpoints, CI values | Shared project defaults |
| `scent_llm.toml` in the **current working directory** | Team-shared non-secret defaults | API keys in a committed file |
| Built-in defaults | Safe fallback for local Ollama | Deployment-specific assumptions |

The config file is looked up from the process current directory, not from the
directory containing the installed package or the source file being analyzed.
For example, this command reads
`C:\work\my-project\scent_llm.toml`:

```powershell
Set-Location C:\work\my-project
scent-llm generate "write a parser"
```

`analyze`, `sandbox`, and `diagram` do not need LLM configuration. They only
load configuration as part of `generate`.

| Setting | CLI flag | Env var | `scent_llm.toml` key | Default |
|---|---|---|---|---|
| Provider | `--provider` | `SCENT_LLM_PROVIDER` | `llm.provider` | `ollama` |
| Model | `--model` | `SCENT_LLM_MODEL` | `llm.model` | provider-specific |
| Ollama host | — | `OLLAMA_HOST` | `llm.ollama_host` | `http://localhost:11434` |
| Groq base URL | — | `GROQ_BASE_URL` | `llm.groq_base_url` | `https://api.groq.com/openai/v1` |
| Groq API key | — | `GROQ_API_KEY` | `llm.groq_api_key` (not recommended — use the env var) | none |
| Gemini API key | — | `GEMINI_API_KEY` | `llm.gemini_api_key` (not recommended — use the env var) | none |
| Temperature | `--temperature` | `SCENT_LLM_TEMPERATURE` | `llm.temperature` | `0.2` |
| Max tokens | `--max-tokens` | `SCENT_LLM_MAX_TOKENS` | `llm.max_tokens` | `2048` |
| Request timeout | — | — | `llm.timeout_seconds` | `60` |
| System message | — | — | `llm.system_message` | "You are a careful, precise senior software engineer." |

`scent_llm.toml` example (place in the directory you run `scent-llm` from):

```toml
[llm]
provider = "groq"
model = "llama-3.3-70b-versatile"
temperature = 0.1
max_tokens = 1024
```

**Never put `groq_api_key` in `scent_llm.toml`** if that file is committed
to a repo — use the `GROQ_API_KEY` environment variable. The config
loader supports the TOML key only for local, gitignored convenience.

A malformed or missing config file is silently ignored (falls back to
env vars / defaults) — it will never crash the tool.

### Setting up each provider

- **Ollama (default, no key needed):** install from ollama.com, then
  `ollama pull llama3.1:8b` (or your chosen model) and `ollama serve`.
  `scent-llm generate` talks to `http://localhost:11434` by default.
- **Groq (cloud, needs a free API key):** get a key at console.groq.com,
  then `export GROQ_API_KEY=...` (or the PowerShell equivalent
  `$env:GROQ_API_KEY = "..."`) and pass `--provider groq`.
- **Gemini (cloud, API key and quota required):** create a key in Google AI
  Studio, set `$env:GEMINI_API_KEY = "..."` in PowerShell (or
  `export GEMINI_API_KEY=...` on macOS/Linux), and pass `--provider gemini`.
  The default model is `gemini-3-flash-preview`; override it with `--model` or
  `SCENT_LLM_MODEL`.

The package installs the current `google-genai` SDK automatically:

```powershell
python -m pip install -e .
```

Gemini example:

```powershell
$env:GEMINI_API_KEY = "your-key"
scent-llm generate "write a Python JSON validator" `
  --provider gemini `
  --model gemini-3-flash-preview `
  --temperature 0.2 `
  --max-tokens 1024 `
  --out validator.py `
  --analyze
```

The Gemini adapter sends a system instruction, explicit temperature, and
`max_output_tokens`. Quotas and model availability are controlled by Google
AI Studio; quota errors should be handled by reducing request frequency or
selecting an available model.

### Hugging Face relevance and future integration

Hugging Face is relevant to the learning model and to a future provider
adapter, but it is **not a currently supported `scent-llm` provider**. The
current implementation accepts `ollama`, `groq`, and `gemini`; do not configure
`provider = "huggingface"` and expect it to work.

Hugging Face's current documentation reinforces several behaviors already
represented by this project:

- chat models expect a model-specific chat template; structured
  `system`/`user`/`assistant` messages are converted into the model's control
  tokens;
- generation should use explicit bounds such as `max_new_tokens`;
- hosted inference uses a Hub model ID and a selected inference provider;
- local inference can connect through servers such as Ollama, vLLM, TGI, or
  other OpenAI-compatible endpoints.

If Hugging Face support is implemented later, the safe design is a new
provider adapter in `llm_client.py`, not a special case in the detectors:

1. Add a provider choice and configuration fields for the Hub model ID,
   endpoint/provider, token, revision, timeout, and generation limit.
2. Keep the token in an environment variable or CI secret; never commit it
   to `scent_llm.toml`.
3. Preserve an explicit system message and use the model's documented chat
   template rather than assuming OpenAI-compatible formatting.
4. Pin the model ID plus a revision or immutable deployment where the backend
   supports it; a mutable Hub name should be treated as the NMVP risk.
5. Add mocked transport tests and an end-to-end test against an explicitly
   opted-in endpoint. Do not make network access part of the default test
   suite.

Reference material:

- [LLM Code Smells: A Taxonomy and Detection Approach](../../materials/LLM%20Code%20Smells-%20A%20Taxonomy%20and%20Detection%20Approach.pdf)
- [Specification and Detection of LLM Code Smells](../../materials/Specification%20and%20Detection%20of%20LLM%20Code%20Smells%20paper.pdf)
- [Hugging Face chat templates](https://huggingface.co/docs/transformers/main/en/chat_templating)
- [Hugging Face text generation](https://huggingface.co/docs/transformers/main/en/llm_tutorial)
- [Hugging Face server inference](https://huggingface.co/docs/huggingface_hub/en/guides/inference)

### Configuration scenarios

#### Scenario A: local Ollama, no secrets

```powershell
ollama pull llama3.1:8b
ollama serve
Set-Location C:\work\DesignPatterns\sdp_lab_tools\scent-llm
scent-llm generate "write a pure Python CSV validator" --out validator.py --analyze
```

Use this for offline development. If Ollama runs on another machine, set
`$env:OLLAMA_HOST` to its reachable URL. The host must also be reachable from
the machine running `scent-llm`.

#### Scenario B: shared project defaults with a local config file

Create `scent_llm.toml` in the project directory (and commit it only after
confirming it contains no secret):

```toml
[llm]
provider = "ollama"
model = "llama3.1:8b"
temperature = 0.0
max_tokens = 1024
timeout_seconds = 90
system_message = "You are a precise code reviewer. Prefer small, testable changes."
```

Then run from that directory:

```powershell
scent-llm generate "review this function for edge cases" --out review.py
```

#### Scenario C: Groq for a cloud/CI run

Keep the provider choice in the file or command, but inject the secret at
runtime:

```powershell
$env:GROQ_API_KEY = "replace-me"
scent-llm generate "write a bounded retry helper" `
  --provider groq --model llama-3.3-70b-versatile --analyze
Remove-Item Env:GROQ_API_KEY
```

In CI, store `GROQ_API_KEY` in the runner's secret store. Do not echo it,
write it to `scent_llm.toml`, or pass it as a command-line argument.

#### Scenario D: troubleshoot precedence

Start with the lowest layer and add one override at a time:

1. Run with no file or environment variables; this checks built-in Ollama defaults.
2. Add `scent_llm.toml`; confirm the request uses its provider/model.
3. Set `SCENT_LLM_MODEL`; confirm it beats the TOML model.
4. Add `--model`; confirm it beats the environment value.

The progress line printed by `generate` includes the resolved
`provider:model`, which is the quickest non-secret configuration check.

## 4.1 Implementation map: where behavior lives

Use this map when extending or reviewing the tool:

| Concern | File | Responsibility |
|---|---|---|
| CLI surface | `scent_llm/cli.py` | Parses commands, flags, progress, exit codes |
| Configuration | `scent_llm/config.py` | Defaults, TOML loading, environment mapping, precedence |
| Provider calls | `scent_llm/llm_client.py` | Ollama/Groq payloads, HTTP errors, code extraction |
| Call-site discovery | `scent_llm/analysis/ast_analyzer.py` | Recognizes SDK call shapes without importing SDKs |
| Smell rules | `scent_llm/smells/detectors.py` | One pure detector per smell |
| Refactoring advice | `scent_llm/smells/refactor.py` | Human-readable fix suggestions |
| Directory orchestration | `scent_llm/runner.py` | Python discovery and report aggregation |
| Dry-run execution | `scent_llm/sandbox/dry_run.py` | Docker-first or subprocess fallback |
| Diagrams | `scent_llm/visualization/architecture.py` | Tree and pipeline text views |

To add a smell, add a detector returning `Finding | None` and register it in
`_ALL_DETECTORS`; then add a focused test. To add a configuration setting,
update the dataclass default, TOML/env loading, any CLI override, and this
table together.

## 5. The five LLM code smells

`analyze` looks for these five patterns in any function call that matches
a known LLM SDK shape (`client.chat.completions.create`, `.messages.create`,
`ollama.chat`, etc.). A call only needs to *look* like an LLM call by
structure — the actual SDK package does not need to be installed for
analysis to find it.

| Code | Name | What triggers it | Why it matters |
|---|---|---|---|
| **NSO** | No Structured Output | No `response_format` / `tools` / `functions` / `format` argument | Output is free text; the caller must parse it by hand, which breaks silently when the model rephrases something |
| **UMM** | Unbounded Max Metrics | No `max_tokens` / `num_predict` / equivalent argument (or it's set to 0/negative) | A single call can run away in cost, latency, or memory with no ceiling |
| **TNES** | Temperature Not Explicitly Set | No `temperature` argument | Determinism depends on the provider's current default, which can change under you |
| **NMVP** | No Model Version Pinning | `model` is a floating alias (`"gpt-4"`, `"gpt-4o"`, `"latest"`, …) or missing entirely | The provider can roll the underlying model forward at any time, changing behavior without your code changing |
| **NSM** | No System Message | A `messages` list with no `role: "system"` entry (and no top-level `system=` argument) | The model's role, constraints, and output contract are never stated |

Detection is heuristic and intentionally conservative: if an argument's
value is a variable or expression rather than a literal (e.g.
`model=settings.MODEL_NAME`), the detector cannot judge it statically and
skips that check rather than guessing.

### Scope relative to the research taxonomy

The two papers in `sdp_lab_tools/materials` provide the research context for
this implementation. The earlier specification defines the same five smells
listed here. The later taxonomy expands the catalog to nine and refines the
categories and effects. It adds concerns such as reasoning-effort
configuration, oversized visual inputs, overspecified sampling parameters,
and weak request identity/observability.

This repository intentionally implements only the original five. In
particular, **UMM currently checks token-output bounds at the call site**; the
papers also recommend bounding request timeouts, retries, and monitoring input
tokens. Those operational controls are useful production requirements, but
they are not findings emitted by this version of `scent-llm`.

The research tools use a richer static observation model than this small
reference implementation. `scent-llm` analyzes literal keyword arguments and
known call-shape suffixes in one Python file. It can therefore miss:

- an SDK call hidden behind a custom wrapper;
- a model, messages list, or generation setting loaded indirectly;
- provider-specific semantics that are not visible in the call syntax;
- newer smells outside the five-detector catalog.

Treat a clean report as “no matching evidence was found,” not as proof that
an integration is production-safe. Pair static analysis with provider
documentation, runtime limits, tests, and monitoring.

## 6. Fixing each smell

**NSO** — constrain the schema instead of parsing prose:

```python
response_format={"type": "json_schema", "json_schema": {...}}   # OpenAI/Groq
format="json"                                                     # Ollama
# Anthropic: use tool-calling with a strict input schema
```

**UMM** — set a bound sized to the task:

```python
max_tokens=512                          # OpenAI / Groq / Anthropic
options={"num_predict": 512}            # Ollama
```

**TNES** — decide and state a temperature:

```python
temperature=0.0   # deterministic — extraction, classification
temperature=0.7   # varied — creative generation
```

**NMVP** — pin to a dated snapshot or tagged build:

```python
model="gpt-4o-2024-08-06"                # not "gpt-4o"
model="claude-3-5-sonnet-20241022"       # not "claude-3.5-sonnet"
model="llama3.1:8b"                      # Ollama tag, not bare "llama3"
```

**NSM** — state the model's role and output contract:

```python
messages=[
    {"role": "system", "content": "You are ... Respond with JSON only."},
    {"role": "user", "content": user_input},
]
# Anthropic: pass system="..." as a top-level argument instead
```

## 7. Sandbox: isolation and supported languages

`sandbox` picks isolation automatically:

1. **Docker**, if `docker` is on PATH: runs in a throwaway container with
   `--network none`, a memory cap, and a process-count cap. This is the
   isolation to use for anything you don't fully trust.
2. **Plain subprocess** otherwise: a temp directory, a trimmed
   environment, and a hard wall-clock timeout — a developer convenience
   for "does this even run", **not** a security boundary. Pass
   `--no-docker` to force this path even when Docker is available.

Supported `--language` values and what they need installed for the
subprocess fallback (Docker mode needs nothing local except Docker
itself — it pulls the image):

| Language | Subprocess-mode requires |
|---|---|
| `python` | `python` |
| `javascript` | `node` |
| `typescript` | `node` + `npx` (runs via `tsx`) |
| `bash` | `bash` |
| `ruby` | `ruby` |
| `go` | `go` |
| `java` | `javac` + `java` (JDK) |
| `csharp` | `dotnet` |
| `c` | `gcc` |
| `cpp` | `g++` |

If neither Docker nor the local toolchain is present, `sandbox` reports
that clearly instead of failing with a confusing traceback.

## 8. Troubleshooting

- **"Could not reach ollama at ...: actively refused"** — Ollama isn't
  running. Start it with `ollama serve`, or point `OLLAMA_HOST` at a
  reachable instance.
- **"Groq provider selected but no API key was found"** — set
  `GROQ_API_KEY`, or drop `--provider groq` to fall back to local Ollama.
- **Garbled tree/box characters on Windows** — shouldn't happen; this
  tool only prints ASCII. If you see mojibake, it's coming from something
  else in your pipeline (e.g. a font or terminal encoding issue upstream).
- **`analyze` finds 0 call sites in a file you know calls an LLM** — the
  detector matches by call *shape* (e.g. `.chat.completions.create`); a
  custom wrapper function that hides the SDK call one level down won't be
  matched. Point `analyze` at the file containing the actual SDK call.

## 9. Testing

```
python -m unittest discover -s tests
```

`tests/test_detectors.py` covers: all five smells firing together on a
deliberately smelly call, zero findings on a clean call, Ollama's
`name:tag` convention and system message both being recognized as
already-fixed, a non-LLM call being ignored entirely, and a syntax error
returning no findings instead of crashing.

Run from `sdp_lab_tools/scent-llm`. On PowerShell, the same command is:

```powershell
Set-Location .\sdp_lab_tools\scent-llm
python -m unittest discover -s tests -p "test_*.py" -v
```

The configuration tests exercise the precedence contract with an isolated
temporary TOML file and environment variables. For a manual end-to-end smoke
test:

```powershell
python -m scent_llm.cli analyze examples\smelly_example.py --json
python -m scent_llm.cli sandbox examples\smelly_example.py --language python --no-docker
python -m scent_llm.cli diagram sequence
```

## 10. Learning path

Read the tool in this order:

1. Run `analyze` against `examples/smelly_example.py` and compare each finding
   with the five smell definitions in §5.
2. Run `sandbox` and observe the difference between Docker isolation and the
   subprocess fallback. Never treat the fallback as a security boundary.
3. Read `ast_analyzer.py` to understand why analysis is syntax-based and why
   custom wrappers may not be discovered.
4. Read `detectors.py` and `refactor.py` together: detection explains the
   evidence; refactoring explains the remediation.
5. Read `config.py` and `llm_client.py` to connect resolved settings to the
   actual provider payload.
6. Use `diagram sequence` to verify the complete generate/analyze flow.

This separation is intentional: configuration, provider transport, discovery,
rules, and presentation can be tested or changed independently.
