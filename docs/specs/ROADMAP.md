# Roadmap

<!-- status: draft | approved -->
| Field | Value |
|---|---|
| status | approved |
| created | 2026-05-03 |

## Features

| ID  | Feature                       | Status  | Spec |
|-----|-------------------------------|---------|------|
| 001 | Core graph runtime            | planned | —    |
| 002 | Tool registry                 | planned | —    |
| 003 | LLM client adapters           | planned | —    |
| 004 | Structured output             | planned | —    |
| 005 | ReAct strategy                | planned | —    |
| 006 | Examples module               | planned | —    |
| 007 | ReWOO strategy                | planned | —    |
| 008 | LLMCompiler strategy          | planned | —    |
| 009 | Reflection loops              | planned | —    |
| 010 | Checkpointing                 | planned | —    |
| 011 | Memory                        | planned | —    |
| 012 | Multi-agent runtime           | planned | —    |
| 013 | Agent-as-tool                 | planned | —    |

Ordering reflects implementation dependencies: the graph runtime, tool registry, LLM adapters, and structured output are foundations; strategies build on them; reflection, checkpointing, memory, multi-agent, and agent-as-tool layer on top.

The **Examples module** (006) is treated as a first-class feature, not a side artifact. It establishes the `examples/` directory conventions, the offline `MockLlmClient` harness, the examples README index, and the canonical first end-to-end example (paired with ReAct). Each subsequent strategy or major feature is expected to ship at least one runnable example as part of its own decision record. For an open-source agent framework, runnable end-to-end examples are how prospective users verify the framework works — they are load-bearing, not decorative.

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
