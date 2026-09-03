// tests/r13_request_verbs.rs
// R13/R11/R04/R05: request new|open|show|approve, the req rows and check request answer exactly
// as the shell answers — through the parity harness, through the refusal wording R11 pins, and
// through the three fixture checkers of R05.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;

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
    assert!(last.ends_with(", differing 0"), "{step} differs:\n{report}");
}

/// A repository with a store, built the way the harness builds its sandboxes: a git repository
/// with one empty commit, the two-tool deps table, and `init` run by the reference dispatcher.
fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p6-{tag}-{}", std::process::id()));
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

fn shell_bin() -> String {
    shell_ref::dispatcher().to_string_lossy().into_owned()
}

/// One dstack call: its stderr with the sandbox path masked, and its exit code. PATH carries no
/// `code`, so `request open` never launches an editor on the machine running the tests.
fn dstack(bin: &str, dir: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("PATH", "/usr/bin:/bin")
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    let stderr = String::from_utf8(out.stderr).expect("utf-8");
    (
        mask_run_id(&stderr.replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>")),
        out.status.code().expect("an exit code"),
    )
}

/// The two sandboxes are minted a second apart when the machine is loaded, so the run id of a
/// message is masked exactly as the harness masks it.
fn mask_run_id(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("Z_") {
        let stamp = &rest[..at];
        let is_stamp = stamp.len() >= 15
            && stamp[stamp.len() - 15..].chars().enumerate().all(|(i, c)| {
                if i == 8 {
                    c == 'T'
                } else {
                    c.is_ascii_digit()
                }
            });
        if is_stamp {
            out.push_str(&stamp[..stamp.len() - 15]);
            out.push_str("<RUNID>_");
        } else {
            out.push_str(stamp);
            out.push_str("Z_");
        }
        rest = &rest[at + 2..];
    }
    out.push_str(rest);
    out
}

/// Every wrong-usage call answers with the shell's stderr line and the shell's exit code. The
/// two sandboxes are driven in step, so a call that writes leaves both stores in the same state.
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
fn r13_request_new_open_and_show_reach_parity() {
    assert_parity("20-request");
}

#[test]
fn r13_the_req_rows_reach_parity() {
    assert_parity("21-req");
}

/// Every fixture of one checker, judged by the checker itself: bad-* must be rejected and
/// good-* must pass, which is what the fixture runner of R05 asserts.
fn assert_fixtures(checker: &str) {
    let home = Home::resolve().expect("the repository of this test binary");
    let dir = home.home.join("lint/fixtures").join(checker);
    let mut ctx = Context::new(
        Home::resolve().expect("the repository of this test binary"),
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(dstack_cli::verbs::all_verbs())),
    );
    let checkers = dstack_cli::verbs::request::selftests();
    let selftest = checkers
        .iter()
        .find(|entry| entry.checker() == checker)
        .unwrap_or_else(|| panic!("no selftest registers the checker {checker}"));
    let mut fixtures = 0;
    for entry in std::fs::read_dir(&dir).expect("the fixture directory") {
        let fixture = entry.expect("a fixture").path();
        let name = fixture
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();
        let wanted = match name.split_once('-') {
            Some(("bad", _)) => Verdict::Reject,
            Some(("good", _)) => Verdict::Pass,
            _ => panic!("{name} is neither a bad- nor a good- fixture"),
        };
        let verdict = selftest
            .run(&mut ctx, &fixture)
            .expect("the checker decides");
        assert_eq!(verdict, wanted, "{checker}/{name}");
        fixtures += 1;
    }
    assert!(fixtures >= 2, "{checker} needs a bad and a good fixture");
}

#[test]
fn r05_req_add_judges_its_fixtures() {
    assert_fixtures("req-add");
}

#[test]
fn r05_check_request_judges_its_fixtures() {
    assert_fixtures("check-request");
}

/// The scenario checker: approve, then edit the file behind the approval and let check request
/// catch it by the hash.
#[test]
fn r05_request_approve_judges_its_fixtures() {
    assert_fixtures("request-approve");
}

/// The run directory CURRENT names in a sandbox.
fn run_dir(dir: &Path) -> PathBuf {
    let current = std::fs::read_to_string(dir.join(".dstack/local/CURRENT")).expect("CURRENT");
    dir.join(".dstack/runs")
        .join(current.trim_end_matches('\n'))
}

/// `req add --assumption` writes three files with no transaction between them, so it reads all
/// three before it writes any: a decision ledger it cannot read stops the verb with nothing
/// written at all, and the question is still open for a second attempt.
#[test]
fn r13_an_assumption_with_an_unreadable_decision_ledger_writes_nothing() {
    let dir = sandbox("rollback");
    let run = run_dir(&dir);
    dstack(&shell_bin(), &dir, &["request", "new", "--type", "cli"]);
    std::fs::write(
        run.join("questions.md"),
        "# Questions (R51)\n\n| Q | Question | Affects | Status |\n|---|---|---|---|\n         | Q-01 | which default? | R01 | open |\n",
    )
    .expect("the question ledger");
    // A directory where the decision row has to be appended: the third write cannot succeed.
    std::fs::create_dir(run.join("decisions.md")).expect("the blocked decision ledger");

    let (stderr, code) = dstack(
        env!("CARGO_BIN_EXE_dstack"),
        &dir,
        &[
            "req",
            "add",
            "the assumed row",
            "--accept",
            "c",
            "--assumption",
            "--from",
            "Q-01",
        ],
    );
    assert_eq!(
        code, 2,
        "a store file it cannot read is a cannot-decide: {stderr}"
    );
    assert!(
        stderr.contains("cannot read") && stderr.contains("decisions.md"),
        "unexpected refusal: {stderr}"
    );
    let questions = std::fs::read_to_string(run.join("questions.md")).expect("the ledger");
    assert!(
        questions.contains("| Q-01 | which default? | R01 | open |"),
        "the question was left assumed:\n{questions}"
    );
    let request = std::fs::read_to_string(run.join("request.md")).expect("the request");
    assert!(
        !request.contains("**R01**"),
        "the row survived the failed write:\n{request}"
    );
    // A decision ledger that exists but cannot be read is never mistaken for one that is not
    // there: a revert that unlinked it would lose every recorded decision.
    assert!(
        run.join("decisions.md").exists(),
        "the unreadable decision ledger was removed"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// A draft snapshot that is not there is a line of prose; one that exists and cannot be read is
/// a store file the verb cannot parse, and D-12 makes that a cannot-decide instead of a diff
/// quietly left out of the approval record.
#[test]
fn r13_an_unreadable_draft_snapshot_cannot_decide() {
    let dir = sandbox("draft");
    let run = run_dir(&dir);
    dstack(&shell_bin(), &dir, &["request", "new", "--type", "cli"]);
    dstack(
        &shell_bin(),
        &dir,
        &["req", "add", "a row", "--accept", "a criterion"],
    );
    std::fs::create_dir(run.join("request.agent-draft.md")).expect("the unreadable draft");

    // A row that a first approval would have to clear: the refusal must leave it marked.
    std::fs::write(run.join("request.approved"), "").expect("a stamp to mark the row pending");
    dstack(
        &shell_bin(),
        &dir,
        &[
            "req",
            "add",
            "a pending row",
            "--accept",
            "another criterion",
        ],
    );
    std::fs::remove_file(run.join("request.approved")).expect("no approval yet");

    let (stderr, code) = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, &["request", "approve"]);
    assert_eq!(code, 2, "an unreadable store file exits 2: {stderr}");
    assert!(
        stderr.contains("cannot read") && stderr.contains("request.agent-draft.md"),
        "unexpected refusal: {stderr}"
    );
    // Nothing was written: no stamp, and the pending marker the approval would have cleared.
    assert!(
        !run.join("request.approved").exists(),
        "the request was stamped although the approval refused"
    );
    let request = std::fs::read_to_string(run.join("request.md")).expect("the request");
    assert!(
        request.contains(
            "**R02** a pending row — accept: another criterion — status: pending-approval"
        ),
        "the pending marker was cleared by a refused approval:\n{request}"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r11_request_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "request",
        &[
            &["request", "show"],
            &["request", "open"],
            &["request", "approve"],
            &["check", "request"],
            &["check", "request", "--bogus"],
            &["request", "new", "--bogus"],
            &["request", "new", "extra"],
            &["request", "new", "--type", "bogus"],
            &["request", "new", "--type=bogus"],
            &["request", "new", "--type"],
            &["request", "new", "--title"],
            &["request", "new", "--type", "cli"],
            &["request", "new", "--type", "cli"],
            &["request", "show", "--bogus"],
            &["check", "request", "--bogus"],
        ],
    );
}

#[test]
fn r11_req_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "req",
        &[
            &["req", "add", "--bogus"],
            &["req", "add"],
            &["req", "accept", "R01", "c"],
            &["req", "split", "R01", "--into", "R02"],
            &["req", "withdraw", "R01", "--why", "x"],
            &["req", "defer", "R01", "--why", "x"],
            &["req", "status"],
            &["request", "new", "--type", "cli"],
            &["req", "add", "a row"],
            &["req", "add", "a row", "--accept", "a — b"],
            &["req", "add", "a row", "--accept", "c", "--assumption"],
            &["req", "add", "a row", "--accept", "c", "--from", "Q-01"],
            &[
                "req",
                "add",
                "a row",
                "--accept",
                "c",
                "--assumption",
                "--from",
                "Q-01",
            ],
            &["req", "add", "a row", "--accept", "c"],
            &["req", "add", "another", "--accept", "c", "--id", "R01"],
            &["req", "add", "another", "--accept", "c", "--id", "R0x"],
            &["req", "accept", "R01"],
            &["req", "accept", "R99", "c"],
            &["req", "accept", "R01", "c"],
            &["req", "split", "R01"],
            &["req", "split", "R99", "--into", "R01,R02"],
            &["req", "split", "R01", "--into", "R01,R02"],
            &["req", "split", "R01", "--into", "R02"],
            &["req", "withdraw", "R01"],
            &["req", "withdraw", "R99", "--why", "x"],
            &["req", "defer", "R99", "--why", "x"],
        ],
    );
}
