use gf_storage::models::{NewProject, NewSource};
use gf_storage::{
    acquire_index_job, cancel_index_job, complete_index_job, connect, create_next_index_generation,
    create_project, create_source, enqueue_index_job, migrate_with_pool, NewIndexJob,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires GF_TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn jobs_are_acquired_atomically_and_locked_by_source() {
    let url = std::env::var("GF_TEST_DATABASE_URL").unwrap();
    let pool = connect(&url).await.unwrap();
    migrate_with_pool(&pool).await.unwrap();
    let suffix = Uuid::new_v4().simple().to_string();
    let tenant: Uuid =
        sqlx::query_scalar("INSERT INTO tenants (slug,name) VALUES ($1,$2) RETURNING id")
            .bind(format!("jobs-{suffix}"))
            .bind("jobs")
            .fetch_one(&pool)
            .await
            .unwrap();
    let project = create_project(
        &pool,
        NewProject {
            tenant_id: tenant,
            slug: format!("jobs-{suffix}"),
            name: "jobs".into(),
            description: None,
            created_by: None,
        },
    )
    .await
    .unwrap();
    let source = create_source(
        &pool,
        NewSource {
            project_id: project.id,
            kind: "git".into(),
            uri: format!("https://example.com/{suffix}.git"),
            branch: Some("main".into()),
            credential_ref: None,
            include_patterns: vec![],
            exclude_patterns: vec![],
            config: json!({}),
            auto_index_interval_seconds: None,
        },
    )
    .await
    .unwrap();
    for _ in 0..2 {
        enqueue_index_job(
            &pool,
            NewIndexJob {
                project_id: project.id,
                source_id: Some(source.id),
                priority: 0,
                max_attempts: 2,
                payload: json!({}),
                created_by: None,
            },
        )
        .await
        .unwrap();
    }

    let first = acquire_index_job(&pool).await.unwrap().unwrap();
    assert_eq!(first.status, "running");
    assert!(acquire_index_job(&pool).await.unwrap().is_none());
    let generation = create_next_index_generation(&pool, project.id)
        .await
        .unwrap();
    assert!(complete_index_job(&pool, first.id, generation.generation)
        .await
        .unwrap());
    let second = acquire_index_job(&pool).await.unwrap().unwrap();
    assert_ne!(first.id, second.id);
    assert!(cancel_index_job(&pool, second.id).await.unwrap());
    sqlx::query("DELETE FROM tenants WHERE id=$1")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
}
