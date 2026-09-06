#[path = "support/mode_settings.rs"]
mod support;

use std::fs;
use support::{tree, Scratch};

#[test]
fn r04_mode_compatibility_absence_defaults_without_creating_a_file() {
    let t = Scratch::new();
    t.init();
    let before = tree(&t.0);
    let mode = t.json(&["mode", "show", "--json"]);
    assert_eq!(mode["main"], "claude");
    assert_eq!(mode["sub"], "codex");
    assert_eq!(mode["source"], "default");
    let text = t.ok(&["mode", "show", "--host", "claude"]);
    assert!(text.contains("main=claude sub=codex"));
    assert!(text.contains("session"));
    assert!(text.contains("dstack run adopt"));
    assert_eq!(tree(&t.0), before);
}

#[test]
fn r04_mode_compatibility_set_without_active_target_guides_the_new_main() {
    let t = Scratch::new();
    t.init();
    let message = t.ok(&["mode", "set", "--main", "codex"]);
    assert!(
        message.contains("new session") && message.contains("main=codex"),
        "{message}"
    );
    assert!(
        !message.contains("launch claude"),
        "stale host guidance: {message}"
    );
}

#[test]
fn r04_mode_compatibility_new_main_host_can_discover_explicit_refresh() {
    let t = Scratch::new();
    t.init();
    t.run_fixture(
        "active",
        Some("{\"main\":\"claude\",\"sub\":\"codex\"}"),
        true,
    );
    t.ok(&["mode", "set", "--main", "codex"]);
    let before = tree(&t.0);
    let out = t.run(&["mode", "show", "--host", "codex"]);
    assert!(!out.status.success());
    let message = String::from_utf8_lossy(&out.stderr);
    assert!(
        message.contains("dstack run adopt active --refresh-mode"),
        "{message}"
    );
    assert!(
        message.contains("ordinary") && message.contains("preserves"),
        "{message}"
    );
    assert_eq!(tree(&t.0), before);
}

#[test]
fn r04_mode_compatibility_legacy_and_active_snapshots_do_not_follow_project_changes() {
    let t = Scratch::new();
    t.init();
    t.ok(&["mode", "set", "--main", "codex", "--sub", "claude"]);
    let legacy = t.run_fixture("legacy", None, true);
    let before = tree(&t.0);
    let mode = t.json(&["mode", "show", "--json"]);
    assert_eq!(mode["main"], "claude");
    assert_eq!(mode["sub"], "codex");
    assert_eq!(mode["source"], "legacy-run");
    assert_eq!(mode["project"]["main"], "codex");
    assert_eq!(
        tree(&t.0),
        before,
        "show must not touch owner or create legacy snapshot"
    );
    let active = t.run_fixture(
        "active",
        Some("{\"main\":\"codex\",\"sub\":\"codex\"}"),
        true,
    );
    let snapshot = fs::read(active.join("mode.json")).unwrap();
    let report = t.ok(&["mode", "set", "--main", "claude"]);
    assert!(report.contains("project: main=claude sub=claude"));
    assert!(report.contains("active: main=codex sub=codex"));
    assert!(report.contains("--refresh-mode"));
    let mode = t.json(&["mode", "show", "--json"]);
    assert_eq!(mode["main"], "codex");
    assert_eq!(mode["source"], "run");
    assert_eq!(fs::read(active.join("mode.json")).unwrap(), snapshot);
    assert!(!legacy.join("mode.json").exists());
}

#[test]
fn r04_mode_compatibility_explicit_targets_and_host_handoff_are_read_only() {
    let t = Scratch::new();
    t.init();
    t.run_fixture(
        "active",
        Some("{\"main\":\"codex\",\"sub\":\"claude\"}"),
        true,
    );
    t.write(
        ".dstack/quick/small/mode.json",
        "{\"main\":\"claude\",\"sub\":\"claude\"}",
    );
    let before = tree(&t.0);
    let mode = t.json(&["mode", "show", "--quick", "small", "--json"]);
    assert_eq!(mode["main"], "claude");
    assert_eq!(mode["source"], "quick");
    assert_eq!(mode["target"]["id"], "small");
    let out = t.run(&["mode", "show", "--run", "active", "--host", "claude"]);
    assert!(!out.status.success());
    let message = String::from_utf8_lossy(&out.stderr);
    assert!(
        message.contains("codex")
            && message.contains("session")
            && message.contains("dstack run adopt active"),
        "{message}"
    );
    assert_eq!(tree(&t.0), before);
    for args in [
        vec!["--run", "active", "--quick", "small"],
        vec!["--run", "active", "--run", "active"],
        vec!["--quick", "small", "--quick=small"],
        vec!["--run", "../outside"],
        vec!["--quick", ".."],
        vec!["--run", "missing"],
        vec!["--quick", "missing"],
    ] {
        let mut all = vec!["mode", "show"];
        all.extend(args);
        assert!(!t.run(&all).status.success(), "accepted {all:?}");
        assert_eq!(tree(&t.0), before);
    }
}

#[test]
fn r04_mode_compatibility_invalid_active_snapshot_prevents_mode_set_mutation() {
    let t = Scratch::new();
    t.init();
    t.ok(&["mode", "set", "--main", "codex"]);
    let active = t.run_fixture("active", Some("{\"main\":\"codex\"}"), true);
    let before = tree(&t.0);
    for args in [
        vec!["mode", "show", "--json"],
        vec!["mode", "set", "--main", "claude"],
    ] {
        let out = t.run(&args);
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("mode.json"));
        assert_eq!(tree(&t.0), before);
    }
    fs::remove_file(active.join("mode.json")).unwrap();
    fs::create_dir(active.join("mode.json")).unwrap();
    assert!(!t.run(&["mode", "show"]).status.success());
    t.write(".dstack/local/CURRENT", "../outside\n");
    assert!(!t.run(&["mode", "show"]).status.success());
}

#[test]
fn r04_mode_compatibility_new_runs_quick_tasks_and_explicit_adoption_refresh() {
    let t = Scratch::new();
    t.init();
    t.ok(&["mode", "set", "--main", "claude", "--sub", "claude"]);
    t.ok(&["run", "new", "fresh"]);
    let id = t.read(".dstack/local/CURRENT").trim().to_string();
    let snapshot = t.0.join(".dstack/runs").join(&id).join("mode.json");
    let original = fs::read(&snapshot).expect("new run snapshots project mode");
    let mode: serde_json::Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(mode, serde_json::json!({"main":"claude", "sub":"claude"}));

    t.ok(&["mode", "set", "--main", "codex", "--sub", "codex"]);
    t.ok(&["quick", "new", "small"]);
    let quick: serde_json::Value =
        serde_json::from_str(&t.read(".dstack/quick/small/mode.json")).unwrap();
    assert_eq!(quick, serde_json::json!({"main":"codex", "sub":"codex"}));
    assert_eq!(t.read(".dstack/local/CURRENT").trim(), id);
    assert_eq!(fs::read(&snapshot).unwrap(), original);

    t.ok(&["run", "adopt", &id, "--force"]);
    assert_eq!(fs::read(&snapshot).unwrap(), original);
    let refresh = t.ok(&["run", "adopt", &id, "--force", "--refresh-mode"]);
    assert!(refresh.contains("main=codex sub=codex"), "{refresh}");
    let mode = t.json(&["mode", "show", "--json"]);
    assert_eq!(mode["main"], "codex");
    assert_eq!(mode["sub"], "codex");
    let status = t.ok(&["status"]);
    assert!(
        status.contains("main=codex") && status.contains("sub=codex"),
        "{status}"
    );

    fs::remove_file(&snapshot).unwrap();
    t.ok(&["run", "adopt", &id, "--force"]);
    assert!(
        !snapshot.exists(),
        "ordinary legacy adoption must not write a snapshot"
    );
    let legacy = t.json(&["mode", "show", "--json"]);
    assert_eq!(legacy["main"], "claude");
    assert_eq!(legacy["sub"], "codex");
    t.ok(&["run", "adopt", &id, "--force", "--refresh-mode"]);
    assert_eq!(t.json(&["mode", "show", "--json"])["main"], "codex");
}

#[test]
fn r04_mode_compatibility_invalid_refresh_preserves_owner_current_and_snapshot() {
    let t = Scratch::new();
    t.init();
    t.ok(&["mode", "set", "--main", "codex"]);
    t.run_fixture(
        "active",
        Some("{\"main\":\"claude\",\"sub\":\"codex\"}"),
        true,
    );
    let other = t.run_fixture(
        "other",
        Some("{\"main\":\"claude\",\"sub\":\"claude\"}"),
        false,
    );
    fs::write(t.project(), "{\"main\":\"codex\"}").unwrap();
    let before = tree(&t.0);
    let out = t.run(&["run", "adopt", "other", "--force", "--refresh-mode"]);
    assert!(!out.status.success());
    assert_eq!(
        tree(&t.0),
        before,
        "invalid project refresh mutated owner or CURRENT"
    );
    fs::write(t.project(), "{\"main\":\"codex\",\"sub\":\"codex\"}").unwrap();
    fs::write(other.join("mode.json"), "{\"main\":\"claude\"}").unwrap();
    let before = tree(&t.0);
    let out = t.run(&["run", "adopt", "other", "--force", "--refresh-mode"]);
    assert!(!out.status.success());
    assert_eq!(
        tree(&t.0),
        before,
        "refresh must not overwrite a damaged snapshot"
    );
}

#[test]
fn r04_mode_compatibility_bad_project_does_not_leave_partial_new_tasks() {
    let t = Scratch::new();
    t.init();
    fs::write(t.project(), "{\"main\":\"codex\"}").unwrap();
    let before = tree(&t.0);
    for args in [vec!["run", "new", "fresh"], vec!["quick", "new", "small"]] {
        let out = t.run(&args);
        assert!(
            !out.status.success(),
            "accepted damaged project for {args:?}"
        );
        assert_eq!(tree(&t.0), before, "partial task from {args:?}");
    }
}
