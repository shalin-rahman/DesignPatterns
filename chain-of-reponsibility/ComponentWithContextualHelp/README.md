# Chain of Responsibility — Contextual UI Help System

> **Source:** [Refactoring Guru — Chain of Responsibility](https://refactoring.guru/design-patterns/chain-of-responsibility)  
> **Category:** Behavioural Design Pattern

---

## What Problem Does It Solve?

When a **help request** (F1 key) is triggered on a UI component, we don't know in advance which component can answer it. Rather than hard-coding "if button, check button; else check dialog; else check panel…", we let the request _travel up the UI hierarchy_ on its own — stopping the moment any component has a relevant answer.

This decouples the **sender** (the focused button) from the **receiver** (whichever handler ultimately responds), and makes it trivial to add or reorder handlers at runtime.

---

## Scenario: GUI Contextual Help

```
Window  ("Getting Started" global guide — always handles if nobody else does)
└── Panel  (optional modal help pop-up)
    └── Dialog  (optional wiki page URL)
        ├── OK Button     (tooltip: "Click OK to submit…")  ← chain stops HERE for Scenario 1
        └── Cancel Button (no tooltip)                      ← passes up to Dialog
```

### Request flow (like a relay race, NOT a broadcast)

```
F1 on Cancel Button
  → Button has no tooltip          → pass up
    → Dialog has a wiki page URL   → HANDLE (chain stops)
```

```
F1 on OK Button
  → Button has tooltip             → HANDLE immediately (chain stops at first link)
```

```
F1 on a Button with no help anywhere
  → Button → Dialog → Panel → Window → HANDLE (global fallback, guaranteed)
```

---

## File Map

| File | Role |
|---|---|
| [`Component.java`](Component.java) | Abstract handler — holds `container` (next handler) reference, default "handle-or-forward" logic |
| [`Container.java`](Container.java) | Intermediate abstract handler — adds child management and auto-wires the chain on `add()` |
| [`Button.java`](Button.java) | Concrete handler (leaf) — first link; uses tooltip text to terminate the chain |
| [`Dialog.java`](Dialog.java) | Concrete handler (mid) — uses wiki page URL to terminate, or forwards |
| [`Panel.java`](Panel.java) | Concrete handler (mid) — uses modal help text to terminate, or forwards |
| [`Window.java`](Window.java) | Concrete handler (root) — always terminates; the guaranteed fallback |
| [`Main.java`](Main.java) | Client driver — wires the hierarchy, runs 3 demonstration scenarios |

---

## How to Run

```bash
# From the ComponentWithContextualHelp/ directory
javac *.java
java Main
```

**Expected output (abbreviated):**

```
=================================================================
  Scenario 1: F1 pressed on OK Button (has tooltip)
  Expected  : Chain stops at Button — most specific answer wins.
=================================================================
[Button] Showing tooltip: "Click OK to submit your login credentials."

=================================================================
  Scenario 2: F1 pressed on Cancel Button (no tooltip)
  Expected  : Chain passes Button → stops at Dialog (wiki page found).
=================================================================
[Button] No tooltip — forwarding to Dialog...
[Dialog] Opening wiki page for context-specific help: https://example.com/docs/login-dialog

=================================================================
  Scenario 3: F1 pressed on a Button with no tooltip...
  Expected  : Chain travels all the way to the Window (global fallback).
=================================================================
[Button] No tooltip — forwarding to Dialog...
[Dialog] No wiki page — forwarding to Panel...
[Panel] No modal help — forwarding to Window...
[Window] No specific help found in the chain.
[Window] Displaying global help guide: "Welcome to the Help Center!..."
```

---

## Master 5-Pattern Comparison

> Observer broadcasts events to multiple subscribers, Proxy acts as a gatekeeper controlling access to a single real object, while Composite, Decorator, and Chain of Responsibility organize trees, add wrapper layers, or pass requests along a sequence.
>
> Mastering these five means you can reason clearly about any system design that involves **objects holding references to other objects and delegating method calls** — the most common confusion cluster in system design interviews.

| Pattern | Category | Structure | Core Purpose | Execution Flow |
|---|---|---|---|---|
| **Composite** | Structural | Tree (1 parent → many children) | Treat individual items and groups uniformly. | Top-down: calls cascade to **all** children in the branch. |
| **Decorator** | Structural | Nested Onion (1 wrapper → 1 wrapped object) | Add extra features/layers dynamically at runtime. | **All** layers run: wrappers add behaviour before/after calling the core. |
| **Chain of Responsibility** | Behavioural | Linked List (1 handler → 1 next handler) | Pass a request along a line until someone handles or rejects it. | Sequential & conditional: can **stop early** at any handler. |
| **Observer** | Behavioural | 1-to-Many (1 Publisher → many Subscribers) | Notify multiple independent objects when a state changes. | Broadcast: Publisher notifies **all** subscribed listeners at once. |
| **Proxy** | Structural | 1-to-1 Stand-in (1 Proxy → 1 Real Subject) | Control access, add caching, lazy-loading, or security to an object. | Gatekeeper: decides if and when to forward the call to the real object. |

---

## Key Confusions Cleared Up

### 1. Proxy vs. Decorator _(both wrap 1 object!)_

| | Proxy | Decorator |
|---|---|---|
| **Goal** | Control **access** / manage lifecycle | Add **new behaviour** |
| **Who chooses?** | Client usually doesn't — it's injected as a stand-in | Client actively stacks multiple decorators |
| **Examples** | Blocking unauthorized users, caching, lazy creation | Encryption + compression on a file stream |

### 2. Observer vs. Chain of Responsibility _(both notify other objects!)_

| | Observer | Chain of Responsibility |
|---|---|---|
| **Topology** | 1-to-Many (broadcast) | 1-to-1-to-1 (relay) |
| **Who acts?** | **All** subscribers simultaneously | **One** handler (the first that can answer) |
| **Stop early?** | No — everyone always gets notified | Yes — chain stops as soon as a handler claims it |
| **Analogy** | A kitchen bell — every waiter hears it at once | Complaining about cold food: Waiter → Chef → Manager |

### 3. Chain of Responsibility vs. Composite _(both use hierarchies!)_

| | Composite | Chain of Responsibility |
|---|---|---|
| **Direction** | Top-down (parent → children) | Bottom-up (child → parent) |
| **Who participates?** | **All** nodes in the branch | **One** node (the first handler found) |
| **Early stop?** | No — the whole subtree runs | Yes — first capable handler terminates the chain |

---

## 5-Second Cheat Sheet (Restaurant Analogy)

| Pattern | Restaurant Analogy |
|---|---|
| **Composite** | A **Family Platter** — ordering it orders all individual dishes grouped inside it. |
| **Decorator** | Ordering a **Burger**, then adding Extra Cheese, then adding BBQ Sauce on top. |
| **Chain of Responsibility** | **Complaining** about cold food: Waiter → Head Chef → Manager (stops when resolved). |
| **Observer** | A **kitchen bell ringing** — every waiter in the restaurant hears it simultaneously. |
| **Proxy** | A **Bouncer/Host** at the door — checks your reservation before you reach the dining table. |

---

## Pattern Structure Diagrams

### Chain of Responsibility (this implementation)
```
Client
  │
  ▼
Button ──(no tooltip)──▶ Dialog ──(no wiki)──▶ Panel ──(no modal)──▶ Window
  │                        │                     │                      │
  └─ STOP (tooltip ✓)      └─ STOP (wiki ✓)      └─ STOP (modal ✓)     └─ STOP (always ✓)
```

### Observer (compare: EventManager example)
```
Editor (Publisher)
  │
  ├──▶ LogOpenListener.update()       ← all three are
  ├──▶ EmailListener(admin).update()  ← notified at
  └──▶ EmailListener(devops).update() ← the same time
```

### Proxy (compare: StockFeedProxy example)
```
Client ──▶ StockFeedProxy ──(if authorized)──▶ RealStockFeed
                │
                └─ BLOCK (if unauthorized) — never reaches the real object
```

---

*Implementation mirrors the [Refactoring Guru Chain of Responsibility](https://refactoring.guru/design-patterns/chain-of-responsibility) GUI contextual help example.*
