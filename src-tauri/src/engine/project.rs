use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::embeddings::EmbeddingEngine;
use super::vault::{resolve_db_path, resolve_vault_path, Engine};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub db_path: String,
    pub note_count: usize,
    pub is_active: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct ProjectsMeta {
    active_project: String,
    #[serde(default)]
    custom_names: HashMap<String, String>,
    #[serde(default)]
    custom_paths: HashMap<String, String>,
    #[serde(default)]
    cached_counts: HashMap<String, usize>,
}

#[derive(Clone)]
pub struct VaultManager {
    base_dir: PathBuf,
    embeddings: Arc<EmbeddingEngine>,
    active_project_id: Arc<RwLock<String>>,
    custom_names: Arc<RwLock<HashMap<String, String>>>,
    custom_paths: Arc<RwLock<HashMap<String, String>>>,
    cached_counts: Arc<RwLock<HashMap<String, usize>>>,
    engines: Arc<RwLock<HashMap<String, Arc<Engine>>>>,
}

pub fn resolve_base_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("GRAPHFINDER_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if Path::new("package.json").exists() {
        PathBuf::from(".")
    } else if Path::new("../package.json").exists() {
        PathBuf::from("..")
    } else if let Some(data) = dirs::data_dir() {
        let p = data.join("GraphFinder");
        let _ = fs::create_dir_all(&p);
        p
    } else {
        PathBuf::from(".")
    }
}

pub fn slugify(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn clean_path_str(p: &Path) -> String {
    let s = p.to_string_lossy().to_string();
    s.trim_start_matches(r"\\?\").to_string()
}

pub fn sanitize_project_id(id: &str) -> String {
    let clean = slugify(id);
    if clean.is_empty() {
        "default".to_string()
    } else {
        clean
    }
}

impl VaultManager {
    pub fn new(
        embeddings: Arc<EmbeddingEngine>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_dir = resolve_base_dir();
        let vaults_dir = base_dir.join("vaults");
        if !vaults_dir.exists() {
            let _ = fs::create_dir_all(&vaults_dir);
        }

        let meta_file = vaults_dir.join("projects.json");
        let (active_id, custom_names, custom_paths, cached_counts) = if meta_file.exists() {
            if let Ok(data) = fs::read_to_string(&meta_file) {
                if let Ok(meta) = serde_json::from_str::<ProjectsMeta>(&data) {
                    (
                        meta.active_project,
                        meta.custom_names,
                        meta.custom_paths,
                        meta.cached_counts,
                    )
                } else {
                    (
                        "default".to_string(),
                        HashMap::new(),
                        HashMap::new(),
                        HashMap::new(),
                    )
                }
            } else {
                (
                    "default".to_string(),
                    HashMap::new(),
                    HashMap::new(),
                    HashMap::new(),
                )
            }
        } else {
            (
                "default".to_string(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            )
        };

        let manager = Self {
            base_dir,
            embeddings,
            active_project_id: Arc::new(RwLock::new(active_id)),
            custom_names: Arc::new(RwLock::new(custom_names)),
            custom_paths: Arc::new(RwLock::new(custom_paths)),
            cached_counts: Arc::new(RwLock::new(cached_counts)),
            engines: Arc::new(RwLock::new(HashMap::new())),
        };

        Ok(manager)
    }

    pub fn embeddings(&self) -> &Arc<EmbeddingEngine> {
        &self.embeddings
    }

    pub async fn active_project_id(&self) -> String {
        self.active_project_id.read().await.clone()
    }

    pub async fn active_engine(&self) -> Arc<Engine> {
        let active_id = self.active_project_id().await;
        self.get_or_create_engine(&active_id).await
    }

    pub async fn get_engine(&self, project_id: Option<&str>) -> Arc<Engine> {
        match project_id {
            Some(id) if !id.trim().is_empty() => self.get_or_create_engine(id).await,
            _ => self.active_engine().await,
        }
    }

    async fn resolve_canonical_id(&self, project_id: &str) -> String {
        let raw = project_id.trim();
        if raw.is_empty() || raw.eq_ignore_ascii_case("default") {
            return "default".to_string();
        }
        let clean = sanitize_project_id(raw);
        {
            let custom_paths = self.custom_paths.read().await;
            if custom_paths.contains_key(raw) {
                return raw.to_string();
            }
            if custom_paths.contains_key(&clean) {
                return clean;
            }
            if let Some((k, _)) = custom_paths
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(raw) || k.eq_ignore_ascii_case(&clean))
            {
                return k.clone();
            }
        }
        clean
    }

    pub async fn try_get_or_create_engine(&self, project_id: &str) -> Result<Arc<Engine>, String> {
        let id = self.resolve_canonical_id(project_id).await;

        // Check if already in cache
        {
            let map = self.engines.read().await;
            if let Some(engine) = map.get(&id) {
                return Ok(Arc::clone(engine));
            }
        }

        // Create new engine for this project vault
        let (vault_path, db_path) = self.resolve_project_paths(&id).await;

        let engine = Engine::new(&vault_path, &db_path, Arc::clone(&self.embeddings))
            .map_err(|e| format!("Failed to initialize Engine for project '{}': {}", id, e))?;
        let count = engine.cached_count();
        let engine_arc = Arc::new(engine);

        // Insert into cache
        {
            let mut map = self.engines.write().await;
            map.insert(id.clone(), Arc::clone(&engine_arc));
        }
        {
            let mut counts = self.cached_counts.write().await;
            counts.insert(id, count);
        }

        Ok(engine_arc)
    }

    pub async fn get_or_create_engine(&self, project_id: &str) -> Arc<Engine> {
        match self.try_get_or_create_engine(project_id).await {
            Ok(e) => e,
            Err(err) => {
                eprintln!("[GraphFinder Error] {}, falling back to default engine.", err);
                self.try_get_or_create_engine("default")
                    .await
                    .expect("Critical failure initializing default vault")
            }
        }
    }

    async fn resolve_project_paths(&self, project_id: &str) -> (PathBuf, PathBuf) {
        let canonical_id = self.resolve_canonical_id(project_id).await;
        if canonical_id == "default" {
            (resolve_vault_path(), resolve_db_path())
        } else {
            let custom_paths = self.custom_paths.read().await;
            if let Some(custom_path_str) = custom_paths.get(&canonical_id) {
                let p = PathBuf::from(custom_path_str);
                if !p.exists() {
                    let _ = fs::create_dir_all(&p);
                }
                let db_path = p.join("index.db");
                (p, db_path)
            } else {
                let safe_id = sanitize_project_id(&canonical_id);
                let project_vault = self.base_dir.join("vaults").join(&safe_id);
                if !project_vault.exists() {
                    let _ = fs::create_dir_all(&project_vault);
                }
                let project_db = project_vault.join("index.db");
                (project_vault, project_db)
            }
        }
    }

    async fn get_fast_project_count(&self, id: &str, vault_path: &Path, db_path: &Path) -> usize {
        // 1. Check loaded Engine atomic counter (0ns)
        {
            let engines = self.engines.read().await;
            if let Some(eng) = engines.get(id) {
                let c = eng.cached_count();
                if c > 0 {
                    return c;
                }
            }
        }
        // 2. Check existing SQLite index.db via fast read-only COUNT(*) (<0.3ms)
        if let Some(db_count) = super::db::DbManager::quick_item_count(db_path) {
            if db_count > 0 {
                let mut counts = self.cached_counts.write().await;
                counts.insert(id.to_string(), db_count);
                return db_count;
            }
        }
        // 3. Check in-memory / persisted cached_counts
        {
            let counts = self.cached_counts.read().await;
            if let Some(&c) = counts.get(id) {
                return c;
            }
        }
        // 4. Shallow bounded fallback for brand-new unindexed directories
        let c = count_notes_in_dir_bounded(vault_path);
        {
            let mut counts = self.cached_counts.write().await;
            counts.insert(id.to_string(), c);
        }
        c
    }

    pub async fn list_projects(&self) -> Vec<ProjectInfo> {
        let active = self.active_project_id().await;
        let names = self.custom_names.read().await.clone();
        let custom_paths = self.custom_paths.read().await.clone();
        let mut list = Vec::new();

        // 1. Default / Principal project
        let (def_vault, def_db) = self.resolve_project_paths("default").await;
        let def_count = self.get_fast_project_count("default", &def_vault, &def_db).await;
        list.push(ProjectInfo {
            id: "default".to_string(),
            name: names
                .get("default")
                .cloned()
                .unwrap_or_else(|| "Principal".to_string()),
            path: def_vault.to_string_lossy().to_string(),
            db_path: def_db.to_string_lossy().to_string(),
            note_count: def_count,
            is_active: active.eq_ignore_ascii_case("default"),
        });

        // 2. Custom path projects
        for (id, path_str) in custom_paths.iter() {
            if id.eq_ignore_ascii_case("default") {
                continue;
            }
            let p = PathBuf::from(path_str);
            let name = names.get(id).cloned().unwrap_or_else(|| id.clone());
            let db_path_buf = p.join("index.db");
            let count = self.get_fast_project_count(id, &p, &db_path_buf).await;

            list.push(ProjectInfo {
                id: id.clone(),
                name,
                path: p.to_string_lossy().to_string(),
                db_path: db_path_buf.to_string_lossy().to_string(),
                note_count: count,
                is_active: active.eq_ignore_ascii_case(id),
            });
        }

        // 3. Subdirectories in ./vaults that are not already listed
        let vaults_dir = self.base_dir.join("vaults");
        if let Ok(entries) = fs::read_dir(vaults_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if folder_name.is_empty()
                        || folder_name.starts_with('.')
                        || folder_name.eq_ignore_ascii_case("default")
                        || custom_paths.keys().any(|k| k.eq_ignore_ascii_case(folder_name))
                    {
                        continue;
                    }

                    let id = folder_name.to_string();
                    let name = names
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(|| folder_name.to_string());
                    let db_path_buf = path.join("index.db");
                    let count = self.get_fast_project_count(&id, &path, &db_path_buf).await;

                    list.push(ProjectInfo {
                        id: id.clone(),
                        name,
                        path: path.to_string_lossy().to_string(),
                        db_path: db_path_buf.to_string_lossy().to_string(),
                        note_count: count,
                        is_active: active.eq_ignore_ascii_case(&id),
                    });
                }
            }
        }

        list
    }

    pub async fn switch_project(&self, project_id: &str) -> Result<ProjectInfo, String> {
        self.switch_project_with_handle(project_id, None).await
    }

    pub async fn switch_project_with_handle(
        &self,
        project_id: &str,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<ProjectInfo, String> {
        let target_id = self.resolve_canonical_id(project_id).await;

        // 1. Update active project in memory and persist immediately
        {
            let mut active = self.active_project_id.write().await;
            *active = target_id.clone();
        }

        self.save_meta().await;

        // 2. Ensure engine is cached
        let engine = self.get_or_create_engine(&target_id).await;
        let initial_count = engine.cached_count();

        // 3. Sync vault in the background:
        //    If the project already has indexed records in SQLite, yield for 200ms first so
        //    the frontend's immediate get_graph / list_notes / list_code_files queries read
        //    the existing DB in <5ms with zero lock contention!
        let engine_clone = Arc::clone(&engine);
        let counts_cache = Arc::clone(&self.cached_counts);
        let tid_for_bg = target_id.clone();
        tokio::spawn(async move {
            if initial_count > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            if let Ok(changed) = engine_clone.sync_vault_with_handle(app_handle.as_ref()).await {
                let updated_total = engine_clone.cached_count();
                {
                    let mut counts = counts_cache.write().await;
                    counts.insert(tid_for_bg, updated_total);
                }
                if let Some(handle) = app_handle {
                    if initial_count == 0 || changed > 0 {
                        use tauri::Emitter;
                        let _ = handle.emit("vault-sync-complete", changed);
                    }
                }
            }
        });

        // 4. Return project info immediately in O(1)
        self.get_project_info(&target_id).await
    }

    pub async fn create_project(
        &self,
        name: &str,
        custom_path: Option<String>,
    ) -> Result<ProjectInfo, String> {
        self.create_project_with_handle(name, custom_path, None).await
    }

    pub async fn create_project_with_handle(
        &self,
        name: &str,
        custom_path: Option<String>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<ProjectInfo, String> {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return Err("Project name cannot be empty".to_string());
        }

        let slug = slugify(clean_name);
        if slug.is_empty() {
            return Err("Invalid project name".to_string());
        }

        // If a custom directory path was chosen by the user
        if let Some(path_str) = custom_path {
            let clean_path = path_str.trim();
            if !clean_path.is_empty() {
                let p = PathBuf::from(clean_path);
                if !p.exists() {
                    fs::create_dir_all(&p)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }
                let canonical = fs::canonicalize(&p).unwrap_or(p);
                let mut paths = self.custom_paths.write().await;
                paths.insert(slug.clone(), clean_path_str(&canonical));
            }
        }

        // Store custom display name
        {
            let mut names = self.custom_names.write().await;
            names.insert(slug.clone(), clean_name.to_string());
        }

        let (vault_path, _) = self.resolve_project_paths(&slug).await;

        // If folder doesn't have any markdown or code files at top levels, create an initial starter note
        let existing_count = count_notes_in_dir_bounded(&vault_path);
        if existing_count == 0 {
            let starter_note = vault_path.join(format!("{}.md", clean_name));
            if !starter_note.exists() {
                let note_body = format!(
                    "# {}\n\nKnowledge vault for project **{}**.\n\n## 1. Overview\n- This vault stores local notes and embeddings independently.\n- Connect your ideas using bidirectional wikilinks like `[[{}#1. Overview]]`.\n\nTags: #project #{}\n",
                    clean_name, clean_name, clean_name, slug
                );
                let _ = fs::write(starter_note, note_body);
            }
        }

        self.save_meta().await;
        self.switch_project_with_handle(&slug, app_handle).await
    }

    /// Checks whether directory `target_dir` is already associated with an existing project (by canonical path).
    /// If it already exists, returns `(project_id, engine, false /* is_new */)`.
    /// If not, automatically registers a new project using folder name, saves metadata, and returns `(project_id, engine, true /* is_new */)`.
    pub async fn get_or_register_project_by_path(
        &self,
        target_dir: &Path,
    ) -> Result<(String, Arc<Engine>, bool), String> {
        let abs_path = if target_dir.is_absolute() {
            target_dir.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| format!("Failed to resolve current directory: {}", e))?
                .join(target_dir)
        };

        let canonical = fs::canonicalize(&abs_path).unwrap_or(abs_path);
        let target_norm = clean_path_str(&canonical).to_lowercase().replace('\\', "/");

        // 1. Check if matches default vault
        let (def_vault, _) = self.resolve_project_paths("default").await;
        let def_norm = clean_path_str(&fs::canonicalize(&def_vault).unwrap_or(def_vault))
            .to_lowercase()
            .replace('\\', "/");
        if def_norm == target_norm {
            let engine = self.get_or_create_engine("default").await;
            return Ok(("default".to_string(), engine, false));
        }

        // 2. Verifica se coincide com algum caminho customizado já registrado
        {
            let custom_paths = self.custom_paths.read().await;
            for (id, p_str) in custom_paths.iter() {
                let p_norm = p_str.trim_start_matches(r"\\?\").to_lowercase().replace('\\', "/");
                if p_norm == target_norm {
                    let found_id = id.clone();
                    drop(custom_paths);
                    let engine = self.get_or_create_engine(&found_id).await;
                    return Ok((found_id, engine, false));
                }
            }
        }

        // 3. Verifica se coincide com alguma subpasta em ./vaults/<id>
        let vaults_dir = self.base_dir.join("vaults");
        if let Ok(entries) = fs::read_dir(&vaults_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let folder_norm = clean_path_str(&fs::canonicalize(&path).unwrap_or(path.clone()))
                        .to_lowercase()
                        .replace('\\', "/");
                    if folder_norm == target_norm {
                        if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                            let id = folder_name.to_string();
                            let engine = self.get_or_create_engine(&id).await;
                            return Ok((id, engine, false));
                        }
                    }
                }
            }
        }

        // 4. Not found: register as new project using folder name
        let folder_name = canonical
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        let base_slug = slugify(&folder_name);
        let slug = if base_slug.is_empty() {
            "project".to_string()
        } else {
            base_slug
        };

        // Ensures unique ID in case slug already exists pointing to another path
        let mut final_id = slug.clone();
        {
            let custom_paths = self.custom_paths.read().await;
            let mut counter = 1;
            while custom_paths.contains_key(&final_id) || final_id == "default" {
                final_id = format!("{}-{}", slug, counter);
                counter += 1;
            }
        }

        let canonical_str = clean_path_str(&canonical);
        {
            let mut paths = self.custom_paths.write().await;
            paths.insert(final_id.clone(), canonical_str);
        }
        {
            let mut names = self.custom_names.write().await;
            names.insert(final_id.clone(), folder_name);
        }

        self.save_meta().await;

        let engine = self.get_or_create_engine(&final_id).await;
        Ok((final_id, engine, true))
    }

    pub async fn get_project_info(&self, project_id: &str) -> Result<ProjectInfo, String> {
        let canonical_id = self.resolve_canonical_id(project_id).await;
        let active = self.active_project_id().await;
        let (vault_path, db_path) = self.resolve_project_paths(&canonical_id).await;
        let count = self
            .get_fast_project_count(&canonical_id, &vault_path, &db_path)
            .await;
        let names = self.custom_names.read().await;
        let name = if canonical_id == "default" {
            names
                .get("default")
                .cloned()
                .unwrap_or_else(|| "Principal".to_string())
        } else {
            names
                .get(&canonical_id)
                .cloned()
                .unwrap_or_else(|| canonical_id.clone())
        };

        Ok(ProjectInfo {
            id: canonical_id.clone(),
            name,
            path: vault_path.to_string_lossy().to_string(),
            db_path: db_path.to_string_lossy().to_string(),
            note_count: count,
            is_active: active.eq_ignore_ascii_case(&canonical_id),
        })
    }

    async fn save_meta(&self) {
        let active = self.active_project_id().await;
        let names = self.custom_names.read().await.clone();
        let custom_paths = self.custom_paths.read().await.clone();
        let cached_counts = self.cached_counts.read().await.clone();
        let meta = ProjectsMeta {
            active_project: active,
            custom_names: names,
            custom_paths,
            cached_counts,
        };

        let vaults_dir = self.base_dir.join("vaults");
        let _ = fs::create_dir_all(&vaults_dir);
        let meta_file = vaults_dir.join("projects.json");
        if let Ok(serialized) = serde_json::to_string_pretty(&meta) {
            let _ = fs::write(meta_file, serialized);
        }
    }
}

fn count_notes_in_dir_bounded(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && name != "node_modules"
                && name != "target"
                && name != "dist"
                && name != "build"
                && name != ".fastembed_cache"
                && name != "vendor"
                && name != "__pycache__"
                && name != "venv"
        })
        .filter_map(|e| e.ok())
        .take(200)
        .filter(|e| {
            e.file_type().is_file()
                && (e.path().extension().and_then(|s| s.to_str()) == Some("md")
                    || super::code::is_supported_code_file(e.path()))
        })
        .count()
}
