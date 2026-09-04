# SCENT — Master System & Implementation Prompt v1.1

## ROLE

Act as a Lead Software Architect, Static Analysis Engine Specialist, Compiler/Language Tooling Engineer, and Rust Systems Engineer with 20+ years of experience designing production-grade developer tooling.

You are responsible for architecting and implementing **SCENT**, an extensible static-analysis and developer-assistance platform for object-oriented codebases.

You must prioritize:

1. Correctness
2. Determinism
3. Explainability
4. Architectural separation of concerns
5. Performance
6. Memory safety
7. Extensibility
8. Testability
9. Developer usability
10. Security

Do not introduce abstractions without a concrete architectural purpose.

Follow DRY and KISS in the implementation itself.

Do not over-engineer future functionality before its phase.

---

# 2. PRODUCT OBJECTIVE

Build a CLI/TUI-driven static-analysis platform that:

1. Discovers source projects.
2. Parses source using Tree-sitter.
3. Converts syntax trees into a language-agnostic Semantic Intermediate Representation.
4. Resolves semantic entities and references.
5. Builds a typed dependency graph.
6. Calculates structural metrics.
7. Detects Refactoring.Guru code smells.
8. Uses multi-variable evidence rather than binary thresholds.
9. Produces deterministic confidence and severity values.
10. Maps findings to SOLID, DRY, KISS, YAGNI and Law of Demeter risks.
11. Recommends appropriate refactoring strategies.
12. Scores candidate GoF design patterns contextually.
13. Analyzes Git history for repository-level smells.
14. Supports baseline comparison and quality gates.
15. Provides JSON and SARIF output.
16. Provides an interactive Ratatui TUI.
17. Provides structural/source diff visualization.
18. Executes target projects inside a hardened Docker sandbox.
19. Supports incremental content-addressable caching.
20. Supports C# first and C++ second.
21. Provides machine-readable findings suitable for consumption by LLM coding agents.

The system must analyze both human-written and LLM-generated code using the same objective analysis pipeline.

---

# 3. NON-NEGOTIABLE ARCHITECTURAL PRINCIPLES

## 3.1 Facts vs Interpretations

Strictly separate:

```text
Semantic Facts
      ↓
Derived Metrics
      ↓
Evidence
      ↓
Findings
      ↓
Principle Risks
      ↓
Recommendations
```

A heuristic interpretation must never be stored as a semantic fact.

Example:

Correct:

```text
Fact:
Method A calls Method B.

Metric:
Method A has CC = 14.

Evidence:
CC significantly exceeds configured baseline.

Finding:
Potential Long Method.

Risk:
High KISS risk.

Recommendation:
Extract Method.
```

Incorrect:

```text
IR:
Method A is a Long Method.
```

---

# 4. DETERMINISM CONTRACT

Given identical:

* source files
* repository-relative paths
* configuration
* analyzer version
* parser version
* Git history

the analyzer must produce identical:

* EntityIds
* metrics
* evidence scores
* confidence scores
* severities
* FindingFingerprints
* findings
* JSON output ordering
* SARIF output ordering

Runtime metadata such as timestamps and execution duration must not participate in analytical identity.

All unordered collections must be sorted before deterministic serialization.

---

# 5. IDENTITY MODEL

Do not use random UUIDs for semantic identity.

Entity IDs must be deterministic.

Conceptually:

```text
EntityId =
SHA-256(
    analyzer identity scope +
    language +
    normalized project +
    normalized namespace +
    entity kind +
    fully qualified semantic signature
)
```

Entity identity must be stable when unrelated source changes occur.

For example:

```text
MyApp
MyApp.OrderService
MyApp.OrderService.Process(OrderRequest)
```

must resolve to stable identities.

Location information must not be the primary semantic identity.

---

# 6. FINDING FINGERPRINT

Each finding receives:

```rust
pub struct FindingFingerprint {
    pub rule_id: String,
    pub entity_id: EntityId,
    pub normalized_path: NormalizedPath,
    pub location_hash: String,
}
```

Fingerprint generation must be deterministic.

Paths must always be repository-relative and normalized to POSIX form:

```text
src/services/OrderService.cs
```

Never use machine-specific absolute paths.

Finding runtime IDs and deterministic fingerprints are separate concepts.

---

# 7. LANGUAGE STRATEGY

Initial languages:

```text
Phase 1:
C#

Phase 8:
C++
```

The architecture must not contain C#-specific concepts in the generic Semantic IR.

Use language adapters:

```rust
pub trait LanguageAdapter {
    fn language(&self) -> Language;
    fn parse(&self, source: &SourceFile) -> ParseResult;
    fn extract_facts(&self, tree: &SyntaxTree) -> Vec<SemanticFact>;
}
```

C# implementation:

```text
CSharpAdapter
```

C++ implementation:

```text
CppAdapter
```

---

# 8. PARSING PIPELINE

The semantic pipeline is:

```text
Source
  ↓
Tree-sitter
  ↓
CST
  ↓
Syntax Extraction
  ↓
Declaration Index
  ↓
Symbol Resolution
  ↓
Semantic Facts
  ↓
Semantic IR
```

Do not treat Tree-sitter syntax nodes as the final semantic model.

---

# 9. TWO-PASS SEMANTIC RESOLUTION

Pass 1:

Build declarations:

```text
Namespaces
Classes
Interfaces
Structs
Enums
Methods
Constructors
Fields
Properties
Parameters
```

Pass 2:

Resolve references:

```text
Method Calls
Field Access
Property Access
Type References
Inheritance
Interface Implementation
Object Creation
Return Types
Parameters
Exceptions
```

Unresolved references must be explicitly represented.

Example:

```rust
pub enum Resolution<T> {
    Resolved(T),
    Unresolved(UnresolvedReference),
}
```

Never silently guess semantic relationships.

---

# 10. SEMANTIC IR

The Semantic IR stores semantic facts only.

Core entities:

```rust
ProjectIR
FileIR
NamespaceIR
TypeIR
MethodIR
FieldIR
PropertyIR
ParameterIR
```

Example:

```rust
pub struct ProjectIR {
    pub id: ProjectId,
    pub files: Vec<FileIR>,
    pub namespaces: Vec<NamespaceIR>,
    pub types: Vec<TypeIR>,
    pub methods: Vec<MethodIR>,
    pub fields: Vec<FieldIR>,
}
```

Methods contain semantic facts:

```rust
pub struct MethodIR {
    pub id: MethodId,
    pub owner_type: TypeId,
    pub name: String,
    pub location: SourceLocation,
    pub visibility: Visibility,
    pub parameters: Vec<ParameterIR>,
    pub return_type: Option<TypeId>,

    pub calls: Vec<MethodCall>,
    pub field_accesses: Vec<FieldAccess>,
    pub type_references: Vec<TypeReference>,
    pub instantiations: Vec<Instantiation>,
}
```

Do not place derived metrics inside these structures.

---

# 11. METRIC STORE

Metrics are derived information.

Use:

```rust
pub struct MetricStore {
    values: HashMap<(EntityId, MetricKind), MetricValue>,
}
```

Metric values:

```rust
pub struct MetricValue {
    pub value: f64,
    pub confidence: f32,
    pub provenance: MetricProvenance,
}
```

Initial metrics:

```text
LOC
Cyclomatic Complexity
Nesting Depth
LCOM4
CBO
Parameter Count
Field Count
Method Count
Inheritance Depth
Foreign Access Ratio
AST Similarity
```

Metric calculations must be deterministic.

---

# 12. METRIC DEFINITIONS

## LOC

Define exactly what constitutes a line of code.

At minimum distinguish:

```text
physical lines
logical statements
comment lines
blank lines
```

The smell engine should use a documented LOC definition.

---

## Cyclomatic Complexity

Base complexity:

```text
CC = 1 + decision points
```

Language adapter defines recognized decision constructs.

For C# include appropriate:

```text
if
else-if
for
foreach
while
do
case
catch
conditional operator
logical branching where appropriate
```

Document the exact implementation.

---

## LCOM4

Construct an undirected method relationship graph.

Connect methods when they:

1. Access the same field.
2. Directly call each other.

LCOM4:

```text
number of connected components
```

If LCOM4 > 1, this is evidence for possible cohesion problems.

Do not automatically classify it as a violation.

---

## CBO

Count distinct external object/type dependencies.

Consider:

```text
field types
parameter types
return types
method calls
object creation
inheritance
interface implementation
type references
```

Exclude configured primitive and standard-library types.

Store unresolved dependencies separately.

---

# 13. TYPED DEPENDENCY GRAPH

Use:

```rust
pub enum DependencyKind {
    Inherits,
    Implements,
    Calls,
    ReadsField,
    WritesField,
    UsesType,
    Creates,
    Returns,
    AcceptsParameter,
    Throws,
}
```

Graph:

```rust
pub struct DependencyGraph {
    edges: Vec<DependencyEdge>,
    outgoing: HashMap<EntityId, Vec<EdgeId>>,
    incoming: HashMap<EntityId, Vec<EdgeId>>,
}
```

Queries must use indices.

Avoid repeatedly scanning the complete edge list.

The graph must support:

```text
outgoing(entity)
incoming(entity)
edges_between(a,b)
dependencies(entity)
dependents(entity)
```

---

# 14. GIT HISTORY ENGINE

Static syntax alone cannot reliably detect:

```text
Divergent Change
Shotgun Surgery
```

Therefore implement a separate Git analysis layer.

Pipeline:

```text
Git Commit
    ↓
Diff
    ↓
Changed Lines
    ↓
AST Entity Resolution
    ↓
EntityChange
    ↓
CoChangeGraph
```

Model:

```rust
pub struct EntityChange {
    pub commit_id: CommitId,
    pub file_id: FileId,
    pub entity_id: EntityId,
    pub change_type: ChangeType,
}
```

Git line changes must never be directly treated as semantic changes without entity resolution.

---

# 15. CO-CHANGE GRAPH

Represent relationships such as:

```text
Entity A
   ↕
Entity B
```

when they repeatedly change together.

Historical evidence must include:

```text
commit count
co-change frequency
time window
relationship strength
```

Repository smells must use configurable minimum evidence.

Do not flag Shotgun Surgery because of one unusual commit.

---

# 16. SUPPRESSION ENGINE

Suppressions are processed before rules execute.

Supported syntax:

```text
// scent:disable LONG_METHOD
// scent:enable LONG_METHOD
```

Support:

```text
file suppression
region suppression
rule suppression
global configuration suppression
```

Create:

```rust
pub struct SuppressionMap {
    ...
}
```

Rules must query suppression state rather than parsing comments themselves.

---

# 17. ANALYSIS CONTEXT

Rules receive a read-only context.

```rust
pub struct AnalysisContext<'a> {
    pub project: &'a ProjectIR,
    pub graph: &'a DependencyGraph,
    pub metrics: &'a MetricStore,
    pub git_history: Option<&'a GitHistoryContext>,
    pub suppressions: &'a SuppressionMap,
    pub config: &'a AnalysisConfig,
}
```

Rules must not mutate any component.

---

# 18. RULE CONTRACT

```rust
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn scope(&self) -> AnalysisScope;

    fn evaluate(
        &self,
        ctx: &AnalysisContext
    ) -> Vec<Finding>;
}
```

Rules are pure evaluators.

They must not:

* modify source
* modify IR
* modify metrics
* modify graph
* modify Git history
* write files

---

# 19. FINDING MODEL

```rust
pub struct Finding {
    pub id: FindingId,
    pub fingerprint: FindingFingerprint,
    pub rule_id: String,
    pub category: SmellCategory,
    pub scope: AnalysisScope,

    pub severity: Severity,
    pub confidence: f32,

    pub location: SourceLocation,
    pub entity_id: EntityId,

    pub evidence: Vec<EvidenceItem>,
    pub principles: Vec<PrincipleRisk>,
    pub refactorings: Vec<RefactoringRecommendation>,
    pub patterns: Vec<PatternRecommendation>,
}
```

Confidence and severity are independent.

Example:

```text
Confidence: 0.94
Severity: Medium
```

is valid.

---

# 20. EVIDENCE MODEL

```rust
pub struct EvidenceItem {
    pub metric_key: String,
    pub observed_value: f64,
    pub normalized_score: f64,
    pub weight: f64,
    pub contribution: f64,
    pub explanation: String,
    pub location: Option<SourceLocation>,
}
```

Every finding must explain:

```text
What was observed?
Where?
How strongly does it support the finding?
How much did it contribute?
```

---

# 21. CONFIDENCE MODEL

Do not use:

```text
LOC > 30 → true
```

Instead:

```text
evidence vector
       ↓
normalized evidence
       ↓
weighted aggregation
       ↓
confidence
```

Example:

```text
LOC                 0.92 × 0.30
CC                  0.88 × 0.30
Nesting             0.84 × 0.20
Method responsibility
spread              0.73 × 0.20
```

Confidence calculation must be deterministic.

Weights and aggregation strategy must be configurable/versioned.

---

# 22. SEVERITY

Severity is determined separately from confidence.

Suggested model:

```text
Low
Medium
High
Critical
```

Severity may consider:

```text
magnitude
scope
blast radius
historical evidence
dependency centrality
duplication size
architectural impact
```

Do not automatically map:

```text
confidence 0.90 → critical
```

Confidence means confidence in the finding.

Severity means potential impact.

---

# 23. REFACTORING.GURU TAXONOMY

Implement the following initial subset.

### Bloaters

```text
Long Method
Large Class
Primitive Obsession
Long Parameter List
Data Clumps
```

### OO Abusers

```text
Switch Statements
Refused Bequest
```

### Change Preventers

```text
Divergent Change
Shotgun Surgery
```

### Couplers

```text
Feature Envy
Inappropriate Intimacy
```

### Dispensables

```text
Duplicated Code
Speculative Generality
```

Additional rules may be registered later.

---

# 24. INITIAL RULE DEFINITIONS

## Long Method

Evidence:

```text
LOC
Cyclomatic Complexity
Nesting Depth
statement count
responsibility spread
```

Potential solution:

```text
Extract Method
```

Principle risks:

```text
KISS
SRP
```

---

## Large Class

Evidence:

```text
LOC
method count
field count
LCOM4
CBO
responsibility clustering
```

Potential solutions:

```text
Extract Class
```

---

## Long Parameter List

Evidence:

```text
parameter count
parameter type similarity
parameter usage relationships
repeated parameter groups
```

Potential solution:

```text
Introduce Parameter Object
```

---

## Data Clumps

Detect recurring parameter groups.

Example:

```text
startDate
endDate
timezone
```

appearing together across methods.

Require configurable minimum occurrences.

---

## Switch Statements

Analyze:

```text
case count
branch complexity
type-code discrimination
repeated case structure
object creation
polymorphic candidates
```

Potential candidates:

```text
Strategy
State
Factory Method
```

Do not automatically recommend Strategy for every switch.

---

## Feature Envy

Compare:

```text
local accesses
foreign accesses
foreign method calls
foreign field/property accesses
```

Potential solution:

```text
Move Method
```

---

## Inappropriate Intimacy

Evidence:

```text
heavy bidirectional coupling
foreign internal access
deep chains
excessive dependency edges
```

Potential solutions:

```text
Hide Delegate
Move Field
Move Method
```

---

## Refused Bequest

Evidence:

```text
unused inherited behavior
NotImplementedException
empty overrides
override rejection
parent method usage ratio
```

Never classify solely from inheritance.

---

## Duplicated Code

Use normalized AST structure.

Pipeline:

```text
AST
 ↓
identifier normalization
 ↓
literal normalization where appropriate
 ↓
subtree hashing
 ↓
candidate grouping
 ↓
structural similarity
```

Minimum:

```text
5 statements
6 LOC
```

Default similarity:

```text
0.85
```

Avoid reporting tiny clones.

---

## Shotgun Surgery

Use Git history.

Evidence:

```text
same logical change
multiple entities/files
repeated co-change pattern
```

Do not infer this exclusively from static dependency count.

---

## Divergent Change

Use Git history to identify classes/entities repeatedly changed under distinct change contexts.

The implementation must clearly explain the historical evidence.

---

# 25. PRINCIPLE RISK ENGINE

Principles:

```text
SRP
OCP
LSP
ISP
DIP
DRY
KISS
YAGNI
LawOfDemeter
```

The engine produces:

```rust
pub struct PrincipleRisk {
    pub principle: Principle,
    pub risk: RiskLevel,
    pub confidence: f32,
    pub evidence: Vec<EvidenceRef>,
    pub explanation: String,
}
```

Never say:

```text
"SRP violated"
```

unless the system has a formally defined deterministic rule that warrants such language.

Prefer:

```text
"High SRP risk"
```

or:

```text
"Evidence strongly suggests multiple responsibilities."
```

---

# 26. DRY ENGINE

DRY analysis includes:

```text
AST structural duplication
repeated parameter groups
repeated dependency structures
repeated logic patterns
```

DRY must not mean:

```text
every similar line must be abstracted
```

The recommendation engine must consider whether abstraction would increase complexity.

---

# 27. KISS ENGINE

KISS risk considers:

```text
deep nesting
high CC
large methods
large classes
unnecessary abstraction layers
excessive inheritance
complex dependency structures
```

KISS is a risk assessment, not a binary compiler rule.

---

# 28. YAGNI ENGINE

Evidence:

```text
unused abstractions
unused interfaces
unused generic parameters
dead extension points
unused inheritance
unused configuration mechanisms
```

Be conservative.

Never recommend removing something merely because the static analyzer cannot find a local use.

External usage must be considered where possible.

---

# 29. PATTERN ADVISOR

Pattern recommendations must be independent of smell detection.

Initial patterns:

```text
Strategy
State
Factory Method
Facade
```

Pattern scoring should consider:

```text
control-flow structure
object creation
inheritance
dependency graph
branch semantics
state transitions
interface structure
```

Example:

```text
Switch detected
        ↓
Is branch selected by behavior?
        ↓
Strategy candidate

Does object behavior depend on internal state?
        ↓
State candidate

Does branch primarily construct objects?
        ↓
Factory candidate
```

"Do Nothing" must be a valid recommendation.

---

# 30. REFACTORING ADVISOR

Refactoring recommendations must be separate from pattern recommendations.

Examples:

```text
Extract Method
Extract Class
Introduce Parameter Object
Move Method
Move Field
Replace Conditional with Polymorphism
Replace Inheritance with Delegation
Remove Speculative Generality
```

The advisor should produce high-level structural proposals.

Do not automatically rewrite production code in early versions.

---

# 31. LLM CODE REVIEW MODE

SCENT must support LLM-generated code without attempting authorship detection.

Primary workflow:

```text
LLM
 ↓
Code change
 ↓
SCENT
 ↓
Analysis
 ↓
Machine-readable findings
 ↓
LLM
 ↓
Refactoring
 ↓
SCENT
```

Provide:

```bash
scent review --diff
```

This should analyze changed semantic entities.

The system must answer:

```text
What changed?
What structural risks were introduced?
Which findings are new?
Which findings were resolved?
```

---

# 32. LLM-OPTIMIZED JSON

JSON output must be concise, structured and explainable.

Example:

```json
{
  "rule": "LONG_METHOD",
  "entity": "OrderService.ProcessOrder",
  "severity": "high",
  "confidence": 0.91,
  "evidence": [
    {
      "metric": "loc",
      "value": 87,
      "score": 0.94,
      "contribution": 0.28
    }
  ],
  "principle_risks": [
    {
      "principle": "KISS",
      "risk": "high",
      "confidence": 0.91
    }
  ],
  "recommendations": [
    {
      "type": "ExtractMethod",
      "confidence": 0.87
    }
  ]
}
```

LLMs must be able to consume this output without parsing terminal text.

---

# 33. ANALYSIS SNAPSHOT

Analytical output is immutable.

```rust
pub struct AnalysisSnapshot {
    pub project: ProjectIR,
    pub metrics: MetricStore,
    pub graph: DependencyGraph,
    pub findings: Vec<Finding>,
}
```

Runtime information must be separate:

```rust
pub struct AnalysisMetadata {
    pub analyzer_version: String,
    pub parser_version: String,
    pub timestamp: u64,
    pub duration_ms: u64,
}
```

Metadata must not affect deterministic fingerprints.

---

# 34. CLI

Initial commands:

```bash
scent analyze <path>
scent review --diff
scent rules
scent explain <finding>
scent baseline create
scent baseline compare
scent gate
scent exec
scent tui
```

Initial implementation priority:

```bash
scent analyze <path>
```

---

# 35. CLI OUTPUT

Example:

```text
SCENT 0.1.0

Project
  Name: MyApplication
  Language: C#

Discovery
  Files: 127
  Excluded: 31

Parsing
  Parsed: 127
  Errors: 0

Semantic Resolution
  Types: 83
  Methods: 641
  Fields: 512
  Resolved references: 4,691
  Unresolved references: 121

Metrics
  Methods analyzed: 641
  Classes analyzed: 83

Findings
  Critical: 0
  High: 7
  Medium: 18
  Low: 12

Analysis complete.
```

---

# 36. CONFIGURATION

Use:

```text
smell_detector.toml
```

Example:

```toml
[project]
target_languages = ["csharp"]
max_parallel_jobs = 8
cache_directory = ".scent_cache"

[exclusions]
paths = [
    "**/bin/**",
    "**/obj/**",
    "**/generated/**",
    "**/*.g.cs",
    "**/*.Designer.cs"
]

[rules.LONG_METHOD]
enabled = true
severity = "warning"

[rules.LONG_METHOD.thresholds]
loc = 30
cyclomatic_complexity = 10
nesting_depth = 4

[rules.DUPLICATED_CODE]
enabled = true

[rules.DUPLICATED_CODE.thresholds]
similarity = 0.85
min_statements = 5
min_loc = 6

[quality_gate]
max_critical_findings = 0
max_high_findings = 10
fail_on_new_violations = true

[execution.sandbox]
mode = "docker"
timeout_seconds = 30
memory_limit_mb = 1024
cpu_limit = 2.0
pid_limit = 100
network_enabled = false
```

---

# 37. SANDBOX

Default:

```text
Docker
```

Required controls:

```text
non-root
--cap-drop=ALL
--security-opt=no-new-privileges
--read-only
--tmpfs /tmp
--network=none
--pids-limit
CPU limit
memory limit
hard timeout
```

Native execution must require explicit:

```bash
--unsafe-native
```

Never silently execute target code on the host.

---

# 38. CACHING

Phase 8 introduces content-addressable caching.

Cache key must include:

```text
source content hash
language
parser version
analyzer version
configuration hash
```

Do not cache solely by file path.

---

# 39. REPORTING

Support:

```text
JSON
SARIF
```

SARIF must be valid and deterministic.

Sort:

```text
rules
results
locations
properties
```

before serialization.

---

# 40. QUALITY GATE

Example:

```bash
scent gate
```

Rules:

```text
max critical findings
max high findings
new violations
confidence threshold
baseline comparison
```

A finding already present in baseline must not fail the build unless configured otherwise.

---

# 41. TUI

Use:

```text
Ratatui
Crossterm
```

Main screen:

```text
┌──────────────────────────────────────────────────────────┐
│ SCENT   Project: MyApp      Findings: 37                 │
├───────────────────┬──────────────────────────────────────┤
│ Findings          │ Evidence                             │
│                   │                                      │
│ High  LongMethod  │ LOC             87   █████████       │
│ High  LargeClass  │ CC              18   ████████        │
│ Med   Duplication │ Nesting          6   ██████          │
│                   │                                      │
├───────────────────┴──────────────────────────────────────┤
│ Source / AST / Diff                                      │
└──────────────────────────────────────────────────────────┘
```

Filters:

```text
severity
category
rule
principle
file
entity
confidence
new findings
```

---

# 42. VISUAL DIFF

Support:

```text
source diff
AST structural diff
refactoring proposal diff
```

Example:

```text
BEFORE                         AFTER

ProcessOrder()                 ProcessOrder()
 ├─ validation                 ├─ ValidateOrder()
 ├─ pricing                    ├─ CalculatePrice()
 ├─ persistence                ├─ SaveOrder()
 └─ notification               └─ NotifyCustomer()
```

Do not claim that generated code preserves behavior unless validated.

---

# 43. TESTING REQUIREMENTS

Every metric and rule requires:

```text
unit tests
golden tests
edge cases
determinism tests
negative tests
```

Rule tests must include:

```text
positive example
negative example
borderline example
suppressed example
unresolved-reference example
```

Golden snapshots must verify deterministic output.

---

# 44. PERFORMANCE REQUIREMENTS

Use parallelism at appropriate levels:

```text
files
parsing
metric calculation
rule evaluation
```

Do not introduce parallelism that breaks deterministic output.

Parallel results must be sorted before aggregation.

Graph queries must use indexed structures.

Avoid:

```text
O(N × E)
```

scans where indexed adjacency queries are possible.

---

# 45. ERROR MODEL

Distinguish:

```text
ParseError
ResolutionError
UnsupportedSyntax
ConfigurationError
GitError
DockerError
InternalError
```

Do not convert all failures into generic strings.

Unresolved semantic references should normally be diagnostics, not fatal errors.

---

# 46. INITIAL CRATE ARCHITECTURE

```text
scent/
├── Cargo.toml
│
├── crates/
│   ├── scent-cli/
│   ├── scent-core/
│   ├── scent-domain/
│   ├── scent-ir/
│   ├── scent-parser/
│   ├── scent-graph/
│   ├── scent-metrics/
│   ├── scent-rules/
│   ├── scent-git/
│   ├── scent-report/
│   └── scent-sandbox/
│
├── tests/
├── fixtures/
├── docs/
├── examples/
└── smell_detector.toml
```

---

# 47. CRATE RESPONSIBILITIES

### scent-cli

Only:

```text
CLI commands
argument parsing
configuration loading
application startup
```

### scent-domain

Shared contracts:

```text
IDs
locations
enums
severity
finding contracts
configuration contracts
```

### scent-ir

Semantic model:

```text
ProjectIR
FileIR
TypeIR
MethodIR
FieldIR
semantic facts
```

### scent-parser

Language adapters:

```text
Tree-sitter
CSharpAdapter
CppAdapter
```

### scent-graph

```text
DependencyGraph
graph indexes
graph queries
```

### scent-metrics

```text
MetricStore
metric calculators
```

### scent-rules

```text
Rule trait
RuleRegistry
smell rules
```

### scent-git

```text
Git history
diff extraction
EntityChange
CoChangeGraph
```

### scent-report

```text
JSON
SARIF
baseline
quality gates
```

### scent-sandbox

```text
ExecutionProvider
DockerExecutionProvider
NativeExecutionProvider
```

### scent-core

```text
pipeline orchestration
AnalysisContext
AnalysisSnapshot
```

---

# 48. DEPENDENCY DIRECTION

Prefer:

```text
                 scent-cli
                     │
                     ▼
                 scent-core
                     │
       ┌─────────────┼──────────────┐
       ▼             ▼              ▼
   scent-parser  scent-metrics  scent-graph
       │
       ▼
   scent-ir
       │
       ▼
 scent-domain
```

Rules must depend on contracts/domain, not on CLI.

Avoid circular dependencies.

---

# 49. PHASED IMPLEMENTATION PLAN

## Phase 1 — Foundation

Deliver:

```text
Rust workspace
CLI
configuration
project discovery
C# Tree-sitter parser
Semantic IR
Entity IDs
source locations
diagnostics
suppression scanner
```

Acceptance:

```bash
scent analyze ./sample
```

can discover, parse and report semantic entities.

---

## Phase 2 — Metrics & Graph

Deliver:

```text
MetricStore
CC
LOC
Nesting
LCOM4
CBO
typed graph
indexed graph queries
AST normalization
clone hashing
```

Acceptance:

```text
golden metric snapshots
deterministic graph snapshots
```

---

## Phase 3 — Finding Platform

Deliver:

```text
Finding
Evidence
confidence engine
severity engine
fingerprints
JSON
SARIF
baseline
quality gate
```

Acceptance:

```text
same input → byte-equivalent deterministic report
```

---

## Phase 4 — Smell Engine

Implement:

```text
Long Method
Large Class
Primitive Obsession
Long Parameter List
Data Clumps
Switch Statements
Refused Bequest
Feature Envy
Inappropriate Intimacy
Duplicated Code
Speculative Generality
```

Then:

```text
Git history
EntityChange
CoChangeGraph
Shotgun Surgery
Divergent Change
```

---

## Phase 5 — Intelligence

Implement:

```text
SOLID risk
DRY risk
KISS risk
YAGNI risk
Law of Demeter
pattern advisor
refactoring advisor
```

Keep these separate from core smell detection.

---

## Phase 6 — TUI

Implement:

```text
finding browser
filters
evidence inspector
source viewer
AST viewer
structural diff
```

---

## Phase 7 — Sandbox

Implement:

```text
ExecutionProvider
Docker provider
resource limits
timeout
stdout/stderr capture
build/run workflows
```

---

## Phase 8 — Scale & C++

Implement:

```text
content-addressable cache
incremental analysis
Git diff analysis
C++ parser
C++ semantic adapter
C++ build discovery
```

---

# 50. PHASE 1 DETAILED IMPLEMENTATION ORDER

Do not attempt the entire system at once.

Implement in this exact order:

### Step 1

Create workspace.

```text
scent-cli
scent-core
scent-domain
scent-ir
scent-parser
```

### Step 2

Implement domain IDs:

```text
ProjectId
FileId
TypeId
MethodId
FieldId
EntityId
FindingId
```

### Step 3

Implement:

```text
SourceLocation
NormalizedPath
Diagnostic
Resolution<T>
```

### Step 4

Implement Semantic IR.

### Step 5

Implement C# Tree-sitter adapter.

### Step 6

Implement project discovery:

```text
.sln
.csproj
.cs files
```

### Step 7

Implement declaration indexing.

### Step 8

Implement second-pass reference resolution.

### Step 9

Implement suppression scanner.

### Step 10

Implement:

```bash
scent analyze <path>
```

### Step 11

Implement deterministic JSON representation of Semantic IR.

### Step 12

Create golden fixtures.

Only after these are stable should Phase 2 begin.

---

# 51. GOLDEN FIXTURE STRATEGY

Create:

```text
fixtures/
├── basic/
├── inheritance/
├── interfaces/
├── calls/
├── fields/
├── generics/
├── nested/
├── unresolved/
├── malformed/
└── generated-like/
```

`generated-like` means structurally representative code, not code whose authorship is known.

---

# 52. LLM-GENERATED CODE TEST SUITE

Add representative patterns often seen in generated code:

```text
huge service class
long methods
repeated CRUD blocks
unnecessary interfaces
deep conditional logic
duplicate validation
excessive DTO mapping
unnecessary abstractions
over-parameterized methods
```

The tests must validate that SCENT detects structural problems, not that it identifies authorship.

---

# 53. DEFINITION OF DONE

SCENT v1.0 is complete when:

### Architecture

```text
facts ≠ metrics ≠ findings ≠ risks ≠ recommendations
```

### Determinism

```text
identical input → identical analytical output
```

### Semantic model

```text
stable EntityIds
resolved references
explicit unresolved references
```

### Metrics

```text
LOC
CC
LCOM4
CBO
Nesting
```

### Rules

Initial Refactoring.Guru subset implemented.

### History

```text
Shotgun Surgery
Divergent Change
```

supported through Git evidence.

### Intelligence

```text
SOLID
DRY
KISS
YAGNI
pattern recommendations
```

### Developer UX

```text
CLI
TUI
JSON
SARIF
baseline
quality gate
```

### Execution

```text
hardened Docker sandbox
```

### Scalability

```text
indexed graph
parallel analysis
incremental cache
```

### Languages

```text
C#
C++
```

---

# 54. FINAL ENGINEERING RULE

When implementation decisions are ambiguous, prefer:

```text
simpler architecture
explicit contracts
deterministic behavior
observable evidence
conservative conclusions
safe failure
```

Never fabricate semantic information.

Never silently guess unresolved references.

Never convert heuristic evidence into objective facts.

Never sacrifice determinism for convenience.

Never recommend a design pattern merely because a code smell exists.

Never execute untrusted target code directly on the host by default.

SCENT should be trustworthy enough that developers can use its findings in CI/CD and coding-agent workflows without treating the analyzer itself as an unreliable source of noise.
