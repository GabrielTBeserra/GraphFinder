use fastembed::{
    EmbeddingModel, InitOptions, InitOptionsUserDefined, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct EmbeddingEngine {
    model: Arc<Mutex<TextEmbedding>>,
}

impl EmbeddingEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut local_dir = None;

        // 1. Explicit env var
        if let Ok(m) = std::env::var("GRAPHFINDER_MODELS") {
            let p = PathBuf::from(m);
            if p.join("model.onnx").exists() {
                local_dir = Some(p);
            }
        }

        // 2. Relative to GRAPHFINDER_VAULT parent
        if local_dir.is_none() {
            if let Ok(v) = std::env::var("GRAPHFINDER_VAULT") {
                if let Some(parent) = PathBuf::from(v).parent() {
                    let p = parent.join("models/all-MiniLM-L6-v2");
                    if p.join("model.onnx").exists() && p.join("tokenizer.json").exists() {
                        local_dir = Some(p);
                    }
                }
            }
        }

        // 3. Search candidates for local offline model files from CWD
        if local_dir.is_none() {
            let candidates = [
                PathBuf::from("models/all-MiniLM-L6-v2"),
                PathBuf::from("../models/all-MiniLM-L6-v2"),
                PathBuf::from("../../models/all-MiniLM-L6-v2"),
                PathBuf::from("../../../models/all-MiniLM-L6-v2"),
            ];

            for c in &candidates {
                if c.join("model.onnx").exists() && c.join("tokenizer.json").exists() {
                    local_dir = Some(c.clone());
                    break;
                }
            }
        }

        // 4. Also check relative to executable directory (for installed bundles and dev builds)
        if local_dir.is_none() {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    let exe_candidates = [
                        parent.join("models/all-MiniLM-L6-v2"),
                        parent.join("resources/models/all-MiniLM-L6-v2"),
                        parent.join("../models/all-MiniLM-L6-v2"),
                        parent.join("../../models/all-MiniLM-L6-v2"),
                        parent.join("../../../models/all-MiniLM-L6-v2"),
                        parent.join("../../../../models/all-MiniLM-L6-v2"),
                    ];
                    for c in &exe_candidates {
                        if c.join("model.onnx").exists() && c.join("tokenizer.json").exists() {
                            local_dir = Some(c.clone());
                            break;
                        }
                    }
                }
            }
        }

        let model = if let Some(dir) = local_dir {
            eprintln!("[GraphFinder] Carregando modelo local offline de {}...", dir.display());
            let onnx_file = std::fs::read(dir.join("model.onnx"))?;
            let tokenizer_files = TokenizerFiles {
                tokenizer_file: std::fs::read(dir.join("tokenizer.json"))?,
                config_file: std::fs::read(dir.join("config.json"))?,
                special_tokens_map_file: std::fs::read(dir.join("special_tokens_map.json"))?,
                tokenizer_config_file: std::fs::read(dir.join("tokenizer_config.json"))?,
            };
            let user_model = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);
            TextEmbedding::try_new_from_user_defined(user_model, InitOptionsUserDefined::default())?
        } else {
            eprintln!("[GraphFinder] Baixando modelo via HuggingFace Hub...");
            let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
            TextEmbedding::try_new(options)?
        };

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let texts = vec![text.to_string()];
        let model = self.model.lock().await;
        let mut embeddings = model.embed(texts, None)?;
        if let Some(emb) = embeddings.pop() {
            Ok(emb)
        } else {
            Err("No embedding generated".into())
        }
    }

    #[allow(dead_code)]
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let model = self.model.lock().await;
        let embeddings = model.embed(texts, None)?;
        Ok(embeddings)
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom <= f32::EPSILON {
        0.0
    } else {
        dot / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-6);
        assert!((cosine_similarity(&v1, &v3) - 0.0).abs() < 1e-6);
    }
}
