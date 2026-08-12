//! Demonstrates swapping the concrete [`Embedder`] behind a single call
//! site, the same way `rusty_search`'s `pluggable_backends` example swaps
//! `SearchBackend` implementations.
//!
//! Run with `cargo run -p rusty-embedder --example pluggable_embedders`.

use std::sync::Arc;

use rusty_embedder::{EmbedError, Embedder, NullEmbedder, Result};

/// A toy `Embedder` standing in for a real backend (`LocalEmbedder`,
/// `HttpEmbedder`, or your own) - it hashes each text into a fixed-size
/// vector instead of doing real inference, just to show a second
/// implementation plugging into the same call site.
struct ToyHashEmbedder;

impl Embedder for ToyHashEmbedder {
    fn dimension(&self) -> usize {
        4
    }

    fn model_name(&self) -> &str {
        "toy-hash"
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.iter().any(|t| t.is_empty()) {
            return Err(EmbedError::InvalidInput("empty text in batch".into()));
        }
        Ok(texts
            .iter()
            .map(|text| {
                let dimension = self.dimension();
                let mut vector = vec![0.0f32; dimension];
                for (i, byte) in text.bytes().enumerate() {
                    vector[i % dimension] += byte as f32;
                }
                vector
            })
            .collect())
    }
}

fn embed_and_report(embedder: &dyn Embedder, texts: &[String]) {
    println!("model: {}", embedder.model_name());
    match embedder.embed(texts) {
        Ok(vectors) => {
            for (text, vector) in texts.iter().zip(vectors.iter()) {
                println!("  {text:?} -> {vector:?}");
            }
        }
        Err(err) => println!("  embed failed: {err}"),
    }
}

fn main() {
    let texts = vec!["hello world".to_string(), "rusty embedder".to_string()];

    // Everything below this line is identical regardless of which
    // Embedder is plugged in - swap `NullEmbedder` for
    // `rusty_embedder::LocalEmbedder::new()?` (feature `local`) or
    // `rusty_embedder::HttpEmbedder::openai(...)?` (feature `http`) and
    // nothing else changes.
    let embedders: Vec<Arc<dyn Embedder>> =
        vec![Arc::new(NullEmbedder::new()), Arc::new(ToyHashEmbedder)];

    for embedder in embedders {
        embed_and_report(embedder.as_ref(), &texts);
    }
}
