# rusty_embedder

A pluggable text-embedding interface for Rust — application code is
written once against the `Embedder` trait, and the concrete embedding
model underneath (local ONNX inference, a remote OpenAI-compatible API, a
no-op default) is swappable at construction time without touching call
sites. Modeled on [`rusty_search`](https://github.com/baileyrd/rusty_search)'s
workspace shape (core trait crate + swappable backend crates + a facade
crate with feature flags), and a Rust equivalent of `knowledge-mcp`'s
Python `Embedder` protocol.

## Status
Early — core trait, `NullEmbedder`, a local backend
(`rusty-embedder-local`, via `fastembed-rs`), and a generic HTTP backend
(`rusty-embedder-http`, any OpenAI-compatible embeddings endpoint) exist
and are tested. First intended consumer: `rusty_knowledge`, storing the
`f32` vectors this crate produces straight into a `sqlite-vec` `vec0`
column (closing its own issue #18). Owner: baileyrd.

## Crates
| Crate | Purpose | Mandatory deps |
| --- | --- | --- |
| `rusty-embedder-core` | The `Embedder` trait, `NullEmbedder`, `EmbedError`/`Result`, `serialize_f32`/`deserialize_f32` | **none** |
| `rusty-embedder-local` | Local inference via `fastembed-rs` (bundled ONNX Runtime + a small MiniLM-class model), no network per call | `fastembed` |
| `rusty-embedder-http` | Generic REST client for any OpenAI-compatible embeddings endpoint | `reqwest` (blocking), `serde`/`serde_json` |
| `rusty-embedder` | Facade — re-exports the trait, `NullEmbedder`, and each backend behind its own feature flag (`local`, `http`) | `rusty-embedder-core` only by default |

## Getting started
```toml
# Depend on the facade with whichever backend(s) you need.
rusty-embedder = { version = "0.1", features = ["local"] }
# or ["http"], or both — neither is on by default.
```

```rust
use std::sync::Arc;
use rusty_embedder::{Embedder, NullEmbedder};

// Swap for rusty_embedder::LocalEmbedder::new()? (feature "local") or
// rusty_embedder::HttpEmbedder::openai("text-embedding-3-small", 1536)?
// (feature "http") — every other line stays the same.
let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new());
let vectors = embedder.embed(&["hello world".to_string()])?;
```

See `crates/rusty-embedder/examples/pluggable_embedders.rs` for a runnable
version (`cargo run -p rusty-embedder --example pluggable_embedders`).

## Architecture
See [ARCHITECTURE.md](./ARCHITECTURE.md) for boundaries, key decisions, and data flow.

## Development
```bash
# Core trait + NullEmbedder — zero dependencies, zero network.
cargo test -p rusty-embedder-core

# HTTP backend — mocked with wiremock, no live API call.
cargo test -p rusty-embedder-http

# Local backend — downloads a real ONNX model on first run, so its
# network-touching tests are #[ignore]d by default.
cargo test -p rusty-embedder-local
cargo test -p rusty-embedder-local -- --ignored

# Facade with backend(s) enabled.
cargo test -p rusty-embedder --features local,http

# Whole workspace, all lint/format checks.
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

## Contributing
See [CONTRIBUTING.md](./CONTRIBUTING.md).

## Security
See [SECURITY.md](./SECURITY.md) to report a vulnerability.

## License
Internal — not for external distribution.
