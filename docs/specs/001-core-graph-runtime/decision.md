# Decision Record: Core Graph Runtime

| Field      | Value                                                     |
|------------|-----------------------------------------------------------|
| feature id | 001                                                       |
| spec       | [spec.md](spec.md)                                        |
| status     | implemented                                               |
| created    | 2026-05-04                                                |

---

## What Shipped

Three Rust modules and a runnable example, all on a foundation of the existing primitives/traits scaffolding (which had to be repaired before any new code could land):

1. **`src/graph/`** — `NodeId`, `NodeFn<S>`, `EdgeCondition<S>`, `Edge<S>`, `AgentCtx<S>`, `Graph<S>`, `GraphBuilder<S>`. Compile-time-typed graph composition with build-time validation.
2. **`src/runtime/single.rs`** — `SingleAgentRuntime<S>` that drives a graph from Start to End with iteration cap and cancellation.
3. **`src/lib.rs::prelude`** — public surface for downstream features.
4. **`examples/graph_basic.rs`** — canonical "hello world" runnable demo.

All testable without an LLM, tool registry, or strategy — the spec's animating goal.

## Acceptance Criteria — Test Evidence

| Spec criterion                                                          | Test evidence                                                                |
|-------------------------------------------------------------------------|------------------------------------------------------------------------------|
| `cargo check` and `cargo test` pass on a clean checkout                 | CI gate; commit 1 (`refactor: get existing src/ compiling`)                  |
| `Graph<S>` for any `S: Send + 'static` via async closures               | `tests/graph_basic.rs::basic_graph_runs_start_to_end_and_returns_final_state`|
| Edges with optional `EdgeCondition<S>` predicates                       | `tests/graph_branching.rs` (both true/false branches)                        |
| `build()` rejects: no Start edge, dangling endpoints, unreachable nodes | `tests/graph_validation.rs` (3 tests) + `src/graph/builder.rs` unit tests    |
| Runtime executes Start → … → End                                        | `tests/graph_basic.rs::multi_node_path_visits_each_node_in_order`            |
| Iteration cap (default 25) returns `LoomError::MaxIterations`           | `tests/graph_iteration_cap.rs` + `src/runtime/single.rs` unit tests          |
| Cancellation via `tokio_util::sync::CancellationToken`                  | `tests/graph_cancellation.rs`                                                |
| Offline tests (no LLM)                                                  | All `tests/graph_*.rs` — zero network dependencies                           |
| `examples/graph_basic.rs` built by CI and runnable                      | `cargo run --example graph_basic` outputs `final state: AppState { ... }`   |

19 tests pass total: 10 unit + 9 integration.

## Resolutions to Spec Open Questions

- **`NodeId` design** — opaque newtype `NodeId(Cow<'static, str>)` with `From<&'static str>` (zero-alloc) and `From<String>` (owned), plus reserved `NodeId::start()`/`NodeId::end()` constructors. Generic-`N` deferred indefinitely; the newtype handles every observed use case ergonomically.
- **`AgentCtx<S>` surface** — `state: S` (owned), `run: RunMeta`, `step: usize`, `cancel: CancellationToken`. Sufficient for the closure-based pattern; no event emitter or logger handle added since no downstream feature has yet demanded one.
- **`InterruptReason` placement** — moved from `traits/checkpoint.rs` into `error.rs` (next to `LoomError::Interrupted`). `traits/checkpoint.rs` re-exports for back-compat. Resolves the import cycle the spec flagged.
- **Prelude** — `weaver_rs::prelude` exports `LoomError`, `Result`, `InterruptReason`, `Graph`, `GraphBuilder`, `NodeId`, `NodeFn`, `EdgeCondition`, `AgentCtx`, `SingleAgentRuntime`. Examples and tests use `use weaver_rs::prelude::*;` exclusively.

## Routing Semantics — Decision

The spec described the contract loosely; implementation forced clarity. The shipped semantics are:

1. **Start** has no node function. The runtime walks outgoing edges from `NodeId::start()` in registration order; the first edge whose condition matches the initial state is the entry node. If none match, `LoomError::Graph("no edge from Start matched...")`.
2. A node returning `NodeId::end()` **terminates** the run.
3. A node returning a **different** concrete `NodeId` routes there directly. If the returned id is not registered (and not `End`), the runtime returns `LoomError::Graph("node X returned unknown next id: Y")`.
4. A node returning **its own** `NodeId` is the "let edges decide" signal: the runtime evaluates outgoing conditional edges from the current node and picks the first whose predicate matches. This is the canonical pattern for state-based branching (see `tests/graph_branching.rs`).

This gives a clean dual mode: explicit routing (rule 3) for simple flows, predicate-based routing (rule 4) for branching, with `EdgeCondition` reusable in both cases.

## Closure API — Decision

Initial design tried a generic `Fut: Future + 'static` to let users return `async {}` directly. This failed to compile under `for<'a> Fn(&'a mut AgentCtx<S>) -> Fut` because `Fut`'s lifetime can't be tied to the higher-ranked `'a`.

Shipped: `F: for<'a> Fn(&'a mut AgentCtx<S>) -> BoxFuture<'a, Result<NodeId, LoomError>> + Send + Sync + 'static`. Callers write `|ctx| Box::pin(async move { ... })`. One extra `Box::pin` per node — small cost for the standard, well-understood Rust async-closure pattern.

## Refactoring Scope

The spec flagged "small refactoring." Scope as it actually played out:

- Wired up empty `lib.rs`, `primitives/mod.rs`, `traits/mod.rs` with module declarations and re-exports.
- Added missing `use` statements across every existing source file (none of them compiled before this commit).
- Moved `InterruptReason` from `traits/checkpoint.rs` to `error.rs` to break the import cycle; `traits/checkpoint.rs` re-exports.
- Added two new dependencies (`tokio-util = "0.7"` for `CancellationToken`, `futures = "0.3"` for `BoxFuture`).
- Added explicit `#[serde(bound(...))]` on `AgentState<T>` so the serde derive resolves cleanly with the `StructuredOutput` trait bound.
- Added `Clone` to the `StructuredOutput` supertrait set so `AgentState<T>: Clone` (required by the existing `GraphState: Clone` blanket impl).

All preparatory; isolated in commit 1 (`refactor: get existing src/ compiling`) so the diff is reviewable on its own.

## Deviations from Spec

- **`Graph<S>: Debug` not implemented.** Closures don't auto-derive `Debug`; integration tests use explicit `match` arms instead of `unwrap_err()` to side-step the `T: Debug` bound on `Result::unwrap_err`. Acceptable cost; revisit if downstream features need to inspect a `Graph` post-build.
- **No checkpoint hook.** The spec's Non-Goals section reserved the option to "expose a future-compatible hook." Skipped — feature 010 (Checkpointing) will be free to add `with_checkpoint(store)` additively without breaking 001 callers.
- **No streaming, no observability.** As spec'd.

## Files Changed

```
Cargo.toml                                 # tokio-util, futures
Cargo.lock                                 # now tracked
src/lib.rs                                 # module declarations + prelude
src/error.rs                               # InterruptReason moved here, Cancelled variant
src/primitives/{mod,message,state,tool}.rs # imports, re-exports, serde bounds
src/traits/{mod,checkpoint,memory,output,tool}.rs # imports, re-exports, Clone bound on StructuredOutput
src/graph/{mod,node,edge,ctx,graph,builder}.rs    # NEW
src/runtime/{mod,single}.rs                # NEW
tests/graph_{basic,branching,iteration_cap,cancellation,validation}.rs # NEW
examples/graph_basic.rs                    # NEW
docs/specs/001-core-graph-runtime/spec.md  # status -> approved
docs/specs/ROADMAP.md                      # 001 -> implemented
docs/specs/001-core-graph-runtime/decision.md # this file
```

## Verification

```bash
cargo check                              # clean
cargo test --all-features                # 19 passed; 0 failed
cargo build --examples --all-features    # graph_basic builds
cargo run --example graph_basic          # prints final state
```

## Follow-ups for Downstream Features

- **005 (ReAct strategy)** will introduce the strategy → node adapter that wraps `Arc<dyn AgentStrategy<S>>` into a `NodeFn<S>` and translates `AgentAction` enum values into `NodeId`. No API change to 001 required.
- **002 (Tool registry)** and **003 (LLM client adapters)** will own their own state, accessed inside node closures via captured `Arc`s. `AgentCtx<S>` does not need to grow.
- **010 (Checkpointing)** can add `SingleAgentRuntime::with_checkpoint(store)` additively. Hook fires between iterations using the existing `Checkpoint` trait.
- **011 (HITL interrupts)** will likely add a non-terminating `LoomError::Interrupted(reason)` return path that the runtime treats as "save checkpoint, exit". The shipped `LoomError::Interrupted(InterruptReason)` variant is already in place.
