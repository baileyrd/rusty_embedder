# ADR-0002: Synchronous, batched `Embedder` trait over a zero-dependency core, with backend crates split like `rusty_search`

Status: Accepted
Date: 2026-08-12

## Context
`rusty_embedder` exists to give the `rusty_search`/`rusty_knowledge` side
of the platform a standard interface for text embedding: application code
(starting with `rusty_knowledge`, closing its own issue #18) should
depend on one trait while the concrete model underneath - a local ONNX
model, a remote OpenAI-compatible API, a no-op in tests - stays
swappable, without touching call sites. It's meant to be a direct Rust
equivalent of `knowledge-mcp`'s Python `Embedder` protocol (`dimension`,
`model_name`, `embed(texts) -> vectors`), and to be modeled on
`rusty_search`'s workspace shape: a dependency-free core trait crate,
swappable backend crates, and a facade crate gating each backend behind
its own feature flag.

That requires deciding, up front: sync or async trait methods, how much
(if anything) the core crate depends on, what the batching contract looks
like, what a zero-dependency default is, and how stored vectors are laid
out on disk for the first real consumer.

## Decision
- Define a single trait, `Embedder`, in a **zero-mandatory-dependency**
  `rusty-embedder-core` crate: `dimension() -> usize`, `model_name() ->
  &str`, `embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>`. No
  crate from crates.io is a dependency of `rusty-embedder-core` - a
  hand-written `EmbedError` enum with a manual `Display`/`Error` impl
  stands in for `thiserror`, so depending on this crate alone (to define
  a custom `Embedder`, or write backend-agnostic application code) pulls
  in nothing else and requires no network at build or run time.
- Make `embed` **synchronous**, not `async fn`/`#[async_trait]` like
  `rusty_search`'s `SearchBackend`. A batched call is the whole unit of
  work a caller pays for here; a sync signature lets each backend pick
  its own concurrency model internally (local ONNX inference has none to
  pick; the HTTP backend uses `reqwest::blocking`) without forcing every
  caller - including ones with no async runtime at all - onto one just to
  produce a vector.
- Batch by construction: `&[String]` in, `Vec<Vec<f32>>` out, one vector
  per input in the same order. No single-text convenience method - unlike
  `SearchBackend::index`/`index_batch`, there's no meaningfully cheaper
  single-item path for an embedding call, so adding one would just be a
  second thing for every backend to implement in lockstep.
- Ship `NullEmbedder` in `rusty-embedder-core` itself (not a separate
  crate) as the zero-dependency default: `dimension() -> 0`, `embed()`
  always returns an empty `Vec` regardless of input - matching
  `knowledge-mcp`'s Python `NullEmbedder` exactly, including returning
  `[]` rather than one empty vector per input text.
- Include `serialize_f32`/`deserialize_f32` in the core crate, packing/
  unpacking `f32` slices as contiguous little-endian bytes with no length
  prefix - the exact on-disk layout `sqlite-vec`'s `vec0` virtual tables
  expect, and the exact layout `knowledge-mcp`'s Python
  `struct.pack(f"<{n}f", ...)` already produces. This crate's first real
  consumer, `rusty_knowledge`, stores vectors straight into a `vec0`
  column; getting this byte-for-byte right here means it doesn't have to
  reimplement or re-verify the layout itself.
- Split backends into their own crates behind their own facade feature
  flags, exactly like `rusty_search`: `rusty-embedder-local` (local
  inference via `fastembed-rs`, bundling ONNX Runtime and a small
  MiniLM-class model - chosen over `candle`/`ort`/`rust-bert` for having
  the least glue code of the local options; those three are the
  documented fallback if `fastembed-rs`'s model selection or licensing
  stops fitting) and `rusty-embedder-http` (a generic client for any
  OpenAI-compatible embeddings endpoint via `reqwest::blocking`, covering
  OpenAI and anything speaking the same request/response shape - no
  Voyage-AI-specific or other bespoke-protocol backend in v1). Both are
  gated behind their own feature (`local`, `http`) on the `rusty-embedder`
  facade crate, off by default, mirroring `memory`/`tantivy`/
  `elasticsearch` on `rusty-search`.

## Alternatives considered
- **Async trait (`#[async_trait]`), matching `SearchBackend`.** Rejected:
  `SearchBackend` needs async because remote search backends are
  I/O-bound behind a runtime-swappable `Arc<dyn SearchBackend>` used
  throughout an async application. `Embedder` doesn't share that
  requirement strongly enough to justify making every embedding-only
  consumer (e.g. a CLI ingesting rows in a tight loop) pull in an async
  runtime it otherwise wouldn't need. Backends that are naturally async
  (the HTTP one) can still be async internally; they just don't leak that
  onto the trait.
- **`thiserror`/`anyhow` in `rusty-embedder-core`**, matching
  `rusty-search-core`'s actual dependency list (`async-trait`, `serde`,
  `serde_json`, `thiserror`, `anyhow`) despite that crate's "backend-
  agnostic types" framing. Rejected specifically because this task calls
  for a *literally* zero-mandatory-dependency core crate - a stricter bar
  than `rusty-search-core` sits at today - so `EmbedError` is hand-rolled
  instead.
- **One vector per text from `NullEmbedder`** (`vec![vec![]; texts.len()]`
  instead of `vec![]`). Rejected in favor of matching the Python
  `Embedder` protocol's actual behavior byte-for-byte, since this crate
  is explicitly meant to be that protocol's Rust equivalent.
- **A length-prefixed or otherwise self-describing vector byte format**
  for `serialize_f32`. Rejected: `sqlite-vec`'s `vec0` columns store the
  dimension in the table schema, not per-row, so a self-describing format
  would need stripping/reconstructing at every read to match what
  `vec0` actually expects on disk - added complexity in service of a
  self-description nothing here needs.
- **A Voyage AI-specific backend crate for v1**, matching
  `knowledge-mcp`'s `AnthropicEmbedder`. Deferred: one local backend and
  one HTTP-generic backend is enough surface to prove the trait shape is
  right. A Voyage (or other bespoke-protocol) backend crate is additive
  later, the same way `rusty_search` adds a new search backend - a new
  crate, no changes to `rusty-embedder-core` or existing callers - once
  something actually needs one.

## Consequences
- Adding a backend is additive: a new crate implementing `Embedder`, a
  new feature flag on the facade. Widening the shared trait itself (a new
  method, a changed signature) is not additive - it requires updating
  every existing backend.
- `rusty-embedder-local`'s `Embedder` impl holds its `fastembed::
  TextEmbedding` behind a `Mutex`, because `TextEmbedding::embed` takes
  `&mut self` while `Embedder::embed` takes `&self` (so callers can share
  one `Arc<dyn Embedder>` across threads without wrapping it themselves).
  That serializes concurrent `embed` calls through one model instance;
  acceptable for v1, revisit (e.g. a small pool of loaded models) if
  local-backend throughput under concurrent callers becomes a real
  bottleneck.
- `rusty-embedder-http`'s API key is read from an environment variable
  once, at construction, and is never interpolated into any
  `EmbedError` message or included in the crate's `Debug` impl (which
  redacts it) - by design, not by accident, per the explicit requirement
  that it never be logged or surfaced in an error.
- Correlating a vector back to a caller's row id, fusing full-text and
  vector search rankings, and choosing a retrieval mode are explicitly
  out of scope here - `rusty_knowledge`'s job once it depends on this
  crate, not this crate's.
