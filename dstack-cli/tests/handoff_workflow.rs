// R10: exercise the rendered role and scratch-home installation; never call a live model.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn repo() -> PathBuf {
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")).unwrap()
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "dstack-r10-workflow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn render(&self, context: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dstack"))
            .args(["prompt", "render", "--role", "handoff", "--context"])
            .arg(context)
            .env("DSTACK_HOME", repo().join("claude"))
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn install(&self, dry: bool) -> String {
        let mut command = Command::new("bash");
        command
            .arg(repo().join("install.sh"))
            .env("HOME", &self.0)
            .env("DSTACK_BACKUP_TS", "handoff-workflow-test");
        if dry {
            command.arg("--dry-run");
        }
        let output = command.output().unwrap();
        succeeded(&output);
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn succeeded(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn r10_handoff_workflow_renders_canonical_role_with_stable_prefix_and_no_state_writes() {
    let scratch = Scratch::new();
    let context = scratch.0.join("context.md");
    let one_task = "{\"snapshot\":{\"items\":[]},\"history\":[]}\n";
    fs::write(&context, one_task).unwrap();
    let first = scratch.render(&context);
    succeeded(&first);
    let source = fs::read_to_string(repo().join("claude/templates/prompts/handoff.md")).unwrap();
    let first_text = String::from_utf8(first.stdout).unwrap();
    assert_eq!(first_text.matches(&source).count(), 1);
    assert!(first_text.ends_with(one_task));
    assert!(first_text.contains("Mode: handoff\n"));
    let boundary = "=== TASK CONTEXT (variable) ===\n";
    let (prefix, _) = first_text.split_once(boundary).unwrap();
    let two_task = "{\"snapshot\":{\"items\":[]},\"history\":[{\"reference\":\"history:12\"}]}\n";
    fs::write(&context, two_task).unwrap();
    let second = scratch.render(&context);
    succeeded(&second);
    let second_text = String::from_utf8(second.stdout).unwrap();
    assert_eq!(prefix, second_text.split_once(boundary).unwrap().0);
    assert_eq!(first.stderr, second.stderr);
    assert!(second_text.ends_with(two_task));
    assert!(!prefix.contains(scratch.0.to_str().unwrap()));
    assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 1);
    assert_eq!(fs::read_to_string(context).unwrap(), two_task);
}

#[test]
fn r10_handoff_workflow_rejects_missing_and_empty_context_without_partial_output() {
    let scratch = Scratch::new();
    let context = scratch.0.join("missing.md");
    for contents in [None, Some(" \n")] {
        if let Some(text) = contents {
            fs::write(&context, text).unwrap();
        }
        let output = scratch.render(&context);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("context"), "{error}");
        assert!(
            !error.contains("usage:"),
            "handoff must resolve before context validation"
        );
    }
    assert!(!scratch.0.join(".dstack").exists());
}

#[test]
fn r10_handoff_workflow_installs_shared_handoff_and_host_entries_idempotently() {
    let scratch = Scratch::new();
    let dry = scratch.install(true);
    for host in ["claude", "codex"] {
        assert!(dry.contains(&format!(".{host}/skills/dstack-handoff")));
    }
    assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 0);
    fs::create_dir_all(scratch.0.join(".codex")).unwrap();
    let config = scratch.0.join(".codex/config.toml");
    fs::write(&config, "model = \"personal-model\"\n").unwrap();
    for pass in 0..2 {
        let printed = scratch.install(false);
        for host in ["claude", "codex"] {
            let skill = scratch.0.join(format!(".{host}/skills/dstack-handoff"));
            assert_eq!(
                fs::read_link(&skill).unwrap(),
                repo().join("claude/skills/dstack-handoff")
            );
            assert!(skill.join("SKILL.md").is_file());
            if pass == 1 {
                let row = printed
                    .lines()
                    .find(|line| line.contains(&format!(".{host}/skills/dstack-handoff")))
                    .unwrap();
                assert!(row.contains("up-to-date"), "{row}");
            }
            let runtime = scratch.0.join(format!(".{host}/runtime.md"));
            assert_eq!(
                fs::read_link(&runtime).unwrap(),
                repo().join("claude/runtime.md")
            );
            installed_entry(&runtime, None);
            let entry = scratch.0.join(if host == "claude" {
                ".claude/CLAUDE.md"
            } else {
                ".codex/AGENTS.md"
            });
            installed_entry(&entry, Some(host));
            let context = scratch.0.join("context.md");
            fs::write(&context, "{\"snapshot\":{\"items\":[]}}\n").unwrap();
            let output = Command::new(scratch.0.join(format!(".{host}/bin/dstack")))
                .args(["prompt", "render", "--role", "handoff", "--context"])
                .arg(&context)
                .env("HOME", &scratch.0)
                .env_remove("DSTACK_HOME")
                .current_dir(&scratch.0)
                .output()
                .unwrap();
            succeeded(&output);
            assert!(String::from_utf8_lossy(&output.stdout).contains("Mode: handoff\n"));
        }
        assert_eq!(
            fs::read_to_string(&config).unwrap(),
            "model = \"personal-model\"\n"
        );
    }
}

// These are installed instruction artifacts, not application source inspected in lieu of execution.
fn installed_entry(path: &Path, host: Option<&str>) {
    let text = fs::read_to_string(path).unwrap();
    let bounded = text
        .find("handoff summarizer")
        .expect("bounded handoff role");
    let handoff = text.find("dstack-handoff").expect("explicit handoff entry");
    let ordinary = text
        .find("dstack mode show --host")
        .expect("ordinary host check");
    assert!(
        bounded < handoff && handoff < ordinary,
        "{}: role and explicit handoff dispatch precede ordinary host check",
        path.display()
    );
    assert!(text.contains("only handoff preparation/resume"));
    if let Some(provider) = host {
        assert!(text.contains(&format!("dstack mode show --host {provider}")));
    }
}
