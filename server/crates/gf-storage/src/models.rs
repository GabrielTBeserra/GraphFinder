use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub active_generation: i64,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewProject {
    pub tenant_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Source {
    pub id: Uuid,
    pub project_id: Uuid,
    pub kind: String,
    pub uri: String,
    pub branch: Option<String>,
    pub credential_ref: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub config: Value,
    pub auto_index_interval_seconds: Option<i32>,
    pub next_index_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewSource {
    pub project_id: Uuid,
    pub kind: String,
    pub uri: String,
    pub branch: Option<String>,
    pub credential_ref: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub config: Value,
    pub auto_index_interval_seconds: Option<i32>,
}

pub struct SourceUpdate {
    pub uri: Option<String>,
    pub branch: Option<String>,
    pub credential_ref: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub config: Option<Value>,
    pub auto_index_interval_seconds: Option<Option<i32>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct IndexGeneration {
    pub project_id: Uuid,
    pub generation: i64,
    pub status: String,
    pub embedding_model: String,
    pub embedding_dimensions: i32,
    pub source_revisions: Value,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub project_id: Uuid,
    pub generation: i64,
    pub source_id: Option<Uuid>,
    pub path: String,
    pub document_type: String,
    pub language: Option<String>,
    pub title: Option<String>,
    pub content_hash: String,
    pub size_bytes: i64,
    pub metadata: Value,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewDocument {
    pub project_id: Uuid,
    pub generation: i64,
    pub source_id: Option<Uuid>,
    pub path: String,
    pub document_type: String,
    pub language: Option<String>,
    pub title: Option<String>,
    pub content_hash: String,
    pub size_bytes: i64,
    pub metadata: Value,
}

#[derive(Debug, Clone, FromRow)]
pub struct Chunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub project_id: Uuid,
    pub generation: i64,
    pub chunk_index: i32,
    pub heading_path: Vec<String>,
    pub content: String,
    pub token_count: i32,
    pub content_hash: String,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

pub struct NewChunk {
    pub document_id: Uuid,
    pub project_id: Uuid,
    pub generation: i64,
    pub chunk_index: i32,
    pub heading_path: Vec<String>,
    pub content: String,
    pub token_count: i32,
    pub content_hash: String,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub metadata: Value,
}
