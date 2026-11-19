//! Bundled AI-agent kit installer.
//!
//! Writes GraphFinder's skills (`.agents/skills/**`) and rule files
//! (`AGENTS.md`, plus a `CLAUDE.md` shim) into a target folder — typically a
//! vault directory — so AI agents that open that folder pick them up.
//! Skill files are GraphFinder-owned and safely overwritten; AGENTS.md and
//! CLAUDE.md use managed marker blocks so user-authored rules are preserved.

use std::fs;
use std::path::Path;

const AGENTS_MD: &str = include_str!("../../../AGENTS.md");
const SKILL_GRAPHFINDER: &str = include_str!("../../../.agents/skills/graphfinder/SKILL.md");
const SKILL_CURATOR: &str =
    include_str!("../../../.agents/skills/graphfinder-curator/SKILL.md");
const REF_MCP_SPEC: &str =
    include_str!("../../../.agents/skills/graphfinder-curator/references/mcp-spec.md");
const REF_NOTE_TEMPLATES: &str =
    include_str!("../../../.agents/skills/graphfinder-curator/references/note-templates.md");

const MANAGED_START: &str = "<!-- graphfinder-kit:start -->";
const MANAGED_END: &str = "<!-- graphfinder-kit:end -->";

const CLAUDE_SHIM: &str =
    "Follow the repository rules in @AGENTS.md and the skills in `.agents/skills/`.\n";

const SKILL_FILES: [(&str, &str); 4] = [
    (".agents/skills/graphfinder/SKILL.md", SKILL_GRAPHFINDER),
    (
        ".agents/skills/graphfinder-curator/SKILL.md",
        SKILL_CURATOR,
    ),
    (
        ".agents/skills/graphfinder-curator/references/mcp-spec.md",
        REF_MCP_SPEC,
    ),
    (
        ".agents/skills/graphfinder-curator/references/note-templates.md",
        REF_NOTE_TEMPLATES,
    ),
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentKitResult {
    pub target: String,
    pub written: Vec<String>,
    pub unchanged: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum Outcome {
    Written,
    Unchanged,
}

fn upsert_file(dest: &Path, content: &str) -> Result<Outcome, String> {
    if dest.exists() {
        if let Ok(existing) = fs::read_to_string(dest) {
            if existing == content {
                return Ok(Outcome::Unchanged);
            }
        }
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    fs::write(dest, content).map_err(|e| format!("Failed to write {}: {}", dest.display(), e))?;
    Ok(Outcome::Written)
}

fn managed_block(body: &str) -> String {
    format!("{}\n{}\n{}", MANAGED_START, body.trim_end(), MANAGED_END)
}

/// Writes `body` inside a managed marker block in `dest`:
/// creates the file if missing, replaces the block if markers exist,
/// or appends the block to user-authored content otherwise.
fn upsert_managed(dest: &Path, body: &str) -> Result<Outcome, String> {
    let block = managed_block(body);
    if !dest.exists() {
        return upsert_file(dest, &format!("{}\n", block));
    }
    let existing = fs::read_to_string(dest)
        .map_err(|e| format!("Failed to read {}: {}", dest.display(), e))?;
    let new_content =
        if let (Some(s), Some(e)) = (existing.find(MANAGED_START), existing.find(MANAGED_END)) {
            let end = e + MANAGED_END.len();
            if end <= s {
                format!("{}\n\n{}\n", existing.trim_end(), block)
            } else {
                format!("{}{}{}", &existing[..s], block, &existing[end..])
            }
        } else {
            format!("{}\n\n{}\n", existing.trim_end(), block)
        };
    upsert_file(dest, &new_content)
}

pub fn install(target_dir: &Path) -> Result<AgentKitResult, String> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create {}: {}", target_dir.display(), e))?;
    }

    let mut result = AgentKitResult {
        target: target_dir.to_string_lossy().to_string(),
        written: Vec::new(),
        unchanged: Vec::new(),
    };

    for (rel, content) in SKILL_FILES {
        match upsert_file(&target_dir.join(rel), content)? {
            Outcome::Written => result.written.push(rel.to_string()),
            Outcome::Unchanged => result.unchanged.push(rel.to_string()),
        }
    }

    for (rel, body) in [("AGENTS.md", AGENTS_MD), ("CLAUDE.md", CLAUDE_SHIM)] {
        match upsert_managed(&target_dir.join(rel), body)? {
            Outcome::Written => result.written.push(rel.to_string()),
            Outcome::Unchanged => result.unchanged.push(rel.to_string()),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("graphfinder_agent_kit_test")
            .join(format!("{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn installs_kit_into_empty_folder() {
        let dir = temp_dir("empty");
        let res = install(&dir).unwrap();
        assert!(dir.join(".agents/skills/graphfinder/SKILL.md").exists());
        assert!(dir
            .join(".agents/skills/graphfinder-curator/references/mcp-spec.md")
            .exists());
        let agents = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
        assert!(agents.contains(MANAGED_START));
        assert!(agents.contains("GraphFinder"));
        assert!(res.written.contains(&"AGENTS.md".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn second_install_is_idempotent() {
        let dir = temp_dir("idem");
        install(&dir).unwrap();
        let res = install(&dir).unwrap();
        assert!(res.written.is_empty());
        assert_eq!(res.unchanged.len(), 6);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn preserves_user_agents_md_content() {
        let dir = temp_dir("preserve");
        fs::write(dir.join("AGENTS.md"), "# My Own Rules\nDo things my way.\n").unwrap();
        install(&dir).unwrap();
        let content = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
        assert!(content.starts_with("# My Own Rules"));
        assert!(content.contains(MANAGED_START));
        assert!(content.contains("GraphFinder"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn updates_existing_managed_block() {
        let dir = temp_dir("update");
        fs::write(
            dir.join("AGENTS.md"),
            "# My Rules\n\n<!-- graphfinder-kit:start -->\nOLD\n<!-- graphfinder-kit:end -->\n\ntrailing\n",
        )
        .unwrap();
        install(&dir).unwrap();
        let content = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
        assert!(content.contains("# My Rules"));
        assert!(!content.contains("\nOLD\n"));
        assert!(content.ends_with("trailing\n"));
        let _ = fs::remove_dir_all(&dir);
    }
}
