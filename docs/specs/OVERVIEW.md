# Overview

<!-- status: draft | approved -->
| Field | Value |
|---|---|
| status | approved |
| created | 2026-05-03 |

## Product Summary <!-- required -->

`weaver-rs` (the loom agent framework) is a Rust framework for building safe, secure, production-grade LLM agents and agentic workflows. It provides a typed, composable graph-based runtime — analogous to LangGraph in spirit, but native to Rust — with first-class support for multiple agent strategies (ReAct, ReWOO, LLMCompiler), structured output bound to user-defined state types, parallel tool dispatch, multi-agent coordination, checkpointing, and reflection loops. The framework is designed so that the compiler, not runtime checks, enforces correctness of state transitions, tool schemas, and graph structure.

## Target Consumer <!-- required -->

Rust developers building production agent systems where safety, performance, and type-correctness matter. Use cases span long-running autonomous agents, RAG pipelines, multi-agent orchestrations, and tool-using research workflows. Consumers are typically backend or systems engineers who want LangGraph-style composition without paying the runtime-cost and stringly-typed-state taxes of the Python ecosystem.

## Job To Be Done <!-- required -->

Give Rust developers a typed, composable runtime for orchestrating LLM agents and tool calls — one where graph structure, state shape, and tool schemas are checked at compile time, and where multiple agent strategies (ReAct, ReWOO, LLMCompiler, multi-agent) are interchangeable behind a common trait.

## Non-Goals <!-- required -->

- Not a vector database. Vector storage and retrieval are pluggable concerns; we provide traits for memory adapters but do not ship our own embedding store.
- Not a hosted service. `weaver-rs` is a library, not a platform. There is no managed runtime, no SaaS dashboard, no opinionated deployment story.
- Not a UI or streaming-frontend toolkit. Streaming hooks may be exposed at the runtime boundary, but rendering them is the consumer's problem.
- Not (for v1) a Python-bindings shim. Python bindings are a deferred future direction and explicitly out of scope until the core API stabilizes.
- Not a prompt-templating DSL. Prompts are constructed by the consumer; the framework does not impose a templating language.

## Tech Stack <!-- required -->

- **Language:** Rust, edition 2021
- **Async runtime:** `tokio` (full features) — task spawning, channels, broadcast for multi-agent message bus
- **Serialization:** `serde`, `serde_json`
- **Schema generation:** `schemars` (derive) — produces JSON Schema for structured output and tool inputs, injected into LLM calls
- **Async traits:** `async-trait`
- **Error handling:** `thiserror`
- **Time / IDs:** `chrono` (with `serde`), `uuid` (v4)
- **Cargo features (strategy-gated):** `react` (default), `rewoo`, `compiler` (LLMCompiler), `multi` (multi-agent runtime)
- **Infrastructure:** none — `weaver-rs` is a pure library crate with no required external services

## Testing Suite <!-- required -->

Test evidence in decision records references this section.

- **Unit tests:** `cargo test`, colocated with each module via `#[cfg(test)] mod tests`. Each primitive, trait impl, and strategy has its own unit tests.
- **Integration tests:** under `tests/` at the crate root. Each integration test exercises a full graph run (state init → strategy steps → terminal node) against a mocked LLM client and a small tool registry.
- **Examples:** under `examples/` at the crate root, runnable via `cargo run --example <name>`. Examples are load-bearing — they are the canonical proof that the framework actually works end-to-end. Each major feature (ReAct, ReWOO, LLMCompiler, multi-agent, checkpointing, reflection) ships at least one example.
- **Doctests:** API-level docs on public traits and structs include runnable examples where the surface is small enough to demonstrate inline.
- **LLM mocking:** A `MockLlmClient` lives behind a `#[cfg(test)]`/test-only feature gate so integration tests are deterministic and offline.
- **CI gate:** `cargo test --all-features` plus `cargo build --examples --all-features` must pass before any decision record is marked complete.

## Open Questions <!-- optional -->

- Which LLM providers ship as built-in adapters in v1 (OpenAI? Anthropic? both? a generic OpenAI-compatible adapter?), and which live behind cargo features.
- Streaming: do we expose token-level streaming through `AgentCtx`, or only message-level events?
- Checkpoint storage backends — in-memory only for v1, or also a filesystem/SQLite default?
- How agent-as-tool composes with cargo features (does wrapping a `multi`-runtime agent require the `multi` feature transitively?).
