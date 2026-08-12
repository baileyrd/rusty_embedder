//! A generic REST [`Embedder`] implementation for any embeddings endpoint
//! that speaks the OpenAI embeddings request/response shape (`POST
//! {base_url}/embeddings`, `{"model": ..., "input": [...]}` in,
//! `{"data": [{"embedding": [...], "index": ...}, ...]}` out). Covers
//! OpenAI itself and any OpenAI-compatible endpoint (Azure OpenAI, a
//! local vLLM/Ollama/text-embeddings-inference server, ...) - one client,
//! not one per vendor.
//!
//! [`Embedder::embed`] is synchronous, so this backend uses
//! [`reqwest::blocking`] rather than pulling every caller onto an async
//! runtime just to embed a batch of texts.
//!
//! The API key is read from an environment variable at construction time
//! and held for the client's lifetime - never logged, and never
//! interpolated into an [`rusty_embedder_core::EmbedError`] message. A
//! manual [`std::fmt::Debug`] impl redacts it too, so an accidental
//! `{:?}` on an [`HttpEmbedder`] can't leak it either.

use std::fmt;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use rusty_embedder_core::{EmbedError, Embedder, Result};

/// The default OpenAI embeddings API base URL, used by [`HttpEmbedder::openai`].
pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

/// A generic REST [`Embedder`] for OpenAI-compatible embeddings endpoints.
pub struct HttpEmbedder {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
    dimension: usize,
}

impl HttpEmbedder {
    /// Connects to an OpenAI-compatible endpoint at `base_url` (e.g.
    /// `"http://localhost:8000/v1"`) with no `Authorization` header - a
    /// good fit for endpoints that don't require one, such as a local
    /// vLLM/Ollama-style server.
    ///
    /// `dimension` must be supplied by the caller: unlike `model_name`,
    /// most OpenAI-compatible APIs have no "describe this model" endpoint
    /// to discover it from, so it's a construction-time input rather than
    /// something this backend can query for itself.
    pub fn new(base_url: impl Into<String>, model: impl Into<String>, dimension: usize) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: None,
            model: model.into(),
            dimension,
        }
    }

    /// Like [`HttpEmbedder::new`], but reads an API key from the
    /// environment variable named `env_var` at construction time (once,
    /// not per call) and sends it as `Authorization: Bearer <key>` on
    /// every request.
    ///
    /// Returns [`EmbedError::InvalidInput`] if `env_var` isn't set -
    /// surfaced at construction, not on the first `embed` call.
    pub fn with_api_key_env(
        base_url: impl Into<String>,
        model: impl Into<String>,
        dimension: usize,
        env_var: &str,
    ) -> Result<Self> {
        let api_key = std::env::var(env_var).map_err(|_| {
            EmbedError::InvalidInput(format!("environment variable `{env_var}` is not set"))
        })?;
        Ok(Self {
            api_key: Some(api_key),
            ..Self::new(base_url, model, dimension)
        })
    }

    /// Convenience for OpenAI itself: base URL
    /// [`OPENAI_BASE_URL`], API key from the `OPENAI_API_KEY`
    /// environment variable.
    pub fn openai(model: impl Into<String>, dimension: usize) -> Result<Self> {
        Self::with_api_key_env(OPENAI_BASE_URL, model, dimension, "OPENAI_API_KEY")
    }

    /// Uses a caller-supplied [`reqwest::blocking::Client`] (for custom
    /// timeouts, proxies, TLS config, ...) instead of a default one.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }
}

impl fmt::Debug for HttpEmbedder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpEmbedder")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("dimension", &self.dimension)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

#[derive(Serialize)]
struct EmbeddingsRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingDatum>,
}

#[derive(Deserialize)]
struct EmbeddingDatum {
    embedding: Vec<f32>,
    index: usize,
}

impl Embedder for HttpEmbedder {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut request = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .json(&EmbeddingsRequest {
                model: &self.model,
                input: texts,
            });
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }

        // The underlying reqwest::Error's Display is deliberately not
        // included here: reqwest doesn't put the Authorization header in
        // it today, but this backend exists specifically to never risk an
        // API key reaching an error message, so it stays out on
        // principle rather than on a library-version guarantee.
        let response = request
            .send()
            .map_err(|_| EmbedError::Backend("request to embeddings endpoint failed".into()))?;

        let status = response.status();
        if !status.is_success() {
            return Err(EmbedError::Backend(format!(
                "embeddings endpoint returned HTTP {status}"
            )));
        }

        let mut parsed: EmbeddingsResponse = response
            .json()
            .map_err(|_| EmbedError::Backend("could not parse embeddings response".into()))?;

        parsed.data.sort_by_key(|datum| datum.index);
        Ok(parsed
            .data
            .into_iter()
            .map(|datum| datum.embedding)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // HttpEmbedder::embed uses reqwest::blocking, which panics if invoked
    // from inside an active Tokio runtime (e.g. a #[tokio::test] fn body).
    // So each test drives its own Runtime with block_on for the async
    // wiremock setup, then calls embed() from plain sync code once
    // block_on has returned control to this thread - the runtime's worker
    // threads keep serving the mock in the background either way.
    fn start_server() -> (tokio::runtime::Runtime, MockServer) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = rt.block_on(MockServer::start());
        (rt, server)
    }

    #[test]
    fn embeds_a_batch_and_reorders_by_index() {
        let (rt, server) = start_server();
        rt.block_on(
            Mock::given(method("POST"))
                .and(path("/embeddings"))
                .and(header("authorization", "Bearer test-key"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": [
                        { "embedding": [0.4, 0.5], "index": 1 },
                        { "embedding": [0.1, 0.2], "index": 0 }
                    ]
                })))
                .mount(&server),
        );

        std::env::set_var("RUSTY_EMBEDDER_TEST_KEY", "test-key");
        let embedder = HttpEmbedder::with_api_key_env(
            server.uri(),
            "test-model",
            2,
            "RUSTY_EMBEDDER_TEST_KEY",
        )
        .unwrap();
        std::env::remove_var("RUSTY_EMBEDDER_TEST_KEY");

        let vectors = embedder.embed(&["a".to_string(), "b".to_string()]).unwrap();
        assert_eq!(vectors, vec![vec![0.1, 0.2], vec![0.4, 0.5]]);
    }

    #[test]
    fn empty_batch_returns_empty_without_a_request() {
        let (_rt, server) = start_server();
        // No mock registered - a request would fail to match and the test
        // would error, proving embed() short-circuits before sending one.
        let embedder = HttpEmbedder::new(server.uri(), "test-model", 2);
        assert_eq!(embedder.embed(&[]).unwrap(), Vec::<Vec<f32>>::new());
    }

    #[test]
    fn non_success_status_becomes_a_backend_error() {
        let (rt, server) = start_server();
        rt.block_on(
            Mock::given(method("POST"))
                .and(path("/embeddings"))
                .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
                .mount(&server),
        );

        let embedder = HttpEmbedder::new(server.uri(), "test-model", 2);
        let err = embedder.embed(&["a".to_string()]).unwrap_err();
        assert!(matches!(err, EmbedError::Backend(_)));
    }

    #[test]
    fn debug_impl_redacts_the_api_key() {
        std::env::set_var("RUSTY_EMBEDDER_TEST_KEY_2", "super-secret");
        let embedder = HttpEmbedder::with_api_key_env(
            "http://example.invalid",
            "test-model",
            2,
            "RUSTY_EMBEDDER_TEST_KEY_2",
        )
        .unwrap();
        std::env::remove_var("RUSTY_EMBEDDER_TEST_KEY_2");

        let debug_output = format!("{embedder:?}");
        assert!(!debug_output.contains("super-secret"));
        assert!(debug_output.contains("<redacted>"));
    }

    #[test]
    fn missing_env_var_is_an_invalid_input_error_not_a_panic() {
        let err = HttpEmbedder::with_api_key_env(
            "http://example.invalid",
            "m",
            2,
            "RUSTY_EMBEDDER_DOES_NOT_EXIST",
        )
        .unwrap_err();
        assert!(matches!(err, EmbedError::InvalidInput(_)));
    }
}
