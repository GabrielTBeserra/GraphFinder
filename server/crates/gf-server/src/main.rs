use anyhow::Result;
use clap::{Parser, Subcommand};

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
    Migrate,
    /// Run the HTTP API and MCP server.
    Api,
    /// Run indexing workers.
    Worker,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Command::Migrate => not_implemented("migrate"),
        Command::Api => not_implemented("api"),
        Command::Worker => not_implemented("worker"),
    }

    Ok(())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn not_implemented(command: &str) {
    println!("graphfinder-server {command}: not implemented yet");
}
