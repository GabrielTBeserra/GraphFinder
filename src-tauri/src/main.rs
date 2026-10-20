#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();

    // O modo stdio foi expressamente proibido e descontinuado.
    // O MCP opera exclusivamente via Streamable HTTP no endpoint http://127.0.0.1:45123/mcp
    if args.iter().any(|a| a == "--mcp" || a == "mcp") {
        eprintln!("\n=======================================================");
        eprintln!("  [GraphFinder MCP] TRANSPORTE STDIO DESCONTINUADO/PROIBIDO");
        eprintln!("=======================================================");
        eprintln!("O modo stdio ('--mcp') foi descontinuado e não é mais permitido.");
        eprintln!("O GraphFinder opera o servidor MCP exclusivamente via Streamable HTTP no endpoint:");
        eprintln!("  http://127.0.0.1:45123/mcp");
        eprintln!("\nPor favor, execute o GraphFinder normalmente (aplicativo de desktop)");
        eprintln!("e configure seu cliente MCP (Cursor, Claude Desktop, Antigravity, VS Code, etc.)");
        eprintln!("com o endpoint HTTP acima.");
        eprintln!("=======================================================\n");
        std::process::exit(1);
    }

    // Modo padrão: Janela desktop nativa do Tauri com servidor MCP HTTP embutido
    app_lib::run();
    Ok(())
}
