use dstack_cli::core::mode::Provider::{Claude, Codex};
use dstack_cli::handoff::history::{load, locate};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "dstack-handoff-history-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(fs::canonicalize(path).unwrap())
    }
    fn file(&self, relative: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }
    fn claude(&self, role: &str, content: Value) -> Value {
        json!({"type":role,"sessionId":"session-1","cwd":self.0,
            "message":{"role":role,"content":content}})
    }
    fn meta(&self) -> Value {
        json!({"type":"session_meta","payload":{"id":"session-1","cwd":self.0,
            "originator":"codex_cli_rs","cli_version":"0.100.0"}})
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn jsonl(values: &[Value]) -> String {
    values.iter().map(|v| format!("{v}\n")).collect()
}
fn response(payload: Value) -> Value {
    json!({"type":"response_item","payload":payload})
}
fn fails(
    path: &Path,
    provider: dstack_cli::core::mode::Provider,
    session: &str,
    cwd: &Path,
    expected: &str,
) {
    let error = load(path, provider, session, cwd).unwrap_err().to_string();
    assert!(error.contains(expected), "expected {expected:?}, got {error}");
}

#[test]
fn r06_handoff_history_claude_original_refs_and_visible_tool_evidence() {
    let t = Scratch::new();
    let source = jsonl(&[
        json!({"type":"file-history-snapshot","messageId":"message-1","snapshot":{}}),
        t.claude("user", json!("Keep the current requirement.")),
        t.claude(
            "assistant",
            json!([
                {"type":"thinking","thinking":"PRIVATE_THINKING"},
                {"type":"redacted_thinking","data":"PRIVATE_ENCRYPTED"},
                {"type":"text","text":""},
                {"type":"text","text":"I will inspect the file."},
                {"type":"tool_use","id":"tool-1","name":"Read","input":{"file_path":"src/main.rs"}}
            ]),
        ),
        t.claude(
            "user",
            json!([{"type":"tool_result","tool_use_id":"tool-1",
            "content":[{"type":"text","text":"pub fn main() {}"}]}]),
        ),
    ]);
    let path = t.file("source.jsonl", &source);
    let history = load(&path, Claude, "session-1", &t.0).unwrap();
    assert_eq!(history.provider, Claude);
    assert_eq!(history.session, "session-1");
    assert_eq!(history.cwd, t.0.to_str().unwrap());
    assert_eq!(history.path, fs::canonicalize(path).unwrap().to_str().unwrap());
    assert_eq!(history.sha256, format!("{:x}", Sha256::digest(source.as_bytes())));
    assert_eq!(
        history.records.iter().map(|r| r.reference.as_str()).collect::<Vec<_>>(),
        ["history:2", "history:3", "history:3", "history:4"]
    );
    assert_eq!(
        history.records.iter().map(|r| r.kind.as_str()).collect::<Vec<_>>(),
        ["user", "assistant", "tool", "tool"]
    );
    let text = serde_json::to_string(&history).unwrap();
    assert!(
        text.contains("src/main.rs") && text.contains("tool-1") && text.contains("pub fn main")
    );
    assert!(!text.contains("PRIVATE_"));
}

#[test]
fn r06_handoff_history_codex_formats_exclude_analysis_and_encrypted_content() {
    let t = Scratch::new();
    let source = jsonl(&[
        t.meta(),
        response(
            json!({"type":"message","role":"user","content":[{"type":"input_text","text":"Start here."}]}),
        ),
        response(json!({"type":"message","role":"assistant","channel":"analysis",
            "content":[{"type":"output_text","text":"PRIVATE_ANALYSIS"}]})),
        response(json!({"type":"reasoning","encrypted_content":"PRIVATE_ENCRYPTED","summary":[]})),
        response(
            json!({"type":"function_call","name":"exec_command","arguments":"{\"cmd\":\"git status\"}","call_id":"call-1"}),
        ),
        response(
            json!({"type":"function_call_output","call_id":"call-1","output":"working tree clean"}),
        ),
        response(json!({"type":"message","role":"assistant","channel":"final",
            "content":[{"type":"output_text","text":"Ready to continue."}]})),
    ]);
    let path = t.file("arbitrary-name.jsonl", &source);
    let history = load(&path, Codex, "session-1", &t.0).unwrap();
    assert_eq!(
        history.records.iter().map(|r| r.reference.as_str()).collect::<Vec<_>>(),
        ["history:2", "history:5", "history:6", "history:7"]
    );
    let text = serde_json::to_string(&history).unwrap();
    assert!(
        text.contains("git status")
            && text.contains("working tree clean")
            && text.contains("call-1")
    );
    assert!(!text.contains("PRIVATE_"));
    assert_eq!(history.sha256, format!("{:x}", Sha256::digest(source.as_bytes())));
}

#[test]
fn r06_handoff_history_current_claude_metadata_records_are_not_messages() {
    let t = Scratch::new();
    let mut records: Vec<_> = [
        "mode",
        "permission-mode",
        "bridge-session",
        "attachment",
        "ai-title",
        "atis-latch",
        "cost-state",
    ]
    .iter()
    .map(|kind| json!({"type":kind}))
    .collect();
    records.push(t.claude("user", json!("Continue the pending task.")));
    let path = t.file("source.jsonl", jsonl(&records));
    let history = load(&path, Claude, "session-1", &t.0).unwrap();
    assert_eq!(history.records.len(), 1);
    assert_eq!(history.records[0].reference, "history:8");
}

#[test]
fn r06_handoff_history_current_codex_metadata_records_are_not_messages() {
    let t = Scratch::new();
    let mut records = vec![t.meta()];
    records.extend(
        ["world_state", "token_usage_record", "inter_agent_communication_metadata"]
            .iter()
            .map(|kind| json!({"type":kind,"payload":{}})),
    );
    records.push(response(json!({"type":"message","role":"user","content":[{"type":"input_text","text":"Continue."}]})));
    let path = t.file("source.jsonl", jsonl(&records));
    let history = load(&path, Codex, "session-1", &t.0).unwrap();
    assert_eq!(history.records.len(), 1);
    assert_eq!(history.records[0].reference, "history:5");
}

#[test]
fn r06_handoff_history_rejects_identity_provider_worktree_and_headerless_input() {
    let t = Scratch::new();
    let path = t.file("session-1.jsonl", jsonl(&[t.claude("user", json!("hello"))]));
    fails(&path, Claude, "", &t.0, "session");
    fails(&path, Claude, "other", &t.0, "session");
    fails(&path, Codex, "session-1", &t.0, "provider");
    let other = Scratch::new();
    fails(&path, Claude, "session-1", &other.0, "worktree");
    let path = t.file(
        "headerless.jsonl",
        jsonl(&[response(json!({"type":"message","role":"user",
        "content":[{"type":"input_text","text":"hello"}]}))]),
    );
    fails(&path, Codex, "session-1", &t.0, "session_meta");
    let path = t.file(
        "missing-identity.jsonl",
        "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"hello\"}}\n",
    );
    fails(&path, Claude, "session-1", &t.0, "session");
    let mut wrong = t.claude("assistant", json!("changed"));
    wrong["sessionId"] = json!("other");
    let path = t.file("mixed.jsonl", jsonl(&[t.claude("user", json!("hello")), wrong]));
    fails(&path, Claude, "session-1", &t.0, "session");
    let path = t.file("duplicate.jsonl", jsonl(&[t.meta(), t.meta()]));
    fails(&path, Codex, "session-1", &t.0, "ambiguous");
    let path = t.file("codex.jsonl", jsonl(&[t.meta()]));
    fails(&path, Claude, "session-1", &t.0, "provider");
    fails(&path, Codex, "other", &t.0, "session");
    fails(&path, Codex, "session-1", &other.0, "worktree");
    let changed = json!({"type":"turn_context","payload":{"cwd":other.0}});
    let path = t.file("changed-cwd.jsonl", jsonl(&[t.meta(), changed]));
    fails(&path, Codex, "session-1", &t.0, "worktree");
}

#[test]
fn r06_handoff_history_missing_malformed_and_incomplete_tail() {
    let t = Scratch::new();
    fails(&t.0.join("missing.jsonl"), Claude, "session-1", &t.0, "history");
    let valid = jsonl(&[t.claude("user", json!("visible"))]);
    for invalid in ["", "{}\n", "[]\n", "not json\n"] {
        let path = t.file("invalid.jsonl", invalid);
        assert!(load(&path, Claude, "session-1", &t.0).is_err(), "accepted {invalid:?}");
    }
    let path = t.file("broken.jsonl", format!("{valid}{{broken\n{valid}"));
    fails(&path, Claude, "session-1", &t.0, "line 2");
    let path = t.file("broken-final.jsonl", format!("{valid}{{broken"));
    fails(&path, Claude, "session-1", &t.0, "line 2");
    let source = format!("{valid}{{\"type\":");
    let path = t.file("incomplete.jsonl", &source);
    let history = load(&path, Claude, "session-1", &t.0).unwrap();
    assert_eq!(history.records.len(), 1);
    assert!(history.warnings.iter().any(|w| w.contains("incomplete") && w.contains("2")));
    assert_eq!(history.sha256, format!("{:x}", Sha256::digest(source.as_bytes())));
}

#[test]
fn r06_handoff_history_long_input_keeps_recent_records_with_bounded_output() {
    let t = Scratch::new();
    let mut values: Vec<_> =
        (0..500).map(|i| t.claude("user", json!(format!("record-{i}")))).collect();
    values.push(t.claude("assistant", json!("한".repeat(20_000))));
    values.push(t.claude("user", json!("newest useful record")));
    let path = t.file("long.jsonl", jsonl(&values));
    let history = load(&path, Claude, "session-1", &t.0).unwrap();
    assert!(history.omitted > 0 && history.records.len() <= 200);
    assert!(history.warnings.iter().any(|w| w.contains("omitted")));
    assert!(history.warnings.iter().any(|w| w.contains("truncated")));
    assert_eq!(history.records.last().unwrap().reference, "history:502");
    assert_eq!(history.records.last().unwrap().text, "newest useful record");
    assert!(serde_json::to_vec(&history).unwrap().len() < 150_000);
}

#[test]
fn r06_handoff_history_rejects_oversized_files_and_lines() {
    let t = Scratch::new();
    let path = t.file("oversized.jsonl", "");
    fs::OpenOptions::new().write(true).open(&path).unwrap().set_len(64 * 1024 * 1024 + 1).unwrap();
    fails(&path, Claude, "session-1", &t.0, "oversized");
    let path = t.file("line.jsonl", jsonl(&[t.claude("user", json!("x".repeat(2 * 1024 * 1024)))]));
    fails(&path, Claude, "session-1", &t.0, "oversized");
}

#[test]
fn r06_handoff_history_locates_only_exact_provider_and_session_filenames() {
    let t = Scratch::new();
    let encoded: String =
        t.0.to_str()
            .unwrap()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
    let claude =
        t.file(&format!(".claude/projects/{encoded}/session-1.jsonl"), "contents are not searched");
    t.file(".claude/projects/unrelated/session-1.jsonl", "ignored");
    assert_eq!(locate(Claude, "session-1", &t.0, &t.0).unwrap(), claude);
    let codex = t.file(
        ".codex/sessions/2026/09/06/rollout-2026-09-06T12-00-00-session-1.jsonl",
        "not inspected",
    );
    t.file(".codex/sessions/2026/09/06/rollout-newer-session-10.jsonl", "ignored");
    t.file(".codex/sessions/2026/09/06/arbitrary-session-1.jsonl", "ignored");
    assert_eq!(locate(Codex, "session-1", &t.0, &t.0).unwrap(), codex);
    assert!(locate(Codex, "missing", &t.0, &t.0).unwrap_err().to_string().contains("missing"));
    assert!(locate(Claude, "../session-1", &t.0, &t.0).is_err());
    t.file(".codex/sessions/2026/09/05/rollout-older-session-1.jsonl", "duplicate");
    assert!(locate(Codex, "session-1", &t.0, &t.0).unwrap_err().to_string().contains("ambiguous"));
}

#[cfg(unix)]
#[test]
fn r06_handoff_history_resolves_canonical_paths_and_rejects_unreadable_candidates() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let t = Scratch::new();
    let path = t.file("real.jsonl", jsonl(&[t.claude("user", json!("visible"))]));
    symlink(&t.0, t.0.join("alias")).unwrap();
    let history =
        load(&t.0.join("alias/real.jsonl"), Claude, "session-1", &t.0.join("alias")).unwrap();
    assert_eq!(history.path, path.to_str().unwrap());
    assert_eq!(history.cwd, t.0.to_str().unwrap());
    let bad = t.file(".codex/sessions/2026/09/06/rollout-old-session-1.jsonl", "unreadable");
    fs::set_permissions(&bad, fs::Permissions::from_mode(0)).unwrap();
    if fs::File::open(&bad).is_err() {
        let error = locate(Codex, "session-1", &t.0, &t.0).unwrap_err().to_string();
        assert!(error.contains("read") && !error.contains("missing"), "{error}");
    }
    fs::set_permissions(&bad, fs::Permissions::from_mode(0o600)).unwrap();
    fs::remove_file(&bad).unwrap();
    symlink(t.0.join("missing-target"), &bad).unwrap();
    assert!(locate(Codex, "session-1", &t.0, &t.0).is_err());
}

#[test]
fn r06_handoff_history_detects_input_changing_during_reading() {
    use std::sync::{atomic::AtomicBool, Arc, Barrier};
    use std::time::{Duration, SystemTime};
    let t = Scratch::new();
    let line = jsonl(&[t.claude("user", json!("visible".repeat(200)))]);
    let path = t.file("changing.jsonl", line.repeat(6000));
    let writer = fs::OpenOptions::new().write(true).open(&path).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let gate = Arc::new(Barrier::new(2));
    let (done, ready) = (stop.clone(), gate.clone());
    let thread = std::thread::spawn(move || {
        ready.wait();
        let mut tick = 0;
        while !done.load(Ordering::Relaxed) {
            writer.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(tick)).unwrap();
            tick += 1;
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    gate.wait();
    let result = load(&path, Claude, "session-1", &t.0);
    stop.store(true, Ordering::Relaxed);
    thread.join().unwrap();
    assert!(result.unwrap_err().to_string().contains("changed"));
}
