#[path = "support/handoff.rs"]
mod support;
use std::fs;
use support::{success, Scratch};

#[test]
fn r08_handoff_summary_destination_provider_and_read_only_role() {
    for (from, to) in [("claude", "codex"), ("codex", "claude")] {
        let s = Scratch::new(from, from);
        let before = s.read(".dstack/runs/sample/meta.tsv");
        success(s.prepare(to, &[]));
        assert_eq!(s.read("trace/calls"), format!("{to}\n"));
        assert_eq!(s.read(".dstack/runs/sample/meta.tsv"), before);
        let args = s.read("trace/argv");
        assert!(!args.contains("WebSearch") && !args.contains("tools.web_search=true"));
        assert!(args.contains(if to == "codex" { "read-only" } else { "dontAsk" }));
        let prompt = s.read("trace/stdin");
        assert!(prompt.contains("=== ROLE INSTRUCTIONS (stable) ==="));
        assert!(prompt.contains("handoff.md"));
        assert!(s.packet().join("RESUME.md").is_file());
    }
}

#[test]
fn r08_handoff_summary_dry_run_is_read_only_and_provider_failure_never_applies() {
    let s = Scratch::new("claude", "claude");
    let out = success(s.prepare("codex", &["--dry-run"]));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["provider"], "codex");
    assert!(!s.0.join("trace/calls").exists());
    assert!(!s.0.join(".dstack/runs/sample/handoffs").exists());
    let before = s.read(".dstack/runs/sample/meta.tsv");
    s.write("scenario", "fail");
    assert_eq!(s.prepare("codex", &[]).status.code(), Some(23));
    assert_eq!(s.read(".dstack/runs/sample/meta.tsv"), before);
    assert_eq!(s.read(".dstack/runs/sample/mode.json"), s.read(".dstack/project/mode.json"));
}

#[test]
fn r08_handoff_summary_rejects_missing_tasks_unknown_refs_and_empty_detail() {
    for scenario in ["missing", "ref", "empty", "duplicate-ref", "duplicate"] {
        let s = Scratch::new("claude", "claude");
        let mut value = Scratch::summary();
        match scenario {
            "missing" => value["active"] = serde_json::json!([]),
            "ref" => value["active"][0]["refs"] = serde_json::json!(["invented:99"]),
            "empty" => value["active"][0]["changes"] = "".into(),
            "duplicate-ref" => {
                value["active"][0]["refs"] = serde_json::json!(["task:T1", "task:T1"])
            }
            _ => {
                let item = value["active"][0].clone();
                value["active"].as_array_mut().unwrap().push(item);
            }
        }
        s.write("trace/summary", &value.to_string());
        let before = s.read(".dstack/runs/sample/meta.tsv");
        let out = s.prepare("codex", &[]);
        assert!(!out.status.success(), "{scenario}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("summary"));
        assert_eq!(s.read(".dstack/runs/sample/meta.tsv"), before);
    }
}

#[test]
fn r09_handoff_resume_requires_stop_ack_and_preserves_project_and_sub() {
    for (from, to, sub) in [("claude", "codex", "claude"), ("codex", "claude", "codex")] {
        let s = Scratch::new(from, sub);
        success(s.prepare(to, &[]));
        let p = s.packet();
        let project = s.read(".dstack/project/mode.json");
        let cases = s.read(".dstack/runs/sample/cases.tsv");
        assert!(!s.resume(&p, to, &[]).status.success());
        assert!(!s.resume(&p, from, &["--source-stopped"]).status.success());
        success(s.resume(&p, to, &["--source-stopped"]));
        let mode: serde_json::Value =
            serde_json::from_str(&s.read(".dstack/runs/sample/mode.json")).unwrap();
        assert_eq!(mode["main"], to);
        assert_eq!(mode["sub"], sub);
        assert_eq!(s.read(".dstack/project/mode.json"), project);
        assert_eq!(s.read(".dstack/runs/sample/cases.tsv"), cases);
        assert!(s.read(".dstack/runs/sample/meta.tsv").contains("owner_session\tdestination"));
        assert!(!s.resume(&p, to, &["--source-stopped"]).status.success());
    }
}

#[test]
fn r09_handoff_resume_refuses_stale_history_git_packet_and_live_execution() {
    for change in ["history", "git", "packet", "live"] {
        let s = Scratch::new("claude", "codex");
        success(s.prepare("codex", &[]));
        let p = s.packet();
        let before = s.read(".dstack/runs/sample/meta.tsv");
        match change {
            "history" => {
                s.write("history.jsonl", "changed");
            }
            "git" => {
                s.write("work.txt", "changed");
            }
            "packet" => {
                fs::write(p.join("summary.json"), "{}").unwrap();
            }
            _ => {
                s.write(".dstack/local/exec/unfinished/started_at", "2026-09-06T00:00:00Z\n");
            }
        }
        assert!(!s.resume(&p, "codex", &["--source-stopped"]).status.success(), "{change}");
        assert_eq!(s.read(".dstack/runs/sample/meta.tsv"), before);
    }
}

#[test]
fn r08_handoff_summary_change_during_summarization_is_not_ready() {
    let s = Scratch::new("claude", "codex");
    s.write("scenario", "mutate");
    let out = s.prepare("codex", &[]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("changed")
            || String::from_utf8_lossy(&out.stderr).contains("stale")
    );
}
