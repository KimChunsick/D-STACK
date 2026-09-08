// Explicit developer composition; no include expansion for arbitrary roles or task data.
use crate::core::error::{Error, Result};
use std::{fs, path::Path};

pub(super) fn append(repo: &Path, role: &str, instructions: &mut String) -> Result<()> {
    if !matches!(role, "frontend-dev" | "general-dev") {
        return Ok(());
    }
    let shared = "claude/templates/prompts/developer.md";
    let selected = format!("claude/agents/{role}.md");
    let common = read(repo, shared)?;
    let agent = read(repo, &selected)?;
    // Native metadata selects Claude tools/model; it is not a Codex prompt instruction.
    let body = agent
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(_, body)| body)
        .filter(|body| !body.trim().is_empty())
        .ok_or_else(|| Error::failed(format!("invalid developer frontmatter/body: {selected}")))?;
    instructions.push_str(&format!(
        "\n=== SHARED DEVELOPER CONTRACT ({shared}) ===\n{common}\n\
         === SELECTED DEVELOPER BODY ({selected}) ===\n{body}"
    ));
    Ok(())
}

fn read(repo: &Path, source: &str) -> Result<String> {
    let text = fs::read_to_string(repo.join(source))
        .map_err(|e| Error::cannot_decide(format!("cannot read role source {source}: {e}")))?;
    if text.trim().is_empty() {
        return Err(Error::failed(format!(
            "role source must not be empty: {source}"
        )));
    }
    Ok(text)
}
