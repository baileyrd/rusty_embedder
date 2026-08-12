use crate::embedder::Embedder;
use crate::error::Result;

/// A no-op [`Embedder`] that produces no vectors.
///
/// `rusty-embedder-core`'s zero-dependency default (the equivalent of
/// `knowledge-mcp`'s Python `NullEmbedder`): any consumer can depend on
/// this crate alone, construct a `NullEmbedder`, and stay fully offline -
/// no model download, no network call, no API key - until (if ever) they
/// opt into a real backend crate (`rusty-embedder-local`,
/// `rusty-embedder-http`).
#[derive(Debug, Default, Clone, Copy)]
pub struct NullEmbedder;

impl NullEmbedder {
    /// Creates a new `NullEmbedder`. Equivalent to `NullEmbedder` (a unit
    /// struct); provided so callers can write `NullEmbedder::new()`
    /// alongside other backends' constructors.
    pub fn new() -> Self {
        Self
    }
}

impl Embedder for NullEmbedder {
    fn dimension(&self) -> usize {
        0
    }

    fn model_name(&self) -> &str {
        "null"
    }

    fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_is_zero() {
        assert_eq!(NullEmbedder.dimension(), 0);
    }

    #[test]
    fn model_name_is_null() {
        assert_eq!(NullEmbedder.model_name(), "null");
    }

    #[test]
    fn embed_returns_empty_regardless_of_input() {
        let texts = vec!["a".to_string(), "b".to_string()];
        assert_eq!(
            NullEmbedder::new().embed(&texts).unwrap(),
            Vec::<Vec<f32>>::new()
        );
    }

    #[test]
    fn embed_handles_empty_batch_without_panicking() {
        assert_eq!(
            NullEmbedder::new().embed(&[]).unwrap(),
            Vec::<Vec<f32>>::new()
        );
    }
}
