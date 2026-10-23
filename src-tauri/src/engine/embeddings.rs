use fastembed::{
    EmbeddingModel, InitOptions, InitOptionsUserDefined, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel, TextRerank, RerankerModel, RerankInitOptions,
};
use super::db::{ChunkHit, CodeChunkHit};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

#[derive(Clone)]
pub struct EmbeddingEngine {
    model: Arc<Mutex<Option<TextEmbedding>>>,
    reranker: Arc<Mutex<Option<TextRerank>>>,
    status: Arc<Mutex<String>>,
}

impl EmbeddingEngine {
    pub fn new_lazy() -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            reranker: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new("initializing".to_string())),
        }
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let model = Self::load_model_internal()?;
        Ok(Self {
            model: Arc::new(Mutex::new(Some(model))),
            reranker: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new("ready".to_string())),
        })
    }

    pub fn is_ready(&self) -> bool {
        self.model.lock().map(|m| m.is_some()).unwrap_or(false)
    }

    pub fn get_status(&self) -> String {
        self.status
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    pub fn load_model_internal() -> Result<TextEmbedding, Box<dyn std::error::Error + Send + Sync>> {
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

        let selected_model = std::env::var("GRAPHFINDER_MODEL")
            .or_else(|_| std::env::var("GRAPHFINDER_EMBEDDING_MODEL"))
            .unwrap_or_else(|_| "auto".to_string())
            .to_lowercase();

        let model = if selected_model == "multilingual"
            || selected_model == "multilingual-e5-small"
            || selected_model == "e5"
        {
            eprintln!("[GraphFinder] Loading multilingual model MultilingualE5Small...");
            let options = InitOptions::new(EmbeddingModel::MultilingualE5Small);
            TextEmbedding::try_new(options)?
        } else if let Some(dir) = local_dir {
            eprintln!(
                "[GraphFinder] Loading local offline model from {}...",
                dir.display()
            );
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
            eprintln!("[GraphFinder] Loading default neural model (all-MiniLM-L6-v2)...");
            let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
            TextEmbedding::try_new(options)?
        };

        Ok(model)
    }

    pub async fn init_model(
        &self,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_ready() {
            if let Some(app) = &app_handle {
                let _ = app.emit(
                    "engine-status",
                    serde_json::json!({
                        "status": "ready",
                        "message": "Neural engine ready!"
                    }),
                );
            }
            return Ok(());
        }

        if let Some(app) = &app_handle {
            let _ = app.emit(
                "engine-status",
                serde_json::json!({
                    "status": "loading",
                    "message": "Loading embedding model (all-MiniLM-L6-v2)..."
                }),
            );
        }

        let model_res = tokio::task::spawn_blocking(move || Self::load_model_internal()).await?;
        let model = model_res?;

        if let Ok(mut guard) = self.model.lock() {
            *guard = Some(model);
        }
        if let Ok(mut st) = self.status.lock() {
            *st = "ready".to_string();
        }

        if std::env::var("GRAPHFINDER_RERANKER")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            let _ = self.init_reranker();
        }

        if let Some(app) = &app_handle {
            let _ = app.emit(
                "engine-status",
                serde_json::json!({
                    "status": "ready",
                    "message": "Motor neural pronto!"
                }),
            );
        }

        Ok(())
    }

    pub async fn ensure_initialized(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_ready() {
            self.init_model(None).await?;
        }
        Ok(())
    }

    pub async fn embed_text(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = self.embed_batch(vec![text.to_string()]).await?;
        results.pop().ok_or_else(|| "No embedding generated".into())
    }

    pub async fn embed_batch(
        &self,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        self.ensure_initialized().await?;
        let model = Arc::clone(&self.model);
        tokio::task::spawn_blocking(move || {
            let guard = model.lock().map_err(|e| e.to_string())?;
            let m = guard.as_ref().ok_or("Neural model not loaded")?;
            let embeddings = m.embed(texts, None)?;
            Ok(embeddings)
        })
        .await?
    }

    pub fn is_reranker_ready(&self) -> bool {
        self.reranker.lock().map(|r| r.is_some()).unwrap_or(false)
    }

    pub fn init_reranker(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut opt = RerankInitOptions::default();
        opt.model_name = RerankerModel::BGERerankerBase;
        opt.show_download_progress = false;
        let reranker = TextRerank::try_new(opt)?;
        if let Ok(mut lock) = self.reranker.lock() {
            *lock = Some(reranker);
        }
        Ok(())
    }

    pub fn rerank_chunks(&self, query: &str, hits: &mut Vec<ChunkHit>, limit: usize) {
        if hits.len() <= 1 {
            hits.truncate(limit);
            return;
        }

        let reranker_guard = match self.reranker.lock() {
            Ok(g) => g,
            Err(_) => {
                hits.truncate(limit);
                return;
            }
        };

        let reranker = match reranker_guard.as_ref() {
            Some(r) => r,
            None => {
                hits.truncate(limit);
                return;
            }
        };

        let docs: Vec<String> = hits
            .iter()
            .map(|h| {
                format!(
                    "{} - {}\n{}",
                    h.note_title,
                    h.heading.as_deref().unwrap_or(""),
                    h.content
                )
            })
            .collect();

        let doc_refs: Vec<&str> = docs.iter().map(|s| s.as_str()).collect();
        if let Ok(rerank_results) = reranker.rerank(query, doc_refs, false, Some(hits.len())) {
            let old_hits = std::mem::take(hits);
            let mut reordered = Vec::with_capacity(rerank_results.len());
            for r in rerank_results {
                if let Some(mut hit) = old_hits.get(r.index).cloned() {
                    hit.score = r.score;
                    reordered.push(hit);
                }
            }
            reordered.truncate(limit);
            *hits = reordered;
        }
    }

    pub fn rerank_code_chunks(&self, query: &str, hits: &mut Vec<CodeChunkHit>, limit: usize) {
        if hits.len() <= 1 {
            hits.truncate(limit);
            return;
        }

        let reranker_guard = match self.reranker.lock() {
            Ok(g) => g,
            Err(_) => {
                hits.truncate(limit);
                return;
            }
        };

        let reranker = match reranker_guard.as_ref() {
            Some(r) => r,
            None => {
                hits.truncate(limit);
                return;
            }
        };

        let docs: Vec<String> = hits
            .iter()
            .map(|h| {
                format!(
                    "{} ({}) - {}\n{}",
                    h.file_name,
                    h.language,
                    h.symbol_name.as_deref().unwrap_or("block"),
                    h.content
                )
            })
            .collect();

        let doc_refs: Vec<&str> = docs.iter().map(|s| s.as_str()).collect();
        if let Ok(rerank_results) = reranker.rerank(query, doc_refs, false, Some(hits.len())) {
            let old_hits = std::mem::take(hits);
            let mut reordered = Vec::with_capacity(rerank_results.len());
            for r in rerank_results {
                if let Some(mut hit) = old_hits.get(r.index).cloned() {
                    hit.score = r.score;
                    reordered.push(hit);
                }
            }
            reordered.truncate(limit);
            *hits = reordered;
        }
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
    // Fast-path for L2 unit-normalized vectors (standard in all-MiniLM-L6-v2)
    if (norm_a - 1.0).abs() < 1e-3 && (norm_b - 1.0).abs() < 1e-3 {
        return dot;
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

    #[test]
    fn test_reranker_configuration() {
        let engine = EmbeddingEngine::new_lazy();
        assert!(!engine.is_reranker_ready());
    }
}
