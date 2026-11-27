//! Local embedding provider abstraction.

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::PathBuf;
use std::sync::Mutex;

pub type EmbeddingResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub trait EmbeddingProvider: Send + Sync {
    fn model_id(&self) -> &str;
    fn dimensions(&self) -> usize;
    fn embed_query(&self, text: &str) -> EmbeddingResult<Vec<f32>>;
    fn embed_passages(&self, texts: &[String]) -> EmbeddingResult<Vec<Vec<f32>>>;
}

/// CPU-first local FastEmbed provider.
///
/// Model initialization downloads the model on first use unless the cache or
/// model directory is pre-provisioned by deployment automation.
pub struct FastEmbedProvider {
    model: Mutex<TextEmbedding>,
    model_id: String,
    dimensions: usize,
    batch_size: usize,
}

impl FastEmbedProvider {
    pub fn multilingual_e5_small(cache_dir: impl Into<PathBuf>) -> EmbeddingResult<Self> {
        let options =
            InitOptions::new(EmbeddingModel::MultilingualE5Small).with_cache_dir(cache_dir.into());
        let model = TextEmbedding::try_new(options)?;
        Ok(Self {
            model: Mutex::new(model),
            model_id: "multilingual-e5-small".into(),
            dimensions: 384,
            batch_size: 16,
        })
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    fn embed_raw(&self, texts: Vec<String>) -> EmbeddingResult<Vec<Vec<f32>>> {
        let batch_size = self.batch_size;
        let model = self.model.lock().map_err(|e| e.to_string())?;
        Ok(model.embed(texts, Some(batch_size))?)
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn model_id(&self) -> &str {
        &self.model_id
    }
    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn embed_query(&self, text: &str) -> EmbeddingResult<Vec<f32>> {
        let mut rows = self.embed_raw(vec![format!("query: {text}")])?;
        rows.pop()
            .ok_or_else(|| "embedding provider returned no rows".into())
    }

    fn embed_passages(&self, texts: &[String]) -> EmbeddingResult<Vec<Vec<f32>>> {
        let prefixed = texts
            .iter()
            .map(|text| format!("passage: {text}"))
            .collect();
        self.embed_raw(prefixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider;
    impl EmbeddingProvider for MockProvider {
        fn model_id(&self) -> &str {
            "mock"
        }
        fn dimensions(&self) -> usize {
            2
        }
        fn embed_query(&self, _: &str) -> EmbeddingResult<Vec<f32>> {
            Ok(vec![1.0, 0.0])
        }
        fn embed_passages(&self, texts: &[String]) -> EmbeddingResult<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![0.0, 1.0]).collect())
        }
    }

    #[test]
    fn provider_contract_is_object_safe() {
        let provider: Box<dyn EmbeddingProvider> = Box::new(MockProvider);
        assert_eq!(provider.dimensions(), 2);
        assert_eq!(provider.embed_query("hello").unwrap().len(), 2);
        assert_eq!(provider.embed_passages(&["a".into()]).unwrap().len(), 1);
    }

    #[test]
    #[ignore = "downloads the local ONNX model on first use"]
    fn fastembed_generates_e5_embeddings() {
        let dir = std::env::temp_dir().join("gf-fastembed-test");
        let provider = FastEmbedProvider::multilingual_e5_small(dir).unwrap();
        let embedding = provider.embed_query("semantic search").unwrap();
        assert_eq!(embedding.len(), 384);
    }
}
