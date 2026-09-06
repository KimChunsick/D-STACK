#[path = "support/mode_settings.rs"]
mod support;

use std::fs;
use support::{tree, Scratch};

#[test]
fn r01_modes_all_four_combinations_persist_and_round_trip() {
    let t = Scratch::new();
    t.init();
    for main in ["claude", "codex"] {
        for sub in ["claude", "codex"] {
            t.ok(&["mode", "set", "--main", main, "--sub", sub]);
            let actual = t.json(&["mode", "show", "--json"]);
            assert_eq!(actual["main"], main);
            assert_eq!(actual["sub"], sub);
            assert_eq!(actual["source"], "project");
            let stored: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(t.project()).unwrap()).unwrap();
            assert_eq!(stored, serde_json::json!({"main": main, "sub": sub}));
            assert_eq!(actual["project"], stored);
        }
    }
}

#[test]
fn r01_modes_one_field_preserves_the_other_and_validates_before_writes() {
    let t = Scratch::new();
    t.init();
    t.ok(&["mode", "set", "--main", "codex"]);
    assert_eq!(t.json(&["mode", "show", "--json"])["sub"], "codex");
    t.ok(&["mode", "set", "--sub=claude"]);
    let mode = t.json(&["mode", "show", "--json"]);
    assert_eq!(mode["main"], "codex");
    assert_eq!(mode["sub"], "claude");
    let before = tree(&t.0);
    for args in [
        vec!["mode", "set"],
        vec!["mode", "set", "--main", "other"],
        vec!["mode", "set", "--main", "Claude"],
        vec!["mode", "set", "--main", "claude", "--sub", "other"],
        vec!["mode", "set", "--main", "claude", "--unknown", "codex"],
        vec!["mode", "set", "--main", "claude", "--sub"],
        vec!["mode", "set", "--main", "claude", "--main=codex"],
        vec!["mode", "set", "--sub=codex", "--sub", "claude"],
        vec!["mode", "set", "--sub="],
        vec!["mode", "set", "--main", "--sub", "claude"],
        vec!["mode", "set", "--main", "claude", "positional"],
        vec!["mode", "show", "--json", "--json"],
        vec!["mode", "show", "--main", "claude"],
        vec!["mode", "show", "--host", "other"],
        vec!["mode", "show", "--host"],
    ] {
        let out = t.run(&args);
        assert!(!out.status.success(), "accepted {args:?}");
        assert!(out.stdout.is_empty(), "partial success output for {args:?}");
        assert_eq!(
            tree(&t.0),
            before,
            "invalid arguments mutated store: {args:?}"
        );
    }
}

#[test]
fn r01_modes_corrupt_project_is_never_overwritten_or_defaulted() {
    let t = Scratch::new();
    t.init();
    for corrupt in [
        "{",
        "{}",
        "null",
        "[]",
        "{\"main\":\"codex\"}",
        "{\"main\":\"codex\",\"sub\":\"unknown\"}",
        "{\"main\":\"codex\",\"sub\":\"claude\",\"extra\":1}",
        "{\"main\":\"codex\",\"main\":\"claude\",\"sub\":\"claude\"}",
    ] {
        fs::write(t.project(), corrupt).unwrap();
        for args in [
            vec!["mode", "show", "--json"],
            vec!["mode", "set", "--main", "claude", "--sub", "codex"],
        ] {
            let out = t.run(&args);
            assert!(!out.status.success(), "accepted {corrupt} with {args:?}");
            assert!(out.stdout.is_empty());
            assert!(String::from_utf8_lossy(&out.stderr).contains("mode.json"));
            assert_eq!(fs::read_to_string(t.project()).unwrap(), corrupt);
        }
    }
    fs::remove_file(t.project()).unwrap();
    fs::create_dir(t.project()).unwrap();
    let out = t.run(&["mode", "set", "--main", "codex"]);
    assert!(!out.status.success());
    assert!(t.project().is_dir());
}

#[test]
fn r01_modes_set_requires_an_initialized_store() {
    let t = Scratch::new();
    let before = tree(&t.0);
    let out = t.run(&["mode", "set", "--sub", "claude"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("dstack init"));
    assert_eq!(tree(&t.0), before);
}
