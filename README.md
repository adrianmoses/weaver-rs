# weaver-rs

> the loom agent framework — a typed, composable runtime for production LLM agents in Rust.

`weaver-rs` is a Rust framework for building safe, secure, production-grade LLM agents and agentic workflows. Think LangGraph in spirit, native to Rust in implementation: graph structure, state shape, and tool schemas are checked at compile time, not at runtime.

**Status:** early / pre-alpha. APIs are unstable.

## Why

- **Type-checked graphs.** State is a generic `S: Send + 'static`; structured output is `S: DeserializeOwned + JsonSchema`. No stringly-typed state keys.
- **Interchangeable strategies.** `ReAct`, `ReWOO`, and `LLMCompiler` all implement a single `AgentStrategy<S>` trait — swap them with one line.
- **Parallel tool dispatch.** ReWOO and LLMCompiler plan tool calls upfront and dispatch them concurrently with `tokio::join_all`.
- **Multi-agent.** Agents coordinate over a `tokio::broadcast` message bus; each runs in its own task.
- **Reflection, checkpointing, memory.** First-class traits, pluggable adapters.
- **Library, not a platform.** No hosted service, no telemetry, no embedded vendor lock-in.

## Cargo features

| Feature    | Purpose                           | Default |
|------------|-----------------------------------|---------|
| `react`    | ReAct strategy                    | yes     |
| `rewoo`    | ReWOO (plan-then-execute)         | no      |
| `compiler` | LLMCompiler (DAG-scheduled tools) | no      |
| `multi`    | Multi-agent runtime + bus         | no      |

## Testing

- `cargo test` — unit tests colocated with each module, plus integration tests under `tests/` against a deterministic `MockLlmClient`
- `cargo run --example <name>` — runnable end-to-end demos under `examples/` (load-bearing: examples are how we prove the framework actually works)

CI gate: `cargo test --all-features` and `cargo build --examples --all-features`.

## Specs and design

The full design lives under [`docs/specs/`](docs/specs/):

- [OVERVIEW.md](docs/specs/OVERVIEW.md) — product summary, target consumer, non-goals, tech stack
- [ARCHITECTURE.md](docs/specs/ARCHITECTURE.md) — system layers, component map, data flow, constraints
- [ROADMAP.md](docs/specs/ROADMAP.md) — feature list and status

## Non-goals

`weaver-rs` is **not** a vector database, **not** a hosted service, **not** a UI/streaming-frontend toolkit, and **not** a prompt-templating DSL. Python bindings are deferred until the core API stabilizes.
