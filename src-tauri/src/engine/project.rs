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
}

#[derive(Clone)]
pub struct VaultManager {
    base_dir: PathBuf,
    embeddings: Arc<EmbeddingEngine>,
    active_project_id: Arc<RwLock<String>>,
    custom_names: Arc<RwLock<HashMap<String, String>>>,
    custom_paths: Arc<RwLock<HashMap<String, String>>>,
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
        let (active_id, custom_names, custom_paths) = if meta_file.exists() {
            if let Ok(data) = fs::read_to_string(&meta_file) {
                if let Ok(meta) = serde_json::from_str::<ProjectsMeta>(&data) {
                    (meta.active_project, meta.custom_names, meta.custom_paths)
                } else {
                    ("default".to_string(), HashMap::new(), HashMap::new())
                }
            } else {
                ("default".to_string(), HashMap::new(), HashMap::new())
            }
        } else {
            ("default".to_string(), HashMap::new(), HashMap::new())
        };

        let manager = Self {
            base_dir,
            embeddings,
            active_project_id: Arc::new(RwLock::new(active_id)),
            custom_names: Arc::new(RwLock::new(custom_names)),
            custom_paths: Arc::new(RwLock::new(custom_paths)),
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

    pub async fn try_get_or_create_engine(&self, project_id: &str) -> Result<Arc<Engine>, String> {
        let id = sanitize_project_id(project_id);

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
            .map_err(|e| format!("Falha ao inicializar Engine do projeto '{}': {}", id, e))?;
        let engine_arc = Arc::new(engine);

        // Sync initial notes in background
        let engine_clone = Arc::clone(&engine_arc);
        tokio::spawn(async move {
            let _ = engine_clone.sync_vault().await;
        });

        // Insert into cache
        {
            let mut map = self.engines.write().await;
            map.insert(id, Arc::clone(&engine_arc));
        }

        Ok(engine_arc)
    }

    pub async fn get_or_create_engine(&self, project_id: &str) -> Arc<Engine> {
        match self.try_get_or_create_engine(project_id).await {
            Ok(e) => e,
            Err(err) => {
                eprintln!("[GraphFinder Error] {}, recorrendo ao engine default.", err);
                self.try_get_or_create_engine("default")
                    .await
                    .expect("Falha crítica ao inicializar o vault padrão")
            }
        }
    }

    async fn resolve_project_paths(&self, project_id: &str) -> (PathBuf, PathBuf) {
        let safe_id = sanitize_project_id(project_id);
        if safe_id == "default" {
            (resolve_vault_path(), resolve_db_path())
        } else {
            let custom_paths = self.custom_paths.read().await;
            if let Some(custom_path_str) = custom_paths.get(&safe_id) {
                let p = PathBuf::from(custom_path_str);
                if !p.exists() {
                    let _ = fs::create_dir_all(&p);
                }
                let db_path = p.join("index.db");
                (p, db_path)
            } else {
                let project_vault = self.base_dir.join("vaults").join(&safe_id);
                if !project_vault.exists() {
                    let _ = fs::create_dir_all(&project_vault);
                }
                let project_db = project_vault.join("index.db");
                (project_vault, project_db)
            }
        }
    }

    pub async fn list_projects(&self) -> Vec<ProjectInfo> {
        let active = self.active_project_id().await;
        let names = self.custom_names.read().await;
        let custom_paths = self.custom_paths.read().await;
        let mut list = Vec::new();

        // 1. Default / Principal project
        let (def_vault, def_db) = self.resolve_project_paths("default").await;
        let def_count = count_notes_in_dir(&def_vault);
        list.push(ProjectInfo {
            id: "default".to_string(),
            name: names
                .get("default")
                .cloned()
                .unwrap_or_else(|| "Principal".to_string()),
            path: def_vault.to_string_lossy().to_string(),
            db_path: def_db.to_string_lossy().to_string(),
            note_count: def_count,
            is_active: active == "default",
        });

        // 2. Custom path projects
        for (id, path_str) in custom_paths.iter() {
            if id == "default" {
                continue;
            }
            let p = PathBuf::from(path_str);
            let name = names.get(id).cloned().unwrap_or_else(|| id.clone());
            let db_path = p.join("index.db").to_string_lossy().to_string();
            let count = count_notes_in_dir(&p);

            list.push(ProjectInfo {
                id: id.clone(),
                name,
                path: p.to_string_lossy().to_string(),
                db_path,
                note_count: count,
                is_active: active == *id,
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
                        || folder_name == "default"
                        || custom_paths.contains_key(folder_name)
                    {
                        continue;
                    }

                    let id = folder_name.to_string();
                    let name = names
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(|| folder_name.to_string());
                    let db_path = path.join("index.db").to_string_lossy().to_string();
                    let count = count_notes_in_dir(&path);

                    list.push(ProjectInfo {
                        id: id.clone(),
                        name,
                        path: path.to_string_lossy().to_string(),
                        db_path,
                        note_count: count,
                        is_active: active == id,
                    });
                }
            }
        }

        list
    }

    pub async fn switch_project(&self, project_id: &str) -> Result<ProjectInfo, String> {
        let target_id = if project_id.trim().is_empty() {
            "default"
        } else {
            project_id.trim()
        };

        // Ensure engine is loaded and synchronized
        let engine = self.get_or_create_engine(target_id).await;
        let _ = engine.sync_vault().await;

        {
            let mut active = self.active_project_id.write().await;
            *active = target_id.to_string();
        }

        self.save_meta().await;

        self.get_project_info(target_id).await
    }

    pub async fn create_project(
        &self,
        name: &str,
        custom_path: Option<String>,
    ) -> Result<ProjectInfo, String> {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return Err("Nome do projeto não pode ser vazio".to_string());
        }

        let slug = slugify(clean_name);
        if slug.is_empty() {
            return Err("Nome do projeto inválido".to_string());
        }

        // If a custom directory path was chosen by the user
        if let Some(path_str) = custom_path {
            let clean_path = path_str.trim();
            if !clean_path.is_empty() {
                let p = PathBuf::from(clean_path);
                if !p.exists() {
                    fs::create_dir_all(&p)
                        .map_err(|e| format!("Falha ao criar diretório: {}", e))?;
                }
                let canonical = fs::canonicalize(&p).unwrap_or(p);
                let mut paths = self.custom_paths.write().await;
                paths.insert(slug.clone(), canonical.to_string_lossy().to_string());
            }
        }

        // Store custom display name
        {
            let mut names = self.custom_names.write().await;
            names.insert(slug.clone(), clean_name.to_string());
        }

        let (vault_path, _) = self.resolve_project_paths(&slug).await;

        // If folder doesn't have any markdown files, create an initial starter note
        let existing_count = count_notes_in_dir(&vault_path);
        if existing_count == 0 {
            let starter_note = vault_path.join(format!("{}.md", clean_name));
            if !starter_note.exists() {
                let note_body = format!(
                    "# {}\n\nCofre de conhecimento para o projeto **{}**.\n\n## 1. Visão Geral\n- Este vault armazena notas e embeddings locais de forma independente.\n- Conecte suas ideias usando wikilinks como `[[{}#1. Visão Geral]]`.\n\nTags: #projeto #{}\n",
                    clean_name, clean_name, clean_name, slug
                );
                let _ = fs::write(starter_note, note_body);
            }
        }

        self.save_meta().await;
        self.switch_project(&slug).await
    }

    pub async fn get_project_info(&self, project_id: &str) -> Result<ProjectInfo, String> {
        let projects = self.list_projects().await;
        projects
            .into_iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| format!("Projeto '{}' não encontrado", project_id))
    }

    async fn save_meta(&self) {
        let active = self.active_project_id().await;
        let names = self.custom_names.read().await.clone();
        let custom_paths = self.custom_paths.read().await.clone();
        let meta = ProjectsMeta {
            active_project: active,
            custom_names: names,
            custom_paths,
        };

        let vaults_dir = self.base_dir.join("vaults");
        let _ = fs::create_dir_all(&vaults_dir);
        let meta_file = vaults_dir.join("projects.json");
        if let Ok(serialized) = serde_json::to_string_pretty(&meta) {
            let _ = fs::write(meta_file, serialized);
        }
    }
}

fn count_notes_in_dir(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("md")
        })
        .count()
}
