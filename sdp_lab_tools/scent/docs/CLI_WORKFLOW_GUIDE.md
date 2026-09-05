# CLI & Workflow Guide

Two things in one place: every `scent` command with a real example and its
real captured output, and a step-by-step trace of what happens internally
between "you press Enter" and "a report appears" — a dry run of the
pipeline, stage by stage, in plain words.

All output below was captured for real from this repository's own test
fixtures, not written from memory or invented. Commands are run from
`sdp_lab_tools/scent/`.

---

## 1. The pipeline, step by step

Every `scent analyze`/`scent gate` invocation runs the same sequence.
`--format human` (the default) prints a progress line to stderr as each
stage starts — this section explains what each line actually means.

```text
scent> cargo run -q -p scent-cli -- analyze crates\scent-core\tests\fixtures\sample-project
```

stderr, in order:

```text
[scent] Discovering C# project files under crates\scent-core\tests\fixtures\sample-project
[scent] Discovered 0 solution file(s), 1 project file(s), 1 source file(s), 0 excluded
[scent] Parsing [1/1] src/Order.cs
[scent] Building declaration index
[scent] Resolving intra-project references
[scent] Building the dependency graph
[scent] Calculating metrics (LOC, CC, nesting, LCOM4, CBO)
[scent] Reading Git history (co-change graph)
[scent] Loading configuration (smell_detector.toml)
[scent] Evaluating rules
[scent] Assessing principle risks, pattern and refactoring recommendations
[scent] Analysis complete
```

What each line is actually doing:

| Step | What happens | Owning crate |
|---|---|---|
| **Discovering** | Walks the given directory for `.sln`/`.slnx`/`.csproj`/`.cs` files, skipping `bin/`, `obj/`, `*.g.cs`, `*.Designer.cs` by default. Nothing about C# syntax is understood yet — just file names. | `scent-parser::discovery` |
| **Discovered N/N/N** | Reports what it found: solution files, project files, source files, and how many were excluded by the rules above. | same |
| **Parsing [i/N] path** | Feeds that file's raw text through Tree-sitter, producing a Concrete Syntax Tree (a generic parse tree — not SCENT's own model yet). Runs once per discovered source file, in sorted path order (determinism). | `scent-parser::csharp::parse` |
| *(implicit, same line)* | Extraction walks that CST and pulls out **facts**: namespaces, types, members, calls, field accesses, instantiations, local variables — what the source plainly says, nothing interpreted. | `scent-parser::csharp::extract` |
| **Building declaration index** | Pass 1 of resolution: records "the project has a type named X," "X has a method named Y," across every file, before trying to resolve anything. | `scent-parser::csharp::index` |
| **Resolving intra-project references** | Pass 2: matches every recorded spelling (`"Order"`, `"Validate"`, ...) against that index. Exactly one match → `Resolved`. Anything ambiguous, external, or too complex (a framework type, an overloaded call, a multi-hop chain) → stays `Unresolved` with a stated reason — never a guess. | `scent-parser::csharp::resolve` |
| **Building the dependency graph** | Turns every `Resolved` fact into a typed edge (`Inherits`, `Calls`, `Creates`, `UsesType`, ...). An `Unresolved` fact produces no edge — you can't draw an arrow to something you don't know. | `scent-graph` |
| **Calculating metrics** | Pure counting over the facts and the graph: LOC (physical lines), cyclomatic complexity (`1 + decision points`), nesting depth, LCOM4 (cohesion), CBO (coupling). No opinions yet. | `scent-metrics` |
| **Reading Git history** | If the analyzed path is inside a Git work tree, shells out to `git log`/`git show` to build a co-change graph (which entities repeatedly change together). If it isn't a repo, or `git` isn't installed, this silently produces nothing — not an error. | `scent-git` |
| **Loading configuration** | Looks for `smell_detector.toml` directly under the analyzed path. Missing or unparseable → every rule just runs at its built-in default; this is never treated as an error. | `scent-config` |
| **Evaluating rules** | Runs every enabled rule (13 built in) against the facts + metrics + graph + git history + suppressions. Each rule combines several weighted, normalized observations into one confidence score — never a bare `metric > threshold` check — and only emits a finding above its minimum confidence. `// scent:disable RULE_ID` comments are applied here, centrally, so no rule has to parse comments itself. | `scent-rules` |
| **Assessing principle risks...** | Reads the findings (and, for a few, the graph/facts directly) to assess all 9 design principles, recommend a pattern for any switch statement found, and map each finding to a standard refactoring name. | `scent-rules::{principles,patterns,refactoring}` |
| **Analysis complete** | The whole `AnalysisReport` is now in memory; the CLI renders it as human/JSON/SARIF, or evaluates it against a quality gate. | `scent-cli` |

This is genuinely the same sequence for every command below — `analyze`,
`gate`, and (internally) `rules` all build on it; only the last step
(render vs. gate-check) differs.

---

## 2. `scent analyze` — human summary (default)

```text
scent> cargo run -q -p scent-cli -- analyze crates\scent-core\tests\fixtures\sample-project
```

```text
SCENT

Discovery
  Solutions: 0
  Projects:  1
  Sources:   1
  Excluded:  0

Semantic IR
  Namespaces: 1
  Types:      2
  Methods:    3
  Fields:     1
  Properties: 1

Metrics
  Methods measured: 3
  Types measured:   2
  Dependency edges: 3

Findings: 0

Principle risks: 0

Pattern recommendations: 0

Refactoring recommendations: 0

Diagnostics: 0
```

The sample fixture (`Order.cs`, two small classes) is too small to trigger
anything — every count past "Metrics" is honestly zero, not hidden.

### The same command against a fixture that *does* trigger a finding

`crates/scent-core/tests/fixtures/config-project` has a `smell_detector.toml`
that lowers `LONG_METHOD`'s thresholds (see §4) so a small method crosses
them:

```text
scent> cargo run -q -p scent-cli -- analyze crates\scent-core\tests\fixtures\config-project
```

```text
SCENT

Discovery
  Solutions: 0
  Projects:  0
  Sources:   1
  Excluded:  0

Semantic IR
  Namespaces: 1
  Types:      1
  Methods:    1
  Fields:     0
  Properties: 0

Metrics
  Methods measured: 1
  Types measured:   1
  Dependency edges: 0

Findings: 1
  [Critical] Long Method — e311e3a6a3055c3647bbfe1abe1a8e45a0ec42f129bcdaa1dc8e8d19dbe70c12 (confidence 54%)

Principle risks: 1
  Medium KISS risk: evidence suggests unnecessary complexity relative to what the entity needs to do

Pattern recommendations: 0

Refactoring recommendations: 1
  e311e3a6a3055c3647bbfe1abe1a8e45a0ec42f129bcdaa1dc8e8d19dbe70c12 -> ExtractMethod

Diagnostics: 0
```

Reading this end to end: a `LONG_METHOD` finding fed the KISS principle
risk (both point at the same method id), and the Refactoring Advisor
mapped that same finding to `ExtractMethod` — three views of one underlying
fact, not three independent guesses.

---

## 3. `scent analyze --format json`

The full deterministic report — every fact, metric, finding, principle
risk, and recommendation, as one JSON object. Two runs over the same input
produce byte-identical output (this is tested).

```text
scent> cargo run -q -p scent-cli -- analyze crates\scent-core\tests\fixtures\config-project --format json
```

```json
{
  "project_id": "14115d16527deef674c56e5f64e6ca96545518d4706b501c2860b97d76a58370",
  "language": "CSharp",
  "types": [{ "id": "3d1d...", "name": "Order", "kind": "class", "...": "..." }],
  "methods": [{
    "id": "e311e3a6a3055c3647bbfe1abe1a8e45a0ec42f129bcdaa1dc8e8d19dbe70c12",
    "name": "Ship",
    "local_variables": [
      { "name": "a", "type_reference": { "status": "unresolved", "reference": { "spelling": "var", "reason": "no intra-project type with this name" } } }
    ]
  }],
  "metrics": [
    { "entity": { "kind": "method", "id": "e311..." }, "metric": "loc", "value": { "value": 6, "confidence": 1 } }
  ],
  "findings": [{
    "rule": "LONG_METHOD",
    "severity": "critical",
    "confidence": 0.5428571701049805,
    "evidence": [
      { "metric": "loc", "observed_value": 6, "explanation": "6 physical lines (threshold 1)" },
      { "metric": "cyclomatic_complexity", "observed_value": 1, "explanation": "cyclomatic complexity 1 (threshold 1)" },
      { "metric": "nesting_depth", "observed_value": 0, "explanation": "max nesting depth 0 (threshold 1)" }
    ]
  }],
  "principle_risks": [{
    "principle": "Kiss",
    "risk": "Medium",
    "explanation": "Medium KISS risk: evidence suggests unnecessary complexity relative to what the entity needs to do",
    "evidence": [{ "rule": "LONG_METHOD", "summary": "Long Method finding at 0.54 confidence" }]
  }],
  "refactoring_recommendations": [
    { "entity": "e311...", "rule": "LONG_METHOD", "refactoring": "extract_method" }
  ]
}
```

(Trimmed for readability — the real output is one unbroken line; run the
command yourself for the untrimmed version, or see
`crates/scent-core/tests/fixtures/sample-project/expected_output.json` for
a full, real, checked-in example.) Notice the `evidence` array under the
finding: `loc = 6` against a *configured* threshold of `1` (not the
built-in default of `30`) — proof the `smell_detector.toml` override in §4
actually took effect, not just that the file parsed.

---

## 4. `smell_detector.toml` — real config file, real effect

`crates/scent-core/tests/fixtures/config-project/smell_detector.toml`:

```toml
[rules.LONG_METHOD.thresholds]
loc = 1
cyclomatic_complexity = 1
nesting_depth = 1
```

`crates/scent-core/tests/fixtures/config-disabled-project/smell_detector.toml`
shows the other kind of override — disabling a rule outright:

```toml
[rules.LONG_PARAMETER_LIST]
enabled = false
```

With that file present, a method with 6 parameters (which would otherwise
trigger `LONG_PARAMETER_LIST` at its default threshold of 5) produces zero
findings for that rule — verified in
`crates/scent-core/tests/config.rs::a_disabled_rule_produces_no_findings_for_it`.

A project with no `smell_detector.toml` at all (like `sample-project`
above) just gets every rule at its built-in default — never an error.

---

## 5. `scent analyze --format sarif`

Same analysis, rendered as [SARIF 2.1.0](https://sarifweb.azurewebsites.net/)
for CI systems and IDEs that consume it directly (GitHub code scanning, VS
Code's SARIF viewer, etc.):

```text
scent> cargo run -q -p scent-cli -- analyze crates\scent-core\tests\fixtures\config-project --format sarif
```

```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [{
    "tool": { "driver": { "name": "SCENT", "rules": [{ "id": "LONG_METHOD", "name": "Long Method" }] } },
    "results": [{
      "ruleId": "LONG_METHOD",
      "level": "error",
      "message": { "text": "Long Method (54% confidence)" },
      "locations": [{
        "physicalLocation": {
          "artifactLocation": { "uri": "src/Order.cs" },
          "region": { "startLine": 4, "startColumn": 8, "endLine": 9, "endColumn": 9 }
        }
      }],
      "properties": { "confidence": 0.5428571701049805, "severity": "critical" }
    }]
  }]
}
```

---

## 6. `scent rules`

Lists every currently registered rule id and name — useful for scripting
(e.g. building a `smell_detector.toml` section per rule) or just confirming
what a build actually enforces:

```text
scent> cargo run -q -p scent-cli -- rules
```

```text
Registered rules
  LONG_PARAMETER_LIST      Long Parameter List
  LONG_METHOD              Long Method
  LARGE_CLASS              Large Class
  DATA_CLUMPS              Data Clumps
  PRIMITIVE_OBSESSION      Primitive Obsession
  INAPPROPRIATE_INTIMACY   Inappropriate Intimacy
  REFUSED_BEQUEST          Refused Bequest
  SPECULATIVE_GENERALITY   Speculative Generality
  FEATURE_ENVY             Feature Envy
  DUPLICATED_CODE          Duplicated Code
  SWITCH_STATEMENTS        Switch Statements
  DIVERGENT_CHANGE         Divergent Change
  SHOTGUN_SURGERY          Shotgun Surgery
```

---

## 7. `scent gate` — CI pass/fail

Runs the same analysis, then checks the findings against thresholds
(`--max-critical`, `--max-high`, or `[quality_gate]` in
`smell_detector.toml`) and exits non-zero on failure — the shape a CI step
actually needs.

**Passing** (the empty sample fixture, generous limits):

```text
scent> cargo run -q -p scent-cli -- gate crates\scent-core\tests\fixtures\sample-project --max-critical 0 --max-high 5
```
```text
scent gate: PASSED (0 finding(s))
```

**Failing** (the config-project fixture, zero critical findings allowed):

```text
scent> cargo run -q -p scent-cli -- gate crates\scent-core\tests\fixtures\config-project --max-critical 0
```
```text
scent gate: FAILED
  - 1 critical finding(s) exceed the allowed 0
```
Exit code `1` — this is exactly what a CI pipeline step checks.

---

## 8. Suppressing a specific finding

`// scent:disable RULE_ID` / `// scent:enable RULE_ID` around a block of
source (the sample fixture's own `Order.cs` uses this around `Validate()`):

```csharp
// scent:disable LONG_METHOD
public void Validate()
{
}
// scent:enable LONG_METHOD
```

Suppression is applied centrally, after every rule runs, so no individual
rule has to parse comments itself — see
`crates/scent-core/tests/fixtures/sample-project/src/Order.cs` and
`crates/scent-core/tests/analyze.rs::scans_suppressions_from_source_comments`
for the real, tested behavior.

---

## Where to go next

- `docs/LEARNING_GUIDE.md` — the same pipeline traced through one file's
  facts in detail, plus a Rust (and C#-comparison) primer for the
  implementation language itself.
- `docs/status.md` — exactly what's implemented, continuously updated.
- `docs/architecture.md` — crate boundaries and the facts/metrics/findings
  separation this whole design rests on.
