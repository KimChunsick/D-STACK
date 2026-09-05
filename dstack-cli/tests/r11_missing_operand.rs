// tests/r11_missing_operand.rs
// R11/R13: a value-taking option whose operand is missing ends exactly as the shell ends —
// exit 1, both streams empty — for the verbs with their own reader and for the target options
// (--run, --quick) core::args::opt reads; the forms that carry a value keep their wording.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The two-tool table the sandboxes read, so no machine-wide deps.tsv reaches the comparison.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
                    jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

fn shell_bin() -> String {
    shell_ref::dispatcher().to_string_lossy().into_owned()
}

/// A git repository with one empty commit, the two-tool deps table and a store made by the
/// reference dispatcher — the sandbox shape the parity harness uses.
fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p511-{tag}-{}", std::process::id()));
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

/// Every call answers with the shell's stdout, the shell's stderr and the shell's exit code.
fn assert_same(tag: &str, cases: &[&[&str]]) {
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

/// The option is the last argument, so the shell's `shift 2` fails under `set -e`: exit 1 with
/// nothing printed. Every value-taking option of the ported verbs ends the same way.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_a_last_option_without_its_operand_exits_silently() {
    assert_same(
        "novalue",
        &[
            &["run", "new", "x", "--type"],
            &["run", "new", "x", "--worktree"],
            &["run", "new", "x", "--type", "cli", "--worktree"],
            &["run", "close", "--abandon"],
        ],
    );
}

/// The neighbouring forms are untouched: an option that carries its value is parsed, an unknown
/// option keeps the pinned wording, `--` is a separator and not an option, and a verb that takes
/// no option ignores what it was given.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_the_other_option_forms_keep_their_answers() {
    assert_same(
        "forms",
        &[
            &["run", "new", "x", "--type=cli"],
            &["run", "new", "x", "--type", "bogus"],
            &["exec", "ok", "--"],
            &["status", "--bogus"],
            // The two forms core::args::opt reads, at the binary boundary.
            &["req", "status", "--run", "nosuch"],
            &["request", "show", "--quick", "nosuch"],
            &["cases", "render", "--run=nosuch"],
        ],
    );
}

/// The same silent exit through core::args::opt: every verb that resolves a target reads --run
/// and --quick there, so the option as the last argument ends the command with 1 and no output.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_a_target_option_without_its_operand_exits_silently() {
    assert_same(
        "target",
        &[
            &["req", "status", "--run"],
            &["request", "show", "--quick"],
            &["cases", "render", "--run"],
            &["check", "request", "--run"],
            &["req", "status", "--run", "nosuch", "--quick"],
        ],
    );
}

/// `--run=` is not the missing operand: the empty string after `=` is a value, so the command
/// runs on and refuses for its own reason. D-10 is the one place the port answers differently —
/// the shell joins the empty id and works on the runs directory itself (it even leaves a
/// meta.tsv there), while the port refuses the id before anything is touched.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_an_empty_value_after_the_equals_sign_is_a_value() {
    let shell_dir = sandbox("equals-shell");
    let rust_dir = sandbox("equals-rust");
    let args = &["req", "status", "--run="];
    let shell = dstack(&shell_bin(), &shell_dir, args);
    let ported = dstack(env!("CARGO_BIN_EXE_dstack"), &rust_dir, args);
    assert_eq!((shell.0.as_str(), shell.2), ("", 1));
    let reason = "no request.md in <SANDBOX>/.dstack/runs/";
    let hint = "(dstack request new --type <work_type>)";
    assert_eq!(shell.1, format!("dstack: {reason} {hint}\n"));
    assert!(shell_dir.join(".dstack/runs/meta.tsv").is_file());
    assert_eq!((ported.0.as_str(), ported.2), ("", 1));
    assert_eq!(ported.1, "dstack: run id must be a plain name (got '')\n");
    assert!(!rust_dir.join(".dstack/runs/meta.tsv").exists());
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
}
