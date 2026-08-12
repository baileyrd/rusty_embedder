# Architecture

## Overview
A Rust embedding generator — produces vector embeddings for downstream use
(e.g. search or RAG). No code exists yet; this section will be filled in once
the first crate lands.

## Boundaries
<!-- Domain logic vs. I/O and framework details (ports-and-adapters).
     List the ports (interfaces) and the adapters that implement them. -->

| Port | Adapter(s) | Notes |
| ---- | ---------- | ----- |
|      |            |       |

## Structure
Modular monolith, composition over inheritance, ports-and-adapters keeping
domain logic free of I/O and framework details. A component gets split into
its own service only for a concrete forcing function — independent scaling,
a team/language boundary, or hard fault isolation.

This isn't just this repo's default — it's `Atlas_Engineering_Standards_Library`
ATLAS-001 Part IV Chapter 21 (Layered Architecture), specifically
`ATLAS-LAYER-0001` (directional dependency: higher layers depend only on a
lower layer's declared interface) and `ATLAS-LAYER-0010` (layer
substitutability). Chapter 22's `ATLAS-BOUND-0001`/`ATLAS-BOUND-0010` govern
boundary ownership and failure contracts once this repo has trust boundaries
worth naming (e.g. an external embedding-model API).

The two stack-specific volumes that would otherwise narrow this further —
`ATLAS-100` (Architecture) and `ATLAS-300` (Rust Workspace/Cargo) — are both
`Seed` status, explicitly not yet triggered (ATLAS-100 waits for a second
real component depending on another; ATLAS-300 waits for a second crate).
Neither applies to a single-crate, pre-code repo yet — revisit once either
trigger condition is met.

## Data flow
<!-- Diagram or short walkthrough of a request/event through the system -->

## Key decisions
See [docs/adr/](./docs/adr/) for the record of individual decisions and their tradeoffs.

## Non-goals
<!-- Explicitly out of scope, and why -->
