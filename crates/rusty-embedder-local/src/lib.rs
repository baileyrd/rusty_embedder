//! A local, no-network-at-runtime [`Embedder`] implementation backed by
//! [`fastembed`](https://docs.rs/fastembed) (bundled ONNX Runtime plus a
//! small MiniLM-class model).
//!
//! Model weights are downloaded once, on first construction, to
//! `fastembed`'s on-disk cache; after that, [`LocalEmbedder::embed`] runs
//! entirely in-process - no API key, no per-call network request.
//!
//! `fastembed-rs` was picked over `candle`/`ort`/`rust-bert` for v1
//! because it bundles model download, tokenizer, and ONNX session setup
//! behind one constructor call - the least glue code of the local
//! options. If its model selection or licensing doesn't fit a future use
//! case, those three are the documented alternatives; add a new local
//! backend the same way `rusty_search` adds a new search backend crate -
//! additively, alongside this one, without touching
//! `rusty-embedder-core` or existing callers.

use std::sync::Mutex;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use rusty_embedder_core::{EmbedError, Embedder, Result};

/// A local [`Embedder`] backed by `fastembed-rs`.
///
/// `TextEmbedding::embed` takes `&mut self`, but [`Embedder::embed`] takes
/// `&self` (so callers can share one `Arc<dyn Embedder>` across threads
/// without wrapping it themselves) - the loaded model is held behind a
/// [`Mutex`] to bridge the two. Not [`Clone`]/[`Copy`]; wrap in an `Arc`
/// to share one loaded model across callers.
pub struct LocalEmbedder {
    model: Mutex<TextEmbedding>,
    model_name: String,
    dimension: usize,
}

impl LocalEmbedder {
    /// Loads the default model (`AllMiniLML6V2`), downloading its weights
    /// to `fastembed`'s cache directory on first use.
    pub fn new() -> Result<Self> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }

    /// Loads a specific `fastembed` model. See
    /// [`TextEmbedding::list_supported_models`] for the full catalog.
    pub fn with_model(model: EmbeddingModel) -> Result<Self> {
        let info = TextEmbedding::get_model_info(&model)
            .map_err(|e| EmbedError::Backend(e.to_string()))?;
        let model_name = info.model_code.clone();
        let dimension = info.dim;

        let text_embedding = TextEmbedding::try_new(InitOptions::new(model))
            .map_err(|e| EmbedError::Backend(e.to_string()))?;

        Ok(Self {
            model: Mutex::new(text_embedding),
            model_name,
            dimension,
        })
    }
}

impl Embedder for LocalEmbedder {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let documents: Vec<&str> = texts.iter().map(String::as_str).collect();
        let mut model = self
            .model
            .lock()
            .map_err(|_| EmbedError::Backend("embedding model lock was poisoned".into()))?;
        model
            .embed(documents, None)
            .map_err(|e| EmbedError::Backend(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Downloads a real ONNX model on first run, so this is opt-in rather
    /// than part of the default `cargo test` run: `cargo test --features
    /// local -- --ignored` from the facade crate, or `cargo test -p
    /// rusty-embedder-local -- --ignored` directly.
    #[test]
    #[ignore = "downloads a real ONNX model on first run - needs network"]
    fn embeds_a_batch_and_matches_declared_dimension() {
        let embedder = LocalEmbedder::new().unwrap();
        let texts = vec!["hello world".to_string(), "rusty embedder".to_string()];
        let vectors = embedder.embed(&texts).unwrap();
        assert_eq!(vectors.len(), 2);
        assert_eq!(vectors[0].len(), embedder.dimension());
        assert_eq!(vectors[1].len(), embedder.dimension());
    }

    #[test]
    #[ignore = "downloads a real ONNX model on first run - needs network"]
    fn empty_batch_returns_empty_without_calling_the_model() {
        let embedder = LocalEmbedder::new().unwrap();
        assert_eq!(embedder.embed(&[]).unwrap(), Vec::<Vec<f32>>::new());
    }
}
