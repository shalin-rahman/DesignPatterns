# CLI & Workflow Guide

Two things in one place: every `scent` command with a real example and its
real captured output, and a step-by-step trace of what happens internally
between "you press Enter" and "a report appears" — a dry run of the
pipeline, stage by stage, in plain words.

All output below was captured for real from this repository's own test
fixtures, not written from memory or invented. Commands are run from
`sdp_lab_tools/scent/` in this guide only so the example `<path>` values
below (`crates\scent-core\tests\fixtures\...`) are short — it is not a
requirement.

---

## 0. Where to actually run these from

**`scent` itself, once installed (`cargo install --path crates/scent-cli`
— see the repo [README](../README.md)), runs from *any* directory, on any
project on disk.** Your current directory only matters because `<path>`
is resolved relative to it — pass an absolute path instead and it doesn't
matter at all:

```powershell
# from anywhere — cwd here is unrelated to this repo
PS C:\Users\you\some-other-project> scent analyze .
PS C:\Users\you\some-other-project> scent analyze C:\Users\you\some-other-project --format table
```

**Tab-completion setup (`completions/scent.ps1` / `scent.bash`) also works
from any directory** — dot-source it once per new terminal window, using
its full path, and it stays active for that window's whole session (see
[../completions/README.md](../completions/README.md) for the underlying
mechanics):

```powershell
PS C:\Users\you\some-other-project> . "C:\path\to\sdp_lab_tools\scent\completions\scent.ps1"
scent: tab-completion enabled. Try: scent <Tab>  or  scent analyze --<Tab>
PS C:\Users\you\some-other-project> scent analyze .
```

If `scent` itself isn't found ("not recognized as the name of a
cmdlet..."), that's a PATH problem, not a "wrong directory" problem — see
the [README's install section](../README.md) for `cargo install`, and
note that a PowerShell window opened *before* installing only picks up
the updated PATH after you open a new window (or manually run
`$env:PATH += ";$env:USERPROFILE\.cargo\bin"` in the current one).

---

## 1. The pipeline, step by step

Every `scent analyze`/`scent gate` invocation runs the same sequence.
`--format human` (the default) prints a progress line to stderr as each
stage starts. Each line is numbered `[step/total]` so a long analysis
tells you how far through it is, not just what it's doing right now —
this section explains what each numbered line actually means.

```text
> scent analyze crates\scent-core\tests\fixtures\sample-project
```

stderr, in order:

```text
[scent] [1/12] Discovering C# project files under crates\scent-core\tests\fixtures\sample-project
[scent] [2/12] Discovered 0 solution file(s), 1 project file(s), 1 source file(s), 0 excluded
[scent] [3/12] Parsing file [1/1] src/Order.cs
[scent] [4/12] Building declaration index
[scent] [5/12] Resolving intra-project references
[scent] [6/12] Building the dependency graph
[scent] [7/12] Calculating metrics (LOC, CC, nesting, LCOM4, CBO)
[scent] [8/12] Reading Git history (co-change graph)
[scent] [9/12] Loading configuration (smell_detector.toml)
[scent] [10/12] Evaluating rules
[scent] [11/12] Assessing principle risks, pattern and refactoring recommendations
[scent] [12/12] Analysis complete
```

The total is 12 when the analyzed path is a Git work tree, 11 when it
isn't (step "Reading Git history" is skipped entirely, not just emptied,
so the total shrinks by one rather than staying fixed and looking stuck).
Step 3 ("Parsing file") also carries its own `[i/N]` — that's a second,
separate counter for *which source file* out of how many, nested inside
the one overall pipeline step.

What each step is actually doing:

| Step | What happens | Owning crate |
|---|---|---|
| **1. Discovering** | Walks the given directory for `.sln`/`.slnx`/`.csproj`/`.cs` files, skipping `bin/`, `obj/`, `*.g.cs`, `*.Designer.cs` by default. Nothing about C# syntax is understood yet — just file names. | `scent-parser::discovery` |
| **2. Discovered N/N/N** | Reports what it found: solution files, project files, source files, and how many were excluded by the rules above. | same |
| **3. Parsing file [i/N] path** | Feeds that file's raw text through Tree-sitter, producing a Concrete Syntax Tree (a generic parse tree — not SCENT's own model yet). Runs once per discovered source file, in sorted path order (determinism). | `scent-parser::csharp::parse` |
| *(implicit, same step)* | Extraction walks that CST and pulls out **facts**: namespaces, types, members, calls, field accesses, instantiations, local variables — what the source plainly says, nothing interpreted. | `scent-parser::csharp::extract` |
| **4. Building declaration index** | Pass 1 of resolution: records "the project has a type named X," "X has a method named Y," across every file, before trying to resolve anything. | `scent-parser::csharp::index` |
| **5. Resolving intra-project references** | Pass 2: matches every recorded spelling (`"Order"`, `"Validate"`, ...) against that index. Exactly one match → `Resolved`. Anything ambiguous, external, or too complex (a framework type, an overloaded call, a multi-hop chain) → stays `Unresolved` with a stated reason — never a guess. | `scent-parser::csharp::resolve` |
| **6. Building the dependency graph** | Turns every `Resolved` fact into a typed edge (`Inherits`, `Calls`, `Creates`, `UsesType`, ...). An `Unresolved` fact produces no edge — you can't draw an arrow to something you don't know. | `scent-graph` |
| **7. Calculating metrics** | Pure counting over the facts and the graph: LOC (physical lines), cyclomatic complexity (`1 + decision points`), nesting depth, LCOM4 (cohesion), CBO (coupling). No opinions yet. | `scent-metrics` |
| **8. Reading Git history** | If the analyzed path is inside a Git work tree, shells out to `git log`/`git show` to build a co-change graph (which entities repeatedly change together). If it isn't a repo, or `git` isn't installed, this step is skipped outright (the total step count drops to 11) — not an error. | `scent-git` |
| **9. Loading configuration** | Looks for `smell_detector.toml` directly under the analyzed path. Missing or unparseable → every rule just runs at its built-in default; this is never treated as an error. | `scent-config` |
| **10. Evaluating rules** | Runs every enabled rule (13 built in) against the facts + metrics + graph + git history + suppressions. Each rule combines several weighted, normalized observations into one confidence score — never a bare `metric > threshold` check — and only emits a finding above its minimum confidence. `// scent:disable RULE_ID` comments are applied here, centrally, so no rule has to parse comments itself. | `scent-rules` |
| **11. Assessing principle risks...** | Reads the findings (and, for a few, the graph/facts directly) to assess all 9 design principles, recommend a pattern for any switch statement found, and map each finding to a standard refactoring name. | `scent-rules::{principles,patterns,refactoring}` |
| **12. Analysis complete** | The whole `AnalysisReport` is now in memory; the CLI renders it as human/JSON/SARIF/table, or evaluates it against a quality gate. | `scent-cli` |

This is genuinely the same sequence for every command below — `analyze`,
`gate`, and (internally) `rules` all build on it; only the last step
(render vs. gate-check) differs.

---

## 2. `--help` — every command's own flags

```text
> scent --help
```

```text
scent — static smell/design analysis for C# projects

USAGE:
  scent <COMMAND> [ARGS]

COMMANDS:
  analyze <path>   Analyze a C# solution/project directory and report findings
  rules            List every registered rule id and name
  gate <path>      Analyze, then pass/fail against a quality gate

<path> can be any directory on disk — inside or outside this repo — that
contains a .sln, .csproj, or .cs files. scent only reads it; it never runs
MSBuild or dotnet.

Run 'scent analyze --help' or 'scent gate --help' for a command's own flags.
See docs/CLI_WORKFLOW_GUIDE.md for real examples of every command.
```

`scent analyze --help`:

```text
scent analyze <path> [OPTIONS]

Analyze a C# solution/project directory. <path> can point anywhere on
disk, including a directory outside this repository.

OPTIONS:
  --format human   Readable summary with findings, risks, recommendations (default)
  --format json    Full machine-readable report — see docs/JSON_OUTPUT_REFERENCE.md
  --format sarif   SARIF output for editors/CI (e.g. GitHub code scanning)
  --format table   Findings and principle risks as aligned columns
  -h, --help       Show this message
```

`scent gate --help`:

```text
scent gate <path> [OPTIONS]

Analyze <path>, then pass or fail against a quality gate. Exits 0 on
PASSED, 1 on FAILED — safe to use as a CI step.

OPTIONS:
  --max-critical N   Fail if more than N Critical findings (default from
                     smell_detector.toml, else built-in default)
  --max-high N       Fail if more than N High findings
  --baseline FILE    Ignore findings already present in this baseline file
  -h, --help         Show this message
```

`scent rules` has no flags of its own; it always lists every registered
rule (§8 below).

`<path>` in every command above works exactly the same whether it's inside
this repository or on a completely different drive/directory — SCENT only
reads the files under it, it never assumes it's analyzing itself.

### Typo'd a command name? scent suggests the closest match

```text
> scent anlyze crates\scent-core\tests\fixtures\sample-project
```

```text
scent: unrecognized command 'anlyze'
       did you mean 'analyze'?
usage:
  scent analyze <path> [--format human|json|sarif|table]
  scent rules
  scent gate <path> [--max-critical N] [--max-high N] [--baseline FILE]
  scent --help                (show full help)
  scent analyze --help        (show analyze's flags)
```

This is a simple edit-distance check against the known commands
(`analyze`, `rules`, `gate`, `--help`) — it only offers a suggestion
within a small distance, so an unrelated word like `scent xyz123` gets
just the usage text, no misleading guess.

---

## 3. `scent analyze` — human summary (default)

```text
> scent analyze crates\scent-core\tests\fixtures\sample-project
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
that lowers `LONG_METHOD`'s thresholds (see §6) so a small method crosses
them:

```text
> scent analyze crates\scent-core\tests\fixtures\config-project
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

## 4. `scent analyze --format table`

The same findings and principle risks as the human summary above, but as
aligned text columns instead of free-form sentences — easier to scan or
paste somewhere that needs straight columns:

```text
> scent analyze crates\scent-core\tests\fixtures\config-project --format table
```

```text
Project: crates\scent-core\tests\fixtures\config-project

FINDINGS (1)
  SEVERITY   RULE                     CONF %   ENTITY
  Critical   Long Method              54       e311e3a6a3055c3647bbfe1abe1a8e45a0ec42f129bcdaa1dc8e8d19dbe70c12

PRINCIPLE RISKS (1)
  RISK     PRINCIPLE      CONF %   EXPLANATION
  Medium   Kiss           54       Medium KISS risk: evidence suggests unnecessary complexity relative to what the entity needs to do
```

This is not JSON — it's plain text, meant for a terminal or a text file.
If you need the data in a structured form to feed another tool, use
`--format json` instead (next section) and see
[docs/JSON_OUTPUT_REFERENCE.md](JSON_OUTPUT_REFERENCE.md) for what each
field means.

---

## 5. `scent analyze --format json`

The full deterministic report — every fact, metric, finding, principle
risk, and recommendation, as one JSON object. Two runs over the same input
produce byte-identical output (this is tested).

```text
> scent analyze crates\scent-core\tests\fixtures\config-project --format json
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
built-in default of `30`) — proof the `smell_detector.toml` override in §6
actually took effect, not just that the file parsed.

---

## 6. `smell_detector.toml` — real config file, real effect

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

## 7. `scent analyze --format sarif`

Same analysis, rendered as [SARIF 2.1.0](https://sarifweb.azurewebsites.net/)
for CI systems and IDEs that consume it directly (GitHub code scanning, VS
Code's SARIF viewer, etc.):

```text
> scent analyze crates\scent-core\tests\fixtures\config-project --format sarif
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

## 8. `scent rules`

Lists every currently registered rule id and name — useful for scripting
(e.g. building a `smell_detector.toml` section per rule) or just confirming
what a build actually enforces:

```text
> scent rules
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

## 9. `scent gate` — CI pass/fail

Runs the same analysis, then checks the findings against thresholds
(`--max-critical`, `--max-high`, or `[quality_gate]` in
`smell_detector.toml`) and exits non-zero on failure — the shape a CI step
actually needs.

**Passing** (the empty sample fixture, generous limits):

```text
> scent gate crates\scent-core\tests\fixtures\sample-project --max-critical 0 --max-high 5
```
```text
scent gate: PASSED (0 finding(s))
```

**Failing** (the config-project fixture, zero critical findings allowed):

```text
> scent gate crates\scent-core\tests\fixtures\config-project --max-critical 0
```
```text
scent gate: FAILED
  - 1 critical finding(s) exceed the allowed 0
```
Exit code `1` — this is exactly what a CI pipeline step checks.

---

## 10. Suppressing a specific finding

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

## 11. A real worked example: a whole messy project at once

Every example above runs against a small fixture with one deliberate
finding. `sdp_lab_tools/code-smells-demo/` is a different kind of example:
a real, buildable C# console app (`dotnet build`/`dotnet run` both work)
written to contain all 12 code smells from a lecture, wired together like
an actual small codebase instead of isolated snippets. Its own
`REFACTORING_GUIDE.md` explains each smell, where it lives, and how to fix
it — this section just shows what SCENT itself reports against it:

```text
> scent analyze sdp_lab_tools\code-smells-demo --format table
```

```text
Project: sdp_lab_tools/code-smells-demo

FINDINGS (15)
  SEVERITY   RULE                     CONF %   ENTITY
  Medium     Duplicated Code          57       a25b9c4b...
  Medium     Duplicated Code          57       dba4181d...
  High       Feature Envy             67       0fdc79eb...
  High       Feature Envy             67       d99f5915...
  High       Feature Envy             67       05ae973e...
  High       Feature Envy             67       d02c711c...
  Medium     Inappropriate Intimacy   64       2dc6f7a3...<->32daca12...
  Medium     Long Method              51       05ae973e...
  Medium     Long Parameter List      62       e95d2b30...
  Medium     Primitive Obsession      57       927d8c31...
  Medium     Primitive Obsession      57       8f54b659...
  High       Primitive Obsession      73       e95d2b30...
  Medium     Refused Bequest          50       94d894ff...
  Medium     Speculative Generality   50       13a17174...
  Medium     Speculative Generality   50       3a090954...

PRINCIPLE RISKS (15)
  RISK     PRINCIPLE      CONF %   EXPLANATION
  ...      Srp / Lsp / Dry / Kiss / Yagni / Dip / Isp / LawOfDemeter
```

(Entity IDs are truncated above for readability — the real output prints
the full SHA-256 hash, see [docs/JSON_OUTPUT_REFERENCE.md](JSON_OUTPUT_REFERENCE.md).)

SCENT catches 7 of the 12 smells under their own name: Duplicated Code,
Feature Envy, Inappropriate Intimacy, Long Method, Long Parameter List,
Primitive Obsession, and Speculative Generality. It has no dedicated rule
(yet) for Large Class/God Class, Divergent Change, Shotgun Surgery, Switch
Statements, or Data Clumps — those 5 are genuinely present in the code and
documented in the demo's refactoring guide, just not flagged by SCENT
under those exact names. This is useful to know honestly rather than
implying full coverage: it's a real gap in the current ruleset, not a bug
in the demo project.

---

## Where to go next

- [docs/JSON_OUTPUT_REFERENCE.md](JSON_OUTPUT_REFERENCE.md) — what every
  field in `--format json` means, in plain words (severity vs. risk,
  confidence, evidence, and where "project name" actually comes from).
- [docs/LEARNING_GUIDE.md](LEARNING_GUIDE.md) — the same pipeline traced
  through one file's facts in detail, plus a Rust (and C#-comparison)
  primer for the implementation language itself.
- [docs/status.md](status.md) — exactly what's implemented, continuously
  updated.
- [docs/architecture.md](architecture.md) — crate boundaries and the
  facts/metrics/findings separation this whole design rests on.
- [../completions/README.md](../completions/README.md) — bash/PowerShell
  tab-completion for `scent`'s command and flag names. Both scripts print
  a one-line confirmation once enabled (registration itself is silent, so
  without it there'd be no sign anything happened), and the PowerShell
  script works whether or not you dot-source it — see that file for why.
