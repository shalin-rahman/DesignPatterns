# SCENT — Learning Guide

This is a plain-language walkthrough of the project: what it does, how to run
it, what happens step by step inside it, how the code is organized, and a
Rust crash course using real snippets from this codebase. Read top to bottom
once, then use it as a reference.

---

## 1. What SCENT is, in one paragraph

SCENT reads a C# codebase and tells you, with evidence, where the design has
problems — methods that are too long, classes that do too much, duplicated
code, and so on. It does this in stages: read the files, understand the
syntax, figure out what each name refers to, count things (lines, branches,
coupling), then apply rules that turn those counts into findings with a
confidence score and a severity. The output is JSON, SARIF, or a short
CI-friendly pass/fail (`scent gate`).

It is **not** a compiler and does not run your code. It only reads source
text and reasons about its structure.

---

## 2. The manual: building, testing, running

Everything below runs from `sdp_lab_tools/scent/` (the Cargo workspace root).

```powershell
# Build everything
cargo build --workspace

# Run every test in every crate
cargo test --workspace

# Format code (auto-fixes spacing/wrapping)
cargo fmt

# Check formatting without changing files (used in CI)
cargo fmt --check

# Lint — this project treats warnings as errors
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Using the tool itself:

```powershell
# Human-readable summary
cargo run -p scent-cli -- analyze <path-to-a-csharp-project>

# Machine-readable JSON (facts + metrics + findings)
cargo run -p scent-cli -- analyze <path> --format json

# SARIF (for GitHub code scanning / CI tools)
cargo run -p scent-cli -- analyze <path> --format sarif

# List every rule that's registered
cargo run -p scent-cli -- rules

# CI gate: fails (non-zero exit code) if too many findings
cargo run -p scent-cli -- gate <path> --max-critical 0 --max-high 10
```

There's a ready-made example project you can point it at:
`crates/scent-core/tests/fixtures/sample-project/`.

---

## 3. The full pipeline, traced through one real file

This is the important part. Everything else in the codebase exists to make
this sequence work. We'll trace one real file,
`crates/scent-core/tests/fixtures/sample-project/src/Order.cs`:

```csharp
namespace Demo.Models
{
    public class Order
    {
        private int total;
        public string Status { get; set; }

        public void Ship()
        {
            this.Validate();
        }

        public void Validate() { }
    }

    public class OrderFactory
    {
        public Order Create()
        {
            return new Order();
        }
    }
}
```

### Stage 1 — Discovery (`scent-parser::discovery`)

Walks the directory tree looking for `.sln`, `.csproj`, and `.cs` files,
skipping `bin/`, `obj/`, generated files, etc. For our example it finds one
project file and one source file. Nothing about C# syntax is understood
yet — this stage only looks at file names and paths.

### Stage 2 — Parsing (`scent-parser::csharp::CSharpAdapter`)

Feeds the raw text of `Order.cs` into **Tree-sitter**, a parser generator
library. Tree-sitter turns the text into a tree of nodes — a **Concrete
Syntax Tree (CST)** — where every keyword, brace, and identifier is a node.
This is not SCENT's own data structure; it's a generic parse tree that
knows nothing about "methods" or "fields" as SCENT's concepts, only as
grammar rules like `method_declaration`.

If the source has a syntax error (a missing brace, garbage tokens), this
stage records a `Diagnostic` instead of crashing, and parsing continues for
the rest of the file.

### Stage 3 — Extraction (`scent-parser::csharp::extract`)

Walks the CST and pulls out **facts**: things the source plainly says,
with no interpretation. For `Order.cs`, extraction produces (informally):

```text
TypeIR   { name: "Order",        kind: Class, ... }
TypeIR   { name: "OrderFactory", kind: Class, ... }
FieldIR  { name: "total",  owner: Order,  type: "int" (not yet resolved) }
PropertyIR { name: "Status", owner: Order, type: "string" (not yet resolved) }
MethodIR { name: "Ship",     owner: Order,
           calls: [ MethodCall { spelling: "Validate", receiver: SelfOrImplicit } ] }
MethodIR { name: "Validate", owner: Order }
MethodIR { name: "Create",   owner: OrderFactory,
           instantiations: [ Instantiation { spelling: "Order" } ] }
```

Notice `"int"`, `"Validate"`, `"Order"` are just **spellings** at this
point — text captured from the source. SCENT doesn't yet know that `"int"`
means the built-in integer type, or that calling `"Validate"` means the
`Validate` method two lines down. That's the next stage's job. This
strict split — record what the text says now, work out what it *means*
later — is why the fact model never has to guess.

### Stage 4 — Two-pass resolution (`scent-parser::csharp::{index, resolve}`)

**Pass 1** builds lookup tables: "the project has a type named `Order`",
"`Order` has a method named `Validate`", etc.

**Pass 2** walks every recorded spelling and tries to match it in those
tables:

- `this.Validate()` inside `Ship()` → receiver is `this`, so look up
  `Validate` on `Order` → found exactly one → **Resolved**.
- `new Order()` inside `Create()` → look up `Order` project-wide → found
  exactly one → **Resolved**.
- The field `total`'s type, `"int"` → look up `int` as a project type →
  not found (it's a C# built-in, not something *this* project declares) →
  stays **Unresolved**, with a reason (`"no intra-project type with this
  name"`).

This is the same rule a real C# compiler would apply for `this.Foo()` —
it's not a guess, it's how the language actually binds names. If a call's
receiver is something more complicated SCENT can't yet track (a chained
expression, a local variable), it's left **Unresolved** with a reason
explaining *why*, rather than silently assuming it means something.

### Stage 5 — Dependency graph (`scent-graph`)

Every **Resolved** fact becomes an edge in a graph: `Ship --Calls--> Validate`,
`Create --Creates--> Order`. Unresolved facts produce no edge — you can't
draw an arrow to something you don't know. This graph is what later stages
use to answer questions like "how many other types does this type touch?"

### Stage 6 — Metrics (`scent-metrics`)

Pure counting, using the facts and the graph, no opinions yet:

| Metric | Meaning | Example here |
|---|---|---|
| LOC | physical lines a method/type spans | `Ship` = 4 lines |
| Cyclomatic Complexity | `1 + decision points` (`if`, `for`, `&&`, ...) | `Ship` = 1 (no branches) |
| Nesting depth | how deeply `if`/`for`/etc. are nested | 0 |
| LCOM4 | how many "islands" of unrelated methods a class has | `Order` = 1 (its two methods are connected: `Ship` calls `Validate`) |
| CBO | how many *other* project types a type touches | `OrderFactory` = 1 (touches `Order`) |

### Stage 7 — Rules (`scent-rules`)

Each rule reads the facts + metrics + graph (never mutates them) and
decides whether to emit a **Finding**. A rule never says "LOC > 30 →
finding"; it combines several weighted observations into a confidence
score between 0 and 1, and only emits a finding above a minimum
confidence. For this tiny example, nothing crosses that bar — `Order` and
`OrderFactory` are both small and simple, so `Findings: 0`.

If `Validate()` were 40 lines with five nested `if`s, the **Long Method**
rule would compute something like:

```text
LOC = 45         → normalized 0.6   × weight 0.4 = 0.24
CC  = 12         → normalized 0.55  × weight 0.4 = 0.22
Nesting = 5      → normalized 0.56  × weight 0.2 = 0.11
                                     confidence  = 0.57  →  emitted
```

### Stage 8 — Suppression + Report

Before a finding is returned, SCENT checks whether a
`// scent:disable RULE_ID` / `// scent:enable RULE_ID` comment pair covers
its line — if so, the finding is dropped. What's left gets rendered as
JSON, SARIF, or a human summary. Here's a real (trimmed) slice of the JSON
this exact file produces:

```json
{
  "name": "Order",
  "kind": "class",
  "location": { "path": "src/Order.cs", "start_line": 2, "end_line": 18 }
},
"calls": [{
  "target": { "status": "resolved", "value": "41610067e9...method-id" },
  "receiver": "self_or_implicit",
  "location": { "path": "src/Order.cs", "start_line": 10, "start_column": 12 }
}]
```

Every ID you see (`"41610067e9..."`) is a SHA-256 hash of a description
string like `"scent:v1:csharp:src/Order.cs::method:Order.Validate()"` —
**not** a random UUID. Run the same source through SCENT twice, or on two
different machines, and you get the exact same ID, because it's derived
from content, not from when/where it ran. That's what "deterministic"
means throughout this project.

---

## 4. Architecture: the crates and why they're separate

```text
scent-cli        (argv parsing, printing)
     ↓
scent-core       (glues every stage together: discover→parse→...→report)
     ↓          ↘
scent-rules   scent-report   (findings; JSON/SARIF/baseline/gate)
     ↓
scent-metrics  scent-graph   (counts and edges, derived from facts)
     ↓
scent-parser                (C# → facts, via Tree-sitter)
     ↓
scent-ir                    (the fact structs: ProjectIR, MethodIR, ...)
     ↓
scent-domain                (shared building blocks: IDs, paths, enums)
```

| Crate | Job | Analogy |
|---|---|---|
| `scent-domain` | IDs, file paths, source locations, small shared enums | The "vocabulary" every other crate speaks |
| `scent-ir` | The fact structs (`ProjectIR`, `MethodIR`, ...) + validation | The nouns — "what exists" |
| `scent-parser` | Turns C# text into those facts, resolves references | The reader |
| `scent-graph` | Turns resolved facts into a queryable graph | The map |
| `scent-metrics` | Counts things (LOC, complexity, coupling) | The measuring tape |
| `scent-rules` | Turns facts+metrics+graph into findings | The judge |
| `scent-report` | Findings → JSON/SARIF/baseline/quality-gate | The printer |
| `scent-core` | Runs all of the above in order | The conductor |
| `scent-cli` | Command-line front door | The receptionist |

**Why split this much?** Because the project's core rule is: *facts,
metrics, and findings must never blend together.* A fact ("this method
calls that one") must stay true regardless of opinion. A metric ("this
method is 45 lines") is derived from facts but still isn't an opinion. A
finding ("this is a Long Method, 57% confidence") is an opinion built on
top of both. Keeping them in different crates makes it physically
impossible to accidentally store an opinion where a fact should be — the
compiler won't let `scent-ir` import `scent-rules`.

---

## 5. Rust crash course, using this codebase's own code

If you're new to Rust, read this section slowly — every example below is
real code pulled from this project, not a toy example.

### 5.1 Cargo, crates, and modules

A **crate** is a compilable unit (roughly: one library or one binary). This
project is a **workspace** — one `Cargo.toml` at the root lists many
crates:

```toml
# scent/Cargo.toml
[workspace]
members = ["crates/scent-cli", "crates/scent-core", ...]
```

Inside a crate, `mod` splits code into files/folders, and `pub` controls
what's visible from outside:

```rust
// crates/scent-parser/src/csharp/mod.rs
mod extract;       // private module, file extract.rs
pub use extract::{extract_file, ExtractedFile};  // re-exported publicly
```

`use` imports names so you don't have to write the full path every time:

```rust
use scent_domain::{Resolution, TypeId};
```

**C# equivalent:** a workspace is like a `.sln` with several `.csproj`s
inside it. `mod`/`pub` map to C#'s file-based namespaces plus
`internal`/`public` — a private `mod` is roughly an `internal` class, a
`pub use` re-export is roughly a `using X = Y.Z;` alias exposed further up.

### 5.2 Structs and enums

A `struct` groups named fields — like a class with no methods by default:

```rust
// crates/scent-ir/src/model.rs
pub struct MethodCall {
    pub target: Resolution<MethodId>,
    pub receiver: CallReceiver,
    pub location: SourceLocation,
}
```

An `enum` is a value that's *one of* several named shapes — much stronger
than most languages' enums, because each variant can carry its own data:

```rust
// crates/scent-ir/src/model.rs
pub enum CallReceiver {
    SelfOrImplicit,      // carries nothing
    Named(String),       // carries a String (e.g. the parameter name "b")
    Other,                // carries nothing
}
```

You build/read them like this:

```rust
let receiver = CallReceiver::Named("warehouse".to_owned());

match receiver {
    CallReceiver::SelfOrImplicit => /* ... */,
    CallReceiver::Named(name) => println!("receiver is {name}"),
    CallReceiver::Other => /* ... */,
}
```

`match` must handle every variant — the compiler refuses to build if you
forget one. This is why adding a new enum variant here is safe: every
`match` on it lights up as a compile error until you've updated it.

**C# equivalent:** `MethodCall` is an ordinary C# class/`record` with three
properties. `CallReceiver` is *not* like a C# `enum` (which is just named
integers) — the nearest C# shape is a small class hierarchy or a `record`
with a discriminator, e.g. `abstract record CallReceiver;
record Named(string Name) : CallReceiver;`, matched with a C# `switch`
expression instead of Rust's `match`. C#'s `switch` isn't exhaustiveness-
checked by default the way Rust's `match` is, so this compile-time safety
net is genuinely Rust-specific here.

### 5.3 `Option<T>` and `Result<T, E>` — no null, no silent failure

Rust has no `null`. "Maybe a value" is spelled `Option<T>`:

```rust
pub struct ParameterIR {
    pub name: String,
    ...
}

// "the method might not have a body" (abstract methods don't)
let body: Option<Node> = decl_node.child_by_field_name("body");

if let Some(body) = body {
    // body is a Node here, guaranteed non-null
}
```

"This might fail, and here's why" is `Result<T, E>`:

```rust
pub fn analyze_path(root: &Path) -> Result<AnalysisReport, AnalyzeError> {
    let discovery = discover_csharp(root, &DiscoveryOptions::default())?;
    // ...
    Ok(AnalysisReport { /* ... */ })
}
```

The `?` at the end of a line means "if this was an error, stop this whole
function and return that error immediately; otherwise, unwrap the success
value and keep going." It's how errors propagate without exceptions.

**C# equivalent:** `Option<Node>` is like C#'s nullable reference types
(`Node?`) — but the compiler *forces* you to check it (`if let Some(...)`)
before use, where C#'s null-checking is only a warning you can ignore.
`Result<T, E>` replaces what C# would do with a thrown exception or an
`out bool success` pattern; `?` is like C#'s null-conditional `?.` in
spirit (short-circuit and propagate) but for errors instead of nulls, with
no `try`/`catch` anywhere in this codebase's own logic.

### 5.4 Ownership and borrowing (the famous part)

Every value has exactly one **owner**. When the owner goes out of scope,
Rust frees the value automatically — no garbage collector, no manual
`free()`. You can temporarily **borrow** a value with `&` (read-only) or
`&mut` (exclusive, allows editing) instead of taking ownership:

```rust
// takes ownership — the caller can't use `project` again after this
fn resolve_project(mut project: ProjectIR, index: &DeclarationIndex) -> ProjectIR { ... }

// borrows read-only — the caller keeps `project`, this fn just looks at it
fn build_graph(project: &ProjectIR) -> DependencyGraph { ... }

// borrows mutably — allowed to edit fields in place, caller gets it back after
for method in &mut project.methods {
    method.return_type = resolve_type_ref(method.return_type.clone(), index);
}
```

The rule the compiler enforces: **at any moment, either one `&mut`
borrow, or any number of `&` borrows — never both at once.** This is what
prevents a whole category of bugs (mutating something while someone else
is reading it) at compile time instead of at 3am in production.

**C# equivalent:** there isn't a direct one — C# objects are
garbage-collected references with no compile-time ownership tracking. The
closest surface-level match is `ref`/`in`/`out` parameters (`in` reads like
`&`, `ref`/`out` like `&mut`), but C# never stops two references from
mutating and reading the same object at once; Rust's borrow checker is
what actually prevents that class of bug, and it has no C# equivalent.

### 5.5 Generics — one function, many types

`Resolution<T>` is generic: it works for a resolved `TypeId`, a resolved
`MethodId`, or anything else, without rewriting it each time:

```rust
pub enum Resolution<T> {
    Resolved(T),
    Unresolved(UnresolvedReference),
}

// used three different ways in this codebase:
let a: Resolution<TypeId> = ...;
let b: Resolution<MethodId> = ...;
let c: Resolution<MemberTarget> = ...;
```

A generic function works the same way — `resolve_type_ref` doesn't care
*which* type `T` is, only that it can plug it into `Resolution<T>`:

```rust
fn mark_out_of_scope<T>(current: Resolution<T>, reason: &str) -> Resolution<T> {
    match current {
        Resolution::Resolved(_) => current,
        Resolution::Unresolved(reference) => Resolution::Unresolved(UnresolvedReference {
            reason: reason.into(),
            ..reference   // "copy every other field from `reference` unchanged"
        }),
    }
}
```

**C# equivalent:** this is C# generics, almost unchanged —
`Resolution<TypeId>` reads exactly like `Resolution<TypeId>` would in C#,
and `mark_out_of_scope<T>` is a generic method the same way
`MarkOutOfScope<T>(...)` would be. The `..reference` spread has no direct
C# equivalent; the closest is a C# 9+ `with` expression on a `record`
(`reference with { Reason = reason }`).

### 5.6 Traits — shared behavior across different types

A `trait` is like an interface: it says "anything implementing this trait
must provide these methods." Every rule in `scent-rules` implements the
same `Rule` trait, so the engine can run all of them the same way without
knowing which specific rule it's looking at:

```rust
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding>;
}

pub struct LongMethod;
impl Rule for LongMethod {
    fn id(&self) -> &'static str { "LONG_METHOD" }
    fn evaluate(&self, ctx: &AnalysisContext) -> Vec<Finding> { /* ... */ }
}

// stored as a list of "anything that implements Rule"
let rules: Vec<Box<dyn Rule>> = vec![Box::new(LongMethod), Box::new(LargeClass)];
for rule in &rules {
    rule.evaluate(&ctx);   // works no matter which concrete rule it is
}
```

**C# equivalent:** almost identical to a C# `interface`. `trait Rule` reads
like `interface IRule { string Id(); List<Finding> Evaluate(AnalysisContext ctx); }`,
`impl Rule for LongMethod` reads like `class LongMethod : IRule { ... }`,
and `Vec<Box<dyn Rule>>` is a `List<IRule>` — same polymorphism, same idea
of "call the interface method, let the concrete type decide what happens."

### 5.7 Closures and function pointers

A closure is an inline, unnamed function that can capture variables from
around it. `scent-rules` uses plain function pointers (`fn(f64, f64) ->
String`, no capturing) for explanation text, so each rule can supply its
own wording without the shared evidence code needing to know it in
advance:

```rust
pub struct Observation {
    pub explain: fn(f64, f64) -> String,   // a function pointer field
}

Observation {
    metric_key: "loc",
    explain: |value, threshold| format!("{value} lines (threshold {threshold})"),
    ...
}
```

**C# equivalent:** `fn(f64, f64) -> String` is a C# delegate type, roughly
`Func<double, double, string>`; the `|value, threshold| format!(...)`
closure is a C# lambda, `(value, threshold) => $"{value} lines..."`. The
one difference worth knowing: this field is a plain function pointer (no
captured variables allowed), whereas a C# lambda assigned to a `Func<...>`
is always allowed to capture outer variables — Rust has a separate,
stricter type (`fn`) for the no-capture case that C# doesn't distinguish.

### 5.8 Iterators — the `.iter().filter().map().collect()` chain

Loops in this codebase are mostly written as iterator chains, which read
left-to-right as a pipeline:

```rust
let primitive_count = method
    .parameters
    .iter()                                   // look at each parameter
    .filter(|parameter| is_primitive(&parameter.type_reference))  // keep only primitives
    .count();                                  // how many are left?
```

Common ones you'll see everywhere: `.iter()` (borrow each item),
`.map(...)` (transform each item), `.filter(...)` (keep matching items),
`.collect()` (turn the chain back into a `Vec`/`HashMap`/etc.), `.find(...)`
(first match or `None`), `.any(...)` (`true` if any item matches).

**C# equivalent:** this is LINQ, just spelled differently. The example
above reads in C# as
`method.Parameters.Where(p => IsPrimitive(p.TypeReference)).Count()`.
`.iter()` has no real C# equivalent (C# collections are enumerable
directly); `.map()` is `.Select()`; `.filter()` is `.Where()`; `.collect()`
is `.ToList()`/`.ToArray()`/`.ToDictionary()` depending on what you collect
into; `.find()` is `.FirstOrDefault()`; `.any()` is `.Any()`.

### 5.9 Common syntax quick-reference

| Syntax | Meaning | Nearest C# |
|---|---|---|
| `let x = 5;` | immutable binding (can't reassign `x`) | `const int x = 5;` (or just `readonly` for the intent, not a real ban on reassignment) |
| `let mut x = 5;` | mutable binding | `int x = 5;` (an ordinary C# local, mutable by default) |
| `&x` | borrow `x` read-only | closest: an `in` parameter |
| `&mut x` | borrow `x` for editing | closest: a `ref` parameter |
| `x.clone()` | make an independent copy (needed a lot with owned `String`/`Vec`) | rarely needed in C# — reference types alias freely under the GC |
| `impl Foo { ... }` | where you write methods for struct/enum `Foo` | the body of `class`/`struct Foo { ... }` itself |
| `Self` | shorthand for "the type this `impl` block is for" | no exact match; closest is returning `Foo` from inside `class Foo` |
| `pub` | visible outside the current module | `public` |
| `pub(crate)` | visible anywhere in this crate, not outside it | `internal` |
| `#[derive(Debug, Clone, ...)]` | auto-generate common trait implementations | closest: a source generator, or manually implementing `ToString`/`ICloneable` |
| `if let Some(x) = maybe_value { ... }` | "if there is a value, bind it to `x` and run this" | `if (maybeValue is { } x) { ... }` |
| `let Some(x) = maybe_value else { return; };` | "unwrap or bail out" in one line | `if (maybeValue is not { } x) return;` |
| `matches!(value, Pattern::A \| Pattern::B)` | quick "is it one of these shapes?" check, returns `bool` | `value is Pattern.A or Pattern.B` (C# 9+ pattern combinators) |
| `format!("{x}")` | build a `String` (variable name directly inside `{}`) | `$"{x}"` (C# interpolated string) |
| `vec![a, b, c]` | build a `Vec` (growable array) with initial values | `new List<T> { a, b, c }` |

---

## 6. Why the design is what it is

- **Determinism above convenience.** Every ID is a content hash, every
  collection gets sorted before being written out, and nothing about
  runtime (timestamps, machine paths) leaks into an ID or a report. Run it
  twice, get the same bytes. This is what makes baselines and CI gates
  trustworthy instead of flaky.
- **Never guess.** If SCENT can't be sure what a name refers to, it says
  so explicitly (`Unresolved`, with a reason) instead of picking the most
  likely answer. A whole real bug this session came from briefly
  violating this rule (resolving every call against the enclosing type
  regardless of receiver) — fixed by adding the `CallReceiver` fact so the
  resolver only acts when it's actually justified.
- **Evidence over thresholds.** No rule says "LOC > 30 → bad." Every rule
  combines several weighted, normalized observations into one confidence
  number, so a method has to be bad on more than one axis to be flagged
  strongly.
- **Facts vs. metrics vs. findings, kept in separate crates.** Described
  in §4 — this is enforced by the compiler, not just convention.

---

## 7. How to extend it

### Add a new rule

1. Create `crates/scent-rules/src/rules/my_rule.rs`.
2. Define a zero-sized struct and `impl Rule for MyRule`, following an
   existing rule (`long_parameter_list.rs` is the simplest) as a template.
3. Build an `[Observation; N]` array, call `build_evidence(&observations)`,
   and only push a `Finding` if `confidence >= your_minimum`.
4. Register it in `rules/mod.rs` and `default_registry()` in `lib.rs`.
5. Add tests in `crates/scent-rules/tests/rules.rs` — a positive case, a
   negative case, and (if relevant) a suppressed case.

### Add a new metric

1. Add a variant to `MetricKind` in `crates/scent-metrics/src/lib.rs`.
2. Write a `calculate_xxx(project, store)` function next to the existing
   ones and call it from `build_metric_store`.
3. Add a test in `crates/scent-metrics/tests/metrics.rs`.

### Run the full check before calling anything "done"

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

---

## 8. Where things live (quick file map)

```text
scent/
├── crates/
│   ├── scent-domain/   IDs, paths, locations, shared enums
│   ├── scent-ir/       ProjectIR and friends (the facts)
│   ├── scent-parser/   discovery, Tree-sitter, extraction, resolution
│   ├── scent-graph/    DependencyGraph
│   ├── scent-metrics/  MetricStore + calculators
│   ├── scent-rules/    Rule trait, evidence engine, the 10 rules
│   ├── scent-report/   JSON/SARIF writer, baseline, quality gate
│   ├── scent-core/     pipeline.rs (orchestration), report.rs (JSON)
│   └── scent-cli/      main.rs (argv → pipeline → printed output)
├── docs/
│   ├── LEARNING_GUIDE.md        this file
│   ├── architecture.md          data-flow contract, layer ownership
│   ├── status.md                what's implemented vs. planned, right now
│   └── contributor-guide.md     workflow checklist for making changes
└── smell_detector.toml          example rule-threshold config (not wired up yet)
```

When in doubt about current status ("is X actually done?"), `docs/status.md`
is the source of truth — it's updated every time a milestone lands.
