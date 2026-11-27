use gf_storage::models::{
    NewChunk, NewDocument, NewProject, NewSource, ProjectUpdate, SourceUpdate,
};
use gf_storage::{
    activate_index_generation, connect, create_chunk, create_document,
    create_next_index_generation, create_project, create_source, delete_chunk, delete_document,
    delete_project, delete_source, get_chunk, get_document, get_project, get_source, list_chunks,
    list_documents, list_projects, list_sources, migrate_with_pool, publish_index_generation,
    update_project, update_source,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires GF_TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn project_source_document_and_chunk_crud_work() {
    let database_url = std::env::var("GF_TEST_DATABASE_URL").expect("GF_TEST_DATABASE_URL");
    let pool = connect(&database_url).await.unwrap();
    migrate_with_pool(&pool).await.unwrap();

    let suffix = Uuid::new_v4().simple().to_string();
    let tenant_id: Uuid =
        sqlx::query_scalar("INSERT INTO tenants (slug, name) VALUES ($1, $2) RETURNING id")
            .bind(format!("tenant-{suffix}"))
            .bind("Repository test tenant")
            .fetch_one(&pool)
            .await
            .unwrap();

    let project = create_project(
        &pool,
        NewProject {
            tenant_id,
            slug: format!("project-{suffix}"),
            name: "Repository test project".into(),
            description: Some("initial".into()),
            created_by: None,
        },
    )
    .await
    .unwrap();

    assert!(get_project(&pool, project.id).await.unwrap().is_some());
    assert_eq!(list_projects(&pool, tenant_id).await.unwrap().len(), 1);

    let project = update_project(
        &pool,
        project.id,
        ProjectUpdate {
            name: Some("Repository test project updated".into()),
            description: Some("updated".into()),
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(project.name, "Repository test project updated");

    let source = create_source(
        &pool,
        NewSource {
            project_id: project.id,
            kind: "git".into(),
            uri: "https://example.com/repo.git".into(),
            branch: Some("main".into()),
            credential_ref: None,
            include_patterns: vec!["docs/**".into()],
            exclude_patterns: vec!["node_modules/**".into()],
            config: json!({}),
            auto_index_interval_seconds: None,
        },
    )
    .await
    .unwrap();

    assert!(get_source(&pool, source.id).await.unwrap().is_some());
    assert_eq!(list_sources(&pool, project.id).await.unwrap().len(), 1);

    let source = update_source(
        &pool,
        source.id,
        SourceUpdate {
            uri: None,
            branch: Some("develop".into()),
            credential_ref: None,
            include_patterns: Some(vec!["**/*.md".into()]),
            exclude_patterns: None,
            config: Some(json!({"shallow": true})),
            auto_index_interval_seconds: None,
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(source.branch.as_deref(), Some("develop"));

    let generation = create_next_index_generation(&pool, project.id)
        .await
        .unwrap();
    assert_eq!(generation.generation, 1);

    let document = create_document(
        &pool,
        NewDocument {
            project_id: project.id,
            generation: generation.generation,
            source_id: Some(source.id),
            path: "docs/guide.md".into(),
            document_type: "markdown".into(),
            language: None,
            title: Some("Guide".into()),
            content_hash: "doc-hash".into(),
            size_bytes: 120,
            metadata: json!({}),
        },
    )
    .await
    .unwrap();

    assert!(get_document(&pool, document.id).await.unwrap().is_some());
    assert_eq!(
        list_documents(&pool, project.id, generation.generation)
            .await
            .unwrap()
            .len(),
        1
    );

    let chunk = create_chunk(
        &pool,
        NewChunk {
            document_id: document.id,
            project_id: project.id,
            generation: generation.generation,
            chunk_index: 0,
            heading_path: vec!["Guide".into()],
            content: "Hello GraphFinder".into(),
            token_count: 3,
            content_hash: "chunk-hash".into(),
            start_line: Some(1),
            end_line: Some(3),
            metadata: json!({}),
        },
    )
    .await
    .unwrap();

    assert!(get_chunk(&pool, chunk.id).await.unwrap().is_some());
    assert_eq!(list_chunks(&pool, document.id).await.unwrap().len(), 1);

    assert!(
        publish_index_generation(&pool, project.id, generation.generation)
            .await
            .unwrap()
    );
    let project = get_project(&pool, project.id).await.unwrap().unwrap();
    assert_eq!(project.active_generation, generation.generation);
    assert!(
        activate_index_generation(&pool, project.id, generation.generation)
            .await
            .unwrap()
    );

    assert!(delete_chunk(&pool, chunk.id).await.unwrap());
    assert!(delete_document(&pool, document.id).await.unwrap());
    assert!(delete_source(&pool, source.id).await.unwrap());
    assert!(delete_project(&pool, project.id).await.unwrap());
    sqlx::query("DELETE FROM tenants WHERE id = $1")
        .bind(tenant_id)
        .execute(&pool)
        .await
        .unwrap();
}
