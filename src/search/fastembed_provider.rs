//! Real semantic embedding provider backed by the `fastembed` crate.
//!
//! This module is only compiled when the `local-embeddings` feature is enabled.
//! It downloads an ~80 MB ONNX model on first use (AllMiniLML6V2) and produces
//! 384-dimensional semantic embeddings locally — no API calls, no keys.

use std::fmt;
use std::sync::Mutex;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use super::{EmbeddingInput, EmbeddingMetadata, EmbeddingProvider, EmbeddingVector};

/// Stable provider identifier for the fastembed AllMiniLML6V2 provider.
pub const FASTEMBED_MINILM_MODEL_ID: &str = "fastembed-all-minilm-l6-v2";

/// Output vector dimension for AllMiniLML6V2.
pub const FASTEMBED_MINILM_DIMENSION: usize = 384;

/// Version label for the AllMiniLML6V2 embedding space.
pub const FASTEMBED_MINILM_VERSION: &str = "1";

/// Opaque string-backed error for `fastembed` failures.
///
/// `fastembed` returns `anyhow::Result`, and `anyhow` is not a direct
/// dependency of this crate. We capture the error message as a `String` so
/// the type satisfies `std::error::Error + Send + Sync + 'static` without
/// depending on `anyhow` directly.
#[derive(Debug)]
pub struct FastEmbedError(String);

impl fmt::Display for FastEmbedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fastembed error: {}", self.0)
    }
}

impl std::error::Error for FastEmbedError {}

/// Local semantic embedding provider using `fastembed` AllMiniLML6V2.
///
/// Wraps `TextEmbedding` in a `Mutex` because fastembed v5's `embed` method
/// takes `&mut self`, while `EmbeddingProvider::embed_batch` takes `&self`.
/// The mutex makes concurrent calls safe while preserving the shared-reference
/// interface.
///
/// # Examples
///
/// ```rust,no_run
/// use mentisdb::search::{EmbeddingInput, EmbeddingProvider};
/// use mentisdb::search::fastembed_provider::FastEmbedProvider;
///
/// let provider = FastEmbedProvider::try_new().expect("model load failed");
/// let inputs = vec![EmbeddingInput::new("doc-1", "hello world")];
/// let vectors = provider.embed_batch(&inputs).expect("embed failed");
/// assert_eq!(vectors[0].values.len(), 384);
/// ```
pub struct FastEmbedProvider {
    model: Mutex<TextEmbedding>,
    metadata: EmbeddingMetadata,
}

impl fmt::Debug for FastEmbedProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FastEmbedProvider")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

impl FastEmbedProvider {
    /// Initialize the AllMiniLML6V2 model.
    ///
    /// Downloads the model (~80 MB) from Hugging Face on first use and caches
    /// it locally. Subsequent calls load from cache and are fast.
    ///
    /// # Errors
    ///
    /// Returns `FastEmbedError` when the model cannot be downloaded or the
    /// ONNX runtime fails to initialise.
    pub fn try_new() -> Result<Self, FastEmbedError> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        )
        .map_err(|e| FastEmbedError(e.to_string()))?;
        Ok(Self {
            model: Mutex::new(model),
            metadata: EmbeddingMetadata::new(
                FASTEMBED_MINILM_MODEL_ID,
                FASTEMBED_MINILM_DIMENSION,
                FASTEMBED_MINILM_VERSION,
            ),
        })
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    type Error = FastEmbedError;

    fn metadata(&self) -> &EmbeddingMetadata {
        &self.metadata
    }

    /// Embed a batch of inputs using AllMiniLML6V2.
    ///
    /// # Errors
    ///
    /// Returns `FastEmbedError` when the ONNX runtime fails during inference,
    /// or when the internal mutex is poisoned (a previous call panicked).
    fn embed_batch(&self, inputs: &[EmbeddingInput]) -> Result<Vec<EmbeddingVector>, Self::Error> {
        let texts: Vec<&str> = inputs.iter().map(|i| i.text.as_str()).collect();
        let mut model = self
            .model
            .lock()
            .map_err(|e| FastEmbedError(format!("fastembed mutex poisoned: {e}")))?;
        let embeddings = model
            .embed(texts, None)
            .map_err(|e| FastEmbedError(e.to_string()))?;
        Ok(embeddings.into_iter().map(EmbeddingVector::new).collect())
    }
}
