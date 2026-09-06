#[path = "support/mode_settings.rs"]
mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output};
use support::Scratch;

fn providers(t: &Scratch, present: &[&str]) {
    let mut table = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n".to_string();
    for provider in ["claude", "codex"] {
        let path = t.0.join(provider);
        if present.contains(&provider) {
            fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        table.push_str(&format!(
            "{provider}\ttest -x {}\tinstall-{provider}\t-\tyes\tgoal-closing\tprovider={provider}\t\n",
            path.display()
        ));
    }
    t.write("deps.tsv", &table);
}

#[test]
fn r03_sub_runtime_preflight_requires_only_the_selected_providers() {
    for main in ["claude", "codex"] {
        for sub in ["claude", "codex"] {
            let t = Scratch::new();
            t.init();
            providers(&t, &[main]);
            t.ok(&["mode", "set", "--main", main, "--sub", sub]);
            let out = t.run(&["run", "new", "selected"]);
            assert_eq!(out.status.success(), main == sub,
                "{main}/{sub}: {}", String::from_utf8_lossy(&out.stdout));
            if main != sub {
                assert!(String::from_utf8_lossy(&out.stdout).contains(&format!("MISSING {sub}")));
                assert!(!t.0.join(".dstack/local/CURRENT").exists());
            }
        }
    }
}

#[test]
fn r03_sub_runtime_quick_preflight_includes_research_even_without_review() {
    let t = Scratch::new();
    t.init();
    providers(&t, &["claude"]);
    t.ok(&["mode", "set", "--main", "claude", "--sub", "codex"]);
    t.ok(&["quick", "new", "plain"]);
    for flag in ["--research", "--review"] {
        let out = t.run(&["quick", "new", "needs-sub", flag]);
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).contains("MISSING codex"));
        assert!(!t.0.join(".dstack/quick/needs-sub").exists());
    }
}

#[test]
fn r03_sub_runtime_approval_uses_its_target_snapshot_not_current_project() {
    let t = Scratch::new();
    t.init();
    providers(&t, &["claude"]);
    t.ok(&["mode", "set", "--main", "claude", "--sub", "claude"]);
    t.ok(&["run", "new", "snapshot"]);
    t.ok(&["request", "new", "--type", "cli", "--title", "선택한 실행 환경을 유지해요"]);
    t.ok(&["req", "add", "선택한 실행 환경을 사용해요.", "--accept", "선택한 환경을 확인해요."]);
    t.ok(&["mode", "set", "--main", "codex", "--sub", "codex"]);
    let approved = t.ok(&["request", "approve"]);
    assert!(approved.contains("ok      claude"), "{approved}");
    assert!(!approved.contains("MISSING codex"), "{approved}");
}

fn session(t: &Scratch, args: &[&str], thread: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_dstack"));
    command.current_dir(&t.0).args(args)
        .env("DSTACK_ROOT", &t.0).env("DSTACK_DEPS", t.0.join("deps.tsv"));
    for key in ["DSTACK_SESSION_ID", "CLAUDE_CODE_SESSION_ID", "CODEX_THREAD_ID", "CODEX_SESSION_ID"] {
        command.env_remove(key);
    }
    if let Some(thread) = thread { command.env("CODEX_THREAD_ID", thread); }
    command.output().unwrap()
}

#[test]
fn r04_mode_compatibility_codex_threads_are_distinct_run_owners() {
    let t = Scratch::new();
    t.init();
    assert!(session(&t, &["run", "new", "owner"], Some("codex-one")).status.success());
    let id = t.read(".dstack/local/CURRENT").trim().to_string();
    let meta = format!(".dstack/runs/{id}/meta.tsv");
    assert!(t.read(&meta).contains("owner_session\tcodex-one\n"));
    assert!(session(&t, &["run", "adopt"], Some("codex-one")).status.success());
    let before = t.read(&meta);
    let other = session(&t, &["run", "adopt"], Some("codex-two"));
    assert!(!other.status.success());
    assert!(String::from_utf8_lossy(&other.stderr).contains("live owner"));
    assert_eq!(t.read(&meta), before);
}

#[test]
fn r04_mode_compatibility_empty_session_ids_do_not_claim_a_live_owner() {
    let t = Scratch::new();
    t.init();
    assert!(session(&t, &["run", "new", "unidentified"], None).status.success());
    let out = session(&t, &["run", "adopt"], None);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--force"));
}
