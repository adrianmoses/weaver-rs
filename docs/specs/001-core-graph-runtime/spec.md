# Spec: Core Graph Runtime

| Field   | Value      |
|---------|------------|
| id      | 001        |
| status  | approved   |
| created | 2026-05-04 |

---

## Why <!-- required -->

The graph runtime is the bedrock primitive that every other feature in the roadmap composes on top of: strategies are nodes, tool calls are dispatched from inside nodes, structured output terminates a run, checkpoints snapshot a graph mid-run, multi-agent runtimes orchestrate multiple graphs. If the graph runtime itself is hard to test, that pain inherits into every downstream feature.

The animating goal of this feature is therefore not "ship a working graph" but **ship a working graph whose builder API is testable without an LLM, a tool registry, or a strategy**. A consumer (or this project's own integration tests) must be able to write a deterministic, fully-typed graph using nothing more than async closures over their own state type, and assert behavior end-to-end with `cargo test`.

### Consumer Impact <!-- required -->

The consumer is a Rust developer composing an agent or agentic workflow. They interact with this feature through:

- **`GraphBuilder<S>`** — a fluent builder where they register nodes (async closures over `&mut AgentCtx<S>`) and edges (with optional `EdgeCondition<S>` predicates over `&S`).
- **`Graph<S>`** — the validated, immutable result.
- **`SingleAgentRuntime<S>`** — runs a `Graph<S>` from start to end, returning the final state.

The litmus test for consumer impact: a developer can write the following test in `tests/` and have it pass without spinning up any external service, mock, or strategy:

```rust
let graph = Graph::<MyState>::builder()
    .add_node(NodeId::from("agent"), |ctx| Box::pin(async move {
        ctx.state.counter += 1;
        Ok(NodeId::end())
    }))
    .add_edge(NodeId::start(), NodeId::from("agent"))
    .build()?;

let final_state = SingleAgentRuntime::new(graph)
    .run(MyState { counter: 0 })
    .await?;

assert_eq!(final_state.counter, 1);
```

If this test reads naturally and runs offline, every downstream feature inherits a testable substrate.

### Roadmap Fit <!-- required -->

001 is the root of the dependency tree. Nothing else can ship until it does. Every roadmap feature 002–014 either composes nodes onto a graph (strategies, reflection, agent-as-tool), needs the runtime to invoke them (tool registry, LLM adapters), persists graph state (checkpointing, HITL interrupts), or layers across multiple graphs (multi-agent). No prerequisites of its own.

---

## What <!-- required -->

### Acceptance Criteria <!-- required -->

- [ ] `cargo check` and `cargo test` pass on a clean checkout. (Today the existing `src/` has empty `mod.rs` files and missing imports — it does not compile.)
- [ ] A consumer can construct a `Graph<S>` for any user-defined `S: Send + 'static` using the `GraphBuilder<S>` API, with nodes as `async` closures over `&mut AgentCtx<S>`.
- [ ] A consumer can register edges with optional `EdgeCondition<S>` predicates that re-route based on `&S`.
- [ ] `GraphBuilder::build()` validates at build time and returns a typed error on:
  - no `Start` edge registered
  - an edge references an unknown `NodeId`
  - a node is unreachable from `Start`
- [ ] `SingleAgentRuntime<S>::run(initial_state)` executes the graph from `Start`, follows node return values and edge conditions, and terminates at `End`, returning the final `S`.
- [ ] The runtime enforces a configurable iteration cap (default 25). Exceeding it returns `LoomError::MaxIterations(n)` rather than looping forever.
- [ ] Cancellation: passing a `tokio::CancellationToken` aborts a run cleanly between node steps.
- [ ] An integration test under `tests/` runs an end-to-end graph with deterministic closures, no LLM, no tools, no strategy.
- [ ] An integration test exercises edge-condition branching (state predicate routes to one of two paths).
- [ ] An integration test exercises the iteration cap (a deliberately looping graph terminates with `MaxIterations`).
- [ ] Unit tests cover each `GraphBuilder` validation path.
- [ ] An `examples/graph_basic.rs` demonstrates the same minimal usage as the consumer-impact snippet above and is built by CI.

### Non-Goals <!-- required -->

- **No strategy dispatch.** `AgentStrategy`, `AgentAction`, `CallTools`, `Respond` belong to features 005/007/008. A node in 001 is just an async closure that returns the next `NodeId`. A strategy will later be modeled as a closure that wraps `Arc<dyn AgentStrategy<S>>` and translates `AgentAction` into a `NodeId`, but that adapter ships with the first strategy, not here.
- **No tool registry integration.** `AgentCtx<S>` does not hold a `ToolRegistry`. That is feature 002.
- **No LLM client.** `AgentCtx<S>` does not hold an `LlmClient`. That is feature 003.
- **No checkpoint persistence.** A `Checkpoint` *hook surface* may be exposed on the runtime so that 010 can plug in without a breaking change, but no store impl ships in 001 and no checkpoint is taken by default.
- **No HITL interrupt mechanism.** That is feature 011.
- **No multi-agent runtime, no message bus.** Feature 013.
- **No graph visualization, dot export, or introspection tooling.** Out of scope.
- **No streaming events.** Runtime returns the final state synchronously when the run completes; token-level streaming is deferred (Open Decision in `ARCHITECTURE.md`).

### Open Questions <!-- optional -->

- **`NodeId` shape.** Proposed: opaque newtype around `Cow<'static, str>` (so both `NodeId::from("agent")` zero-alloc and `NodeId::from(format!(...))` work), with reserved constructors `NodeId::start()` and `NodeId::end()`. Alternative: make `NodeId` generic over the graph (`Graph<S, N>` where `N: Hash + Eq + Clone`). The newtype is simpler and matches the LangGraph mental model; the generic is more typesafe but more ceremony at use sites. **Defer the generic** unless the closure-only API proves too error-prone in practice — it can be added later additively without breaking callers if `NodeId` is opaque.
- **`AgentCtx<S>` surface.** Minimum: `state: &mut S`, `run: &RunMeta`, `cancel: CancellationToken`, `step: usize`. Anything else (event emitters, structured logger handle) we add when a downstream feature actually needs it. Confirm this minimum is enough to start.
- **`InterruptReason` placement.** Currently referenced in `error.rs` (variant `Interrupted`) and *defined* in `traits/checkpoint.rs`. This is a circular import waiting to bite. Proposal: move `InterruptReason` into `error.rs` (or into a new top-level `interrupt.rs`) and have `traits/checkpoint.rs` re-export from there. Resolve as part of the refactor in this feature.
- **Re-exports / prelude.** `weaver_rs::prelude` exporting `Graph`, `GraphBuilder`, `NodeId`, `EdgeCondition`, `AgentCtx`, `SingleAgentRuntime`, `LoomError`, `Result`. Worth deciding the public surface now even if more is added later.

---

## How <!-- required -->

### Approach <!-- required -->

Implementation breaks into three steps. Step 1 is the "small refactoring" the user flagged; steps 2 and 3 are the new feature.

**Step 1 — Get the existing code compiling.**

The current `src/` is a sketch: `lib.rs`, `primitives/mod.rs`, and `traits/mod.rs` are empty, and every existing file is missing its `use` declarations.

- Wire up `src/lib.rs` to declare `pub mod error; pub mod primitives; pub mod traits;` (and the new `graph`, `runtime` modules added below).
- Populate `primitives/mod.rs` and `traits/mod.rs` to re-export their submodules.
- Add the missing `use` statements across existing files (`serde::{Serialize, Deserialize}`, `serde::de::DeserializeOwned`, `schemars::JsonSchema`, `async_trait::async_trait`, `chrono::{DateTime, Utc}`, plus internal cross-module references).
- Resolve the `InterruptReason` cycle: move the type to `error.rs` (it is referenced by `LoomError::Interrupted` already) and have `traits/checkpoint.rs` import it from there.
- Run `cargo check` and `cargo test` (no tests yet — should still succeed) before adding the new graph code.

**Step 2 — Build the graph module.**

```
src/graph/
  mod.rs
  node.rs        -- NodeId, NodeFn<S> trait alias, BoxedNode<S>
  edge.rs        -- EdgeCondition<S>, Edge<S>
  ctx.rs         -- AgentCtx<S>
  graph.rs       -- Graph<S> (validated, immutable)
  builder.rs     -- GraphBuilder<S> + validation errors
```

Sketches:

```rust
// graph/node.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(Cow<'static, str>);

impl NodeId {
    pub fn start() -> Self { Self(Cow::Borrowed("__start")) }
    pub fn end()   -> Self { Self(Cow::Borrowed("__end")) }
}

pub type NodeFn<S> = Arc<
    dyn for<'a> Fn(&'a mut AgentCtx<S>)
        -> BoxFuture<'a, Result<NodeId, LoomError>>
        + Send + Sync,
>;

// graph/edge.rs
pub type EdgeCondition<S> = Arc<dyn Fn(&S) -> bool + Send + Sync>;

pub struct Edge<S> {
    pub from: NodeId,
    pub to:   NodeId,
    pub when: Option<EdgeCondition<S>>,
}

// graph/ctx.rs
pub struct AgentCtx<S> {
    pub state:  S,
    pub run:    RunMeta,
    pub step:   usize,
    pub cancel: CancellationToken,
}

// graph/graph.rs
pub struct Graph<S> {
    nodes:  HashMap<NodeId, NodeFn<S>>,
    edges:  Vec<Edge<S>>,
}

// graph/builder.rs
pub struct GraphBuilder<S> { /* ... */ }

impl<S: Send + 'static> GraphBuilder<S> {
    pub fn add_node<F, Fut>(self, id: NodeId, f: F) -> Self
        where F:   Fn(&mut AgentCtx<S>) -> Fut + Send + Sync + 'static,
              Fut: Future<Output = Result<NodeId, LoomError>> + Send + 'static;

    pub fn add_edge(self, from: NodeId, to: NodeId) -> Self;
    pub fn add_conditional_edge(self, from: NodeId, to: NodeId, when: EdgeCondition<S>) -> Self;
    pub fn build(self) -> Result<Graph<S>, LoomError>;
}
```

`build()` validates: `Start` has at least one outgoing edge, every edge endpoint exists as a registered node (or is the reserved `End`), and a BFS from `Start` reaches every node (no orphans).

**Step 3 — Build the runtime.**

```
src/runtime/
  mod.rs
  single.rs      -- SingleAgentRuntime<S>
```

```rust
pub struct SingleAgentRuntime<S> {
    graph:          Graph<S>,
    iteration_cap:  usize,    // default 25
    cancel:         CancellationToken,
}

impl<S: Send + 'static> SingleAgentRuntime<S> {
    pub fn new(graph: Graph<S>) -> Self;
    pub fn with_iteration_cap(self, cap: usize) -> Self;
    pub fn with_cancellation(self, token: CancellationToken) -> Self;

    pub async fn run(self, initial_state: S) -> Result<S, LoomError>;
}
```

`run` constructs an `AgentCtx<S>`, starts at the node reachable from `NodeId::start()`, invokes its `NodeFn<S>` to get the next `NodeId`, applies edge conditions if any, and loops until it reaches `NodeId::end()`. Each iteration increments `ctx.step`; if `step` exceeds `iteration_cap`, return `LoomError::MaxIterations(cap)`. Each iteration also checks `cancel.is_cancelled()` and returns `LoomError::Graph("cancelled".into())` (or a dedicated variant — see decision below) if so.

### Confidence <!-- required -->

**Level:** Medium

**Rationale:**

The conceptual design is well-trodden ground (LangGraph, petgraph, state machines generally). What's well-understood:

- Closure-based nodes returning the next node ID.
- BFS validation at build time.
- Iteration cap to prevent runaway loops.

What's uncertain enough to drop confidence from High to Medium:

- **Rust closure ergonomics with `BoxFuture<'_>` and `Send` bounds.** The trait alias `NodeFn<S>` above will work in theory, but real consumer code that captures references or `Arc`s into closures often runs into subtle lifetime/Send issues. The first end-to-end test will tell us whether the API is actually pleasant to use.
- **Existing code may need more than "small" refactoring.** The empty `mod.rs` files plus missing imports plus the `InterruptReason` import cycle suggest the existing files were never compiled. There may be more cracks once we run `cargo check`.
- **`NodeId` design.** The opaque-newtype proposal is the right starting point but generic-`N` may turn out to be necessary for ergonomics; we should validate the newtype works before committing.

**Validate before proceeding:**

1. Land Step 1 (get existing code compiling) on its own commit, separate from new feature work, so the diff is reviewable. If `cargo check` reveals more work than expected, surface that and decide whether to expand scope or split into a precursor PR.
2. Spike a minimal `Graph<()>` with one stub node and a `SingleAgentRuntime<()>::run` to confirm the closure/lifetime/Send story holds before fleshing out the validation, edge-condition, and cancellation paths.
3. Confirm the `AgentCtx<S>` surface (state + run meta + step + cancellation) is sufficient by writing the consumer-impact snippet from the **Why** section as a real test before declaring the API stable.

### Key Decisions <!-- optional -->

- **Nodes return `NodeId`, not `AgentAction`.** Strategies are not a concern of 001. The strategy pattern from `core-traits.md` (now in `docs/design/`) layers in via 005 by wrapping a strategy as a node closure that translates `AgentAction` into `NodeId` — additive, no breaking change required here.
- **`NodeId` is an opaque newtype** wrapping `Cow<'static, str>`. Defer the generic-`N` design unless ergonomics demand it.
- **Validation happens at `build()`, not at `run()`.** Errors caught early are cheaper than errors caught at minute 17 of a long run.
- **Iteration cap is on the runtime, not the graph.** Same graph can run with different caps per call site.
- **Cancellation is a `tokio::CancellationToken`** passed in via builder method, checked between node steps. Synchronous mid-step cancellation is out of scope.
- **No checkpoint surface in 001.** A future-compatible hook (e.g., `with_checkpoint(store)` taking an opaque trait object) can be added in 010 without breaking 001 callers.

### Testing Approach <!-- required -->

Per `OVERVIEW.md` ("Testing Suite") — `cargo test` for unit + integration, `examples/` runnable demos, `MockLlmClient` for offline determinism (not used here since 001 has no LLM dependency).

**Unit tests** (colocated `#[cfg(test)] mod tests` blocks):

- `graph/node.rs`: `NodeId` equality / hashing / `start()`/`end()` reserved.
- `graph/builder.rs`: each validation path
  - missing start edge → `LoomError::Graph(...)`
  - edge to unregistered node → `LoomError::Graph(...)`
  - unreachable node → `LoomError::Graph(...)`
  - happy path returns `Ok(Graph)`
- `runtime/single.rs`: iteration-cap default = 25; `with_iteration_cap` overrides.

**Integration tests** (under `tests/`):

- `tests/graph_basic.rs` — happy path: `Start → A → End`, deterministic closure mutates `S`, runtime returns final state.
- `tests/graph_branching.rs` — two outgoing edges from a node; conditional `EdgeCondition<S>` routes based on state predicate; assert correct path was taken.
- `tests/graph_iteration_cap.rs` — node always returns its own `NodeId` (loop); runtime returns `LoomError::MaxIterations(cap)` once cap is hit.
- `tests/graph_cancellation.rs` — cancellation token tripped before run; runtime returns the cancellation error variant before invoking any node.
- `tests/graph_validation.rs` — exercises each builder validation error from a consumer perspective (i.e., `build()` returning `Err(...)`).

**Examples** (load-bearing per `OVERVIEW.md`):

- `examples/graph_basic.rs` — smallest possible runnable graph; serves as the canonical "hello world" for the framework. CI gate `cargo build --examples` covers it.

**CI gate** (per `OVERVIEW.md`): `cargo test --all-features` and `cargo build --examples --all-features` must pass before this feature's decision record is marked complete.
