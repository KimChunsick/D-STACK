// tests/r01_external_commands.rs
// R01/R13: the binary spawns no external command but git and the editor of request open (D-14).

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read src") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(path);
        }
    }
}

/// Every `Command::new("literal")` in the crate, with the source file it sits in, in file order.
fn spawned_literals() -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    rust_sources(&crate_dir().join("src"), &mut files);
    let mut found = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read source");
        for (idx, _) in text.match_indices("Command::new(\"") {
            let rest = &text[idx + "Command::new(\"".len()..];
            let end = rest.find('"').expect("closing quote");
            found.push((file.clone(), rest[..end].to_string()));
        }
    }
    found
}

#[test]
fn r01__spawns_only_git() {
    let literals = spawned_literals();
    assert!(
        !literals.is_empty(),
        "expected the port to spawn git somewhere"
    );
    // D-14: the optional editor of deps.tsv is the one exception, and only in the verb that
    // opens the request — every other file may name git alone.
    for (file, name) in &literals {
        let editor = name == "code" && file.ends_with("verbs/request/open_show.rs");
        assert!(
            name == "git" || editor,
            "the CLI may only spawn git, found {name} in {}",
            file.display()
        );
    }
    // The selftest sandbox spawns the dstack binary itself, which is a path, not a literal.
    assert!(
        literals.iter().any(|(_, name)| name == "git"),
        "git is the one runtime dependency"
    );
    assert_eq!(
        literals.iter().filter(|(_, name)| name == "code").count(),
        1,
        "request open is the only place that launches the editor"
    );
}

#[test]
fn r01__binary_prints_the_roster() {
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .arg("help")
        .output()
        .expect("run dstack help");
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert!(out.status.success(), "dstack help must exit 0");
    assert!(
        stdout.starts_with("dstack — machine state of the pipeline."),
        "unexpected help header: {stdout}"
    );
}

/// Every probe the repository's deps.tsv declares must be a form the port reads without a shell,
/// or run new would refuse to check tools at all.
#[test]
fn r01__every_repository_probe_is_supported() {
    let table = std::fs::read_to_string(crate_dir().join("../deps.tsv")).expect("deps.tsv");
    let mut probes = 0;
    for line in table.lines().skip(1) {
        let probe = match line.split('\t').nth(1) {
            Some(probe) if !probe.is_empty() && !line.starts_with('#') => probe,
            _ => continue,
        };
        dstack_cli::core::tools::tool_present(probe)
            .unwrap_or_else(|e| panic!("{}: {}", probe, e.message()));
        probes += 1;
    }
    assert!(
        probes >= 8,
        "deps.tsv should declare at least 8 probes, found {probes}"
    );
}

/// The one-tool table the sandbox gets, so no machine-wide deps.tsv is read.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n";

/// A PATH with no `code` on it: the branch that prints the path instead of opening it.
const BARE_PATH: &str = "/usr/bin:/bin";

fn git(dir: &Path, args: &[&str]) {
    let done = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(done.status.success(), "git {args:?} failed in {dir:?}");
}

fn dstack(dir: &Path, path: &str, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_dstack"))
        .args(args)
        .current_dir(dir)
        .env("PATH", path)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "r01")
        .output()
        .expect("run dstack")
}

/// A scratch repository with a store and a request in it, built by the binary under test.
fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p61-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    let dir = std::fs::canonicalize(&dir).expect("the physical path of the scratch directory");
    std::fs::write(dir.join(".deps.tsv"), DEPS).expect("write the deps table");
    git(&dir, &["init", "-q"]);
    git(
        &dir,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ],
    );
    for args in [
        &["init"][..],
        &["run", "new", "sandbox", "--type", "cli"][..],
        &["request", "new", "--type", "cli"][..],
    ] {
        let out = dstack(&dir, BARE_PATH, args);
        assert!(
            out.status.success(),
            "dstack {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    dir
}

/// Writes an executable `code` of the given script into <dir>/bin and answers that directory,
/// which the caller puts on PATH.
fn code_on_path(dir: &Path, script: &str) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("bin directory");
    std::fs::write(bin.join("code"), script).expect("write the stand-in editor");
    std::fs::set_permissions(bin.join("code"), std::fs::Permissions::from_mode(0o755))
        .expect("make the stand-in editor executable");
    bin
}

/// A stand-in `code` that records its argv and returns 0, so the launch is proven without an
/// editor ever appearing on the machine that runs the tests.
fn fake_code(dir: &Path) -> (PathBuf, PathBuf) {
    let log = dir.join("argv.txt");
    let script = format!("#!/bin/sh\nprintf '%s\\n' \"$@\" >> {}\n", log.display());
    (code_on_path(dir, &script), log)
}

/// The one request.md of the sandbox, by the absolute path the verb prints.
fn request_file(dir: &Path) -> PathBuf {
    let runs = dir.join(".dstack/runs");
    let run = std::fs::read_dir(&runs)
        .expect("the runs directory")
        .next()
        .expect("one run")
        .expect("dir entry")
        .path();
    run.join("request.md")
}

/// R44/D-14: the optional editor of deps.tsv is still launched — `code -g <file>:1`, never -w,
/// and the shell's line about it. Without `code` on PATH the other line stands.
#[test]
fn r13__request_open_launches_the_editor() {
    let dir = sandbox("open");
    let (bin, log) = fake_code(&dir);
    let spot = format!("{}:1", request_file(&dir).display());

    let out = dstack(
        &dir,
        &format!("{}:{BARE_PATH}", bin.display()),
        &["request", "open"],
    );
    assert!(out.status.success(), "request open must exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    let argv = std::fs::read_to_string(&log).unwrap_or_default();
    assert_eq!(argv, format!("-g\n{spot}\n"), "code was called as: {argv}");
    assert!(
        stdout.contains(&format!("  opened: code -g {spot}\n")),
        "unexpected open output: {stdout}"
    );

    let out = dstack(&dir, BARE_PATH, &["request", "open"]);
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert!(
        stdout.contains("  code is not on PATH; open the path above by hand\n"),
        "unexpected output without code on PATH: {stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(&log).unwrap_or_default(),
        format!("-g\n{spot}\n"),
        "no editor may run when code is not on PATH"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// A `code` that resolves on PATH and cannot be run: the shell's `if code -g "$f:1"` is false for
/// that as for a non-zero exit, so the verb prints the fallback line, finishes its summary and
/// exits 0. A file without the exec bit is a different case — neither `command -v` nor the port's
/// PATH scan resolves it, so it takes the "not on PATH" branch instead.
#[test]
fn r13__an_editor_that_cannot_run_is_not_an_error() {
    let dir = sandbox("open-broken");
    let bin = code_on_path(&dir, "#!/dstack/no-such-interpreter\n");
    let out = dstack(
        &dir,
        &format!("{}:{BARE_PATH}", bin.display()),
        &["request", "open"],
    );
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert_eq!(
        out.status.code(),
        Some(0),
        "an editor that cannot run is not a failure of the verb: {stdout}"
    );
    assert!(
        stdout.contains("  code -g failed; open the path above by hand\n"),
        "unexpected output: {stdout}"
    );
    assert!(
        stdout.contains(", approved: no\n"),
        "the summary line must still print: {stdout}"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}
