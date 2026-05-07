# Roadmap

<!-- status: draft | approved -->
| Field | Value |
|---|---|
| status | approved |
| created | 2026-05-03 |

## Features

| ID  | Feature                       | Status  | Spec |
|-----|-------------------------------|---------|------|
| 001 | Core graph runtime            | implemented | [spec](001-core-graph-runtime/spec.md) · [decision](001-core-graph-runtime/decision.md) |
| 002 | Typed state deltas            | in-progress | [spec](002-typed-state-deltas/spec.md) |
| 003 | Tool registry                 | planned | —    |
| 004 | LLM client adapters           | planned | —    |
| 005 | Structured output             | planned | —    |
| 006 | ReAct strategy                | planned | —    |
| 007 | Examples module               | planned | —    |
| 008 | ReWOO strategy                | planned | —    |
| 009 | LLMCompiler strategy          | planned | —    |
| 010 | Reflection loops              | planned | —    |
| 011 | Checkpointing                 | planned | —    |
| 012 | Human-in-the-loop interrupts  | planned | —    |
| 013 | Memory                        | planned | —    |
| 014 | Multi-agent runtime           | planned | —    |
| 015 | Agent-as-tool                 | planned | —    |

Ordering reflects implementation dependencies: the graph runtime, the state-delta contract, tool registry, LLM adapters, and structured output are foundations; strategies build on them; reflection, checkpointing, human-in-the-loop, memory, multi-agent, and agent-as-tool layer on top.

**Typed state deltas** (002) lock in the contract that node functions return *typed deltas* against a `GraphState` trait, rather than mutating state in place. This is foundational rather than ergonomic: it is what enables parallel fan-out within a single graph, multi-agent merge semantics (014), efficient checkpoints (011), and clean HITL pause boundaries (012). Doing it now — while 001 is still the only caller — costs one rewrite of 001's tests and example. Doing it later would be a breaking change to every downstream feature.

The **Examples module** (007) is treated as a first-class feature, not a side artifact. It establishes the `examples/` directory conventions, the offline `MockLlmClient` harness, the examples README index, and the canonical first end-to-end example (paired with ReAct). Each subsequent strategy or major feature is expected to ship at least one runnable example as part of its own decision record. For an open-source agent framework, runnable end-to-end examples are how prospective users verify the framework works — they are load-bearing, not decorative.

**Human-in-the-loop interrupts** (012) extend checkpointing with a typed `interrupt(payload) -> resume_value` primitive: a node can pause the run, surface a payload to the caller (an approval prompt, a clarification question, a draft for review), and resume on a subsequent `runtime.resume(run_id, value)` call. State persists across the pause via the same `Checkpoint` machinery, so a human can take seconds or days to respond. This is what turns the framework from "autonomous agents only" into a substrate for agentic workflows that include real human review steps.

## Status Values

- `planned` — not yet started
- `in-progress` — spec written, implementation underway
- `implemented` — decision record complete
- `deprecated` — removed from product

## Revision History

| Date       | Change                                                                                  |
|------------|-----------------------------------------------------------------------------------------|
| 2026-05-03 | Initial roadmap created                                                                 |
| 2026-05-03 | Added Examples module as feature 006; subsequent IDs (ReWOO → Agent-as-tool) bumped +1 |
| 2026-05-03 | Added Human-in-the-loop interrupts as feature 011; Memory → Agent-as-tool bumped +1     |
| 2026-05-04 | Core graph runtime (001) moved to in-progress; spec drafted at 001-core-graph-runtime/  |
| 2026-05-04 | Core graph runtime (001) implemented; decision record at 001-core-graph-runtime/        |
| 2026-05-07 | Added Typed state deltas as feature 002; subsequent IDs (Tool registry → Agent-as-tool) bumped +1; spec drafted at 002-typed-state-deltas/ |
