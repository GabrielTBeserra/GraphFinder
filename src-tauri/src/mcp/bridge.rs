// MCP stdio → HTTP bridge. When `graphfinder --mcp` detects a running
// GraphFinder instance (GUI), it relays JSON-RPC messages over localhost
// Streamable HTTP instead of loading a second embedding engine, a second
// VaultManager and a second SQLite connection set — one process, one model.

use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const PROTOCOL_VERSION: &str = "2025-03-26";

fn auth(req: ureq::Request) -> ureq::Request {
    match std::env::var("GRAPHFINDER_MCP_TOKEN") {
        Ok(t) if !t.trim().is_empty() => {
            req.set("Authorization", &format!("Bearer {}", t.trim()))
        }
        _ => req,
    }
}

/// Extract JSON-RPC payloads from an SSE body and print each as a line.
fn write_sse_payloads(body: &str, out: &mut impl Write) {
    for line in body.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            let data = data.trim();
            if !data.is_empty() {
                let _ = writeln!(out, "{}", data);
            }
        }
    }
    let _ = out.flush();
}

pub fn run_stdio_bridge(port: u16) -> Result<(), String> {
    let mcp_url = format!("http://127.0.0.1:{}/mcp", port);
    let session: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let running = Arc::new(AtomicBool::new(true));

    // GET /mcp SSE pump — forwards server-initiated requests/notifications.
    {
        let session = Arc::clone(&session);
        let running = Arc::clone(&running);
        let url = mcp_url.clone();
        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let mut req = ureq::get(&url).set("Accept", "text/event-stream");
                if let Some(sid) = session.lock().unwrap().clone() {
                    req = req.set("Mcp-Session-Id", &sid);
                }
                req = auth(req);
                match req.call() {
                    Ok(resp) => {
                        let mut reader = BufReader::new(resp.into_reader());
                        let mut line = String::new();
                        loop {
                            line.clear();
                            match reader.read_line(&mut line) {
                                Ok(0) | Err(_) => break,
                                Ok(_) => {
                                    if let Some(data) = line.trim_end().strip_prefix("data:") {
                                        let data = data.trim();
                                        if !data.is_empty() {
                                            let stdout = std::io::stdout();
                                            let mut out = stdout.lock();
                                            let _ = writeln!(out, "{}", data);
                                            let _ = out.flush();
                                        }
                                    }
                                }
                            }
                            if !running.load(Ordering::Relaxed) {
                                break;
                            }
                        }
                    }
                    Err(_) => std::thread::sleep(std::time::Duration::from_secs(1)),
                }
            }
        });
    }

    // stdin → POST /mcp → stdout
    let stdin = std::io::stdin();
    let reader = stdin.lock();
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let mut req = ureq::post(&mcp_url)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json, text/event-stream")
            .set("MCP-Protocol-Version", PROTOCOL_VERSION);
        if let Some(sid) = session.lock().unwrap().clone() {
            req = req.set("Mcp-Session-Id", &sid);
        }
        req = auth(req);

        match req.send_string(&line) {
            Ok(resp) => {
                if let Some(sid) = resp.header("Mcp-Session-Id") {
                    *session.lock().unwrap() = Some(sid.to_string());
                }
                let content_type = resp.header("content-type").unwrap_or("").to_string();
                let body = resp.into_string().unwrap_or_default();
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                if content_type.contains("text/event-stream") {
                    write_sse_payloads(&body, &mut out);
                } else if !body.trim().is_empty() {
                    let _ = writeln!(out, "{}", body.trim());
                    let _ = out.flush();
                }
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                if body.trim().starts_with('{') {
                    let stdout = std::io::stdout();
                    let mut out = stdout.lock();
                    let _ = writeln!(out, "{}", body.trim());
                    let _ = out.flush();
                } else {
                    eprintln!("[GraphFinder Bridge] HTTP {} from instance", code);
                }
            }
            Err(e) => {
                eprintln!("[GraphFinder Bridge] Instance unreachable: {}", e);
                break;
            }
        }
    }

    running.store(false, Ordering::Relaxed);
    Ok(())
}
