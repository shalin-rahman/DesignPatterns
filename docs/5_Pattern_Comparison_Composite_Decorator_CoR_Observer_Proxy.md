# 5-Pattern Deep Comparison: Composite · Decorator · Chain of Responsibility · Observer · Proxy

> **Why these five?**
> All five involve objects holding references to other objects and delegating method calls.
> They are the most commonly confused cluster in system design interviews and code reviews.
> This document defines each pattern precisely, explains when to reach for it, shows how
> they interlock when combined, and provides decision rules to eliminate confusion.

---

## Table of Contents

1. [Quick Identity Card](#1-quick-identity-card)
2. [Pattern-by-Pattern Deep Dive](#2-pattern-by-pattern-deep-dive)
   - [Composite](#21-composite)
   - [Decorator](#22-decorator)
   - [Chain of Responsibility](#23-chain-of-responsibility)
   - [Observer](#24-observer)
   - [Proxy](#25-proxy)
3. [Head-to-Head Confusion Pairs](#3-head-to-head-confusion-pairs)
4. [When Multiple Patterns Combine](#4-when-multiple-patterns-combine)
5. [Decision Flowchart — Which Pattern?](#5-decision-flowchart--which-pattern)
6. [Implementation Examples in This Repo](#6-implementation-examples-in-this-repo)

---

## 1. Quick Identity Card

| Pattern | Category | Structure | Intent in one sentence | Execution model |
|---|---|---|---|---|
| **Composite** | Structural | Tree (1 parent → 0..* children) | Treat a single object and a group of objects identically. | **Top-down, all-inclusive** — every node in the branch runs. |
| **Decorator** | Structural | Nested onion (1 wrapper → 1 wrappee) | Add responsibilities to an object at runtime without subclassing. | **All-layers, ordered** — every wrapper runs; none can skip. |
| **Chain of Resp.** | Behavioural | Linked list (1 handler → 1 next) | Pass a request along handlers until one handles or discards it. | **Sequential, early-stop** — the chain halts at the first handler that claims the request. |
| **Observer** | Behavioural | 1-to-Many (1 publisher → many subscribers) | Notify all dependents automatically when the publisher's state changes. | **Broadcast, simultaneous** — every subscriber receives the event at once. |
| **Proxy** | Structural | 1-to-1 stand-in (1 proxy → 1 real subject) | Control access to, or augment, a single real object transparently. | **Gatekeeper** — decides once per call whether, and how, to forward to the real object. |

---

## 2. Pattern-by-Pattern Deep Dive

### 2.1 Composite

**Problem it solves**

You have part-whole hierarchies (file systems, UI trees, menus, org charts) where client code
would otherwise need to distinguish leaves from containers with `instanceof` checks.

**Core structure**

```
Component (interface)
    |
    +-- Leaf           (no children — executes directly)
    +-- Composite      (holds children — delegates to all of them)
```

**Key mechanic**

`Composite.operation()` calls `child.operation()` for every child.
Children may themselves be Composites, so the call recurses to any depth automatically.

**When to use**

- The domain is hierarchical by nature (tree = natural fit).
- You want one code path that works for both a single item and a whole subtree.
- Adding new leaf or composite types must not change client code.

**When NOT to use**

- The structure is not a tree (use a graph, list, or map instead).
- Operations differ so much between leaf and container that a shared interface becomes a forced abstraction.

**Repo example:** [`composite-pattern/`](../composite-pattern/README.md) — Searchable file system
(`IFileSystemItem` ← `File` / `Folder`)

---

### 2.2 Decorator

**Problem it solves**

A class needs optional, combinable behaviours (encryption, compression, logging, caching)
that would explode into `2^n` subclasses if handled by inheritance.

**Core structure**

```
Component (interface)
    |
    +-- ConcreteComponent          (innermost wrappee — the real object)
    +-- BaseDecorator              (implements Component; holds a Component reference)
            |
            +-- ConcreteDecoratorA
            +-- ConcreteDecoratorB
```

**Key mechanic**

Each decorator holds a reference to any `Component` (which may itself be a decorator).
On each call, it adds behaviour **before and/or after** delegating to its inner wrappee.
The chain unwinds from outer to inner on write, inner to outer on read.

**When to use**

- Responsibilities are orthogonal and should be stackable in any combination.
- You can't (or shouldn't) subclass the target class.
- Behaviour needs to be attached and detached at runtime.

**When NOT to use**

- Only one combination will ever exist in practice (just subclass once).
- The order of wrappers doesn't matter to the result — Composite may be simpler.
- You need to skip layers conditionally — use Chain of Responsibility instead.

**Repo example:** [`decorator_design_pattern/`](../decorator_design_pattern/README.md) — File data source
(`DataSource` ← `FileDataSource` + `EncryptionDecorator` + `CompressionDecorator`)

---

### 2.3 Chain of Responsibility

**Problem it solves**

A request may be handled by one of several objects, but the sender should not know which one,
and the chain of candidates can change at runtime.

**Core structure**

```
Handler (interface / abstract)
    |
    +-- ConcreteHandlerA  --next--> ConcreteHandlerB  --next--> ConcreteHandlerC  --next--> null
```

**Key mechanic**

Each handler checks whether it can process the request.
If yes → it handles it and the chain stops.
If no  → it forwards to `next` (which may be null = end of chain).

**When to use**

- The set of handlers and their priority order changes at runtime.
- More than one object may be capable of handling a request but only one should.
- You want to decouple the sender from all concrete handlers.
- Classic cases: middleware pipelines, UI event bubbling, support escalation, GUI contextual help.

**When NOT to use**

- Exactly one handler is always correct — use Strategy or simple delegation.
- All handlers must always act — use Observer or Composite.
- You need a guaranteed handler — ensure the last link is a catch-all, or add a null check.

**Repo example:** [`chain-of-reponsibility/ComponentWithContextualHelp/`](../chain-of-reponsibility/ComponentWithContextualHelp/README.md) —
GUI contextual help (Button → Dialog → Panel → Window)

---

### 2.4 Observer

**Problem it solves**

Multiple, unrelated parts of a system need to react when an object's state changes,
without creating hard-coded dependencies between them.

**Core structure**

```
Publisher
    |-- subscriber list: [Sub1, Sub2, Sub3, ...]
    |
    notify() --> Sub1.update()
             --> Sub2.update()
             --> Sub3.update()
```

**Key mechanic**

The Publisher keeps a list of Subscribers and iterates through it on `notify()`.
Every subscriber is called; there is no early stop. Subscribers are added/removed at runtime
without touching the Publisher.

**When to use**

- State changes in one object must trigger reactions in an unknown or varying number of others.
- Changes should propagate without tight coupling.
- Subscribers don't know about each other.
- Classic cases: event systems, MVC (model notifying views), stock feeds, GUI event listeners,
  message buses, reactive streams.

**When NOT to use**

- The number of subscribers is always exactly one — use a simple callback or Strategy.
- Subscribers need to influence whether the event is "consumed" — use Chain of Responsibility.
- Order of notification is critical — Observer provides no guaranteed order across different implementations.
- You need synchronous, transactional behaviour — loose coupling can make debugging harder.

**Repo example:** [`observer_pattern/EventManager/`](../observer_pattern/EventManager/) —
Editor event system (Editor publishes `open`/`save` events;
LogOpenListener, EmailNotificationListener subscribe)

---

### 2.5 Proxy

**Problem it solves**

A client needs access to a potentially expensive, remote, or sensitive object, but that
object shouldn't be directly reachable: you need lazy initialisation, access control, caching,
logging, or remote communication without changing the client.

**Core structure**

```
Subject (interface)
    |
    +-- RealSubject    (the actual object — does the real work)
    +-- Proxy          (implements Subject; holds a reference to RealSubject)
            |
            checkPermission() / cache lookup / lazy init
            |
            --> RealSubject.operation()   (only if conditions are met)
```

**Key mechanic**

The Proxy and the RealSubject share the same interface, so the client cannot tell them apart.
The Proxy intercepts every call and decides: forward or not, transform before or after, cache, log.
There is always exactly one RealSubject behind the Proxy.

**Common Proxy types**

| Type | What it does |
|---|---|
| **Virtual Proxy** | Lazy-creates the expensive RealSubject on first access. |
| **Protection Proxy** | Checks permissions before forwarding. |
| **Caching Proxy** | Returns cached results instead of calling the real object. |
| **Remote Proxy** | Hides the fact that the real object lives on another machine. |
| **Logging Proxy** | Records all calls for auditing. |

**When to use**

- Object creation is expensive and should be deferred until actually needed.
- Access control must be enforced transparently.
- You want caching without changing the client or the real object.
- The real object lives remotely (network/RPC).

**When NOT to use**

- Multiple behaviours need to be combined freely — use Decorator (the client chooses the stack).
- Multiple objects need to react — use Observer.
- Access control is handled at the infrastructure/framework level already.

**Repo example:** [`observer_pattern/StockMonitoring/StockFeedProxy.java`](../observer_pattern/StockMonitoring/StockFeedProxy.java) —
StockFeedProxy wraps RealStockFeed, enforcing UserType-based access control.

---

## 3. Head-to-Head Confusion Pairs

### 3A. Proxy vs. Decorator — "Both wrap exactly one object!"

This is the single most common confusion. The structure looks identical from the outside.

| Dimension | Proxy | Decorator |
|---|---|---|
| **Goal** | Control **access** / manage the object's lifecycle | Add **new behaviour** to an existing interface |
| **Number of real objects** | Exactly 1 — the Proxy IS the stand-in for 1 real subject | Exactly 1 — but the client **chooses** the wrapping depth |
| **Who decides the wrapping?** | Injected transparently — the client usually doesn't know it's talking to a proxy | The client actively builds the stack (`new A(new B(new C()))`) |
| **Can layers be stacked?** | No — two Proxies chained would be unusual and probably a design smell | Yes — stacking is the entire point |
| **Early termination?** | Yes — Proxy may block the call entirely | No — the decorator **always** calls its wrappee eventually |
| **Wrappee lifecycle** | Proxy may **own and manage** the RealSubject (lazy-create, remote connect) | Decorator never controls wrappee creation |
| **Interface** | Identical to the RealSubject (the client can't tell the difference) | Same, but the client explicitly selects decorators |

**Decision rule:**
> _"Am I standing in for something to control access or defer its creation?"_ → **Proxy**
> _"Am I layering extra behaviour the client explicitly asks for?"_ → **Decorator**

---

### 3B. Observer vs. Chain of Responsibility — "Both pass something to other objects!"

| Dimension | Observer | Chain of Responsibility |
|---|---|---|
| **Topology** | 1-to-Many (publisher → subscriber list) | 1-to-1-to-1 (linked list) |
| **Who receives?** | **All** subscribers receive the event | **One** handler ultimately claims it |
| **Early stop?** | No — all subscribers always run | Yes — the chain stops immediately once handled |
| **Direction** | Publisher → subscribers (push) | Sender → handler → next handler (relay) |
| **Subscribers know each other?** | No — they're independent | Yes — each handler holds a reference to the next |
| **Request consumed?** | No — the event is broadcast | Yes — one handler "consumes" the request |

**Decision rule:**
> _"Do I need everyone to react?"_ → **Observer**
> _"Do I need exactly one to react, and I don't know which upfront?"_ → **Chain of Responsibility**

---

### 3C. Chain of Responsibility vs. Decorator — "Both traverse a linked chain!"

| Dimension | Decorator | Chain of Responsibility |
|---|---|---|
| **All layers run?** | **Yes always** — every wrapper adds its behaviour | **No** — chain stops at the first capable handler |
| **Can a layer skip the rest?** | No — a decorator always calls its wrappee | Yes — a handler can handle and return without forwarding |
| **Purpose of the chain** | Accumulate/transform behaviour | Route a request to the right handler |
| **Who builds the chain?** | Client, eagerly at construction time | Can be dynamic; handlers register themselves |

**Decision rule:**
> _"Should every layer in the chain always do something?"_ → **Decorator**
> _"Should the chain stop at the first layer that can answer?"_ → **Chain of Responsibility**

---

### 3D. Composite vs. Chain of Responsibility — "Both use parent-child hierarchies!"

| Dimension | Composite | Chain of Responsibility |
|---|---|---|
| **Direction of propagation** | Top-down (parent delegates to children) | Bottom-up (child passes to parent) |
| **Who participates?** | Every node in the branch | Only the nodes traversed before a handler claims it |
| **Early stop?** | No — the whole subtree runs | Yes — first capable handler terminates |
| **Purpose** | Uniform treatment of leaf and container | Routing a single request to the right handler |

**Decision rule:**
> _"Should the whole subtree execute the operation?"_ → **Composite**
> _"Should the request bubble up until one node can handle it?"_ → **Chain of Responsibility**

---

### 3E. Proxy vs. Observer — "Both react to events on another object!"

| Dimension | Observer | Proxy |
|---|---|---|
| **Number of reactive objects** | Many subscribers | One proxy |
| **Who initiates?** | Publisher broadcasts to all | Client calls proxy; proxy may forward |
| **Transparency** | Subscribers are known to the publisher | Client doesn't know it's talking to a proxy |
| **Purpose** | Loose coupling on state change | Access control or augmentation |

---

## 4. When Multiple Patterns Combine

These patterns are not mutually exclusive. Real systems often combine two or three.
Here are the most natural pairings — each is used in this repository.

---

### 4.1 Observer + Proxy (StockMonitoring example)

**Setup:** A `StockFeedProxy` controls access to a `RealStockFeed` publisher.
Subscribers (Observers) only receive events if they pass the Proxy's permission check.

```
Client
  |
  v
StockFeedProxy (Proxy)
  -- checks UserType permission
  |-- if allowed --> RealStockFeed (Publisher)
                         |
                         +--> StockObserver1.update()   (broadcast to all)
                         +--> StockObserver2.update()
                         +--> Dashboard.update()
```

**How they interact:**
- The Proxy is a **gatekeeper** — it decides who can subscribe and who gets notified.
- Once past the Proxy, the Observer pattern takes over — all authorized subscribers
  receive the broadcast simultaneously.
- Neither pattern knows about the other's existence at the class level.

**When to combine:** When you need access control (Proxy) on top of an event system (Observer).

---

### 4.2 Composite + Decorator (UI rendering)

**Setup:** A UI tree (Composite) where each node can have visual decorators applied
(borders, shadows, scrollbars) without changing the tree structure.

```
Window (Composite root)
  |
  +-- ScrollbarDecorator          <-- Decorator wraps the Panel
        |
        v
       Panel (Composite)
         |
         +-- BorderDecorator      <-- Decorator wraps a Button
               |
               v
              Button (Leaf)
```

**How they interact:**
- The Composite gives the structural tree and the uniform interface.
- The Decorator adds visual behaviour to individual nodes without altering the tree.
- Both use the same `Component` interface — a Decorator is indistinguishable from a Leaf or Composite to the parent.

**When to combine:** When you have a tree of objects (Composite) and need to add behaviours
to specific nodes without subclassing or restructuring the tree (Decorator).

---

### 4.3 Composite + Chain of Responsibility (UI help propagation — this repo)

**Setup:** The UI hierarchy (Composite) doubles as the handler chain.
When F1 is pressed, the help request starts at the focused component and
bubbles up through the containment hierarchy until a handler responds.

```
Window  (top of Composite tree; last handler in CoR chain)
  |
Panel   (Composite node; intermediate CoR handler)
  |
Dialog  (Composite node; intermediate CoR handler)
  |
Button  (Composite leaf; first CoR handler)

Request: Button --> Dialog --> Panel --> Window
                                        (chain stops at first handler with an answer)
```

**How they interact:**
- The Composite provides the parent-child hierarchy and the `setContainer(parent)` wiring.
- The Chain of Responsibility uses that same hierarchy as its handler chain, traversing
  it bottom-up — the opposite direction to Composite's normal top-down delegation.
- Each node plays both roles: a Composite node for rendering/layout, a CoR handler for help.

**When to combine:** When you already have a containment hierarchy and want requests
to bubble up through it (browser DOM events, GUI help systems, exception handling).

---

### 4.4 Decorator + Proxy (logging/caching proxy on a decorated object)

**Setup:** A Proxy provides caching for an object that is itself decorated with
logging and encryption. The Proxy is the outermost shell; it doesn't know about
the decorators inside.

```
Client
  |
  v
CachingProxy (Proxy)
  -- checks cache; on miss:
  |
  v
LoggingDecorator (Decorator)
  |
  v
EncryptionDecorator (Decorator)
  |
  v
FileDataSource (ConcreteComponent)
```

**How they interact:**
- The Proxy handles caching (read path) and blocks repeated expensive calls.
- The Decorators handle transformation (write path) independently.
- The Proxy's `RealSubject` interface is the same `DataSource` as the decorators use —
  it doesn't need to know that decorators are inside.

**When to combine:** When you need access control or caching (Proxy) around an object
that also needs composable behaviour transformations (Decorator).

---

### 4.5 Observer + Chain of Responsibility (event filtering pipeline)

**Setup:** An event enters a filtering chain (CoR) that sanitises, validates, and routes it.
Only events that pass all filters are broadcast to subscribers (Observer).

```
EventSource
  |
  v
AuthFilter (CoR handler 1) -- blocks unauthorized events
  |
  v
RateLimitFilter (CoR handler 2) -- blocks flood events
  |
  v
EventManager (Publisher) -- broadcasts to all subscribers
      |
      +--> LoggingSubscriber.update()
      +--> DashboardSubscriber.update()
      +--> AlertSubscriber.update()
```

**How they interact:**
- The Chain decides whether the event is worthy of broadcasting.
- The Observer broadcasts to all if the chain passes it through.
- The chain and the subscriber list are completely independent — adding a new filter
  does not affect subscribers and vice versa.

**When to combine:** Middleware pipelines, API gateways, event-driven systems with
pre-processing requirements.

---

## 5. Decision Flowchart — Which Pattern?

```
START: I need objects to delegate work to other objects
         |
         v
    Is the structure a tree?
    (parent owns many children)
         |
        YES --> Do ALL children need to execute?
                    |
                   YES --> COMPOSITE
                    |
                    NO --> Does the request bubble UP through parents?
                                |
                               YES --> CHAIN OF RESPONSIBILITY
                                |
                                NO  --> (unusual; reconsider the design)
         |
        NO
         |
         v
    Is it a 1-to-1 relationship (one object wraps/controls exactly one other)?
         |
        YES --> Does the client actively choose the layers?
                    |
                   YES --> Can multiple layers stack freely?
                                |
                               YES --> DECORATOR
                                |
                                NO  --> PROXY (single wrap, transparent to client)
                    |
                    NO  --> PROXY (injected transparently by framework/DI)
         |
        NO
         |
         v
    Does one state change need to notify MULTIPLE independent objects?
         |
        YES --> Should every listener always react?
                    |
                   YES --> OBSERVER
                    |
                    NO  --> Combine OBSERVER + CHAIN OF RESPONSIBILITY
                            (filter chain then broadcast)
         |
        NO
         |
         v
    Does a request need to reach ONE handler from a list of candidates?
    (you don't know which one at design time)
         |
        YES --> CHAIN OF RESPONSIBILITY
         |
        NO  --> revisit the problem — it may not need a structural pattern
```

---

## 6. Implementation Examples in This Repo

| Pattern | Location | Context / Scenario |
|---|---|---|
| **Composite** | [`composite-pattern/`](../composite-pattern/) | Searchable file system: `IFileSystemItem` ← `File` / `Folder` |
| **Decorator** | [`decorator_design_pattern/`](../decorator_design_pattern/) | File data source: `FileDataSource` + `EncryptionDecorator` + `CompressionDecorator` |
| **Chain of Responsibility** | [`chain-of-reponsibility/ComponentWithContextualHelp/`](../chain-of-reponsibility/ComponentWithContextualHelp/) | GUI contextual help: `Button` → `Dialog` → `Panel` → `Window` |
| **Observer (event system)** | [`observer_pattern/EventManager/`](../observer_pattern/EventManager/) | Editor events: `Editor` publishes `open`/`save`; `LogOpenListener`, `EmailNotificationListener` subscribe |
| **Observer + Proxy** | [`observer_pattern/StockMonitoring/`](../observer_pattern/StockMonitoring/) | Stock feed: `StockFeedProxy` gates access; `RealStockFeed` broadcasts to `StockObserver`, `Dashboard`, `PriceAlert` |
| **Proxy** | [`proxy_design_pattern/YouTubeManager/`](../proxy_design_pattern/YouTubeManager/) | Caching proxy: `YouTubeCacheProxy` wraps `ThirdPartyYouTubeClass` to cache expensive remote video downloads |

---

## Appendix: 5-Second Restaurant Cheat Sheet

| Pattern | Restaurant Analogy |
|---|---|
| **Composite** | A **Family Platter** — ordering it orders every dish grouped inside it. All dishes are served. |
| **Decorator** | Ordering a **Burger**, adding Extra Cheese, then adding BBQ Sauce. Every layer always applies. |
| **Chain of Resp.** | Complaining about cold food: **Waiter → Head Chef → Manager**. Stops the moment someone resolves it. |
| **Observer** | A **kitchen bell ringing** — every waiter in the restaurant hears it simultaneously and checks their table. |
| **Proxy** | A **Bouncer/Host** at the entrance — checks your reservation before you can reach the dining room. |

---

## Appendix: Anti-Pattern Warnings

> [!CAUTION]
> **Don't confuse Proxy with Decorator** by looking only at the structure.
> Ask: _"Does the client know about the wrapper?"_ (Decorator: yes. Proxy: no.)
> Ask: _"Can you stack multiples freely?"_ (Decorator: yes. Proxy: rarely.)

> [!CAUTION]
> **Don't confuse Observer with CoR** by thinking "both forward to other objects."
> Ask: _"Should ALL react, or just ONE?"_ (Observer: all. CoR: one.)

> [!WARNING]
> **Don't use Composite** for structures that aren't genuinely hierarchical.
> Forcing a flat or graph-shaped domain into a tree produces a leaky abstraction.

> [!WARNING]
> **Don't use Chain of Responsibility** when you need a guaranteed handler.
> Always ensure the last link in the chain is a catch-all, or add a null-check fallback.

> [!NOTE]
> **Combining patterns is normal** — a real system often uses Observer + Proxy (access-controlled events),
> Composite + CoR (UI event bubbling), or Decorator + Proxy (caching around a transformed object).
> The key is that each pattern solves its own distinct concern; they compose cleanly precisely because they don't overlap.
