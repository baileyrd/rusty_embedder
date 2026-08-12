//! `rusty_embedder`: a pluggable text-embedding interface for Rust.
//!
//! In the spirit of `rusty_search`'s backend-agnostic `SearchBackend`
//! (and as a Rust equivalent of `knowledge-mcp`'s Python `Embedder`
//! protocol), application code is written once against [`Embedder`] and
//! the concrete embedding model underneath is chosen - and swappable - at
//! construction time:
//!
//! ```
//! use std::sync::Arc;
//! use rusty_embedder::{Embedder, NullEmbedder};
//!
//! // Swap this for `rusty_embedder::LocalEmbedder::new()?` (needs the
//! // `local` feature) or `rusty_embedder::HttpEmbedder::openai(...)?`
//! // (needs the `http` feature) and every line below stays the same.
//! let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new());
//!
//! let vectors = embedder.embed(&["hello world".to_string()]).unwrap();
//! assert_eq!(vectors.len(), 0); // NullEmbedder never produces vectors
//! ```
//!
//! [`NullEmbedder`] is always available - depending on
//! `rusty-embedder-core` alone (which this facade re-exports in full)
//! pulls in nothing else. Enable the `local` feature for
//! [`LocalEmbedder`], local ONNX inference via `fastembed-rs` with no
//! network call per `embed()`, or the `http` feature for
//! [`HttpEmbedder`], a thin `reqwest`-based client for any
//! OpenAI-compatible embeddings endpoint. Neither is enabled by default.
//!
//! Out of scope for this crate: correlating a vector back to a caller's
//! row id, fusing full-text and vector search rankings, and choosing a
//! retrieval mode. Those belong to whatever depends on this crate to
//! store and query the vectors it produces (e.g. `rusty_knowledge`,
//! writing them into a `sqlite-vec` `vec0` column) - not to
//! `rusty_embedder` itself.

pub use rusty_embedder_core::*;

#[cfg(feature = "local")]
pub use rusty_embedder_local::LocalEmbedder;

#[cfg(feature = "http")]
pub use rusty_embedder_http::HttpEmbedder;
