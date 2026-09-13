// tests/r01_audit_writable.rs
// The optional `--writable <dir>` of `dstack mode exec`: an audit sub may write into exactly one
// directory. Codex keeps its working root writable under workspace-write, so the flag decides the
// sandbox and the root together; every other role, target, path or sub is refused by name.
mod support {
    pub mod mode_runtime;
}
use std::fs;
use support::mode_runtime::Scratch;

/// The one directory the audit sub is allowed to write into, inside the session worktree.
fn artifacts(t: &Scratch) -> String {
    let dir = t.0.join("work/artifacts");
    fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap().to_str().unwrap().to_string()
}

fn argv(stdout: &[u8]) -> (serde_json::Value, Vec<String>) {
    let plan: serde_json::Value = serde_json::from_slice(stdout).unwrap();
    let argv = plan["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect();
    (plan, argv)
}

fn after(argv: &[String], flag: &str) -> String {
    let at = argv
        .iter()
        .position(|value| value == flag)
        .unwrap_or_else(|| panic!("no {flag} in {argv:?}"));
    argv[at + 1].clone()
}

#[test]
fn r01_audit_writable_roots_the_codex_sandbox_at_the_writable_directory() {
    let t = Scratch::new("claude", "codex");
    let dir = artifacts(&t);
    let before = t.tree();
    let output = t.run(
        "audit",
        &[
            "--run",
            "sample",
            "--writable",
            "work/artifacts",
            "--dry-run",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let (plan, argv) = argv(&output.stdout);
    assert_eq!(after(&argv, "--sandbox"), "workspace-write");
    assert_eq!(after(&argv, "-C"), dir);
    assert!(!argv.iter().any(|value| value == "read-only"), "{argv:?}");
    assert_eq!(plan["cwd"], dir);
    assert_eq!(before, t.tree(), "dry-run changed files or metadata");
}

#[test]
fn r02_audit_writable_refuses_by_naming_the_rule_and_reserves_nothing() {
    for (role, extra, rule) in [
        (
            "review",
            vec!["--run", "sample", "--writable", "work/artifacts"],
            "--role audit",
        ),
        ("audit", vec!["--writable", "work/artifacts"], "--run"),
        (
            "audit",
            vec!["--run", "sample", "--writable", "work/missing"],
            "invalid --writable",
        ),
        (
            "audit",
            vec!["--run", "sample", "--writable", "trace"],
            "--worktree",
        ),
    ] {
        let t = Scratch::new("claude", "codex");
        artifacts(&t);
        let before = t.tree();
        let output = t.run(role, &extra);
        assert!(!output.status.success(), "accepted {rule}");
        assert!(output.stdout.is_empty(), "printed a plan for {rule}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr.lines().count(), 1, "{stderr}");
        assert!(stderr.contains(rule), "{rule} is not named: {stderr}");
        assert!(
            !t.0.join(".dstack/local/exec").exists(),
            "reserved a capture for {rule}"
        );
        assert_eq!(before, t.tree(), "mutated state for {rule}");
    }
}

#[test]
fn r03_audit_writable_is_refused_for_a_claude_sub() {
    let t = Scratch::new("claude", "claude");
    artifacts(&t);
    let before = t.tree();
    let output = t.run(
        "audit",
        &[
            "--run",
            "sample",
            "--writable",
            "work/artifacts",
            "--dry-run",
        ],
    );
    assert!(!output.status.success(), "accepted a claude sub");
    assert!(output.stdout.is_empty(), "printed a plan for a claude sub");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("codex-only"), "{stderr}");
    assert_eq!(before, t.tree(), "mutated state for a claude sub");
}

#[test]
fn r04_audit_writable_keeps_the_sandbox_and_root_in_the_capture() {
    let t = Scratch::new("claude", "codex");
    let dir = artifacts(&t);
    let output = t.run(
        "audit",
        &["--run", "sample", "--writable", "work/artifacts"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cmd = t.read(".dstack/local/exec/check/cmd");
    assert!(cmd.contains("--sandbox workspace-write"), "{cmd}");
    assert!(cmd.contains(&format!("-C {dir}")), "{cmd}");
    assert_eq!(
        t.read(".dstack/local/exec/check/sandbox"),
        format!("workspace-write {dir}\n")
    );
    assert_eq!(t.read("trace/cwd").trim(), dir);
}

#[test]
fn r05_without_writable_the_read_only_argv_and_receipt_stay() {
    let t = Scratch::new("claude", "codex");
    let planned = t.run("audit", &["--run", "sample", "--dry-run"]);
    assert!(
        planned.status.success(),
        "{}",
        String::from_utf8_lossy(&planned.stderr)
    );
    let (plan, argv) = argv(&planned.stdout);
    assert_eq!(after(&argv, "--sandbox"), "read-only");
    let worktree = t.0.join("work").canonicalize().unwrap();
    assert_eq!(after(&argv, "-C"), worktree.to_str().unwrap());
    assert!(
        !argv.iter().any(|value| value == "workspace-write"),
        "{argv:?}"
    );
    assert_eq!(plan["cwd"], worktree.to_str().unwrap());
    let output = t.run("audit", &["--run", "sample"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !t.0.join(".dstack/local/exec/check/sandbox").exists(),
        "wrote a sandbox receipt without --writable"
    );
}
