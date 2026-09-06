// tests/r08_install.rs
// R08/R01: install.sh builds the release binary and links ~/.claude/bin/dstack to it, and deps.tsv
// names cargo so dstack doctor probes it. install.sh runs against a scratch $HOME, never the real
// one: the switch of the installed link is the Goal-close step (D-03).
#![allow(non_snake_case)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The binary install.sh links to, as install.sh names it (D-DESIGN-01).
const BIN_REL: &str = "dstack-cli/target/release/dstack";

fn repo() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
        .expect("the physical path of the repository")
}

/// The release binary, built once, into the directory install.sh links from: the build is pinned
/// with --target-dir because an ambient CARGO_TARGET_DIR would send it somewhere else and leave
/// these tests reading whatever stale file sits at BIN_REL. Cargo serialises a nested build
/// against the outer one itself.
fn release_binary() -> &'static PathBuf {
    static ONCE: OnceLock<PathBuf> = OnceLock::new();
    ONCE.get_or_init(|| {
        let out = Command::new("cargo")
            .args(["build", "--release", "--manifest-path"])
            .arg(repo().join("dstack-cli/Cargo.toml"))
            .arg("--target-dir")
            .arg(repo().join("dstack-cli/target"))
            .output()
            .expect("run cargo build --release");
        assert!(
            out.status.success(),
            "cargo build --release failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        repo().join(BIN_REL)
    })
}

fn scratch_home(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-r08-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    std::fs::canonicalize(&dir).expect("the physical path of the scratch directory")
}

/// ./install.sh from the repository root with a scratch $HOME → stdout, stderr, exit code.
fn install(home: &Path, args: &[&str]) -> (String, String, i32) {
    run(&repo().join("install.sh"), home, args, &[])
}

/// An installer script (the repository's, or a copy in a scratch repository) with a scratch $HOME
/// and the environment a case needs → stdout, stderr, exit code.
fn run(script: &Path, home: &Path, args: &[&str], env: &[(&str, &str)]) -> (String, String, i32) {
    let mut command = Command::new(script);
    command
        .args(args)
        .current_dir(script.parent().expect("the directory of the script"))
        .env("HOME", home);
    for (name, value) in env {
        command.env(name, value);
    }
    let out = command.output().expect("run install.sh");
    (
        String::from_utf8(out.stdout).expect("utf-8"),
        String::from_utf8(out.stderr).expect("utf-8"),
        out.status.code().expect("an exit code"),
    )
}

/// The installer's row table: (source, target, status) for every printed row.
fn rows(printed: &str) -> Vec<(String, String, String)> {
    printed
        .lines()
        .filter_map(|line| {
            let (src, rest) = line.split_once('→')?;
            let (target, status) = rest.trim_start().split_once("  ")?;
            Some((
                src.trim().to_string(),
                target.trim().to_string(),
                status.trim().to_string(),
            ))
        })
        .collect()
}

fn dstack_row(printed: &str) -> (String, String, String) {
    let mut found: Vec<(String, String, String)> = rows(printed)
        .into_iter()
        .filter(|(_, target, _)| target == ".claude/bin/dstack")
        .collect();
    assert_eq!(
        found.len(),
        1,
        "exactly one row installs ~/.claude/bin/dstack:\n{printed}"
    );
    found.remove(0)
}

#[test]
fn r08__the_dry_run_names_the_built_binary_and_changes_nothing() {
    let binary = release_binary();
    let home = scratch_home("dry-run");
    let (printed, err, code) = install(&home, &["--dry-run"]);
    assert_eq!(code, 0, "install.sh --dry-run: {err}");
    let (src, _, status) = dstack_row(&printed);
    assert_eq!(src, BIN_REL, "the dstack row points at the built binary");
    assert!(binary.is_file(), "the built binary is at {BIN_REL}");
    assert!(
        !status.starts_with("skipped"),
        "the dstack row is installable: {status}"
    );
    assert!(
        !rows(&printed)
            .iter()
            .any(|(source, _, _)| source == "claude/bin/dstack"),
        "no row installs the shell dispatcher:\n{printed}"
    );
    assert!(
        !home.join(".claude").exists(),
        "--dry-run changed the home directory"
    );
}

#[test]
fn r08__the_install_links_the_binary_and_the_link_runs() {
    let home = scratch_home("install");
    let (printed, err, code) = install(&home, &[]);
    assert_eq!(code, 0, "install.sh: {err}\n{printed}");
    let link = home.join(".claude/bin/dstack");
    assert_eq!(
        std::fs::read_link(&link).expect("~/.claude/bin/dstack is a symlink"),
        repo().join(BIN_REL)
    );
    let out = Command::new(&link)
        .arg("help")
        .env("HOME", &home)
        .output()
        .expect("run the installed dstack");
    let help = String::from_utf8(out.stdout).expect("utf-8");
    assert!(
        help.lines().any(|line| line == "verbs: 64"),
        "the installed link is the port:\n{help}"
    );
}

#[test]
fn r08__doctor_probes_cargo() {
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .arg("doctor")
        .current_dir(repo())
        .env_remove("DSTACK_DEPS")
        .output()
        .expect("run dstack doctor");
    let printed = String::from_utf8(out.stdout).expect("utf-8");
    let row = printed
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("cargo | "))
        .unwrap_or_else(|| panic!("no cargo row in the deps section:\n{printed}"));
    // name | present | needed_when | install — the install column may carry a pipe of its own.
    let column: Vec<&str> = row.splitn(4, " | ").collect();
    assert_eq!(
        column[2], "optional",
        "cargo builds the CLI; it never gates an ordinary run (R105): {row}"
    );
    assert!(
        column[3].contains("rustup") || column[3].contains("rust"),
        "the row names how to install cargo: {row}"
    );
}

/// The link is the repository's own target directory, whatever CARGO_TARGET_DIR says: cargo must
/// be told where to build, or it succeeds elsewhere and the row goes to "source missing".
#[test]
fn r08__a_foreign_cargo_target_dir_does_not_move_the_binary() {
    let home = scratch_home("target-dir");
    let elsewhere = scratch_home("target-dir-elsewhere");
    std::fs::remove_dir_all(&elsewhere).expect("cargo decides whether this directory exists");
    let (printed, err, code) = run(
        &repo().join("install.sh"),
        &home,
        &[],
        &[("CARGO_TARGET_DIR", &elsewhere.display().to_string())],
    );
    assert_eq!(code, 0, "install.sh: {err}\n{printed}");
    assert_eq!(
        std::fs::read_link(home.join(".claude/bin/dstack")).expect("a symlink"),
        repo().join(BIN_REL)
    );
    assert!(
        !elsewhere.exists(),
        "the build went to CARGO_TARGET_DIR instead of {BIN_REL}"
    );
}

/// A build that leaves no binary behind is a failed install, never a skipped row.
#[test]
fn r08__a_build_that_produces_no_binary_aborts() {
    let dir = scratch_home("no-binary");
    let script = dir.join("install.sh");
    std::fs::copy(repo().join("install.sh"), &script).expect("copy the installer");
    std::fs::copy(repo().join("deps.tsv"), dir.join("deps.tsv")).expect("copy the deps table");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("executable");
    // A cargo that reports success and compiles nothing: the scratch repository has no crate.
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("a directory for the stub");
    std::fs::write(bin.join("cargo"), "#!/bin/sh\nexit 0\n").expect("the stub cargo");
    std::fs::set_permissions(bin.join("cargo"), std::fs::Permissions::from_mode(0o755))
        .expect("executable");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("a scratch home");

    let (printed, _, code) = run(&script, &home, &[], &[("PATH", &path)]);
    assert_eq!(code, 1, "the install fails:\n{printed}");
    assert!(
        printed
            .lines()
            .any(|line| line.trim_start().starts_with("ABORT:") && line.contains(BIN_REL)),
        "the abort names the binary that is not there:\n{printed}"
    );
    assert!(
        !home.join(".claude").exists(),
        "a failed build installs nothing"
    );
}

#[test]
fn r01__the_release_binary_runs_help() {
    let out = Command::new(release_binary())
        .arg("help")
        .output()
        .expect("run the built dstack");
    assert!(out.status.success(), "dstack help exits 0");
    let help = String::from_utf8(out.stdout).expect("utf-8");
    assert!(
        help.lines().any(|line| line == "verbs: 64"),
        "the roster of the built binary:\n{help}"
    );
}
