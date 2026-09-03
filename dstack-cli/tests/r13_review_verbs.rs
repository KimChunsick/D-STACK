// tests/r13_review_verbs.rs
// R13/R11/R04/R05: dstack review, check review-bundle, review seal and review close answer
// exactly as the shell answers — through the parity harness, through the refusal wording R11
// pins, and through the fixture runner of the two checkers they own.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::verbs::{self, review};

/// The two-tool table the harness gives its sandboxes, so no machine-wide deps.tsv is read.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
                    jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn shell_bin() -> String {
    shell_ref::dispatcher().to_string_lossy().into_owned()
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
    assert!(last.ends_with(", differing 0"), "{step} differs:\n{report}");
}

/// A repository with a store and one open run, built by the reference dispatcher the way the
/// harness builds its sandboxes.
fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p11-{tag}-{}", std::process::id()));
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
    dstack(
        &shell_bin(),
        &dir,
        &["run", "new", "sandbox", "--type", "cli"],
    );
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

/// The run id CURRENT names: it carries the UTC second the run was minted, so two sandboxes
/// built a second apart differ in every line that prints a run directory.
fn run_id(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(".dstack/local/CURRENT"))
        .unwrap_or_default()
        .trim_end_matches('\n')
        .to_string()
}

/// One dstack call: its stderr with the sandbox path and the run id masked, and its exit code.
fn dstack(bin: &str, dir: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    let stderr = String::from_utf8(out.stderr).expect("utf-8");
    let masked = stderr
        .replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>")
        .replace(&run_id(dir), "<RUNID>");
    (masked, out.status.code().expect("an exit code"))
}

/// Every wrong-usage call answers with the shell's stderr line and the shell's exit code.
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

/// The fixture runner of one checker, the way doctor --self drives it: every bad-* fixture has
/// to be rejected and every good-* fixture has to pass.
fn assert_fixtures(checker: &str) {
    let home = Home::resolve().expect("the repository");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let selftests = review::selftests();
    let selftest = selftests
        .iter()
        .find(|selftest| selftest.checker() == checker)
        .unwrap_or_else(|| panic!("no checker named {checker}"));
    let dir = repo().join("claude/lint/fixtures").join(checker);
    let mut fixtures = 0;
    for entry in std::fs::read_dir(&dir).expect("the fixture directory") {
        let path = entry.expect("a fixture").path();
        let name = path
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .into_owned();
        let wanted = match name.starts_with("bad-") {
            true => Verdict::Reject,
            false => Verdict::Pass,
        };
        let verdict = selftest
            .run(&mut ctx, &path)
            .unwrap_or_else(|e| panic!("{checker}/{name} cannot decide: {e}"));
        assert_eq!(verdict, wanted, "{checker}/{name}");
        fixtures += 1;
    }
    assert!(fixtures >= 2, "{checker} needs a bad and a good fixture");
}

#[test]
fn r13_the_review_verbs_reach_parity() {
    assert_parity("31-review");
}

#[test]
fn r11_review_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "review",
        &[
            &["review"],
            &["review", "--scope", "bogus"],
            &["review", "--scope", "plan"],
            &["review", "--scope", "milestone"],
            &["review", "--scope", "plan", "--plan", "P1", "--nope", "x"],
            &["review", "--scope"],
            &["review", "--scope", "plan", "--plan", "P1"],
            &["check", "review-bundle"],
            &["check", "review-bundle", "nosuch.txt"],
        ],
    );
}

#[test]
fn r11_the_rounds_refuse_with_the_shell_wording() {
    assert_same_refusal(
        "rounds",
        &[
            &["review", "seal"],
            &["review", "seal", "--nope", "x"],
            &["review", "seal", "--from"],
            &["review", "seal", "--from", "nosuch.md", "--scope", "plan", "--id", "P1"],
            &["review", "seal", "--from", "nosuch.md", "--scope", "bogus"],
            &["review", "close"],
            &["review", "close", "--nope", "x"],
            &["review", "close", "--scope"],
            &["review", "close", "--scope", "bogus", "--why", "no such scope"],
            &["review", "close", "--scope", "plan", "--why", "no id at all"],
            &["review", "close", "--scope", "plan", "--id", "P1", "--why", "no plan file"],
            &["review", "close", "--scope", "quick", "--why", "the target is a run"],
        ],
    );
}

#[test]
fn r05_the_bundle_checker_judges_its_fixtures() {
    assert_fixtures("check-review-bundle");
}

#[test]
fn r05_the_close_checker_judges_its_fixtures() {
    assert_fixtures("review-close");
}
