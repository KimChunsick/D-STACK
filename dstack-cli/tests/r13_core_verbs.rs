// tests/r13_core_verbs.rs
// R13/R11/R04: init, the six run verbs, status and exec answer exactly as the shell answers,
// both through the parity harness and through the refusal wording R11 pins.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The two-tool table the harness gives its sandboxes, so no machine-wide deps.tsv is read.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
                    jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// One harness run over a single step, asserting that nothing differs — stdout, stderr, exit
/// codes and the store files of both sandboxes.
fn assert_parity(step: &str) {
    let out = Command::new("bash")
        .arg(repo().join("dstack-cli/parity/run.sh"))
        .args(["--rust", env!("CARGO_BIN_EXE_dstack"), "--only", step])
        .output()
        .expect("run the parity harness");
    let report = String::from_utf8(out.stdout).expect("utf-8");
    let aborted = String::from_utf8(out.stderr).expect("utf-8");
    assert!(aborted.is_empty(), "the harness aborted: {aborted}");
    let last = report.lines().last().unwrap_or("");
    assert!(
        last.ends_with(", differing 0"),
        "{step} differs:\n{report}"
    );
}

/// A repository with a store, built the way the harness builds its sandboxes: a git repository
/// with one empty commit, the two-tool deps table, and `init` run by the reference dispatcher.
fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p5-{tag}-{}", std::process::id()));
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
    dstack(&shell_bin(), &dir, &["init"]);
    dir
}

fn git(dir: &Path, args: &[&str]) {
    let done = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(done.status.success(), "git {args:?} failed in {dir:?}");
}

fn shell_bin() -> String {
    shell_ref::dispatcher().to_string_lossy().into_owned()
}

/// One dstack call: its two streams with the sandbox path masked, and its exit code.
fn dstack(bin: &str, dir: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    let mask = |stream: Vec<u8>| {
        String::from_utf8(stream)
            .expect("utf-8")
            .replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>")
    };
    (
        mask(out.stdout),
        mask(out.stderr),
        out.status.code().expect("an exit code"),
    )
}

/// Every wrong-usage call answers with the shell's two streams and the shell's exit code.
fn assert_same_refusal(tag: &str, cases: &[&[&str]]) {
    let shell_dir = sandbox(&format!("{tag}-shell"));
    let rust_dir = sandbox(&format!("{tag}-rust"));
    for args in cases {
        let shell = dstack(&shell_bin(), &shell_dir, args);
        let ported = dstack(env!("CARGO_BIN_EXE_dstack"), &rust_dir, args);
        assert_eq!(ported, shell, "dstack {args:?}");
    }
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
}

#[test]
fn r13_init_and_the_run_verbs_reach_parity() {
    assert_parity("10-init-run");
}

#[test]
fn r13_status_and_exec_reach_parity() {
    assert_parity("11-status-exec");
}

#[test]
fn r11_init_and_run_refuse_with_the_shell_wording() {
    assert_same_refusal("run", &[
        &["init", "--bogus"],
        &["run", "new", "--bogus"],
        &["run", "new"],
        &["run", "new", "Bad_Slug"],
        &["run", "new", "x", "--type", "bogus"],
        &["run", "new", "x", "--type=cli"],
        &["run", "adopt", "--bogus"],
        &["run", "adopt", "nosuch"],
        &["run", "adopt"],
        &["run", "close", "--bogus"],
        &["run", "close", "nosuch", "--abandon", "why"],
        &["run", "close"],
        &["run", "pause"],
    ]);
}

/// A command that cannot be spawned at all: the shell's redirection makes bash answer, with 127
/// for a command it cannot find and 126 for a file it finds and cannot run.
#[test]
fn r13_exec_not_found_exits_127() {
    assert_same_exec_failure("execnf", &["exec", "nf", "--", "no-such-command-of-mine"], "nf", 127);
}

#[test]
fn r13_exec_not_executable_exits_126() {
    assert_same_exec_failure("execne", &["exec", "ne", "--", "./not-runnable"], "ne", 126);
}

/// Both dispatchers on one call: the same exit code, and the port's err.txt carries the reason
/// (bash's line names the script and the line number, which no port can reproduce).
fn assert_same_exec_failure(tag: &str, args: &[&str], label: &str, expected: i32) {
    let shell_dir = sandbox(&format!("{tag}-shell"));
    let rust_dir = sandbox(&format!("{tag}-rust"));
    for dir in [&shell_dir, &rust_dir] {
        std::fs::write(dir.join("not-runnable"), "not a program\n").expect("a file with no x bit");
    }
    let shell = dstack(&shell_bin(), &shell_dir, args).2;
    let ported = dstack(env!("CARGO_BIN_EXE_dstack"), &rust_dir, args).2;
    assert_eq!(ported, shell, "dstack {args:?}");
    assert_eq!(ported, expected, "dstack {args:?}");
    let err = rust_dir.join(".dstack/local/exec").join(label).join("err.txt");
    let said = std::fs::read_to_string(&err).expect("read err.txt");
    assert!(!said.trim().is_empty(), "err.txt of a failed spawn says nothing: {err:?}");
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
}

/// D-10: a run id names one directory inside the store and nothing else. The shell joined the
/// id unchecked, so `run adopt ../x` took the owner of a meta.tsv outside the runs directory and
/// `run close ../x --abandon why` stamped it closed. The port refuses before it builds the path.
#[test]
fn r13_a_run_id_that_is_not_a_plain_name_is_refused() {
    let dir = sandbox("plainid");
    // Where `../x` lands: one level above the runs directory, with a run-shaped file in it.
    let escaped = dir.join(".dstack/x");
    std::fs::create_dir_all(&escaped).expect("a directory the id could escape to");
    std::fs::write(escaped.join("meta.tsv"), "status\topen\n").expect("write the bait");
    for (args, id) in [
        (vec!["run", "adopt", "../x"], "../x"),
        (vec!["run", "adopt", "/tmp/x"], "/tmp/x"),
        (vec!["run", "close", "../x", "--abandon", "why"], "../x"),
    ] {
        let call = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, &args);
        let refusal = format!("dstack: run id must be a plain name (got '{id}')\n");
        assert_eq!(call, (String::new(), refusal, 1), "dstack {args:?}");
    }
    // run pause reads the id from CURRENT instead of the command line; a poisoned CURRENT is the
    // same traversal, so it is refused before the id becomes a path.
    std::fs::write(dir.join(".dstack/local/CURRENT"), "../x\n").expect("poison CURRENT");
    let paused = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, &["run", "pause"]);
    let refusal = "dstack: run id must be a plain name (got '../x')\n".to_string();
    assert_eq!(paused, (String::new(), refusal, 1), "dstack run pause");
    let bait = std::fs::read_to_string(escaped.join("meta.tsv")).expect("read the bait");
    assert_eq!(bait, "status\topen\n", "a refused id wrote outside the runs directory");
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// A value-taking option whose value is missing: the shell's `shift 2` fails under `set -e`, so
/// the command ends with 1 and says nothing. The port answers with the same silent exit.
#[test]
fn r11_a_missing_option_value_exits_like_the_shell() {
    assert_same_refusal("novalue", &[
        &["run", "new", "x", "--type"],
        &["run", "new", "x", "--worktree"],
        &["run", "new", "x", "--type", "cli", "--worktree"],
        &["run", "close", "--abandon"],
    ]);
}

#[test]
fn r11_status_and_exec_refuse_with_the_shell_wording() {
    assert_same_refusal("statusexec", &[
        &["status", "--bogus"],
        &["exec"],
        &["exec", "ok"],
        &["exec", "ok", "--"],
        &["exec", "bad/label", "--", "echo", "hi"],
        &["exec", "..", "--", "echo", "hi"],
        &["exec", "two words", "--", "echo", "hi"],
    ]);
}

/// R09: every source file of the crate stays inside the 350-line rule of the repository.
#[test]
fn r09_no_source_file_is_over_the_line_limit() {
    for file in source_files() {
        let lines = std::fs::read_to_string(&file).expect("read the source").lines().count();
        assert!(lines <= 350, "{} is {lines} lines — split it by responsibility", file.display());
    }
}

/// R13: only main() decides the exit code of the process, so a verb called in process (the
/// self-call of Context::call) cannot kill its caller; it returns Error::Exit instead.
#[test]
fn r13_no_verb_ends_the_process_itself() {
    for file in source_files() {
        if file.ends_with("main.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read the source");
        assert!(
            !text.contains("process::exit"),
            "{} calls process::exit; return Error::Exit instead",
            file.display()
        );
    }
}

/// Every .rs file under dstack-cli/src, in name order.
fn source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![repo().join("dstack-cli/src")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).expect("read the source tree").flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}
