use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use gf_core::FastEmbedProvider;
use gf_indexer::{process_next_index_job, WorkerConfig};

mod api;

#[derive(Debug, Parser)]
#[command(name = "graphfinder-server")]
#[command(about = "Multi-user GraphFinder Server", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Apply database migrations.
    Migrate {
        /// PostgreSQL connection URL. Defaults to DATABASE_URL.
        #[arg(long)]
        database_url: Option<String>,
    },
    /// Create or update the initial tenant admin.
    BootstrapAdmin {
        /// PostgreSQL connection URL. Defaults to DATABASE_URL.
        #[arg(long)]
        database_url: Option<String>,
        #[arg(long, default_value = "default")]
        tenant_slug: String,
        #[arg(long, default_value = "admin@example.com")]
        email: String,
        #[arg(long, default_value = "Administrator")]
        display_name: String,
    },
    /// Run the HTTP API and MCP server.
    Api {
        /// PostgreSQL connection URL. Defaults to DATABASE_URL.
        #[arg(long)]
        database_url: Option<String>,
        /// HTTP bind address.
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: String,
    },
    /// Run indexing workers.
    Worker {
        /// PostgreSQL connection URL. Defaults to DATABASE_URL.
        #[arg(long)]
        database_url: Option<String>,
        /// Process at most one queued job and exit.
        #[arg(long)]
        once: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Command::Migrate { database_url } => {
            let database_url = database_url
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .expect("DATABASE_URL or --database-url is required");
            gf_storage::migrate(&database_url).await?;
            println!("database migrations applied");
        }
        Command::BootstrapAdmin {
            database_url,
            tenant_slug,
            email,
            display_name,
        } => bootstrap_admin(database_url, tenant_slug, email, display_name).await?,
        Command::Api { database_url, bind } => run_api(database_url, bind).await?,
        Command::Worker { database_url, once } => run_worker(database_url, once).await?,
    }

    Ok(())
}

async fn bootstrap_admin(
    database_url: Option<String>,
    tenant_slug: String,
    email: String,
    display_name: String,
) -> Result<()> {
    let database_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL or --database-url is required");
    let password = std::env::var("GF_BOOTSTRAP_ADMIN_PASSWORD")
        .expect("GF_BOOTSTRAP_ADMIN_PASSWORD is required");
    let pool = gf_storage::connect(&database_url).await?;
    let tenant_id = gf_storage::ensure_tenant(&pool, &tenant_slug, &tenant_slug).await?;
    let hash = gf_storage::hash_password(&password)?;
    let user_id = gf_storage::upsert_user(&pool, tenant_id, &email, &display_name, &hash).await?;
    gf_storage::ensure_tenant_membership(&pool, tenant_id, user_id, "owner").await?;
    println!("bootstrap admin upserted: tenant={tenant_slug} email={email} user_id={user_id}");
    Ok(())
}

async fn run_api(database_url: Option<String>, bind: String) -> Result<()> {
    let database_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL or --database-url is required");
    let pool = gf_storage::connect(&database_url).await?;
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "graphfinder api listening");
    let model_cache = std::env::var("GF_MODEL_CACHE_DIR")
        .unwrap_or_else(|_| "/var/lib/graphfinder/models".into());
    let embedder = std::sync::Arc::new(api::LazyEmbedder::new(model_cache));
    axum::serve(listener, api::router(pool, embedder)).await?;
    Ok(())
}

async fn run_worker(database_url: Option<String>, once: bool) -> Result<()> {
    let database_url = database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL or --database-url is required");
    let pool = gf_storage::connect(&database_url).await?;
    let model_cache = std::env::var("GF_MODEL_CACHE_DIR")
        .unwrap_or_else(|_| "/var/lib/graphfinder/models".into());
    let embedder = FastEmbedProvider::multilingual_e5_small(model_cache)
        .map_err(|e| anyhow::anyhow!("failed to initialize embedding provider: {e}"))?;
    let config = WorkerConfig {
        repo_cache_root: PathBuf::from(
            std::env::var("GF_REPO_CACHE_ROOT")
                .unwrap_or_else(|_| "/var/lib/graphfinder/repo-cache".into()),
        ),
        allow_private_git_hosts: env_bool("GF_ALLOW_PRIVATE_GIT_HOSTS"),
        allowed_git_hosts: std::env::var("GF_ALLOWED_GIT_HOSTS")
            .unwrap_or_default()
            .split(',')
            .filter(|v| !v.trim().is_empty())
            .map(|v| v.trim().to_string())
            .collect(),
        git_timeout: Duration::from_secs(env_u64("GF_GIT_TIMEOUT_SECS", 120)),
        max_repo_bytes: env_u64("GF_MAX_REPO_BYTES", 1_000_000_000),
    };
    let poll_interval = Duration::from_millis(env_u64("GF_WORKER_POLL_MS", 2_000));

    loop {
        match process_next_index_job(&pool, &config, &embedder).await {
            Ok(true) if once => break,
            Ok(true) => {}
            Ok(false) if once => break,
            Ok(false) => tokio::time::sleep(poll_interval).await,
            Err(error) if once => return Err(error.into()),
            Err(error) => {
                tracing::error!(error = %error, "index job failed");
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
    Ok(())
}

fn env_bool(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    if std::env::var("GF_LOG_FORMAT").as_deref() == Ok("json") {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }
}
