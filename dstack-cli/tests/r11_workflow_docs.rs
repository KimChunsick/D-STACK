// tests/r11_workflow_docs.rs
// R11: the main workflow skills point at the request steps of M1 — dstack-workflow names the
// section writer, the design gate's skip verb, the merge path's background writer and the brief
// regenerated before `request open`; dstack-quick names the quick body and its summary writer;
// README.md lists the verbs; every skill file stays at or under 300 lines.
#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

const WORKFLOW: &str = "claude/skills/dstack-workflow/SKILL.md";
const QUICK: &str = "claude/skills/dstack-quick/SKILL.md";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every phrase `path` must carry, reported together so one run names every gap.
fn assert_names(path: &str, phrases: &[&str]) {
    let text = read(path);
    let missing: Vec<&str> = phrases
        .iter()
        .copied()
        .filter(|phrase| !text.contains(phrase))
        .collect();
    assert!(missing.is_empty(), "{path} does not name {missing:?}");
}

/// The body of the `## <title>` section: up to the next `## ` heading or the end.
fn section<'a>(text: &'a str, title: &str) -> &'a str {
    let start = text
        .find(&format!("\n## {title}\n"))
        .unwrap_or_else(|| panic!("no '## {title}' section"));
    let body = &text[start + 1..];
    match body[3..].find("\n## ") {
        Some(end) => &body[..end + 3],
        None => body,
    }
}

fn position(text: &str, phrase: &str) -> usize {
    text.find(phrase)
        .unwrap_or_else(|| panic!("missing {phrase:?}"))
}

#[test]
fn R11_workflow_skill_points_at_the_request_design_steps() {
    assert_names(
        WORKFLOW,
        &[
            "dstack request section <key> --from <file>",
            "dstack request design-skip --why",
            "dstack request background --run <id> --from <file>",
            "dstack request brief",
        ],
    );
    let text = read(WORKFLOW);
    let approval = section(&text, "8. Approval loop (R44, R45, R46)");
    assert!(
        position(approval, "dstack request brief") < position(approval, "dstack request open"),
        "the brief is regenerated before the request is opened"
    );
}

#[test]
fn R11_quick_skill_points_at_the_quick_body_and_its_summary_writer() {
    assert_names(
        QUICK,
        &[
            "dstack request section summary --from <file> --quick <slug>",
            "`## 요구사항`",
        ],
    );
}

#[test]
fn R11_readme_lists_the_new_request_verbs() {
    assert_names(
        "README.md",
        &[
            "dstack request section",
            "dstack request design-skip",
            "dstack request background",
            "dstack request brief",
        ],
    );
}

#[test]
fn R11_every_skill_file_stays_within_300_lines() {
    let mut over = Vec::new();
    for root in ["claude/skills", "codex/skills"] {
        let skills = std::fs::read_dir(repo().join(root)).unwrap().flatten();
        for skill in skills.filter(|entry| entry.path().is_dir()) {
            for file in std::fs::read_dir(skill.path()).unwrap().flatten() {
                let path = file.path();
                if path.extension().is_some_and(|ext| ext == "md") {
                    let lines = std::fs::read_to_string(&path).unwrap().lines().count();
                    if lines > 300 {
                        over.push(format!("{}: {lines}", relative(&path)));
                    }
                }
            }
        }
    }
    assert!(over.is_empty(), "skill files over 300 lines: {over:?}");
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repo()).unwrap_or(path).display().to_string()
}
