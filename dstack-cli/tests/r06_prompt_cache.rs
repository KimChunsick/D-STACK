// Stable prompt bytes and provider-specific usage accounting, without model/network calls.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn r06_multiple_logs_use_token_weighting_and_reject_duplicate_files() {
    let t = Scratch::new();
    let a = t.file("a", "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":100,\"cached_input_tokens\":100,\"output_tokens\":0}}\n");
    let b = t.file("b", "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":900,\"cached_input_tokens\":0,\"output_tokens\":0}}\n");
    let out = t.run(&[
        "prompt",
        "usage",
        "--provider",
        "codex",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
    ]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["cache_read_ratio"], 0.1);
    let out = t.run(&[
        "prompt",
        "usage",
        "--provider",
        "codex",
        a.to_str().unwrap(),
        a.to_str().unwrap(),
    ]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    let zero = t.file("zero", "{\"type\":\"result\",\"usage\":{\"input_tokens\":0,\"cache_read_input_tokens\":0,\"cache_creation_input_tokens\":0,\"output_tokens\":0}}\n");
    let out = t.run(&[
        "prompt",
        "usage",
        "--provider",
        "claude",
        zero.to_str().unwrap(),
    ]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["cache_read_ratio"].is_null());
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "dstack-cache-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn file(&self, name: &str, text: &str) -> PathBuf {
        let p = self.0.join(name);
        fs::write(&p, text).unwrap();
        p
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dstack"))
            .env(
                "DSTACK_HOME",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"),
            )
            .current_dir(&self.0)
            .args(args)
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
fn r06_role_prefix_is_identical_across_paths_rounds_and_korean_context() {
    let t = Scratch::new();
    for role in ["review", "research", "audit", "worker"] {
        let a = t.file(
            "a.md",
            "회차: 001\n- [ ] **R01** 캐시를 재사용해요 — accept: 원문을 보존해요\n",
        );
        let b = t.file("b.md", "회차: 002\n경로: /other/worktree\n변경: +새 코드\n");
        let render = |p: &PathBuf| {
            t.run(&[
                "prompt",
                "render",
                "--role",
                role,
                "--context",
                p.to_str().unwrap(),
            ])
        };
        let one = render(&a);
        assert!(
            one.status.success(),
            "{}",
            String::from_utf8_lossy(&one.stderr)
        );
        let two = render(&b);
        assert!(two.status.success());
        let one_text = String::from_utf8(one.stdout).unwrap();
        let two_text = String::from_utf8(two.stdout).unwrap();
        let split = "=== TASK CONTEXT (variable) ===\n";
        assert_eq!(
            one_text.split_once(split).unwrap().0,
            two_text.split_once(split).unwrap().0
        );
        assert!(one_text.ends_with(&fs::read_to_string(a).unwrap()));
        assert!(two_text.ends_with(&fs::read_to_string(b).unwrap()));
        assert_eq!(
            one.stderr, two.stderr,
            "metadata must describe only the stable prefix"
        );
        assert!(!one_text.contains(t.0.to_str().unwrap()));
    }
}

#[test]
fn r06_render_fails_without_partial_prompt() {
    let t = Scratch::new();
    let empty = t.file("empty", " \n");
    for args in [
        vec!["--role", "unknown", "--context", "missing"],
        vec!["--role", "review", "--context", "missing"],
        vec!["--role", "review", "--context", empty.to_str().unwrap()],
        vec![
            "--role",
            "review",
            "--role",
            "audit",
            "--context",
            "missing",
        ],
    ] {
        let mut all = vec!["prompt", "render"];
        all.extend(args);
        let out = t.run(&all);
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn r06_research_and_audit_share_the_canonical_role_source() {
    let t = Scratch::new();
    let p = t.file("context", "R01: 조사 내용을 검증해요\n");
    let render = |role| {
        t.run(&[
            "prompt",
            "render",
            "--role",
            role,
            "--context",
            p.to_str().unwrap(),
        ])
    };
    let research = render("research");
    let audit = render("audit");
    assert!(research.status.success() && audit.status.success());
    assert_eq!(research.stderr, audit.stderr);
    let text = String::from_utf8(research.stdout).unwrap();
    let source = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../codex/skills/dstack-researcher/SKILL.md"),
    )
    .unwrap();
    assert!(text.contains(&source));
    assert_eq!(text.matches(&source).count(), 1);
}

#[test]
fn r06_exec_captures_usage_and_preserves_stdin_and_failure_exit() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Stdio;
    let t = Scratch::new();
    let fake = t.file("codex", "#!/bin/sh\ncat > received\necho '{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":10,\"cached_input_tokens\":5,\"output_tokens\":1}}'\nexit 7\n");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .env(
            "DSTACK_HOME",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"),
        )
        .env("DSTACK_ROOT", &t.0)
        .current_dir(&t.0)
        .args([
            "exec",
            "sample",
            "--",
            fake.to_str().unwrap(),
            "--json",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("원문\n\n".as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(7));
    assert_eq!(
        fs::read_to_string(t.0.join("received")).unwrap(),
        "원문\n\n"
    );
    let capture = t.0.join(".dstack/local/exec/sample");
    assert_eq!(fs::read_to_string(capture.join("exit")).unwrap(), "7\n");
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(capture.join("usage.json")).unwrap()).unwrap();
    assert_eq!(report["cache_read_ratio"], 0.5);
    fs::write(&fake, "#!/bin/sh\necho not-json\nexit 9\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .env(
            "DSTACK_HOME",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"),
        )
        .env("DSTACK_ROOT", &t.0)
        .current_dir(&t.0)
        .args(["exec", "sample", "--", fake.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(9));
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(t.0.join(".dstack/local/exec/sample.1/usage.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["status"], "skipped");
    assert!(report.get("cache_read_ratio").is_none());
}

#[test]
fn r06_usage_counts_codex_turns_and_claude_result_once() {
    let t = Scratch::new();
    let cases = [
        ("codex", concat!(
            "{\"type\":\"item.completed\",\"usage\":{\"input_tokens\":999}}\n",
            "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":1000,\"cached_input_tokens\":900,\"cache_write_input_tokens\":100,\"output_tokens\":10}}\n",
            "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":100,\"cached_input_tokens\":0,\"cache_write_input_tokens\":100,\"output_tokens\":5}}\n"), 1100, 900, 200, 2),
        ("claude", concat!(
            "{\"type\":\"assistant\",\"message\":{\"usage\":{\"input_tokens\":999}}}\n",
            "{\"type\":\"result\",\"usage\":{\"input_tokens\":100,\"cache_read_input_tokens\":900,\"cache_creation_input_tokens\":200,\"output_tokens\":15}}\n"), 1200, 900, 200, 1),
    ];
    for (provider, events, input, read, write, samples) in cases {
        let p = t.file(provider, events);
        let out = t.run(&[
            "prompt",
            "usage",
            "--provider",
            provider,
            p.to_str().unwrap(),
        ]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["input_tokens"], input);
        assert_eq!(v["cache_read_tokens"], read);
        assert_eq!(v["cache_write_tokens"], write);
        assert_eq!(v["samples"], samples);
        assert_eq!(v["output_tokens"], 15);
        assert!(
            (v["cache_read_ratio"].as_f64().unwrap() - read as f64 / input as f64).abs() < 1e-10
        );
    }
}

#[test]
fn r06_usage_never_turns_missing_or_invalid_telemetry_into_zero_hits() {
    let t = Scratch::new();
    for events in [
        "plain text", "{\"type\":\"turn.started\"}\n",
        "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":1}}\n",
        "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":1,\"cached_input_tokens\":2,\"output_tokens\":0}}\n",
    ] {
        let p = t.file("bad", events);
        let out = t.run(&["prompt", "usage", "--provider", "codex", p.to_str().unwrap()]);
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
    }
    let p = t.file("older", "{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":100,\"cached_input_tokens\":0,\"output_tokens\":1}}\n");
    let out = t.run(&[
        "prompt",
        "usage",
        "--provider",
        "codex",
        p.to_str().unwrap(),
    ]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        v["cache_write_tokens"].is_null(),
        "older CLI omitted writes; unknown is not zero"
    );
}
