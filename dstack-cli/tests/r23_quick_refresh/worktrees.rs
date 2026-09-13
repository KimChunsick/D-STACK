use super::{
    fixture, refresh,
    support::{tree, Scratch},
};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn R23__symlinked_quick_targets_and_roots_cannot_refresh_another_path() {
    for relative in [".dstack/quick/selected", ".dstack/quick", ".dstack"] {
        let t = fixture();
        let external = Scratch::new();
        let source = t.0.join(relative);
        let moved = external.0.join("moved");
        fs::rename(&source, &moved).unwrap();
        symlink(&moved, &source).unwrap();
        let before = tree(&external.0);
        let out = t.run(&refresh("selected", "claude"));
        assert!(!out.status.success(), "accepted redirected {relative}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("path"));
        assert_eq!(tree(&external.0), before);
        assert_eq!(fs::read_link(source).unwrap(), moved);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn in_worktree(t: &Scratch, dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dstack"))
        .current_dir(dir)
        .args(args)
        .env_remove("DSTACK_ROOT")
        .env(
            "DSTACK_HOME",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../claude"),
        )
        .env("DSTACK_DEPS", t.0.join("deps.tsv"))
        .output()
        .unwrap()
}

#[test]
fn R23__real_git_worktrees_refresh_only_their_local_quick_snapshot() {
    let t = fixture();
    git(&t.0, &["init", "-q"]);
    git(
        &t.0,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-qm",
            "fixture",
        ],
    );
    let holder = Scratch::new();
    let wt = holder.0.join("worktree");
    git(&t.0, &["worktree", "add", "--detach", wt.to_str().unwrap()]);
    assert!(in_worktree(&t, &wt, &["quick", "new", "selected"])
        .status
        .success());
    t.ok(&["mode", "set", "--main", "codex", "--sub", "codex"]);
    let main_before = tree(&t.0.join(".dstack"));
    let out = in_worktree(&t, &wt, &refresh("selected", "codex"));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(tree(&t.0.join(".dstack")), main_before);
    let mode: serde_json::Value =
        serde_json::from_slice(&fs::read(wt.join(".dstack/quick/selected/mode.json")).unwrap())
            .unwrap();
    assert_eq!(mode, serde_json::json!({"main":"codex","sub":"codex"}));
    let out = in_worktree(&t, &wt, &refresh("other", "codex"));
    assert!(
        !out.status.success(),
        "must not find main worktree's other task"
    );
    assert_eq!(tree(&t.0.join(".dstack")), main_before);
}
