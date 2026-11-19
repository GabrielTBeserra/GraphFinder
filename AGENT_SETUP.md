# GraphFinder — AI Agent Setup Guide

Step-by-step installation of GraphFinder for AI coding agents, written for beginners. Three components exist — only the first one is required.

| # | Component | What it does | Required? |
|---|-----------|--------------|-----------|
| 1 | **MCP server** | Gives the agent GraphFinder's tools (`search_semantic`, `search_code`, `store_context`, …) | **Yes** |
| 2 | **Skills** | Teaches the agent *how* to use the tools well (token economy, atomic notes, linking) | Recommended |
| 3 | **Rules** (`AGENTS.md`) | Behavioral guardrails: never read whole files, always suggest links, keep sections small | Recommended |

```
┌──────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ 1. MCP       │────►│ 2. SKILLS        │────►│ 3. RULES        │
│ (the tools)  │     │ (how to use them)│     │ (the discipline)│
└──────────────┘     └──────────────────┘     └─────────────────┘
```

---

## Step 0 — Prerequisites

- A GraphFinder binary. Either the built desktop app, or `cargo` available to run it from source (`cargo run --manifest-path src-tauri/Cargo.toml -- --mcp`).
- Know which agent you use: **Cursor, Claude Code, Claude Desktop, Codex, Devin, Windsurf, Antigravity/Gemini, Cline, Roo Code, Continue, Copilot, Aider**.

> **Tip:** You can stop reading after Step 1 if you just want the tools working. Steps 2–3 make the agent noticeably smarter with the vault.

---

## Fastest path — install skills + rules from the app

You do **not** need to copy files by hand. The desktop app can install the agent kit (Skills + `AGENTS.md` rules + a `CLAUDE.md` shim) into any vault folder in one click:

1. Open the vault actions menu — the **≡ (hamburger)** button next to the vault name in the *Explorer* header.
2. Each vault row has a **↓ (download)** button. Click it to install into that vault's folder, or click **New Vault / Choose Folder** and tick *"Install agent kit into this folder"* while creating.
3. The installer writes:
   - `.agents/skills/graphfinder/SKILL.md`
   - `.agents/skills/graphfinder-curator/SKILL.md` (+ `references/`)
   - `AGENTS.md` — **merged safely**: if the file already exists, GraphFinder's rules go inside a managed `<!-- graphfinder-kit -->` block and your own content is never touched
   - `CLAUDE.md` — same managed-block shim pointing at `@AGENTS.md`

Skills and rules are then done for agents that read `.agents/` + `AGENTS.md` natively (Codex, Devin, Cursor, Antigravity, Amp, OpenCode, Jules). For other tools, copy the installed files using the per-tool paths in Step 2/3 below. **MCP (Step 1) is always configured manually** — the button only installs the Markdown files.

---

## Step 1 — Connect the MCP Server (required)

Two modes. Pick one:

| Mode | When to use | Config value |
|------|-------------|--------------|
| **A. SSE** (recommended) | The GraphFinder desktop app is open — notes appear on the graph in real time | `"url": "http://127.0.0.1:45123/sse"` |
| **B. Stdio headless** | No GUI — the agent launches the binary itself | `"command": "C:\\Path\\To\\GraphFinder.exe", "args": ["--mcp"]` |

### Cursor

Edit `.cursor/mcp.json` (project) or `~/.cursor/mcp.json` (global):

```json
{ "mcpServers": { "graphfinder": { "url": "http://127.0.0.1:45123/sse" } } }
```

### Claude Code / Claude Desktop

Config file: `%APPDATA%\Claude\claude_desktop_config.json` (Windows) or `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{ "mcpServers": { "graphfinder": { "command": "C:\\Path\\To\\GraphFinder.exe", "args": ["--mcp"] } } }
```

### Codex

Add to `~/.codex/config.toml`:

```toml
[mcp_servers.graphfinder]
command = "C:\\Path\\To\\GraphFinder.exe"
args = ["--mcp"]
```

### Devin

Add to `.devin/mcp_config.local.json` in the project (or `~/.config/devin/mcp_config.json` for global):

```json
{ "mcpServers": { "graphfinder": { "url": "http://127.0.0.1:45123/sse" } } }
```

### Windsurf / Antigravity / Cline / Roo Code

Open the tool's MCP settings UI and add a server named `graphfinder` with URL `http://127.0.0.1:45123/sse`. All of them accept the same JSON block:

```json
{ "mcpServers": { "graphfinder": { "url": "http://127.0.0.1:45123/sse" } } }
```

> **Windows note:** inside JSON, either escape backslashes (`C:\\Path\\GraphFinder.exe`) or use forward slashes (`C:/Path/GraphFinder.exe`).

---

## Step 2 — Install the Skills

Two skills ship inside this repository:

- `.agents/skills/graphfinder/` — graph analytics (communities, God Nodes, pathfinding, exports)
- `.agents/skills/graphfinder-curator/` — note authoring, semantic linking, MCP retrieval discipline

### Option A — Project scope (recommended)

The skills already live inside the repo. Agents that read the `.agents/skills/` convention (Devin, Codex, OpenCode, and anything honoring the Agent Skills spec) detect them **automatically — nothing to do**.

Tools with their own folder need a copy. From the repository root:

**Windows (PowerShell):**
```powershell
# Claude Code
Copy-Item -Recurse .agents\skills\graphfinder, .agents\skills\graphfinder-curator .claude\skills\ -Force
# Cursor
Copy-Item -Recurse .agents\skills\graphfinder, .agents\skills\graphfinder-curator .cursor\skills\ -Force
# Devin
Copy-Item -Recurse .agents\skills\graphfinder, .agents\skills\graphfinder-curator .devin\skills\ -Force
```

**macOS / Linux (bash):**
```bash
for tool in .claude .cursor .devin; do
  mkdir -p "$tool/skills"
  cp -r .agents/skills/graphfinder .agents/skills/graphfinder-curator "$tool/skills/"
done
```

Windsurf, Cline, Roo Code and Continue use plain `.md` rule files instead of `SKILL.md` folders — copy the file and rename it:

```powershell
# Windsurf (Windows)
New-Item -ItemType Directory .windsurf\rules\graphfinder -Force
Copy-Item .agents\skills\graphfinder-curator\SKILL.md .windsurf\rules\graphfinder\graphfinder.md
```

```bash
# Windsurf / Cline / Roo / Continue (bash)
for d in .windsurf .cline .roo .continue; do
  mkdir -p "$d/rules/graphfinder"
  cp .agents/skills/graphfinder-curator/SKILL.md "$d/rules/graphfinder/graphfinder.md"
done
```

### Option B — Global scope (available in every project)

Copy to your home directory once; the skills follow you into any repo:

```powershell
# Windows — covers Agents-standard + Claude Code + Codex
$skills = "$HOME\.agents\skills", "$HOME\.claude\skills", "$HOME\.codex\skills"
foreach ($s in $skills) {
  New-Item -ItemType Directory $s -Force | Out-Null
  Copy-Item -Recurse .agents\skills\graphfinder, .agents\skills\graphfinder-curator $s -Force
}
```

```bash
# macOS / Linux
for s in ~/.agents/skills ~/.claude/skills ~/.codex/skills; do
  mkdir -p "$s"
  cp -r .agents/skills/graphfinder .agents/skills/graphfinder-curator "$s/"
done
```

> **Choose project scope** if skills should travel with the repo for collaborators. **Choose global** if you want GraphFinder available everywhere on your machine.

---

## Step 3 — Install the Rules (`AGENTS.md`)

`AGENTS.md` at the repository root defines the vault discipline: atomic chunks, `suggest_links` before saving, never reading whole files.

### Auto-detected — nothing to do

These tools read `AGENTS.md` from the project root automatically: **Codex, Devin, Antigravity/Gemini, Jules, Amp, OpenCode, Cursor** (recent versions). Open the repo and the rules are active.

### Claude Code

Create a one-line `CLAUDE.md` in the project root so the rules are imported:

```powershell
Set-Content CLAUDE.md "@AGENTS.md"          # Windows
echo "@AGENTS.md" > CLAUDE.md               # bash
```

(Claude Code also reads `AGENTS.md` directly in recent versions; the shim keeps it working on older ones.)

### Cursor (explicit)

```powershell
New-Item -ItemType Directory .cursor\rules -Force
@'
---
alwaysApply: true
---
'@ + (Get-Content AGENTS.md -Raw) | Set-Content .cursor\rules\graphfinder.mdc
```

```bash
mkdir -p .cursor/rules
{ printf -- "---\nalwaysApply: true\n---\n"; cat AGENTS.md; } > .cursor/rules/graphfinder.mdc
```

### GitHub Copilot

```powershell
New-Item -ItemType Directory .github -Force
Copy-Item AGENTS.md .github\copilot-instructions.md   # Windows
mkdir -p .github && cp AGENTS.md .github/copilot-instructions.md   # bash
```

### Claude Desktop / Chat projects

Paste the contents of `AGENTS.md` into the project's system prompt or instructions field.

---

## Step 4 — Verify It Works

Restart the agent session (or reload the window), then ask:

1. *"What GraphFinder tools can you call?"* → should list `search_semantic`, `search_code`, `store_context`, …
2. *"Call `list_projects`"* → the MCP server answers with your vaults. If the app is closed, headless mode launches instead.
3. *"Search the GraphFinder vault for `sqlite`"* → the agent calls `search_semantic` and answers with `[[Note#Section]]` links — **without dumping whole files into chat** (that's the rules working).

If all three pass, you're done.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `graphfinder` tools don't appear | MCP not configured or wrong mode | Recheck the JSON block; for SSE mode make sure the app is running; for stdio use an **absolute** binary path |
| Agent never mentions the skill | Session started before install, or wrong folder | Folder must be `<tool>/skills/<name>/SKILL.md` (uppercase filename); restart the session |
| Agent reads whole `.md` files | Rules not loaded | Check Step 3 for your tool; verify `AGENTS.md` exists at repo root; restart |
| `command not found` in stdio config | Relative path | Use the full absolute path to `GraphFinder.exe` |
| JSON parse error on Windows | Unescaped backslashes | Use `C:/Path/GraphFinder.exe` instead of `C:\Path\...` |
| Symlink copies fail on Windows | No Developer Mode | Use `Copy-Item` (real copies) as shown — symlinks are unnecessary |

---

## Quick Reference — Where Everything Lives

| Tool | Skills folder | Rules file | MCP config |
|------|---------------|------------|------------|
| Agents-standard (Devin, Codex, OpenCode) | `.agents/skills/` or `~/.agents/skills/` | `AGENTS.md` (auto) | per tool |
| Claude Code | `.claude/skills/` or `~/.claude/skills/` | `CLAUDE.md` → `@AGENTS.md` | `claude_desktop_config.json` |
| Cursor | `.cursor/skills/` or `~/.cursor/skills/` | `.cursor/rules/*.mdc` or `AGENTS.md` | `.cursor/mcp.json` |
| Devin | `.devin/skills/` or `.agents/skills/` | `AGENTS.md` (auto) | `.devin/mcp_config.local.json` |
| Codex | `~/.codex/skills/` | `AGENTS.md` (auto) | `~/.codex/config.toml` |
| Windsurf | `.windsurf/rules/<n>/<n>.md` | `.windsurf/rules/` or `.windsurfrules` | MCP settings UI |
| Cline / Roo / Continue | `.cline/`, `.roo/`, `.continue/` `rules/` | same folder | MCP settings UI |
| Copilot | — | `.github/copilot-instructions.md` | VS Code MCP settings |
| Antigravity / Gemini | — | `AGENTS.md` (auto) | `mcp_config.json` |
