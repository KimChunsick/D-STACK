#![allow(non_snake_case)]
use dstack_cli::core::mode::Provider::{self, Claude, Codex};
use dstack_cli::handoff::history::load;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Trees {
    base: PathBuf,
    root: PathBuf,
    linked: PathBuf,
    other: PathBuf,
}
impl Trees {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "dstack-R11-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&base).unwrap();
        let base = base.canonicalize().unwrap();
        let root = base.join("root");
        let linked = base.join("linked");
        let other = base.join("other");
        for repo in [&root, &other] {
            fs::create_dir(repo).unwrap();
            git(repo, &["init", "-q"]);
            git(
                repo,
                &[
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.test",
                    "-c",
                    "commit.gpgsign=false",
                    "commit",
                    "--allow-empty",
                    "-qm",
                    "fixture",
                ],
            );
        }
        git(&root, &["worktree", "add", "-q", "--detach", linked.to_str().unwrap()]);
        Self { base, root, linked, other }
    }
    fn source(&self, records: &[Value]) -> (PathBuf, String) {
        let bytes: String = records.iter().map(|v| format!("{v}\n")).collect();
        let path = self.base.join("source.jsonl");
        fs::write(&path, &bytes).unwrap();
        (path, bytes)
    }
}
impl Drop for Trees {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
fn git(root: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    for key in [
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        cmd.env_remove(key);
    }
    let out = cmd.arg("-C").arg(root).args(args).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}
fn claude(cwd: &Path) -> Value {
    json!({"type":"user","sessionId":"session-1","cwd":cwd,"message":{"role":"user","content":"visible"}})
}
fn meta(cwd: &Path) -> Value {
    json!({"type":"session_meta","payload":{"id":"session-1","cwd":cwd}})
}
fn context(cwd: &Path) -> Value {
    json!({"type":"turn_context","payload":{"cwd":cwd}})
}
fn message() -> Value {
    json!({"type":"response_item","payload":{"type":"message","role":"user","content":"visible"}})
}
fn rejected(t: &Trees, records: &[Value], provider: Provider, target: &Path, expected: &str) {
    let (path, _) = t.source(records);
    let error = load(&path, provider, "session-1", target).unwrap_err().to_string();
    assert!(error.contains(expected), "expected {expected}, got {error}");
}

#[test]
fn R11__claude_and_codex_linked_moves_preserve_original_evidence() {
    let t = Trees::new();
    for (provider, records, refs) in [
        (
            Claude,
            vec![claude(&t.root), claude(&t.linked), claude(&t.root)],
            vec!["history:1", "history:2", "history:3"],
        ),
        (
            Codex,
            vec![meta(&t.root), message(), context(&t.linked), message(), context(&t.root)],
            vec!["history:2", "history:4"],
        ),
    ] {
        let (path, bytes) = t.source(&records);
        for target in [&t.root, &t.linked] {
            let history = load(&path, provider, "session-1", target).unwrap();
            assert_eq!(history.cwd, target.to_str().unwrap());
            assert_eq!(history.sha256, format!("{:x}", Sha256::digest(bytes.as_bytes())));
            assert_eq!(
                history.records.iter().map(|r| r.reference.as_str()).collect::<Vec<_>>(),
                refs
            );
            let warnings = history.warnings.join("\n");
            assert!(
                warnings.contains(t.root.to_str().unwrap())
                    && warnings.contains(t.linked.to_str().unwrap())
            );
            assert!(warnings.contains("history:1"));
            assert!(warnings.contains(if provider == Claude { "history:2" } else { "history:3" }));
            assert_eq!(fs::read_to_string(&path).unwrap(), bytes);
        }
    }
}

#[test]
fn R11__rejects_unrelated_nonroot_nonexistent_relative_and_unvisited_cwds() {
    let t = Trees::new();
    let subdir = t.root.join("nested");
    fs::create_dir(&subdir).unwrap();
    for bad in [&t.other, &subdir, &t.base.join("missing"), Path::new("relative")] {
        rejected(&t, &[claude(&t.linked), claude(bad)], Claude, &t.linked, "worktree");
        rejected(&t, &[meta(&t.linked), context(bad), message()], Codex, &t.linked, "worktree");
    }
    rejected(&t, &[claude(&t.root)], Claude, &t.linked, "selected worktree");
    rejected(&t, &[meta(&t.root), message()], Codex, &t.linked, "selected worktree");
    // An exact non-Git cwd remains valid, but cannot confer repository membership.
    let (path, _) = t.source(&[claude(&subdir)]);
    assert!(load(&path, Claude, "session-1", &subdir).is_ok());
    rejected(&t, &[claude(&subdir), claude(&t.root)], Claude, &subdir, "worktree");
}

#[test]
fn R11__retains_session_and_header_validation_across_moves() {
    let t = Trees::new();
    let mut wrong = claude(&t.linked);
    wrong["sessionId"] = json!("other-session");
    rejected(&t, &[claude(&t.root), wrong], Claude, &t.linked, "session identity");
    let mut wrong = context(&t.linked);
    wrong["payload"]["session_id"] = json!("other-session");
    rejected(&t, &[meta(&t.root), wrong, message()], Codex, &t.linked, "session identity");
    rejected(&t, &[context(&t.linked), message()], Codex, &t.linked, "session_meta header");
    rejected(&t, &[meta(&t.root), meta(&t.linked), message()], Codex, &t.linked, "duplicate");
}

#[test]
fn R11__claude_relocation_metadata_stays_nonmessage_and_checks_identity() {
    let t = Trees::new();
    let records = vec![
        json!({"type":"relocated","sessionId":"session-1","relocatedCwd":t.linked}),
        json!({"type":"worktree-state","sessionId":"session-1","worktreeSession":{"private":"HIDDEN_SENTINEL"}}),
        json!({"type":"file-history-delta","backup":{"private":"HIDDEN_SENTINEL"},"messageId":"m1","snapshotMessageId":"m0","timestamp":"now","trackingPath":"private"}),
        claude(&t.linked),
    ];
    let (path, _) = t.source(&records);
    let history = load(&path, Claude, "session-1", &t.linked).unwrap();
    assert_eq!(history.records.len(), 1);
    assert_eq!(history.records[0].reference, "history:4");
    assert!(!serde_json::to_string(&history).unwrap().contains("HIDDEN_SENTINEL"));
    rejected(&t, &[records[0].clone(), claude(&t.root)], Claude, &t.linked, "selected worktree");
    for kind in ["relocated", "worktree-state", "file-history-delta"] {
        rejected(
            &t,
            &[claude(&t.linked), json!({"type":kind,"sessionId":"other"})],
            Claude,
            &t.linked,
            "session identity",
        );
        rejected(
            &t,
            &[claude(&t.linked), json!({"type":kind,"cwd":t.other})],
            Claude,
            &t.linked,
            "worktree",
        );
    }
}

#[cfg(unix)]
#[test]
fn R11__canonical_aliases_and_repeated_cwds_have_bounded_warnings() {
    let t = Trees::new();
    let alias = t.base.join("alias");
    std::os::unix::fs::symlink(&t.linked, &alias).unwrap();
    let records: Vec<_> =
        (0..1000).map(|i| claude(if i % 2 == 0 { &t.root } else { &alias })).collect();
    let (path, _) = t.source(&records);
    let history = load(&path, Claude, "session-1", &t.linked).unwrap();
    assert_eq!(history.cwd, t.linked.to_str().unwrap());
    assert!(history.warnings.join("\n").len() < 8192);
    assert_eq!(history.records.last().unwrap().reference, "history:1000");
    let aliases: Vec<_> = (0..65)
        .map(|i| {
            let alias = t.base.join(format!("alias-{i}"));
            std::os::unix::fs::symlink(&t.linked, &alias).unwrap();
            claude(&alias)
        })
        .collect();
    rejected(&t, &aliases, Claude, &t.linked, "bound");
}

#[test]
fn R11__git_environment_cannot_spoof_repository_identity() {
    if std::env::var_os("DSTACK_R11_CHILD").is_none() {
        let t = Trees::new();
        let result = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "R11__git_environment_cannot_spoof_repository_identity",
                "--nocapture",
            ])
            .env("DSTACK_R11_CHILD", "1")
            .env("GIT_DIR", t.root.join(".git"))
            .env("GIT_COMMON_DIR", t.root.join(".git"))
            .env("GIT_WORK_TREE", &t.linked)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    let t = Trees::new();
    let (path, _) = t.source(&[claude(&t.root), claude(&t.linked)]);
    assert!(load(&path, Claude, "session-1", &t.linked).is_ok());
    rejected(&t, &[claude(&t.linked), claude(&t.other)], Claude, &t.linked, "worktree");
}
