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
against `groq` or a remote Ollama host. `analyze`, `sandbox`, and `diagram`
never make a network call.

## 2. Install

```
cd sdp_lab_tools/scent-llm
python -m pip install -e .
```

Requires Python 3.10+. No third-party runtime dependency on 3.11+; on 3.10
`tomli` is pulled in for reading `scent_llm.toml`. Everything else (HTTP,
AST parsing, subprocess) is standard library.

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
| `--provider {ollama,groq}` | override the configured provider for this run |
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

If neither Ollama is running locally nor a Groq key is set, the command
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

| Setting | CLI flag | Env var | `scent_llm.toml` key | Default |
|---|---|---|---|---|
| Provider | `--provider` | `SCENT_LLM_PROVIDER` | `llm.provider` | `ollama` |
| Model | `--model` | `SCENT_LLM_MODEL` | `llm.model` | `llama3.1:8b` (ollama) / `llama-3.3-70b-versatile` (groq) |
| Ollama host | — | `OLLAMA_HOST` | `llm.ollama_host` | `http://localhost:11434` |
| Groq base URL | — | `GROQ_BASE_URL` | `llm.groq_base_url` | `https://api.groq.com/openai/v1` |
| Groq API key | — | `GROQ_API_KEY` | `llm.groq_api_key` (not recommended — use the env var) | none |
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
