#![allow(non_snake_case)]
#[path = "support/handoff.rs"]
mod support;
use dstack_cli::core::fsx::sha256_bytes;
use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use support::{success, Scratch};

const META: &str = ".dstack/runs/sample/meta.tsv";
const RECEIPT: &str = ".dstack/runs/sample/owner-recovery/intent.json";
fn fixture() -> Scratch {
    let s = Scratch::new("claude", "codex");
    s.write("trace/source.jsonl", &s.read("history.jsonl"));
    let old = s
        .read(META)
        .replace("owner_session\tsource", "owner_session\tdestination");
    s.write(
        META,
        &format!(
            "{old}transcript_path\t{}\nowner_pid\t4242\nowner_ts\t2026-09-06T00:00:00Z\n",
            s.0.join("trace/source.jsonl").display()
        ),
    );
    s
}
fn recovery_command(s: &Scratch, host: &str, path: &str, stopped: bool) -> Command {
    let mut c = s.command();
    c.args([
        "handoff",
        "recover-owner",
        "--run",
        "sample",
        "--host",
        host,
        "--session",
        "source",
        "--history",
        path,
    ]);
    if stopped {
        c.arg("--source-stopped");
    }
    c
}
fn recover(s: &Scratch, stopped: bool) -> Output {
    recovery_command(s, "codex", "trace/source.jsonl", stopped)
        .output()
        .unwrap()
}
fn rejected(s: &Scratch, output: Output, expected: &str, old: &str) {
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains(expected), "{error}");
    assert_eq!(s.read(META), old);
    assert!(!s.0.join(RECEIPT).exists());
}
#[test]
fn R12__queries_and_positional_cases_preserve_foreign_and_unowned_metadata() {
    for owner in ["source", ""] {
        let s = Scratch::new("claude", "codex");
        let old = s
            .read(META)
            .replace("owner_session\tsource", &format!("owner_session\t{owner}"));
        s.write(META, &old);
        for args in [
            vec!["gate"],
            vec!["request", "show", "--run", "sample"],
            vec!["cases", "sync", "sample"],
        ] {
            let output = s.command().args(&args).output().unwrap();
            if args[0] != "gate" {
                success(output);
            }
            assert_eq!(s.read(META), old);
        }
    }
}
#[test]
fn R12__matching_owner_queries_leave_metadata_unchanged() {
    let s = fixture();
    let old = s.read(META);
    for args in [
        vec!["request", "show", "--run", "sample"],
        vec!["cases", "sync", "sample"],
        vec!["gate"],
    ] {
        let output = s.command().args(&args).output().unwrap();
        if args[0] != "gate" {
            success(output);
        }
        assert_eq!(s.read(META), old);
    }
}
#[test]
fn R12__recovery_has_audited_receipt_and_preserves_workflow_and_normal_guards() {
    let s = fixture();
    s.write(META, &(s.read(META) + "custom\tkeep\r\n"));
    let old = s.read(META);
    let preserved: Vec<_> = [
        "mode.json",
        "request.md",
        "request.approved",
        "cases.tsv",
        "plan.json",
    ]
    .into_iter()
    .map(|p| (p, s.read(&format!(".dstack/runs/sample/{p}"))))
    .collect();
    success(recover(&s, true));
    let after = s.read(META);
    assert!(after.contains("owner_session\tsource\n"));
    assert!(!after.contains("owner_pid\t"));
    assert!(!after.contains("owner_ts\t"));
    assert!(after.contains("custom\tkeep\r\n"));
    let receipt: Value = serde_json::from_str(&s.read(RECEIPT)).unwrap();
    assert_eq!(receipt["original_meta"], old);
    assert_eq!(receipt["original_sha256"], sha256_bytes(old.as_bytes()));
    assert_eq!(receipt["proposed_meta"], after);
    assert_eq!(receipt["caller_session"], "destination");
    assert_eq!(receipt["source_session"], "source");
    assert_eq!(
        receipt["source_sha256"],
        sha256_bytes(s.read("trace/source.jsonl").as_bytes())
    );
    assert_eq!(receipt["source_stopped"], true);
    assert!(s
        .0
        .join(".dstack/runs/sample/owner-recovery/completed")
        .is_file());
    for (p, text) in preserved {
        assert_eq!(s.read(&format!(".dstack/runs/sample/{p}")), text);
    }
    assert_eq!(s.read(".dstack/local/CURRENT"), "sample\n");
    s.command().arg("gate").output().unwrap();
    assert_eq!(s.read(META), after);
    let bad = s.prepare("codex", &["--session", "destination", "--dry-run"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("--session must match"));
    success(s.prepare("codex", &[]));
    let packet = s.packet();
    let same = s
        .command()
        .env("DSTACK_SESSION_ID", "source")
        .args([
            "handoff",
            "resume",
            packet.file_name().unwrap().to_str().unwrap(),
            "--host",
            "codex",
            "--source-stopped",
        ])
        .output()
        .unwrap();
    assert!(!same.status.success());
    assert!(String::from_utf8_lossy(&same.stderr).contains("distinct nonempty"));
    success(s.resume(&packet, "codex", &["--source-stopped"]));
}
#[test]
fn R12__recovery_refuses_other_owner_wrong_source_corrupt_history_and_no_ack() {
    for scenario in [
        "owner",
        "source",
        "corrupt",
        "ack",
        "host",
        "cwd",
        "empty",
        "missing-path",
        "trailing",
    ] {
        let s = fixture();
        match scenario {
            "owner" => {
                s.write(
                    META,
                    &s.read(META)
                        .replace("owner_session\tdestination", "owner_session\tthird"),
                );
            }
            "source" => {
                s.write(
                    META,
                    &s.read(META)
                        .replace("trace/source.jsonl", "trace/other.jsonl"),
                );
            }
            "corrupt" => {
                s.write("trace/source.jsonl", "broken\n");
            }
            "host" => {
                s.write(
                    ".dstack/runs/sample/mode.json",
                    "{\"main\":\"codex\",\"sub\":\"claude\"}",
                );
            }
            "cwd" => {
                s.write(
                    "trace/source.jsonl",
                    &s.read("history.jsonl").replace(s.0.to_str().unwrap(), "/"),
                );
            }
            "empty" => {
                s.write("trace/source.jsonl", "");
            }
            "missing-path" => {
                s.write(
                    META,
                    &s.read(META)
                        .lines()
                        .filter(|l| !l.starts_with("transcript_path\t"))
                        .map(|l| format!("{l}\n"))
                        .collect::<String>(),
                );
            }
            "trailing" => {
                s.write(
                    "trace/source.jsonl",
                    &(s.read("history.jsonl") + "{\"type\":"),
                );
            }
            _ => (),
        }
        let old = s.read(META);
        rejected(
            &s,
            recover(&s, scenario != "ack"),
            match scenario {
                "owner" => "current saved owner",
                "source" => "saved transcript filename",
                "corrupt" => "malformed",
                "ack" => "--source-stopped",
                "host" => "different",
                "cwd" => "worktree",
                "empty" => "history",
                "missing-path" => "transcript_path",
                _ => "incomplete",
            },
            &old,
        );
    }
}
#[test]
fn R12__existing_path_must_match_but_moved_exact_file_can_recover() {
    let s = fixture();
    s.write("trace/moved/source.jsonl", &s.read("history.jsonl"));
    let old = s.read(META);
    let mut command = recovery_command(&s, "codex", "trace/moved/source.jsonl", true);
    rejected(
        &s,
        command.output().unwrap(),
        "stored transcript path",
        &old,
    );
    fs::remove_file(s.0.join("trace/source.jsonl")).unwrap();
    success(command.output().unwrap());
    assert!(s.read(META).contains(&format!(
        "transcript_path\t{}\n",
        s.0.join("trace/moved/source.jsonl").display()
    )));
}
#[test]
fn R12__recovery_refuses_handoff_and_uncertain_recovery_guards() {
    for guard in ["ready", "resuming", "consumed", "context.md", "recovery"] {
        let s = fixture();
        let path = if guard == "recovery" {
            ".dstack/runs/sample/owner-recovery/uncertain".into()
        } else {
            format!(".dstack/runs/sample/handoffs/old/{guard}")
        };
        s.write(&path, "partial");
        let old = s.read(META);
        rejected(
            &s,
            recover(&s, true),
            if guard == "recovery" {
                "prior owner recovery"
            } else {
                "existing handoff"
            },
            &old,
        );
    }
}
#[test]
fn R12__recovery_requires_real_distinct_caller_and_idle_snapshot() {
    for scenario in ["empty-caller", "same-source", "active", "duplicate-meta"] {
        let s = fixture();
        if scenario == "same-source" {
            s.write(
                META,
                &s.read(META)
                    .replace("owner_session\tdestination", "owner_session\tsource"),
            );
        }
        if scenario == "active" {
            s.write(
                ".dstack/local/exec/pending/started_at",
                "2026-09-06T00:00:00Z\n",
            );
        }
        if scenario == "duplicate-meta" {
            s.write(META, &(s.read(META) + "owner_session\tdestination\n"));
        }
        let mut c = recovery_command(&s, "codex", "trace/source.jsonl", true);
        if scenario == "empty-caller" {
            for key in [
                "DSTACK_SESSION_ID",
                "CLAUDE_CODE_SESSION_ID",
                "CODEX_THREAD_ID",
                "CODEX_SESSION_ID",
            ] {
                c.env_remove(key);
            }
        }
        if scenario == "same-source" {
            c.env("DSTACK_SESSION_ID", "source");
        }
        let old = s.read(META);
        let out = c.output().unwrap();
        rejected(
            &s,
            out,
            match scenario {
                "empty-caller" => "actual caller",
                "same-source" => "differ",
                "active" => "unresolved exec",
                _ => "duplicate",
            },
            &old,
        );
    }
}
#[test]
fn R12__codex_source_uses_exact_rollout_filename_and_keeps_saved_mode() {
    let s = Scratch::new("codex", "claude");
    let mode = s.read(".dstack/runs/sample/mode.json");
    let name = "trace/rollout-2026-09-06-source.jsonl";
    s.write(name, &s.read("history.jsonl"));
    s.write(
        META,
        &(s.read(META)
            .replace("owner_session\tsource", "owner_session\tdestination")
            + &format!("transcript_path\t{}\n", s.0.join(name).display())),
    );
    let out = recovery_command(&s, "claude", name, true).output().unwrap();
    success(out);
    assert!(s.read(META).contains("owner_session\tsource\n"));
    assert_eq!(s.read(".dstack/runs/sample/mode.json"), mode);
}
