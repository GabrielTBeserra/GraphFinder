use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Chunk, Document, IndexGeneration, NewChunk, NewDocument, NewProject, NewSource, Project,
    ProjectUpdate, Source, SourceUpdate,
};
use crate::StorageResult;

pub async fn create_project(pool: &PgPool, project: NewProject) -> StorageResult<Project> {
    Ok(sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (tenant_id, slug, name, description, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(project.tenant_id)
    .bind(project.slug)
    .bind(project.name)
    .bind(project.description)
    .bind(project.created_by)
    .fetch_one(pool)
    .await?)
}

pub async fn get_project(pool: &PgPool, id: Uuid) -> StorageResult<Option<Project>> {
    Ok(
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn list_projects(pool: &PgPool, tenant_id: Uuid) -> StorageResult<Vec<Project>> {
    Ok(sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?)
}

pub async fn update_project(
    pool: &PgPool,
    id: Uuid,
    update: ProjectUpdate,
) -> StorageResult<Option<Project>> {
    Ok(sqlx::query_as::<_, Project>(
        r#"
        UPDATE projects
        SET name = COALESCE($2, name),
            description = COALESCE($3, description),
            updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(update.name)
    .bind(update.description)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_project(pool: &PgPool, id: Uuid) -> StorageResult<bool> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn create_source(pool: &PgPool, source: NewSource) -> StorageResult<Source> {
    Ok(sqlx::query_as::<_, Source>(
        r#"
        INSERT INTO sources (
            project_id, kind, uri, branch, credential_ref,
            include_patterns, exclude_patterns, config
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(source.project_id)
    .bind(source.kind)
    .bind(source.uri)
    .bind(source.branch)
    .bind(source.credential_ref)
    .bind(source.include_patterns)
    .bind(source.exclude_patterns)
    .bind(source.config)
    .fetch_one(pool)
    .await?)
}

pub async fn get_source(pool: &PgPool, id: Uuid) -> StorageResult<Option<Source>> {
    Ok(
        sqlx::query_as::<_, Source>("SELECT * FROM sources WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn list_sources(pool: &PgPool, project_id: Uuid) -> StorageResult<Vec<Source>> {
    Ok(sqlx::query_as::<_, Source>(
        "SELECT * FROM sources WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?)
}

pub async fn update_source(
    pool: &PgPool,
    id: Uuid,
    update: SourceUpdate,
) -> StorageResult<Option<Source>> {
    Ok(sqlx::query_as::<_, Source>(
        r#"
        UPDATE sources
        SET uri = COALESCE($2, uri),
            branch = COALESCE($3, branch),
            credential_ref = COALESCE($4, credential_ref),
            include_patterns = COALESCE($5, include_patterns),
            exclude_patterns = COALESCE($6, exclude_patterns),
            config = COALESCE($7, config),
            updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(update.uri)
    .bind(update.branch)
    .bind(update.credential_ref)
    .bind(update.include_patterns)
    .bind(update.exclude_patterns)
    .bind(update.config)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_source(pool: &PgPool, id: Uuid) -> StorageResult<bool> {
    let result = sqlx::query("DELETE FROM sources WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn create_index_generation(
    pool: &PgPool,
    project_id: Uuid,
    generation: i64,
) -> StorageResult<IndexGeneration> {
    Ok(sqlx::query_as::<_, IndexGeneration>(
        r#"
        INSERT INTO index_generations (project_id, generation)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .fetch_one(pool)
    .await?)
}

pub async fn get_index_generation(
    pool: &PgPool,
    project_id: Uuid,
    generation: i64,
) -> StorageResult<Option<IndexGeneration>> {
    Ok(sqlx::query_as::<_, IndexGeneration>(
        "SELECT * FROM index_generations WHERE project_id = $1 AND generation = $2",
    )
    .bind(project_id)
    .bind(generation)
    .fetch_optional(pool)
    .await?)
}

pub async fn create_document(pool: &PgPool, document: NewDocument) -> StorageResult<Document> {
    Ok(sqlx::query_as::<_, Document>(
        r#"
        INSERT INTO documents (
            project_id, generation, source_id, path, document_type,
            language, title, content_hash, size_bytes, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(document.project_id)
    .bind(document.generation)
    .bind(document.source_id)
    .bind(document.path)
    .bind(document.document_type)
    .bind(document.language)
    .bind(document.title)
    .bind(document.content_hash)
    .bind(document.size_bytes)
    .bind(document.metadata)
    .fetch_one(pool)
    .await?)
}

pub async fn get_document(pool: &PgPool, id: Uuid) -> StorageResult<Option<Document>> {
    Ok(
        sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn list_documents(
    pool: &PgPool,
    project_id: Uuid,
    generation: i64,
) -> StorageResult<Vec<Document>> {
    Ok(sqlx::query_as::<_, Document>(
        "SELECT * FROM documents WHERE project_id = $1 AND generation = $2 ORDER BY path",
    )
    .bind(project_id)
    .bind(generation)
    .fetch_all(pool)
    .await?)
}

pub async fn delete_document(pool: &PgPool, id: Uuid) -> StorageResult<bool> {
    let result = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn create_chunk(pool: &PgPool, chunk: NewChunk) -> StorageResult<Chunk> {
    Ok(sqlx::query_as::<_, Chunk>(
        r#"
        INSERT INTO chunks (
            document_id, project_id, generation, chunk_index, heading_path,
            content, token_count, content_hash, start_line, end_line, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(chunk.document_id)
    .bind(chunk.project_id)
    .bind(chunk.generation)
    .bind(chunk.chunk_index)
    .bind(chunk.heading_path)
    .bind(chunk.content)
    .bind(chunk.token_count)
    .bind(chunk.content_hash)
    .bind(chunk.start_line)
    .bind(chunk.end_line)
    .bind(chunk.metadata)
    .fetch_one(pool)
    .await?)
}

pub async fn get_chunk(pool: &PgPool, id: Uuid) -> StorageResult<Option<Chunk>> {
    Ok(
        sqlx::query_as::<_, Chunk>("SELECT * FROM chunks WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn list_chunks(pool: &PgPool, document_id: Uuid) -> StorageResult<Vec<Chunk>> {
    Ok(sqlx::query_as::<_, Chunk>(
        "SELECT * FROM chunks WHERE document_id = $1 ORDER BY chunk_index",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?)
}

pub async fn delete_chunk(pool: &PgPool, id: Uuid) -> StorageResult<bool> {
    let result = sqlx::query("DELETE FROM chunks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
