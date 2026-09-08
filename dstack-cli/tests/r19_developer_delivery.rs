#![allow(non_snake_case)]
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "dstack-r19-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn render(&self, role: &str, home: &std::path::Path) -> Output {
        let context = self.0.join("context.md");
        fs::write(&context, "Bounded task data\n").unwrap();
        Command::new(env!("CARGO_BIN_EXE_dstack"))
            .env("DSTACK_HOME", home)
            .current_dir(&self.0)
            .args(["prompt", "render", "--role", role, "--context"])
            .arg(context)
            .output()
            .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn R19__real_render_delivers_common_and_only_selected_body() {
    let t = Scratch::new();
    let common = fs::read_to_string(repo().join("claude/templates/prompts/developer.md"))
        .unwrap_or_default();
    let worker = fs::read_to_string(repo().join("claude/templates/prompts/worker.md")).unwrap();
    for (role, other) in [
        ("frontend-dev", "general-dev"),
        ("general-dev", "frontend-dev"),
    ] {
        let out = t.render(role, &repo().join("claude"));
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let rendered = String::from_utf8(out.stdout).unwrap();
        let stable = rendered
            .split_once("=== TASK CONTEXT (variable) ===")
            .unwrap()
            .0;
        let agent = fs::read_to_string(repo().join(format!("claude/agents/{role}.md"))).unwrap();
        let body = agent
            .strip_prefix("---\n")
            .unwrap()
            .split_once("\n---\n")
            .unwrap()
            .1;
        assert!(!common.is_empty());
        assert_eq!(stable.matches(&common).count(), 1);
        assert_eq!(stable.matches(&worker).count(), 1);
        assert!(stable.contains(body));
        assert!(stable.find(&worker).unwrap() < stable.find(&common).unwrap());
        assert!(stable.find(&common).unwrap() < stable.find(body).unwrap());
        assert!(
            !stable.contains(&format!("name: {role}")),
            "frontmatter is not a model instruction"
        );
        let other_body =
            fs::read_to_string(repo().join(format!("claude/agents/{other}.md"))).unwrap();
        assert!(!stable.contains(other_body.split_once("\n---\n").unwrap().1));
    }
    for role in ["worker", "review", "research", "audit", "handoff"] {
        let out = t.render(role, &repo().join("claude"));
        assert!(out.status.success());
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(!text.contains("# Developer implementation responsibility"));
        if role == "worker" {
            assert!(text.contains(&worker));
        }
    }
    for role in ["recon", "e2e-runner", "ko-polish"] {
        let out = t.render(role, &repo().join("claude"));
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn R19__missing_empty_or_malformed_developer_source_never_leaks_partial_prompt() {
    let t = Scratch::new();
    for source in [
        "claude/templates/prompts/worker.md",
        "claude/templates/prompts/developer.md",
        "claude/agents/frontend-dev.md",
    ] {
        let p = t.0.join(source);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let original = fs::read_to_string(repo().join(source)).unwrap_or_default();
        fs::write(p, original).unwrap();
    }
    for source in [
        "claude/templates/prompts/worker.md",
        "claude/templates/prompts/developer.md",
        "claude/agents/frontend-dev.md",
    ] {
        let p = t.0.join(source);
        let original = fs::read_to_string(&p).unwrap();
        for bad in [None, Some(" \n"), Some("---\nname: frontend-dev\n---\n")] {
            if let Some(text) = bad {
                fs::write(&p, text).unwrap();
            } else {
                fs::remove_file(&p).unwrap();
            }
            if bad.is_some_and(|s| s.starts_with("---")) && !source.contains("agents/") {
                continue;
            }
            let out = t.render("frontend-dev", &t.0.join("claude"));
            assert!(!out.status.success(), "accepted broken {source}");
            assert!(out.stdout.is_empty());
        }
        fs::write(p, original).unwrap();
    }
}
