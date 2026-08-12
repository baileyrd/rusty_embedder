# Changelog

All notable changes to this repo are documented here.
Format: Added / Changed / Deprecated / Removed / Fixed / Security, newest first.

## [Unreleased]
### Added
- `rusty-embedder-core`: the `Embedder` trait, `NullEmbedder`, `EmbedError`/
  `Result`, and `serialize_f32`/`deserialize_f32` (sqlite-vec `vec0`-compatible
  little-endian `f32` byte layout). Zero mandatory dependencies.
- `rusty-embedder-local`: `LocalEmbedder`, local ONNX inference via
  `fastembed-rs` (bundled ONNX Runtime + a small MiniLM-class model).
- `rusty-embedder-http`: `HttpEmbedder`, a generic REST client for any
  OpenAI-compatible embeddings endpoint, via `reqwest::blocking`. API key
  read from an environment variable; never logged, never in an error message.
- `rusty-embedder`: the facade crate, re-exporting the core API plus
  `LocalEmbedder`/`HttpEmbedder` behind `local`/`http` feature flags.
### Changed
### Fixed
### Security

<!-- ## [0.1.0] - YYYY-MM-DD
### Added
- Initial release -->
