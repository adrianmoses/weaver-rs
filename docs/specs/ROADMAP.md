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
| 006 | ReWOO strategy                | planned | —    |
| 007 | LLMCompiler strategy          | planned | —    |
| 008 | Reflection loops              | planned | —    |
| 009 | Checkpointing                 | planned | —    |
| 010 | Memory                        | planned | —    |
| 011 | Multi-agent runtime           | planned | —    |
| 012 | Agent-as-tool                 | planned | —    |

Ordering reflects implementation dependencies: the graph runtime, tool registry, LLM adapters, and structured output are foundations; strategies build on them; reflection, checkpointing, memory, multi-agent, and agent-as-tool layer on top.

## Status Values

- `planned` — not yet started
- `in-progress` — spec written, implementation underway
- `implemented` — decision record complete
- `deprecated` — removed from product

## Revision History

| Date       | Change                  |
|------------|-------------------------|
| 2026-05-03 | Initial roadmap created |
