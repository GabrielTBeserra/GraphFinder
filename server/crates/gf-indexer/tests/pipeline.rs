use gf_core::embeddings::EmbeddingResult;
use gf_core::EmbeddingProvider;
use gf_indexer::index_checkout;
use gf_storage::models::NewProject;
use gf_storage::{connect, create_project, hybrid_search, migrate_with_pool, SearchFilters};
use uuid::Uuid;

struct MockEmbedder;
impl EmbeddingProvider for MockEmbedder {
    fn model_id(&self) -> &str {
        "multilingual-e5-small"
    }
    fn dimensions(&self) -> usize {
        384
    }
    fn embed_query(&self, _: &str) -> EmbeddingResult<Vec<f32>> {
        Ok(vec![1.0; 384])
    }
    fn embed_passages(&self, texts: &[String]) -> EmbeddingResult<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| vec![1.0; 384]).collect())
    }
}

#[tokio::test]
#[ignore = "requires GF_TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn indexes_fixture_end_to_end() {
    let url = std::env::var("GF_TEST_DATABASE_URL").unwrap();
    let pool = connect(&url).await.unwrap();
    migrate_with_pool(&pool).await.unwrap();
    let suffix = Uuid::new_v4().simple().to_string();
    let tenant: Uuid =
        sqlx::query_scalar("INSERT INTO tenants (slug,name) VALUES ($1,$2) RETURNING id")
            .bind(format!("pipe-{suffix}"))
            .bind("pipe")
            .fetch_one(&pool)
            .await
            .unwrap();
    let project = create_project(
        &pool,
        NewProject {
            tenant_id: tenant,
            slug: format!("repo-{suffix}"),
            name: "repo".into(),
            description: None,
            created_by: None,
        },
    )
    .await
    .unwrap();

    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("guide.md"),
        "# Guide\n\n## Search\nGraphFinder semantic search.\n",
    )
    .unwrap();
    std::fs::write(
        temp.path().join("main.rs"),
        "use std::io;\n\nfn main() { println!(\"hi\"); }\n",
    )
    .unwrap();
    let report = index_checkout(
        &pool,
        temp.path(),
        project.id,
        None,
        &[],
        &[],
        &MockEmbedder,
    )
    .await
    .unwrap();

    assert!(report.published);
    assert_eq!(report.documents_indexed, 2);
    assert!(report.chunks_indexed >= 2);
    assert_eq!(report.embeddings_written, report.chunks_indexed);
    let hits = hybrid_search(
        &pool,
        project.id,
        "GraphFinder",
        &vec![1.0; 384],
        &SearchFilters::default(),
        5,
    )
    .await
    .unwrap();
    assert!(!hits.is_empty());
    sqlx::query("DELETE FROM tenants WHERE id=$1")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
}
