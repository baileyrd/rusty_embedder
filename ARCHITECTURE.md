# Architecture

## Overview
`rusty_embedder` gives the platform a standard, swappable interface for
text embedding: application code depends on the `Embedder` trait, and the
concrete model underneath — local ONNX inference, a remote
OpenAI-compatible API, a no-op default for tests — is chosen at
construction time and replaceable without touching call sites. Modeled on
`rusty_search`'s `SearchBackend`/workspace shape, and a Rust equivalent of
`knowledge-mcp`'s Python `Embedder` protocol.

## Boundaries
| Port | Adapter(s) | Notes |
| ---- | ---------- | ----- |
| `rusty_embedder_core::Embedder` | `NullEmbedder` (core, always available), `LocalEmbedder` (`rusty-embedder-local`, feature `local`), `HttpEmbedder` (`rusty-embedder-http`, feature `http`) | Sync, batched (`&[String]` → `Vec<Vec<f32>>`). Callers hold `Arc<dyn Embedder>` (or a generic `E: Embedder`) and swap the concrete adapter at construction time. |
| `serialize_f32` / `deserialize_f32` | n/a — free functions, not a trait | Fixed on-disk contract: little-endian, contiguous `f32` bytes, no header — exactly what `sqlite-vec`'s `vec0` virtual tables expect. Any consumer serializing/deserializing a vector for storage goes through these rather than reimplementing the layout. |

## Structure
A workspace of four crates, mirroring `rusty_search`'s shape:

- **`rusty-embedder-core`** — the `Embedder` trait, `NullEmbedder`,
  `EmbedError`/`Result`, and the `serialize_f32`/`deserialize_f32` byte
  helpers. **Zero mandatory dependencies** — no crate from crates.io, so
  depending on this crate alone (to write backend-agnostic application
  code, or to implement a custom `Embedder`) needs no network at build or
  run time and stays fully offline via `NullEmbedder`.
- **`rusty-embedder-local`** — `LocalEmbedder`, backed by `fastembed-rs`
  (bundled ONNX Runtime + a small MiniLM-class model). Downloads model
  weights once, on first construction; every `embed()` call after that is
  in-process, no network.
- **`rusty-embedder-http`** — `HttpEmbedder`, a generic REST client for
  any OpenAI-compatible embeddings endpoint (`reqwest::blocking`, since
  `Embedder::embed` is synchronous). Covers OpenAI and any endpoint
  speaking the same request/response shape — not one crate per vendor.
- **`rusty-embedder`** — the facade. Re-exports
  `rusty-embedder-core`'s full public API, plus `LocalEmbedder`/
  `HttpEmbedder` behind their own feature flags (`local`, `http`), off by
  default — the same shape `rusty-search`'s facade uses for
  `memory`/`tantivy`/`elasticsearch`.

Modular monolith, composition over inheritance, ports-and-adapters
keeping the `Embedder` trait free of any one backend's I/O or SDK
details — this repo's default per `Atlas_Engineering_Standards_Library`
ATLAS-001 Part IV Chapter 21/22 (`ATLAS-LAYER-0001`/`0010`,
`ATLAS-BOUND-0001`/`0010`), now concretely instantiated: `rusty-embedder`
is the higher layer depending only on `rusty-embedder-core`'s declared
interface, and each backend crate is independently substitutable behind
it. `ATLAS-100`/`ATLAS-300` (the Rust-workspace-specific standards
volumes) are triggered now that this repo has more than one crate with a
real dependency relationship between them — nothing in this workspace's
shape currently deviates from either, so no exception is recorded.

## Data flow
```
caller
  │  texts: &[String]
  ▼
Arc<dyn Embedder>            (NullEmbedder | LocalEmbedder | HttpEmbedder | ...)
  │  embed(texts) -> Result<Vec<Vec<f32>>>
  ▼
Vec<Vec<f32>>                 one vector per input text, same order
  │  serialize_f32(&vector)
  ▼
Vec<u8>                       little-endian f32 bytes, sqlite-vec vec0 layout
  ▼
caller's storage (e.g. rusty_knowledge's sqlite-vec vec0 column)
```

`LocalEmbedder` and `HttpEmbedder` both do their own thing internally
(ONNX session behind a `Mutex`; a blocking HTTP POST) but present the
same synchronous, batched interface to the caller above.

## Key decisions
See [docs/adr/](./docs/adr/) for the record of individual decisions and
their tradeoffs — notably
[ADR-0002](./docs/adr/0002-pluggable-embedder-trait-and-backend-crates.md),
covering the sync-vs-async trait choice, the zero-dependency core, the
batching contract, and the backend/feature-flag split.

## Non-goals
Explicitly out of scope for this crate (and left to whatever depends on
it — e.g. `rusty_knowledge`, closing its own issue #18):
- Correlating a returned vector back to a caller's row id.
- Fusing full-text and vector search rankings (hybrid retrieval).
- Declaring or choosing a retrieval mode.
- Voyage-AI-specific or other bespoke-protocol embedding backends — v1
  ships one local backend and one HTTP-generic backend; a bespoke-
  protocol backend is a future, additive crate if something needs one,
  not part of this repo's current scope.
