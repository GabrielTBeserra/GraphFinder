// In-process CLI dispatcher. When the GraphFinder GUI is already running,
// external `graphfinder <cmd>` invocations are forwarded here over
// `POST /api/cli` so they execute against the SAME VaultManager/engines the
// interface uses — results appear in the UI immediately and no second
// process ever touches the vault.

use crate::engine::project::VaultManager;
use crate::instance::CliProxyResponse;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Deserialize)]
pub struct CliExecRequest {
    pub args: Vec<String>,
    pub cwd: String,
}

fn clean(p: &Path) -> String {
    p.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// Resolve the target vault directory for a command. `--path <p>` wins,
/// then — only when `allow_positional` is set — the first positional after
/// the subcommand (for `search`/`path`/`explain` positionals are queries
/// or node names, never directories), then `cwd`.
fn resolve_target(args: &[String], cwd: &Path, allow_positional: bool) -> PathBuf {
    let raw = arg_value(args, "--path")
        .or_else(|| {
            if allow_positional {
                args.iter()
                    .skip(2)
                    .find(|a| !a.starts_with("--"))
                    .cloned()
            } else {
                None
            }
        })
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.to_path_buf());
    let abs = if raw.is_absolute() {
        raw
    } else {
        cwd.join(raw)
    };
    std::fs::canonicalize(&abs).unwrap_or(abs)
}

fn ok(stdout: String) -> CliProxyResponse {
    CliProxyResponse {
        code: 0,
        stdout,
        stderr: String::new(),
    }
}

fn fail(code: i32, stderr: String) -> CliProxyResponse {
    CliProxyResponse {
        code,
        stdout: String::new(),
        stderr,
    }
}

pub async fn execute(
    args: Vec<String>,
    cwd: PathBuf,
    manager: Arc<VaultManager>,
    app_handle: Option<AppHandle>,
) -> CliProxyResponse {
    let cmd = args.get(1).cloned().unwrap_or_default();

    // Commands that auto-register a project may write projects.json — refresh
    // the in-memory metadata and notify the interface afterwards.
    let result = dispatch(&cmd, &args, &cwd, &manager, &app_handle).await;
    if let Some(handle) = &app_handle {
        manager.reload_meta().await;
        let _ = handle.emit("projects-changed", ());
    }
    result
}

async fn dispatch(
    cmd: &str,
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
    app_handle: &Option<AppHandle>,
) -> CliProxyResponse {
    match cmd {
        "sync" | "syn" | "index" => cmd_sync(args, cwd, manager, app_handle).await,
        "status" | "info" => cmd_status(args, cwd, manager).await,
        "projects" | "list" => cmd_projects(manager).await,
        "search" => cmd_search(args, cwd, manager).await,
        "path" => cmd_path(args, cwd, manager).await,
        "explain" => cmd_explain(args, cwd, manager).await,
        "export" => cmd_export(args, cwd, manager).await,
        "report" | "audit" => cmd_report(args, cwd, manager, app_handle).await,
        _ => fail(
            2,
            format!(
                "Command '{}' cannot be delegated to the running GraphFinder instance.\n\
                 Close the GUI (or its tray icon) and retry.\n",
                cmd
            ),
        ),
    }
}

async fn cmd_sync(
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
    app_handle: &Option<AppHandle>,
) -> CliProxyResponse {
    let target = resolve_target(args, cwd, true);
    let is_json = args.iter().any(|a| a == "--json");

    let (project_id, engine, is_new) = match manager.get_or_register_project_by_path(&target).await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error registering project: {}\n", e)),
    };

    let start = std::time::Instant::now();
    let sync_result = match app_handle {
        Some(h) => engine.sync_vault_with_handle(Some(h)).await,
        None => engine.sync_vault().await,
    };
    let elapsed = start.elapsed();

    match sync_result {
        Ok(changed) => {
            if is_json {
                let stats = engine.get_vault_stats().await.ok();
                let out = serde_json::json!({
                    "status": "success",
                    "project_id": project_id,
                    "is_new": is_new,
                    "changed_items": changed,
                    "elapsed_ms": elapsed.as_millis(),
                    "stats": stats,
                });
                ok(format!("{}\n", serde_json::to_string_pretty(&out).unwrap_or_default()))
            } else {
                ok(format!(
                    "[OK] '{}' synced in {:.2}s — {} item(s) updated.\n\
                     The GraphFinder interface has been refreshed.\n",
                    project_id,
                    elapsed.as_secs_f32(),
                    changed
                ))
            }
        }
        Err(e) => fail(1, format!("Error during synchronization: {}\n", e)),
    }
}

async fn cmd_status(
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
) -> CliProxyResponse {
    let target = resolve_target(args, cwd, true);
    let (project_id, engine, is_new) = match manager.get_or_register_project_by_path(&target).await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error checking path: {}\n", e)),
    };
    let stats = match engine.get_vault_stats().await {
        Ok(s) => s,
        Err(e) => return fail(1, format!("Error reading stats: {}\n", e)),
    };
    ok(format!(
        "Path             : {}\n\
         Project ID       : {}\n\
         Status           : {}\n\
         Notes            : {} ({} sections)\n\
         Code             : {} files ({} symbols)\n\
         Connections      : {} graph links\n\
         SQLite Database  : {}\n",
        clean(&target),
        project_id,
        if is_new { "Newly registered" } else { "Registered" },
        stats.total_notes,
        stats.total_chunks,
        stats.total_code_files,
        stats.total_code_chunks,
        stats.total_links,
        clean(engine.db_path()),
    ))
}

async fn cmd_projects(manager: &Arc<VaultManager>) -> CliProxyResponse {
    let list = manager.list_projects().await;
    let mut out = String::from("Registered Projects:\n");
    for (i, p) in list.iter().enumerate() {
        let active = if p.is_active { " * (active)" } else { "" };
        out.push_str(&format!(
            "{}. {} [{}]{}\n   Path:  {}\n   Notes: {} (+{} code files)\n",
            i + 1,
            p.name,
            p.id,
            active,
            p.path,
            p.note_count,
            p.code_count
        ));
    }
    ok(out)
}

async fn cmd_search(
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
) -> CliProxyResponse {
    let query = match args.get(2) {
        Some(q) if !q.starts_with("--") => q.clone(),
        _ => return fail(1, "Usage: graphfinder search <query> [--path <path>] [--limit <n>]\n".into()),
    };
    let limit = arg_value(args, "--limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5);
    let target = resolve_target(args, cwd, false);
    let (_, engine, _) = match manager.get_or_register_project_by_path(&target).await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error: {}\n", e)),
    };

    let note_hits = engine.search_chunks(&query, limit).await.unwrap_or_default();
    let code_hits = engine
        .search_code(&query, limit, None)
        .await
        .unwrap_or_default();

    let mut out = format!("Results for '{}':\n\n=== Notes ===\n", query);
    if note_hits.is_empty() {
        out.push_str("(none)\n");
    }
    for h in note_hits {
        out.push_str(&format!(
            "* [[{}]] (score: {:.2})\n  {}\n",
            h.deep_link,
            h.score,
            h.content.lines().next().unwrap_or("")
        ));
    }
    out.push_str("\n=== Code ===\n");
    if code_hits.is_empty() {
        out.push_str("(none)\n");
    }
    for h in code_hits {
        out.push_str(&format!(
            "* [[{}]] - {} [{}:{}] (score: {:.2})\n",
            h.deep_link, h.language, h.start_line, h.end_line, h.score
        ));
    }
    ok(out)
}

async fn cmd_path(
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
) -> CliProxyResponse {
    let (from, to) = match (args.get(2), args.get(3)) {
        (Some(f), Some(t)) => (f.clone(), t.clone()),
        _ => return fail(1, "Usage: graphfinder path <from> <to> [--path <path>] [--json]\n".into()),
    };
    let is_json = args.iter().any(|a| a == "--json");
    let target = resolve_target(args, cwd, false);
    let (_, engine, _) = match manager.get_or_register_project_by_path(&target).await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error: {}\n", e)),
    };
    match engine.find_path(&from, &to).await {
        Ok(path_opt) => {
            if is_json {
                return ok(format!(
                    "{}\n",
                    serde_json::to_string_pretty(&path_opt).unwrap_or_default()
                ));
            }
            match path_opt {
                Some(p) => {
                    let mut out = format!(
                        "Shortest path '{}' -> '{}' ({} hops):\n",
                        p.source, p.target, p.length
                    );
                    for (i, step) in p.steps.iter().enumerate() {
                        let prefix = if i == 0 { "START".to_string() } else { step.direction.clone() };
                        let edge = step
                            .edge_label
                            .as_deref()
                            .map(|l| format!(" [{}]", l))
                            .unwrap_or_default();
                        out.push_str(&format!(
                            "  {} {} [{}] {}{}\n",
                            prefix, step.title, step.node_type, step.node_id, edge
                        ));
                    }
                    ok(out)
                }
                None => ok(format!("No path connecting '{}' to '{}'.\n", from, to)),
            }
        }
        Err(e) => fail(1, format!("Error: {}\n", e)),
    }
}

async fn cmd_explain(
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
) -> CliProxyResponse {
    let node = match args.get(2) {
        Some(n) if !n.starts_with("--") => n.clone(),
        _ => return fail(1, "Usage: graphfinder explain <node> [--path <path>] [--json]\n".into()),
    };
    let is_json = args.iter().any(|a| a == "--json");
    let target = resolve_target(args, cwd, false);
    let (_, engine, _) = match manager.get_or_register_project_by_path(&target).await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error: {}\n", e)),
    };
    match engine.explain_node(&node).await {
        Ok(explanation) => {
            if is_json {
                return ok(format!(
                    "{}\n",
                    serde_json::to_string_pretty(&explanation).unwrap_or_default()
                ));
            }
            match explanation {
                Some(e) => {
                    let god = if e.is_god_node { " [GOD NODE]" } else { "" };
                    ok(format!(
                        "Node: {}{}\nType: {}\nCommunity: C{} ({})\nDegree: {} (In: {}, Out: {}) rank #{}\nInbound: {}\nOutbound: {}\n",
                        e.title,
                        god,
                        e.node_type,
                        e.community_id,
                        e.community_name,
                        e.degree,
                        e.in_degree,
                        e.out_degree,
                        e.centrality_rank,
                        e.incoming_dependents.join(", "),
                        e.outgoing_dependencies.join(", "),
                    ))
                }
                None => ok(format!("Entity '{}' not found in the graph.\n", node)),
            }
        }
        Err(e) => fail(1, format!("Error: {}\n", e)),
    }
}

async fn cmd_export(
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
) -> CliProxyResponse {
    let target = resolve_target(args, cwd, true);
    let out_dir = arg_value(args, "--out-dir").map(|s| {
        let p = PathBuf::from(&s);
        if p.is_absolute() {
            p
        } else {
            cwd.join(p)
        }
    });
    let (_, engine, _) = match manager.get_or_register_project_by_path(&target).await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error: {}\n", e)),
    };
    match engine.export_artifacts(out_dir.as_deref()).await {
        Ok(exp) => ok(format!(
            "[OK] Artifacts exported:\n  Directory : {}\n  Graph JSON: {}\n  Report MD : {}\n  HTML View : {}\n",
            exp.output_dir, exp.graph_json_path, exp.report_markdown_path, exp.standalone_html_path
        )),
        Err(e) => fail(1, format!("Error: {}\n", e)),
    }
}

async fn cmd_report(
    args: &[String],
    cwd: &Path,
    manager: &Arc<VaultManager>,
    app_handle: &Option<AppHandle>,
) -> CliProxyResponse {
    let is_json = args.iter().any(|a| a == "--json");
    let fail_on_cycles = args.iter().any(|a| a == "--fail-on-cycles");
    let force_sync = args.iter().any(|a| a == "--sync");
    let max_god_nodes = arg_value(args, "--max-god-nodes").and_then(|s| s.parse::<usize>().ok());
    let out_dir = arg_value(args, "--out-dir").map(|s| {
        let p = PathBuf::from(&s);
        if p.is_absolute() { p } else { cwd.join(p) }
    });
    let target = resolve_target(args, cwd, true);

    let (project_id, engine, is_new) = match manager.get_or_register_project_by_path(&target).await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error: {}\n", e)),
    };

    if force_sync || is_new {
        let _ = match app_handle {
            Some(h) => engine.sync_vault_with_handle(Some(h)).await,
            None => engine.sync_vault().await,
        };
    }

    let report = match engine.analyze_graph().await {
        Ok(r) => r,
        Err(e) => return fail(1, format!("Error: {}\n", e)),
    };
    let export_summary = match engine.export_artifacts(out_dir.as_deref()).await {
        Ok(s) => s,
        Err(e) => return fail(1, format!("Error: {}\n", e)),
    };

    let mut violations: Vec<String> = Vec::new();
    if fail_on_cycles && !report.cycles.is_empty() {
        violations.push(format!("Dependency cycles detected: {} (--fail-on-cycles)", report.cycles.len()));
    }
    if let Some(max) = max_god_nodes {
        if report.god_nodes.len() > max {
            violations.push(format!("God Nodes ({}) exceed threshold {} (--max-god-nodes)", report.god_nodes.len(), max));
        }
    }
    let ci_passed = violations.is_empty();

    if is_json {
        let out = serde_json::json!({
            "project_id": project_id,
            "ci_passed": ci_passed,
            "violations": violations,
            "report": report,
            "artifacts": export_summary,
        });
        return CliProxyResponse {
            code: if ci_passed { 0 } else { 1 },
            stdout: format!("{}\n", serde_json::to_string_pretty(&out).unwrap_or_default()),
            stderr: String::new(),
        };
    }

    let mut out = format!(
        "Architectural Report — {}\nNodes: {} | Edges: {} | Density: {:.4}\nCommunities: {} | God Nodes: {} | Surprises: {} | Cycles: {}\n\nTop hubs:\n",
        project_id, report.total_nodes, report.total_edges, report.density,
        report.communities.len(), report.god_nodes.len(), report.surprises.len(), report.cycles.len()
    );
    for (i, g) in report.god_nodes.iter().take(6).enumerate() {
        out.push_str(&format!("  {}. {} [{}] degree {}\n", i + 1, g.title, g.node_type, g.degree));
    }
    out.push_str(&format!(
        "\nArtifacts:\n  {}\n  {}\n  {}\n",
        export_summary.report_markdown_path, export_summary.graph_json_path, export_summary.standalone_html_path
    ));
    if !ci_passed {
        for v in &violations {
            out.push_str(&format!("[FAIL] {}\n", v));
        }
    }
    CliProxyResponse {
        code: if ci_passed { 0 } else { 1 },
        stdout: out,
        stderr: String::new(),
    }
}
