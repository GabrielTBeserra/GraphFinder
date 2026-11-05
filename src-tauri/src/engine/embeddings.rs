use fastembed::{
    EmbeddingModel, InitOptions, InitOptionsUserDefined, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel, TextRerank, RerankerModel, RerankInitOptions,
};
use ort::execution_providers::{CPUExecutionProvider, ExecutionProviderDispatch};
#[cfg(any(target_os = "macos", feature = "gpu-coreml"))]
use ort::execution_providers::CoreMLExecutionProvider;
#[cfg(feature = "gpu-cuda")]
use ort::execution_providers::CUDAExecutionProvider;
#[cfg(any(windows, feature = "gpu-dml"))]
use ort::execution_providers::DirectMLExecutionProvider;
#[cfg(feature = "gpu-rocm")]
use ort::execution_providers::ROCmExecutionProvider;
use super::db::{ChunkHit, CodeChunkHit};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

/// Model identity stamped into the DB meta table so migrations can detect
/// when vault vectors were produced by a different embedder.
pub const MODEL_ID_E5_SMALL: &str = "multilingual-e5-small";
pub const MODEL_ID_MINILM: &str = "all-MiniLM-L6-v2";

/// Ordered ONNX execution-provider chain from `GRAPHFINDER_DEVICE`
/// (auto|cpu|dml|coreml|cuda|rocm). GPU providers register first and are
/// silently skipped by ONNX Runtime when unavailable; CPU is always last.
fn provider_chain_for(pref: &str) -> Vec<ExecutionProviderDispatch> {
    let mut eps: Vec<ExecutionProviderDispatch> = Vec::new();
    match pref {
        "cpu" => {}
        "dml" | "directml" => {
            #[cfg(any(windows, feature = "gpu-dml"))]
            eps.push(DirectMLExecutionProvider::default().build());
        }
        "coreml" => {
            #[cfg(any(target_os = "macos", feature = "gpu-coreml"))]
            eps.push(CoreMLExecutionProvider::default().build());
        }
        "cuda" => {
            #[cfg(feature = "gpu-cuda")]
            eps.push(CUDAExecutionProvider::default().build());
        }
        "rocm" => {
            #[cfg(feature = "gpu-rocm")]
            eps.push(ROCmExecutionProvider::default().build());
        }
        _ => {
            #[cfg(any(windows, feature = "gpu-dml"))]
            eps.push(DirectMLExecutionProvider::default().build());
            #[cfg(any(target_os = "macos", feature = "gpu-coreml"))]
            eps.push(CoreMLExecutionProvider::default().build());
            #[cfg(feature = "gpu-cuda")]
            eps.push(CUDAExecutionProvider::default().build());
            #[cfg(feature = "gpu-rocm")]
            eps.push(ROCmExecutionProvider::default().build());
        }
    }
    eps.push(CPUExecutionProvider::default().build());
    eps
}

fn provider_chain() -> Vec<ExecutionProviderDispatch> {
    let pref = std::env::var("GRAPHFINDER_DEVICE")
        .unwrap_or_else(|_| "auto".to_string())
        .to_lowercase();
    provider_chain_for(&pref)
}

fn init_options(model_name: EmbeddingModel) -> InitOptions {
    InitOptions::new(model_name).with_execution_providers(provider_chain())
}

/// Session creation prefers the provider chain; retries CPU-only on failure.
fn try_new_embedding(
    opts: InitOptions,
) -> Result<TextEmbedding, Box<dyn std::error::Error + Send + Sync>> {
    let cpu_opts = InitOptions::new(opts.model_name.clone());
    match TextEmbedding::try_new(opts) {
        Ok(m) => Ok(m),
        Err(e) => {
            eprintln!("[GraphFinder] execution-provider init failed ({e}); retrying CPU-only");
            Ok(TextEmbedding::try_new(cpu_opts)?)
        }
    }
}

fn try_new_embedding_user_defined(
    model: UserDefinedEmbeddingModel,
) -> Result<TextEmbedding, Box<dyn std::error::Error + Send + Sync>> {
    let opts = InitOptionsUserDefined::default().with_execution_providers(provider_chain());
    match TextEmbedding::try_new_from_user_defined(model.clone(), opts) {
        Ok(m) => Ok(m),
        Err(e) => {
            eprintln!("[GraphFinder] execution-provider init failed ({e}); retrying CPU-only");
            Ok(TextEmbedding::try_new_from_user_defined(
                model,
                InitOptionsUserDefined::default(),
            )?)
        }
    }
}

#[derive(Clone)]
pub struct EmbeddingEngine {
    model: Arc<Mutex<Option<TextEmbedding>>>,
    reranker: Arc<Mutex<Option<TextRerank>>>,
    status: Arc<Mutex<String>>,
    model_id: Arc<Mutex<String>>,
    /// E5-family models require "query: "/"passage: " prefixes.
    needs_prefix: Arc<Mutex<bool>>,
}

impl EmbeddingEngine {
    pub fn new_lazy() -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            reranker: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new("initializing".to_string())),
            model_id: Arc::new(Mutex::new("unknown".to_string())),
            needs_prefix: Arc::new(Mutex::new(false)),
        }
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (model, model_id, needs_prefix) = Self::load_model_internal()?;
        Ok(Self {
            model: Arc::new(Mutex::new(Some(model))),
            reranker: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new("ready".to_string())),
            model_id: Arc::new(Mutex::new(model_id.to_string())),
            needs_prefix: Arc::new(Mutex::new(needs_prefix)),
        })
    }

    /// Identity of the currently loaded embedder (used for the DB stamp).
    pub fn model_id(&self) -> String {
        self.model_id
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|_| "unknown".to_string())
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

    /// Loads the embedder and reports which model actually loaded so the
    /// engine can stamp it into the DB and decide on E5-style prefixes.
    /// Resolution order: explicit env model > local E5 dir > fastembed
    /// MultilingualE5Small (auto-download) > local MiniLM dir > MiniLM download.
    pub fn load_model_internal() -> Result<
        (TextEmbedding, &'static str, bool),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let mut local_dir = None;

        // 1. Explicit env var (honored verbatim — user picks the folder)
        if let Ok(m) = std::env::var("GRAPHFINDER_MODELS") {
            let p = PathBuf::from(m);
            if p.join("model.onnx").exists() {
                local_dir = Some(p);
            }
        }

        // 2. Search candidates for local offline model files (E5 preferred)
        if local_dir.is_none() {
            let mut candidates: Vec<PathBuf> = Vec::new();
            if let Ok(v) = std::env::var("GRAPHFINDER_VAULT") {
                if let Some(parent) = PathBuf::from(v).parent() {
                    candidates.push(parent.join("models/multilingual-e5-small"));
                    candidates.push(parent.join("models/all-MiniLM-L6-v2"));
                }
            }
            for name in ["multilingual-e5-small", "all-MiniLM-L6-v2"] {
                for base in ["models", "../models", "../../models", "../../../models"] {
                    candidates.push(PathBuf::from(format!("{}/{}", base, name)));
                }
            }
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    for name in ["multilingual-e5-small", "all-MiniLM-L6-v2"] {
                        candidates.push(parent.join(format!("models/{}", name)));
                        candidates.push(parent.join(format!("resources/models/{}", name)));
                        for up in ["..", "../..", "../../..", "../../../.."] {
                            candidates.push(parent.join(format!("{}/models/{}", up, name)));
                        }
                    }
                }
            }
            for c in &candidates {
                if c.join("model.onnx").exists() && c.join("tokenizer.json").exists() {
                    local_dir = Some(c.clone());
                    break;
                }
            }
        }

        let selected_model = std::env::var("GRAPHFINDER_MODEL")
            .or_else(|_| std::env::var("GRAPHFINDER_EMBEDDING_MODEL"))
            .unwrap_or_else(|_| "auto".to_string())
            .to_lowercase();

        let wants_minilm = matches!(
            selected_model.as_str(),
            "minilm" | "all-minilm-l6-v2" | "l6"
        );
        let wants_e5 = matches!(
            selected_model.as_str(),
            "multilingual" | "multilingual-e5-small" | "e5"
        );

        let (model, model_id, needs_prefix) = if wants_minilm {
            eprintln!("[GraphFinder] Loading legacy all-MiniLM-L6-v2 (env override)...");
            (
                try_new_embedding(init_options(EmbeddingModel::AllMiniLML6V2))?,
                MODEL_ID_MINILM,
                false,
            )
        } else if let Some(dir) = local_dir.filter(|_| !wants_e5) {
            // Local dir wins when no explicit e5 request; detect E5 by folder name
            let is_e5_dir = dir
                .file_name()
                .map(|n| n.to_string_lossy().contains("e5"))
                .unwrap_or(false);
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
            (
                try_new_embedding_user_defined(user_model)?,
                if is_e5_dir {
                    MODEL_ID_E5_SMALL
                } else {
                    MODEL_ID_MINILM
                },
                is_e5_dir,
            )
        } else {
            // Default: multilingual-e5-small via fastembed (auto-download once)
            eprintln!("[GraphFinder] Loading multilingual-e5-small (PT/EN, 384d)...");
            (
                try_new_embedding(init_options(EmbeddingModel::MultilingualE5Small))?,
                MODEL_ID_E5_SMALL,
                true,
            )
        };

        Ok((model, model_id, needs_prefix))
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
        let (model, model_id, needs_prefix) = model_res?;

        if let Ok(mut guard) = self.model.lock() {
            *guard = Some(model);
        }
        if let Ok(mut guard) = self.model_id.lock() {
            *guard = model_id.to_string();
        }
        if let Ok(mut guard) = self.needs_prefix.lock() {
            *guard = needs_prefix;
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

    /// Applies the E5 "query: "/"passage: " prefix when the loaded model
    /// requires it; returns the text unchanged for legacy models.
    fn prefixed(&self, text: String, role: &str) -> String {
        let needs = self
            .needs_prefix
            .lock()
            .map(|g| *g)
            .unwrap_or(false);
        if needs {
            format!("{}: {}", role, text)
        } else {
            text
        }
    }

    /// Embed a search query ("query: " prefix on E5 models).
    pub async fn embed_query(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        self.embed_text(&self.prefixed(text.to_string(), "query"))
            .await
    }

    /// Embed one document for indexing ("passage: " prefix on E5 models).
    pub async fn embed_passage(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        self.embed_text(&self.prefixed(text.to_string(), "passage"))
            .await
    }

    /// Embed documents for indexing in batch ("passage: " prefix on E5).
    pub async fn embed_passages(
        &self,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let prefixed: Vec<String> = texts
            .into_iter()
            .map(|t| self.prefixed(t, "passage"))
            .collect();
        self.embed_batch(prefixed).await
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
            let trimmed_texts: Vec<String> = texts
                .into_iter()
                .map(|mut s| {
                    if s.len() > 800 {
                        let mut end = 800;
                        while end > 0 && !s.is_char_boundary(end) {
                            end -= 1;
                        }
                        s.truncate(end);
                    }
                    s
                })
                .collect();
            let guard = model.lock().map_err(|e| e.to_string())?;
            let m = guard.as_ref().ok_or("Neural model not loaded")?;
            let embeddings = m.embed(trimmed_texts, Some(64))?;
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

    #[test]
    fn test_provider_chain_cpu_only() {
        let eps = provider_chain_for("cpu");
        assert_eq!(eps.len(), 1);
    }

    #[test]
    fn test_provider_chain_always_ends_with_cpu() {
        for pref in ["auto", "dml", "coreml", "cuda", "rocm", "bogus"] {
            let eps = provider_chain_for(pref);
            assert!(!eps.is_empty(), "chain for {pref} must not be empty");
        }
    }

    #[test]
    fn test_provider_chain_platform_gpu() {
        let eps = provider_chain_for("auto");
        #[cfg(windows)]
        assert_eq!(eps.len(), 2, "windows auto chain should be [DML, CPU]");
        #[cfg(target_os = "macos")]
        assert_eq!(eps.len(), 2, "macos auto chain should be [CoreML, CPU]");
        #[cfg(all(not(windows), not(target_os = "macos")))]
        assert_eq!(eps.len(), 1, "other platforms default to CPU only");
    }
}
