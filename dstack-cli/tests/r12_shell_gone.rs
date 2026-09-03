// tests/r12_shell_gone.rs
// R12: the shell implementation is out of the tree — no library directory, no bash dispatcher,
// and no document or comment left naming either of them or the bash-3.2 rule the CLI was written
// under. This file is swept by that same sweep, so it spells the two words in pieces.
// What replaced it has to stand on its own: `dstack doctor` passes, and the reference the parity
// harness and the comparison tests still drive is reachable through the shell-final tag.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::PathBuf;
use std::process::{Command, Output};

/// The two scripts R12 allows to keep naming the shell: the installer and the hook wrapper.
const ALLOWED: [&str; 2] = ["install.sh", "claude/hooks/dstack-hook.sh"];

/// The library directory of the shell implementation, and the sweep R12 accepts as its proof.
/// Both are spelled in pieces so that this file is not the one stray the sweep reports.
const LIB_DIR: &str = concat!("claude", "/lib");
const SWEEP: &str = concat!("claude", r"/lib|bash 3\.2");

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn git(args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(repo())
        .output()
        .expect("run git")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf-8")
}

#[test]
fn r12__the_shell_implementation_is_not_in_the_tree() {
    let listed = git(&["ls-files", LIB_DIR]);
    assert_eq!(listed.status.code(), Some(0));
    assert_eq!(stdout(&listed), "", "{LIB_DIR} still holds files");
    assert_eq!(
        stdout(&git(&["ls-files", "claude/bin"])),
        "",
        "the bash dispatcher is still tracked"
    );
}

#[test]
fn r12__nothing_but_the_installer_and_the_hook_wrapper_names_the_shell() {
    // git grep exits 1 when nothing matches at all, which is a pass here.
    let found = git(&["grep", "-n", "-E", SWEEP]);
    assert!(
        matches!(found.status.code(), Some(0) | Some(1)),
        "git grep could not sweep the tree"
    );
    let printed = stdout(&found);
    let strays: Vec<&str> = printed
        .lines()
        .filter(|line| {
            let file = line.split(':').next().unwrap_or(line);
            !ALLOWED.contains(&file)
        })
        .collect();
    assert!(
        strays.is_empty(),
        "files outside {ALLOWED:?} still name the shell:\n{}",
        strays.join("\n")
    );
}

#[test]
fn r12__doctor_passes_over_the_tree_the_shell_left_behind() {
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .arg("doctor")
        .current_dir(repo())
        .output()
        .expect("run dstack doctor");
    assert_eq!(
        out.status.code(),
        Some(0),
        "dstack doctor:\n{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn r12__the_reference_is_still_reachable_through_the_tag() {
    let dispatcher = shell_ref::dispatcher();
    assert!(
        dispatcher.is_file(),
        "the tag holds no dispatcher at {}",
        dispatcher.display()
    );
    assert!(
        shell_ref::lib().join("common.sh").is_file(),
        "the tag holds no shell library next to the dispatcher"
    );
    let out = Command::new("bash")
        .arg(&dispatcher)
        .arg("help")
        .output()
        .expect("run the reference dispatcher");
    assert_eq!(out.status.code(), Some(0), "the reference still answers");
    assert_eq!(
        stdout(&out).lines().last(),
        Some("verbs: 59"),
        "the reference roster is the one R13 compares against"
    );
}
