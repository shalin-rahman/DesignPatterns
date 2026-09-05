# JSON Output Reference

`scent analyze <path> --format json` prints one JSON object: the full
`AnalysisReport`. This doc explains what every field means, in plain words,
so you don't have to read the Rust structs to make sense of it. For a real,
full, checked-in example, see
`crates/scent-core/tests/fixtures/sample-project/expected_output.json`.
For a shorter trimmed example with commentary, see §3 of
`docs/CLI_WORKFLOW_GUIDE.md`.

If you'd rather read the report as columns than as JSON, use
`--format table` instead (see §2 of `CLI_WORKFLOW_GUIDE.md`) — it prints
findings and principle risks as aligned text columns, not JSON.

## Top-level shape

| Field | What it is, in plain words |
|---|---|
| `project_id` | A stable hash identifying the analyzed project. Not a human-readable name — the project's name is the path you gave `scent analyze`, shown at the top of `--format human`/`--format table` output (and available to you directly, since you're the one who typed it). |
| `language` | The source language SCENT parsed. Always `"CSharp"` today. |
| `discovery` | What files were found before any parsing happened: counts of `.sln`, `.csproj`, and `.cs` files, and which ones were excluded (e.g. `bin/`, `obj/`, generated files) and why. |
| `namespaces` / `types` / `methods` / `fields` / `properties` | The semantic facts SCENT extracted from your C# source — what the code plainly declares. Each entry has a stable `id` (a hash, not a line number, so it survives unrelated edits) and a `name`. |
| `graph` | The dependency graph built from resolved references: typed edges like `Calls`, `Inherits`, `Creates`, `UsesType` between entity ids. |
| `metrics` | Raw measurements per entity: lines of code, cyclomatic complexity, nesting depth, coupling (CBO), cohesion (LCOM4). Plain counts — no judgment attached yet. |
| `findings` | The smells SCENT actually flagged. See "A finding" below. |
| `principle_risks` | Design-principle concerns (SRP, DRY, KISS, ...) inferred from findings/facts. See "A principle risk" below. |
| `pattern_recommendations` | Suggested design patterns (e.g. Factory Method) for specific code shapes, such as a switch statement that constructs several different types. |
| `refactoring_recommendations` | A standard refactoring name (e.g. `ExtractMethod`) attached to each finding's entity — "the well-known fix for this smell." |
| `diagnostics` | Parse problems SCENT could not make sense of (malformed syntax) — reported, never silently dropped or guessed at. |

## A finding

```json
{
  "rule": "LONG_METHOD",
  "rule_name": "Long Method",
  "entity": "e311e3a6...",
  "fingerprint": "LONG_METHOD:9f2a...",
  "severity": "critical",
  "confidence": 0.54,
  "location": {
    "path": "src/OrderService.cs",
    "start_line": 12,
    "start_column": 4,
    "end_line": 63,
    "end_column": 5
  },
  "snippet": {
    "lines": [
      { "line": 13, "text": "    public void ProcessOrder(Order order, string paymentType)" },
      { "line": 14, "text": "    {" }
    ],
    "omitted_lines": 32
  },
  "evidence": [
    { "metric": "loc", "observed_value": 6, "explanation": "6 physical lines (threshold 1)" }
  ]
}
```

| Field | Plain-words meaning |
|---|---|
| `rule` | Which of the 13 built-in rules fired (run `scent rules` for the full list). |
| `rule_name` | The same rule, as the human-readable name shown in `--format human`/`--format table` ("Long Method" instead of `LONG_METHOD`). |
| `entity` | The stable id of the method/type/field this finding is about. Cross-reference it against `methods`/`types`/`fields` in the same report to get its full extracted facts — `location` below already gives you the file and line range directly, without needing to do that cross-reference just to find *where* it is. |
| `fingerprint` | A stable identity for this exact finding (`rule` + `entity`, hashed), used for baseline comparison (`scent gate --baseline`) — unaffected by unrelated edits elsewhere in the file. |
| `severity` | **How bad this is**, one of `low` / `medium` / `high` / `critical` (fixed order, worst last). Each rule has a built-in default severity; `smell_detector.toml` can override it per rule. |
| `confidence` | **How sure SCENT is**, 0.0–1.0, shown as a percentage in `--format human`/`--format table`. Built from several weighted, normalized measurements — never a single `metric > threshold` check — so a value near the rule's minimum confidence means "borderline," not "wrong." |
| `location` | Where this finding actually lives: `path` is relative to the analyzed project root (the same `<path>` you passed to `scent analyze`), `start_line`/`end_line`/`start_column`/`end_column` are the exact span, **0-based** (row 0 is the file's first line) — `--format table`'s `LOCATION` column and the `SOURCE` section both add 1 before printing, so what you see there is 1-based. |
| `snippet` | The real source lines `location` covers, read from disk **at report-generation time** — `lines` is `{line, text}` pairs (1-based, matching what an editor shows), capped at 20 lines from the start; `omitted_lines` says how many more there were (0 if none). `null` if the file couldn't be read (moved/deleted since analysis) or the location doesn't fit the file anymore. Because this is read fresh each run, it can differ between two reports if the file changed in between — unlike every other field here, it is not derived purely from the parsed IR. |
| `evidence` | The specific measurements that added up to this finding — each one names the metric, its observed value, and the threshold it crossed. This is what makes a finding explainable rather than a black box. |

**Severity vs. confidence — the difference**: severity is "how serious is this
kind of problem" (fixed per rule, like a smoke detector's alarm level).
Confidence is "how sure am I this specific instance is real" (computed per
finding). A `LONG_METHOD` finding is always at least `high` severity by
default, but two different long methods can have very different confidence
scores depending on how far past the threshold each one is.

## A principle risk

```json
{
  "principle": "Kiss",
  "risk": "Medium",
  "confidence": 0.54,
  "explanation": "Medium KISS risk: evidence suggests unnecessary complexity relative to what the entity needs to do",
  "evidence": [{ "rule": "LONG_METHOD", "summary": "Long Method finding at 0.54 confidence" }]
}
```

| Field | Plain-words meaning |
|---|---|
| `principle` | Which of the 9 design principles this is about: `Srp`, `Ocp`, `Lsp`, `Isp`, `Dip`, `Dry`, `Kiss`, `Yagni`, `LawOfDemeter`. |
| `risk` | **How much risk this represents**, one of `Low` / `Medium` / `High` — a separate scale from `severity` above (findings use `severity`, principle risks use `risk`; same idea, different name, don't confuse the two). |
| `confidence` | Same meaning as a finding's `confidence` — how sure SCENT is, 0.0–1.0. |
| `explanation` | A ready-to-read sentence that already states the risk level in words (`"Medium KISS risk: ..."`) — that's why `--format human`/`--format table` don't print `risk` and `explanation` side by side; it would just repeat "Medium" twice. |
| `evidence` | Which finding(s) this risk was inferred from — a principle risk is never invented from nothing; it always points back to real findings or facts. |

## Where "project name" actually lives

There is no `project_name` field in the JSON, because SCENT never assumes
your project has one canonical name — a directory can be a bare folder of
`.cs` files with no `.sln` at all. The identifying information you have is:

- **The path you passed to `scent analyze`** — this is what `--format
  human` and `--format table` print as `Project: <path>` at the top.
- **`project_id`** — a stable hash, useful for confirming two runs are
  analyzing the same project (e.g. across CI runs), not for display.

## Severity and risk, side by side

| Severity (findings) | Risk (principle risks) |
|---|---|
| `low` | `Low` |
| `medium` | `Medium` |
| `high` | `High` |
| `critical` | *(no fourth level — principle risk only ever goes up to `High`)* |

## See also

- `docs/CLI_WORKFLOW_GUIDE.md` — every command with real captured output,
  including a full untrimmed JSON example.
- `docs/architecture.md` — why findings/metrics/facts are kept as separate
  layers (this is why a finding's `evidence` looks the way it does).
