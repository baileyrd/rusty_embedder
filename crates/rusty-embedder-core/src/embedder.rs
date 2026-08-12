use crate::error::Result;

/// The standard interface every embedding backend plugs into.
///
/// This plays the role `rusty_search`'s `SearchBackend` plays for search
/// engines, and is a direct Rust equivalent of `knowledge-mcp`'s Python
/// `Embedder` protocol: application code is written once against
/// [`Embedder`], and the concrete model underneath - a local ONNX model,
/// a remote OpenAI-compatible API, an in-test no-op - is an implementation
/// detail selected at construction time and swappable without touching
/// call sites.
///
/// Unlike `SearchBackend`, this trait is deliberately synchronous: a
/// batched `embed` call is the unit of work callers pay for, and a sync
/// signature lets a backend choose its own concurrency model internally
/// (a blocking HTTP client, a background thread pool, pure in-process
/// inference) without forcing every caller onto an async runtime just to
/// produce a vector.
pub trait Embedder: Send + Sync {
    /// Dimensionality of the vectors this embedder produces.
    ///
    /// Fixed for the lifetime of an instance, so callers can size a
    /// storage column (e.g. a `sqlite-vec` `vec0` table) or preallocate
    /// buffers without embedding a probe text first.
    fn dimension(&self) -> usize;

    /// Identifier for the underlying model (e.g. `"all-MiniLM-L6-v2"`,
    /// `"text-embedding-3-small"`, `"null"`). Purely informational - useful
    /// for logging/metrics and for tagging stored vectors with the model
    /// that produced them.
    fn model_name(&self) -> &str;

    /// Embeds a batch of texts, returning one vector per input text, in
    /// the same order.
    ///
    /// Batched by design: callers embedding many rows at ingestion pay one
    /// call's worth of overhead (network round trip, tokenizer setup,
    /// ...) rather than one per text. An empty `texts` slice returns
    /// `Ok(Vec::new())` rather than an error - there's nothing invalid
    /// about asking for zero embeddings.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
