# Implementation Status

Status is intentionally specific. “Planned” means no production capability is
claimed, even if a type or placeholder crate exists.

| Area | Status | Evidence |
| --- | --- | --- |
| Rust workspace and CI gate | Implemented | workspace manifests and `.github/workflows/ci.yml` |
| Deterministic typed IDs | Implemented | `scent-domain::id` tests |
| Repository-relative path normalization | Implemented | `scent-domain::path` tests |
| Source locations, diagnostics, resolution contracts | Implemented | `scent-domain` modules |
| Facts-only IR and basic validation | Implemented | `scent-ir::model` and validation tests, including `FieldAccess`/`PropertyId` (closed a gap against `docs/prompt.md` §10 found during review) |
| C# discovery and default exclusions | Implemented | `scent-parser::discovery` fixture |
| C# Tree-sitter parsing and parse diagnostics | Implemented | `CSharpAdapter` tests |
| Namespace and type declaration extraction | Implemented | `csharp.rs` tests |
| Member extraction | Implemented | methods, constructors, fields, properties, parameters (`csharp/extract.rs`) |
| Calls, creations, and field accesses inside method bodies | Implemented | recorded as `Unresolved` facts; see `collect_method_facts` |
| Inheritance/interface-list extraction | Implemented | `TypeIR.interface_types` holds every `base_list` entry; resolution moves a resolved first-position `Class`/`Record` entry into `base_types` (see `parse_base_list`, `resolve_project`) |
| Two-pass symbol index/resolution | Implemented | `csharp/index.rs` (`DeclarationIndex`), `csharp/resolve.rs` (`resolve_project`); exact intra-project matches only, ambiguous/unknown stay `Unresolved` with an explanatory reason |
| Suppression scanning | Implemented | `scent_domain::suppression` (`SuppressionMap::scan`), text-based, language-neutral |
| `scent analyze <path> [--format human\|json\|sarif\|table]` | Implemented | `scent-core::pipeline` orchestrates the whole pipeline; `scent-cli` wires it to argv, plus `scent rules` and `scent gate`. `<path>` works for any directory on disk, not only paths inside this repo. `--help`/`-h` is recognized at the top level and by `analyze`/`gate`, exits 0, and never falls through to the failure-usage message. An unrecognized top-level command (e.g. `scent anlyze`) suggests the closest known command by Levenshtein distance (`did you mean 'analyze'?`), capped at distance 2 so an unrelated word gets no misleading guess |
| Deterministic IR JSON | Implemented | `scent-core::report` (hand-rolled writer, no serde dependency yet); byte-identical across runs (tested) |
| Dependency graph | Implemented | `scent-graph::build_graph`; edges only from `Resolved` facts (Inherits/Implements/Calls/ReadsField/Creates/Returns/AcceptsParameter/UsesType). `WritesField`/`Throws` variants exist but nothing emits them yet (no write/exception facts extracted) |
| Metrics | Implemented | `scent-metrics`: LOC (physical lines), cyclomatic complexity (`1 + decision_points`), nesting depth, LCOM4 (connected components via shared-field/calls), CBO (symmetric distinct-type coupling via the graph) |
| CLI progress visibility | Implemented | `analyze_path_with_progress` reports each pipeline stage; `scent-cli` prints them to stderr, keeping stdout a clean report. Each line is numbered `[step/total]` (`scent-core::pipeline`'s `stage!` macro), where `total` is 12 with Git history available or 11 without (the only stage that's skipped rather than emptied) |
| Shell tab-completion | Implemented | `completions/scent.bash` (bash) and `completions/scent.ps1` (PowerShell); hand-written since the CLI's argument parser isn't built on a framework that generates one — completes command/flag *names* only, not `<path>` (the shell's own path completion already covers that). Both print a one-line confirmation once enabled, since registration itself succeeds silently |
| Rule platform | Implemented | `scent-rules`: `AnalysisContext`, `Rule` trait, `RuleRegistry` (suppression enforced centrally in `evaluate_all`) |
| Evidence/confidence/severity engine | Implemented | `scent-rules::evidence`: shared `normalize_ratio`/`build_evidence`/`severity_from_observations`, so no rule hand-rolls a binary threshold |
| Rules | Implemented (11 of 11 Refactoring.Guru + 2 of 2 Git-history) | Long Method, Large Class, Long Parameter List, Data Clumps, Primitive Obsession, Inappropriate Intimacy, Refused Bequest, Speculative Generality, Feature Envy, Duplicated Code (structural clone hashing via `CloneSignature`, exact-match only — see limits below), Switch Statements (`SwitchShape`: case count, densest-case branch complexity, distinct constructed types — reports only the smell, never a candidate pattern), plus Divergent Change and Shotgun Surgery (see the Git history rows below) |
| Receiver-scoped call/member resolution | Implemented | `CallReceiver` fact (`SelfOrImplicit`/`Named(name)`/`Other`) records what a call/access syntax says its receiver is; the resolver follows `this`/bare (against the enclosing type) and a named local variable, parameter, or field (checked in that shadowing order, against *that* declaration's resolved type) — anything else stays honestly `Unresolved`. This replaced an earlier bug where every call/access was resolved against the enclosing type regardless of receiver, which could have silently matched the wrong member by name coincidence |
| Local variable extraction | Implemented | `MethodIR.local_variables: Vec<LocalVariable>` — a real fact from each `local_declaration_statement`'s declared type (never inferred from `var`'s initializer); lets the resolver follow a call/access routed through a local (`var w = ...; w.Reserve();`), closing the recall gap Feature Envy/Inappropriate Intimacy previously had |
| Finding fingerprints | Implemented | `FindingFingerprint` — `location_hash` derived from `(rule_id, entity_id)`, not a line number |
| JSON/SARIF/baseline/quality gate | Implemented | `scent-report`; `scent analyze --format json\|sarif`, `scent gate` (real pass/fail exit code), `scent rules` all wired into the CLI and manually verified |
| Finding location + source snippet | Implemented | Every finding already carried a real `location: SourceLocation` (used internally for fingerprinting); now also surfaced everywhere a finding is shown. `--format table` adds a `LOCATION` column (`path:start-end`, 1-based) plus a `SOURCE` section with the real code from disk, capped at 20 lines with an "N more line(s) omitted" note. `--format json`'s `finding.snippet` (`scent-report::findings::read_snippet`) carries the same lines/cap, read fresh from `root.join(location.path)` at report time — the one field in the JSON report that isn't purely derived from the parsed IR, so it can differ between runs if the source file changed in between (`null` if the file can't be read) |
| Git history engine | Implemented | `scent-git`: shells out to the `git` CLI (`log`/`show --unified=0`) rather than adding a `git2`/libgit2 dependency; `log_commits` -> `resolve_changes` (maps new-file diff-hunk line ranges onto current type/method `SourceLocation`s, skipping pure deletions rather than guessing which entity they belonged to) -> `CoChangeGraph` (commit count, commit IDs, first/last-seen timestamps per co-changing pair) |
| Divergent Change / Shotgun Surgery | Implemented | `scent-rules::rules::{divergent_change, shotgun_surgery}`; both read `AnalysisContext::history` and produce no findings when it is `None` (no repo, or `git` unavailable) or below their minimum-evidence thresholds — Shotgun Surgery never fires on fewer than `MIN_COMMITS` (3) shared commits, matching `docs/prompt.md` §15's "not from one unusual commit" |
| Principle risk engine | Implemented (9 of 9 principles) | `scent-rules::principles`: SRP (Large Class/Feature Envy), LSP (Refused Bequest), DRY (Duplicated Code/Data Clumps), KISS (Long Method/Large Class/Switch Statements), YAGNI (Speculative Generality), and OCP (Switch Statements — a type-discriminating switch must be modified, not extended, to add a case) aggregate real findings. DIP (dependency-graph `Creates`/`AcceptsParameter`/`Returns`/`UsesType` edges: concrete-class ratio per type), ISP (reuses `RefusedBequest`'s "looks trivially unimplemented" check against `Implements` edges instead of `Inherits`), and Law of Demeter (new `receiver_chain_depth` fact on `MethodCall`/`FieldAccess`, flagging chains of depth 2+) read `AnalysisContext` directly since none has a natural single-`Finding` shape to reuse. All use the spec's hedged wording ("High X risk", never "X violated") |
| Pattern advisor | Implemented (Factory Method / Do Nothing only) | `scent-rules::patterns`: reads `SwitchShape.distinct_created_types` independently of the Switch Statements finding (per `prompt.md` §29, pattern advice is independent of smell detection); recommends Factory Method when a switch constructs 2+ distinct types, else the explicitly-valid "Do Nothing". Strategy/State are not offered — both need a "does behavior depend on internal state" signal not yet computed |
| Refactoring advisor | Implemented | `scent-rules::refactoring`: a static `rule_id -> Refactoring` table using only the 8 refactorings `prompt.md` §30 names; a test asserts every currently-registered rule has an entry |
| Config loading (`smell_detector.toml`) | Implemented | new `scent-config` crate: a minimal hand-rolled TOML parser (`[section]`/`[section.sub]` headers, scalars, string arrays — no dates/inline-tables/multi-line strings, since this schema never uses them) plus `ScentConfig` (per-rule `enabled`/`severity`/`thresholds`, `[quality_gate]` overrides). `scent-rules::build_registry(&config)` skips a disabled rule entirely and applies threshold overrides by rule id; `scent-core::pipeline` applies severity overrides post-evaluation; `scent-cli gate` uses `[quality_gate]` as the base, CLI flags still override on top. A missing or unparseable file is never an error — just every rule at its built-in default. Verified end-to-end in `scent-core/tests/config.rs` (a real threshold override and a real disabled rule, not just "the file parses") |
| TUI, Docker sandbox, cache, C++ | Planned | later milestones |

## Immediate next steps

1. Config loading currently overrides one threshold per rule (matching its
   dominant `Observation`), not every field on the rule struct (e.g.
   `DataClumps.min_clump_size` has no override yet) — a deliberate, small
   first slice, not the full field set.
2. Git history is read against the *current* checkout's entity locations,
   not a historical reconstruction — a pure deletion hunk (no new-file
   lines) is not attributed to any entity, and a renamed/moved file breaks
   the association between its old and new history.
3. Receiver-scoped resolution still does not resolve through a chain (`a.b.c`)
   or a receiver reassigned mid-method to a different type; it resolves one
   local/parameter/field level deep, using that declaration's single
   originally-declared type.

The detailed milestone and acceptance criteria are maintained in
`../../docs/implementation_plan.md`.
