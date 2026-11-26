use sqlx::PgPool;
use uuid::Uuid;

use crate::models::IndexGeneration;
use crate::StorageResult;

pub async fn create_next_index_generation(
    pool: &PgPool,
    project_id: Uuid,
) -> StorageResult<IndexGeneration> {
    let mut tx = pool.begin().await?;

    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")
        .bind(project_id)
        .execute(&mut *tx)
        .await?;

    let next_generation: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(generation), 0) + 1 FROM index_generations WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(&mut *tx)
    .await?;

    let generation = sqlx::query_as::<_, IndexGeneration>(
        r#"
        INSERT INTO index_generations (project_id, generation, status, started_at)
        VALUES ($1, $2, 'running', now())
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(next_generation)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(generation)
}

pub async fn list_index_generations(
    pool: &PgPool,
    project_id: Uuid,
) -> StorageResult<Vec<IndexGeneration>> {
    Ok(sqlx::query_as::<_, IndexGeneration>(
        r#"
        SELECT * FROM index_generations
        WHERE project_id = $1
        ORDER BY generation DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?)
}

pub async fn publish_index_generation(
    pool: &PgPool,
    project_id: Uuid,
    generation: i64,
) -> StorageResult<bool> {
    let mut tx = pool.begin().await?;

    let publishing = sqlx::query(
        r#"
        UPDATE index_generations
        SET status = 'publishing'
        WHERE project_id = $1
          AND generation = $2
          AND status IN ('pending', 'running', 'failed')
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if publishing == 0 {
        tx.rollback().await?;
        return Ok(false);
    }

    sqlx::query(
        r#"
        UPDATE index_generations
        SET status = 'archived'
        WHERE project_id = $1
          AND generation <> $2
          AND status = 'published'
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE index_generations
        SET status = 'published', published_at = now(), error = NULL
        WHERE project_id = $1 AND generation = $2
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE projects
        SET active_generation = $2, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(true)
}

pub async fn activate_index_generation(
    pool: &PgPool,
    project_id: Uuid,
    generation: i64,
) -> StorageResult<bool> {
    let mut tx = pool.begin().await?;

    let reactivating = sqlx::query(
        r#"
        UPDATE index_generations
        SET status = 'published', published_at = COALESCE(published_at, now()), error = NULL
        WHERE project_id = $1
          AND generation = $2
          AND status IN ('published', 'archived')
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if reactivating == 0 {
        tx.rollback().await?;
        return Ok(false);
    }

    sqlx::query(
        r#"
        UPDATE index_generations
        SET status = 'archived'
        WHERE project_id = $1
          AND generation <> $2
          AND status = 'published'
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE projects
        SET active_generation = $2, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(true)
}

pub async fn fail_index_generation(
    pool: &PgPool,
    project_id: Uuid,
    generation: i64,
    error: &str,
) -> StorageResult<bool> {
    let rows = sqlx::query(
        r#"
        UPDATE index_generations
        SET status = 'failed', error = $3
        WHERE project_id = $1
          AND generation = $2
          AND status IN ('pending', 'running', 'publishing')
        "#,
    )
    .bind(project_id)
    .bind(generation)
    .bind(error)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub async fn delete_archived_generations_before(
    pool: &PgPool,
    project_id: Uuid,
    before_generation: i64,
) -> StorageResult<u64> {
    let rows = sqlx::query(
        r#"
        DELETE FROM index_generations
        WHERE project_id = $1
          AND status = 'archived'
          AND generation < $2
        "#,
    )
    .bind(project_id)
    .bind(before_generation)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows)
}
