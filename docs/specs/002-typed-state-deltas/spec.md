# Spec: Typed State Deltas

| Field   | Value      |
|---------|------------|
| id      | 002        |
| status  | draft      |
| created | 2026-05-07 |

---

## Why <!-- required -->

The 001 runtime mutates state via `&mut AgentCtx<S>` inside node closures. This is the simplest possible model and works for sequential graphs — but the contract forecloses on a class of behavior the framework needs as it grows:

1. **Parallel work within a single graph.** A node that wants to call three tools concurrently and aggregate results cannot today. Two `&mut S` references to the same state cannot coexist; users have to serialize or wrap in `Arc<Mutex<…>>`, neither of which the framework should require.
2. **Multi-agent coordination (now 014).** When a coordinator dispatches sub-agents and synthesizes their outputs, "merge two state changes" needs a defined semantics. Without one, every multi-agent application invents its own.
3. **Checkpointing (now 011).** Snapshot semantics are clearer when state changes are explicit applied transactions rather than mid-flight in-place mutations. Replay is natural over a delta log; less natural over snapshots.
4. **HITL interrupts (now 012).** Pausing between "node decided what to change" and "change is applied" is a clean barrier. With `&mut S` mutations, a node can be half-done when we pause.

Locking the contract in *now*, before any consumer-facing strategy (006 ReAct), tool integration (003 Tool registry), or multi-agent runtime (014) is built on top, is cheap. The 001 PR is still open; existing tests and the example are the only callers. Doing this at 014 instead means breaking every downstream feature in the roadmap.

This feature only locks in the **contract**. The runtime stays sequential — actual parallel fan-out execution is deferred to 014 (multi-agent runtime) or its own future feature, and slots in additively because the contract supports it.

### Consumer Impact <!-- required -->

The consumer continues writing nodes as async closures. The signature changes shape.

**Today (after 001):**

```rust
.add_node("agent".into(), |ctx| Box::pin(async move {
    ctx.state.counter += 1;
    Ok(NodeId::end())
}))
```

**Proposed:**

```rust
.add_node("agent".into(), |ctx| Box::pin(async move {
    let mut next = ctx.state.clone();
    next.counter += 1;
    Ok(NodeOutput::end().with(next))
}))
```

For state types that opt into a custom `Delta` (e.g., `MyState: GraphState<Delta = MyDelta>`), nodes return concrete deltas:

```rust
.add_node("chat".into(), |ctx| Box::pin(async move {
    Ok(NodeOutput::goto("respond".into()).with(ChatDelta::Append(msg)))
}))
```

For state types that don't define one, a blanket impl provides `Delta = S` (full replace) so users can return a mutated copy of the state — preserving today's ergonomics at the cost of one `clone()` per node.

The `prelude` exposes `GraphState`, `NodeOutput`, and `NextStep` so consumers `use weaver_rs::prelude::*;` and write nodes without ceremony.

The litmus test for consumer impact: a developer can write either of these tests in `tests/` and have them pass without spinning up any service:

```rust
// 1. Simple state, blanket-impl Delta = S.
let final_state = run_graph(SimpleState { counter: 0 }).await?;
assert_eq!(final_state.counter, 1);

// 2. Custom Delta with append semantics.
let final_state = run_graph(ChatState { messages: vec![] }).await?;
assert_eq!(final_state.messages.len(), 1);
```

If both read naturally, the trait shape works for the 001 audience and the future multi-agent audience.

### Roadmap Fit <!-- required -->

Slots in as **002** (between 001 Core graph runtime and the now-bumped 003 Tool registry). Every later feature inherits the new contract:

- **003 Tool registry** is unaffected — tools are dispatched from inside nodes; the registry itself doesn't touch state.
- **004 LLM client adapters** are unaffected for the same reason.
- **005 Structured output** terminates a run by returning a final delta containing the parsed structured value.
- **006 ReAct strategy** is the first feature where typed deltas pay off — a ReAct node returns a delta carrying the new `Vec<Message>` entries.
- **011 Checkpointing** records `(state_at_step_n, applied_deltas, pending_node)` rather than `(state_snapshot,)` — smaller, replayable.
- **012 HITL interrupts** insert their pause boundary between "node returned delta" and "runtime applied delta".
- **014 Multi-agent runtime** uses `S::merge` to fold N sub-agents' deltas into the coordinator's state.

Renumbering: existing 002–014 each shift to 003–015. Revision history in `ROADMAP.md` records this.

---

## What <!-- required -->

### Acceptance Criteria <!-- required -->

- [ ] A `GraphState` trait with associated `Delta` type, `apply(&mut self, Self::Delta)`, and `merge(Self::Delta, Self::Delta) -> Self::Delta`.
- [ ] A blanket implementation for any `S: Send + Clone + 'static` with `type Delta = S`, where `apply` is `*self = delta` and `merge` is "second wins". Preserves the simple case ergonomically. (Subject to the orphan-rule resolution noted in Open Questions.)
- [ ] `NodeFn<S>` is generic over `S: GraphState` and returns `Result<NodeOutput<S>, LoomError>`.
- [ ] `NodeOutput<S>` is a struct carrying `next: NextStep` and `delta: Option<S::Delta>`, with builder methods: `NodeOutput::goto(id)`, `NodeOutput::end()`, `NodeOutput::edges()`, `.with(delta)`. Nodes that don't change state can call `NodeOutput::end()` without a delta.
- [ ] `NextStep` enum: `Goto(NodeId)`, `Edges`, `End`. Replaces the magic "return your own NodeId means edges decide" convention from 001.
- [ ] The `SingleAgentRuntime` accumulates one delta per node call and applies it via `S::apply` between iterations. Single-threaded behavior is observably identical to today.
- [ ] Every existing 001 integration test (`tests/graph_*.rs`) and the example (`examples/graph_basic.rs`) is updated to the new API and passes.
- [ ] A new integration test exercises a custom `Delta` type with append semantics on a `Vec<T>` field, demonstrating non-replace behavior.
- [ ] A new integration test exercises `NodeOutput::edges()` for predicate-driven branching, confirming the explicit replacement for 001's self-return convention.
- [ ] Public re-exports in `prelude` extended with `GraphState`, `NodeOutput`, `NextStep`.
- [ ] The decision record explicitly **defers** parallel fan-out runtime (the *use* of `merge`) to 014; this feature only locks in the contract.

### Non-Goals <!-- required -->

- **No parallel fan-out at the runtime level.** `merge` is defined and proved correct for at least one custom delta type, but the runtime still executes nodes sequentially. Parallel execution is 014.
- **No `derive(GraphState)` macro.** Hand-rolled trait impls are verbose for non-trivial deltas; a derive is a quality-of-life follow-up, not part of this feature. Spec'd separately when justified.
- **No checkpoint integration.** 011 will define how deltas are persisted; 002 only ensures the contract supports it.
- **No state validation / invariants.** `apply` is total; users are responsible for the soundness of their own delta semantics.
- **No streaming events on delta application.** Observability hooks (per the 001 review) are a separate concern; they can be added additively to the runtime without changing this contract.
- **No `AgentCtx` redesign beyond what the contract requires.** The runtime may shift from giving nodes `&mut AgentCtx<S>` to `&AgentCtx<S>` (read-only state, mutate via returned delta), but only as far as needed; everything else on `AgentCtx` (run meta, step, cancel) stays.

### Open Questions <!-- optional -->

- **Blanket impl + orphan rules.** `impl<S: Send + Clone + 'static> GraphState for S` blocks downstream consumers from writing `impl GraphState for MyState { type Delta = MyDelta; … }` — Rust's coherence rules forbid overlapping impls. Three resolutions:
  1. **Ship the blanket impl, drop opt-in.** Users wanting custom deltas wrap in a newtype or use a `#[derive]` macro that generates a non-conflicting impl. Simple state stays trivial; advanced state pays a small ceremony tax.
  2. **Drop the blanket impl, require explicit.** Every state type must `impl GraphState`. Most ergonomic for advanced users; meanest for "I just want to count things." A `#[derive(GraphState)]` macro mitigates but creates dependency on a proc-macro crate.
  3. **Specialization.** Unstable; not viable for OSS framework targeting stable Rust.

  **Recommend (1)** with an explicit "if you want a typed delta, derive or hand-roll" note. Validate with a spike before committing.

- **`Clone` bound on `S`.** Required by the blanket impl (since `Delta = S`) and natural for parallel branching (each branch needs its own copy). Most state types derive `Clone` already, but heavy state (embedding caches, large vectors) may not. Possible mitigation: a `GraphState::Static` marker or a `Cow`-shaped delta that avoids unnecessary clones. **Defer** unless an early example hits the constraint.
- **`merge` associativity.** The runtime will fold N deltas left-to-right when the parallel path lands (014). Should the trait *require* associativity? Documenting "best-effort, order-dependent merge OK for the sequential path; associativity required for parallel" is simpler than enforcing. **Recommend documenting requirement, not enforcing.**
- **`NodeOutput` API shape.** Three options:
  1. **Builder** — `NodeOutput::goto(id).with(delta)`. Reads best for unchanged-state returns (`NodeOutput::end()`). Adds slight ceremony for the always-with-delta case.
  2. **Struct literal** — `NodeOutput { next: NextStep::Goto(id), delta: Some(delta) }`. Most explicit; most verbose.
  3. **Enum** — `NodeOutput::Goto { id, delta: Option<…> }`. Most type-safe; harder to extend with non-routing fields later (e.g., a tracing event).

  **Recommend (1)** for ergonomics + extensibility; struct internals stay private so we can refactor freely.

- **Migration of 001 tests and example.** Mechanical but not free — every existing test and example rewrites. Confirm scope acceptable. (Counterfactual: leave 001's tests in place and add 002 tests on the new API. Two APIs in the codebase forever; rejected.)
- **Replacing the magic self-return convention.** 001 ships "return your own NodeId means edges decide." 002 introduces `NodeOutput::edges()` as the explicit form. This is a *better* design (per the 001 review) but it is a breaking change vs. 001's currently-shipped public API. Confirm: yes, breaking is acceptable here because 001 has no consumers yet and the PR is still open.

---

## How <!-- required -->

### Approach <!-- required -->

Three steps. Each lands as its own commit so the diff is reviewable.

**Step 1 — Define `GraphState`, `NodeOutput`, `NextStep`.**

```
src/graph/state.rs        -- GraphState trait, blanket impl
src/graph/output.rs       -- NodeOutput<S>, NextStep, builder methods
src/graph/mod.rs          -- re-exports
src/lib.rs::prelude       -- add GraphState, NodeOutput, NextStep
```

Sketches:

```rust
// graph/state.rs
pub trait GraphState: Send + 'static {
    type Delta: Send + 'static;
    fn apply(&mut self, delta: Self::Delta);
    fn merge(a: Self::Delta, b: Self::Delta) -> Self::Delta;
}

// Blanket impl — subject to orphan-rule resolution from Open Questions.
impl<S: Send + Clone + 'static> GraphState for S {
    type Delta = S;
    fn apply(&mut self, delta: S) { *self = delta; }
    fn merge(_a: S, b: S) -> S { b }   // second writer wins
}
```

```rust
// graph/output.rs
pub enum NextStep {
    Goto(NodeId),
    Edges,
    End,
}

pub struct NodeOutput<S: GraphState> {
    pub(crate) next:  NextStep,
    pub(crate) delta: Option<S::Delta>,
}

impl<S: GraphState> NodeOutput<S> {
    pub fn goto(id: NodeId) -> Self  { Self { next: NextStep::Goto(id), delta: None } }
    pub fn end()  -> Self             { Self { next: NextStep::End,      delta: None } }
    pub fn edges() -> Self            { Self { next: NextStep::Edges,    delta: None } }
    pub fn with(mut self, delta: S::Delta) -> Self { self.delta = Some(delta); self }
}
```

Spike before committing: confirm the blanket impl doesn't block a downstream `impl GraphState for MyState`. If it does, switch to an opt-in derive (Open Question item 1).

**Step 2 — Wire the new contract through `NodeFn` and the runtime.**

- `NodeFn<S>` becomes `Arc<dyn for<'a> Fn(&'a AgentCtx<S>) -> BoxFuture<'a, Result<NodeOutput<S>, LoomError>> + Send + Sync>` *or* keeps `&mut AgentCtx<S>` if the migration is heavier than expected. Decision in spike.
- `SingleAgentRuntime::run`:
  1. Call the node function, await its `NodeOutput<S>`.
  2. If `output.delta.is_some()`, apply via `S::apply(&mut ctx.state, delta)`.
  3. Route based on `output.next`:
     - `Goto(id)` → set current to id (validate registered or `End`).
     - `End` → return `Ok(ctx.state)`.
     - `Edges` → call `graph.next_node(current, &ctx.state)`.
- The "return your own NodeId means edges" convention from 001 is removed. `NextStep::Edges` is the explicit replacement.

Update every existing test, example, and unit test in `src/graph/builder.rs` and `src/runtime/single.rs`.

**Step 3 — Verify.**

- `cargo test --all-features` — all 001 tests still pass (semantically equivalent on the new API), plus new ones below.
- `cargo build --examples --all-features` — `graph_basic` builds.
- `cargo run --example graph_basic` — runs.
- New tests:
  - `tests/graph_state_delta_replace.rs` — blanket impl path; assert state replacement works as expected.
  - `tests/graph_state_delta_append.rs` — custom `Delta` with append semantics on `Vec<T>`.
  - `tests/graph_next_step_edges.rs` — explicit `NodeOutput::edges()` predicate routing (replaces 001's self-return convention).

### Confidence <!-- required -->

**Level:** Medium

**Rationale:**

The trait shape is well-trodden (it's a `Monoid` over deltas, with `apply` linking deltas to state). The Rust-specific risks:

- **Blanket impl + orphan rules.** Whether a downstream user can override `Delta = MyDelta` for their type while we ship `impl<S: Clone> GraphState for S` is the central uncertainty. Spike *before* committing to (1) above; if blocked, fall back to opt-in via derive.
- **Closure ergonomics with the new return type.** `NodeOutput::goto(id).with(delta)` reads naturally for one delta; a node returning multiple deltas (rare, but possible) needs `S::merge` upfront. Validate with a real test.
- **Migration noise.** Rewriting 001's tests is mechanical but the diff is large. Acceptable risk: we own the only callers.

**Validate before proceeding:**

1. **Spike the blanket impl in a scratch crate.** Confirm whether `impl GraphState for MyState { type Delta = MyDelta; … }` coexists with `impl<S: Clone> GraphState for S`. If not, pick the opt-in path.
2. **Rewrite `tests/graph_basic.rs` first.** Read it cold. If the new API reads as natural as the 001 form, proceed. If not, redesign `NodeOutput`.
3. **Rewrite the example.** Same litmus.
4. **Confirm a `Vec<T>`-append delta works without cloning the whole state.** Custom delta path is the value-add; if it's painful, the feature isn't earning its keep.

### Key Decisions <!-- optional -->

- **Trait, not associated function or pointer.** `GraphState` is a trait so consumers control the impl per state type. Per-state-type pluggability is the whole point.
- **Blanket impl with full-replace as the simple-case backstop**, accepting orphan-rule friction for advanced users. The 80% case (small POD state) stays trivial. Subject to spike confirmation.
- **`NodeOutput` is a struct, not an enum.** Keeping it a struct lets us add fields later (e.g., `tracing_event: Option<Event>`, `metrics: …`) without breaking callers using the builder. The `next` discriminant lives inside as a `NextStep` enum.
- **`NextStep::Edges` is the explicit replacement for 001's self-return convention.** Removes a magic-value footgun called out in the 001 review.
- **No parallel runtime in 002.** This feature locks in the contract. Implementing fan-out comes with 014.
- **Renumber existing 002–014 to 003–015.** Roadmap is dependency-ordered; 002 belongs immediately after 001.

### Testing Approach <!-- required -->

Per `OVERVIEW.md` ("Testing Suite") — `cargo test` for unit + integration, `examples/` runnable demos, `MockLlmClient` not used (no LLM dependency in 002).

**Unit tests** (colocated `#[cfg(test)] mod tests`):

- `graph/state.rs`: blanket impl `apply` (replace) and `merge` (second wins) for `()`, `u32`, simple struct.
- `graph/output.rs`: builder methods (`goto`, `end`, `edges`, `with`) produce the expected struct shape.
- `runtime/single.rs`: applying a delta between iterations updates `ctx.state` correctly; absence of delta leaves state untouched.

**Integration tests** (under `tests/`):

- `tests/graph_state_delta_replace.rs` — blanket-impl path; node returns mutated copy of state, assert replacement.
- `tests/graph_state_delta_append.rs` — custom `Delta` type (e.g., `enum ChatDelta { Append(Message) }`); node appends; final state has appended message; multiple iterations accumulate.
- `tests/graph_next_step_edges.rs` — explicit `NodeOutput::edges()` predicate routing; replaces 001's self-return convention.
- All existing 001 integration tests rewritten to the new API:
  - `tests/graph_basic.rs`, `tests/graph_branching.rs`, `tests/graph_iteration_cap.rs`, `tests/graph_cancellation.rs`, `tests/graph_validation.rs`.

**Example**:

- `examples/graph_basic.rs` rewritten to the new API; remains the canonical "hello world."

**CI gate** (per `OVERVIEW.md`): `cargo test --all-features` and `cargo build --examples --all-features` must pass before this feature's decision record is marked complete.
