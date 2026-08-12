use std::error::Error as StdError;
use std::fmt;

/// The result type returned by every [`crate::Embedder`] operation (and by
/// [`crate::deserialize_f32`]).
pub type Result<T> = std::result::Result<T, EmbedError>;

/// Errors that can occur while producing or (de)serializing embeddings.
///
/// Kept to two variants deliberately: callers handling embedding failures
/// generically only need to know whether the *input* was unusable or the
/// *backend* failed to produce a result, regardless of which concrete
/// backend is plugged in.
#[derive(Debug)]
pub enum EmbedError {
    /// The input itself was unusable, independent of any backend (e.g. a
    /// byte blob passed to [`crate::deserialize_f32`] whose length isn't a
    /// multiple of 4). Backends may also return this for input they can
    /// detect is invalid before making a call (e.g. an empty string a
    /// remote API is known to reject).
    InvalidInput(String),

    /// The underlying model or service failed to produce embeddings
    /// (network error, non-2xx HTTP response, ONNX runtime failure, ...).
    /// Backends should stringify their own error type into this variant
    /// rather than exposing it directly, so callers written against
    /// `rusty-embedder-core` get one uniform error type regardless of
    /// which backend is plugged in - and so backend-specific error
    /// payloads (which may echo request details) never propagate as-is
    /// into a message a caller might log.
    Backend(String),
}

impl fmt::Display for EmbedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbedError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            EmbedError::Backend(msg) => write!(f, "embedding backend error: {msg}"),
        }
    }
}

impl StdError for EmbedError {}
