//! Backend-agnostic text-embedding interface for `rusty_embedder`: the
//! standard interface that makes embedding models pluggable and
//! replaceable, in the spirit of `rusty_search`'s `SearchBackend` for
//! search engines - and a Rust equivalent of `knowledge-mcp`'s Python
//! `Embedder` protocol.
//!
//! This crate defines *only* the shared vocabulary - the [`Embedder`]
//! trait, [`EmbedError`]/[`Result`], and the sqlite-vec-compatible
//! [`serialize_f32`]/[`deserialize_f32`] helpers - plus [`NullEmbedder`],
//! a zero-op default, and no concrete model. Pick a backend crate
//! (`rusty-embedder-local` for local ONNX inference,
//! `rusty-embedder-http` for a remote OpenAI-compatible API) or implement
//! [`Embedder`] yourself to plug in a real one.
//!
//! **Zero mandatory dependencies.** Nothing above is behind a feature
//! flag and nothing pulls in a crate from crates.io - any consumer can
//! depend on `rusty-embedder-core` alone, construct a [`NullEmbedder`],
//! and stay fully offline until they opt into a real backend.
//!
//! ```
//! use rusty_embedder_core::{Embedder, NullEmbedder};
//!
//! let embedder = NullEmbedder::new();
//! assert_eq!(embedder.dimension(), 0);
//! assert!(embedder.embed(&["hello".to_string()]).unwrap().is_empty());
//! ```
//!
//! Explicitly out of scope for this crate (and the `rusty-embedder`
//! facade as a whole): correlating a vector back to a caller's row id,
//! fusing full-text and vector search rankings, and choosing a retrieval
//! mode. Those are a consumer's job - e.g. `rusty_knowledge`, which
//! stores the vectors this crate produces straight into a `sqlite-vec`
//! `vec0` column - not this crate's.

mod embedder;
mod error;
mod null;
mod vector;

pub use embedder::Embedder;
pub use error::{EmbedError, Result};
pub use null::NullEmbedder;
pub use vector::{deserialize_f32, serialize_f32};
