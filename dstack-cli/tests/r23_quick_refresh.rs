#![allow(non_snake_case)]
#[path = "r23_quick_refresh/refusals.rs"]
mod refusals;
#[path = "support/mode_settings.rs"]
mod support;
#[path = "r23_quick_refresh/worktrees.rs"]
mod worktrees;

use std::fs;
use support::{tree, Scratch};

fn fixture() -> Scratch {
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "selected"]);
    t.ok(&["quick", "new", "other"]);
    t.run_fixture("active", Some(r#"{"main":"claude","sub":"codex"}"#), true);
    t.write(
        ".dstack/quick/selected/request.approved",
        "approval hash retained\n",
    );
    t.write(
        ".dstack/quick/selected/cases.tsv",
        "existing evidence ledger\n",
    );
    t.write(
        ".dstack/quick/selected/codex-review-001.md",
        "sealed review\n",
    );
    t.write(
        ".dstack/quick/selected/artifacts/check.txt",
        "evidence bytes\n",
    );
    t
}

fn refresh<'a>(slug: &'a str, host: &'a str) -> [&'a str; 6] {
    ["quick", "resume", slug, "--refresh-mode", "--host", host]
}

#[test]
fn R23__explicit_refresh_changes_only_selected_mode_for_all_provider_pairs() {
    for main in ["claude", "codex"] {
        for sub in ["claude", "codex"] {
            let t = fixture();
            t.ok(&["mode", "set", "--main", main, "--sub", sub]);
            let mut expected = tree(&t.0);
            let receipt = t.ok(&refresh("selected", main));
            assert!(
                receipt.contains("quick mode refreshed: selected"),
                "{receipt}"
            );
            assert!(receipt.contains("previous: main=claude sub=codex"));
            assert!(receipt.contains(&format!("current: main={main} sub={sub}")));
            assert!(
                receipt.contains("engine unchanged") && receipt.contains("ownership unchanged")
            );
            let mode = t.json(&[
                "mode", "show", "--quick", "selected", "--host", main, "--json",
            ]);
            assert_eq!(mode["main"], main);
            assert_eq!(mode["sub"], sub);
            let file = std::path::Path::new(".dstack/quick/selected/mode.json");
            expected.iter_mut().find(|(p, _)| p == file).unwrap().1 =
                fs::read(t.0.join(file)).unwrap();
            assert_eq!(tree(&t.0), expected, "only selected mode may change");
            assert!(
                !t.run(&["quick", "resume", "selected"]).status.success(),
                "refresh succeeds even while ordinary resume reports missing evidence"
            );
            assert_eq!(tree(&t.0), expected);
        }
    }
}

#[test]
fn R23__ordinary_resume_is_read_only_and_legacy_snapshot_refresh_is_explicit() {
    let t = fixture();
    fs::remove_file(t.0.join(".dstack/quick/selected/mode.json")).unwrap();
    t.ok(&["mode", "set", "--main", "codex", "--sub", "claude"]);
    let before = tree(&t.0);
    let out = t.run(&["quick", "resume", "selected"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("what is still missing"));
    assert_eq!(tree(&t.0), before);
    assert_eq!(
        t.json(&["mode", "show", "--quick", "selected", "--json"])["source"],
        "legacy-quick"
    );
    let text = t.ok(&[
        "quick",
        "resume",
        "selected",
        "--host=codex",
        "--refresh-mode",
    ]);
    assert!(text.contains("previous: main=claude sub=codex"));
    assert_eq!(
        t.json(&["mode", "show", "--quick", "selected", "--json"])["source"],
        "quick"
    );
}

#[test]
fn R23__mode_guidance_and_help_name_the_selected_quick_refresh() {
    let t = fixture();
    t.ok(&["mode", "set", "--main", "codex"]);
    let before = tree(&t.0);
    let hint = "dstack quick resume selected --refresh-mode --host codex";
    let out = t.run(&["mode", "show", "--host", "codex", "--quick", "selected"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains(hint));
    let shown = t.ok(&["mode", "show", "--quick", "selected"]);
    assert!(shown.contains(hint));
    assert!(!shown.contains("dstack run adopt <id> --refresh-mode"));
    let help = t.ok(&["help"]);
    assert!(help.lines().any(|l| l.contains("quick resume")
        && l.contains("--refresh-mode")
        && l.contains("--host")));
    assert_eq!(tree(&t.0), before);
}
