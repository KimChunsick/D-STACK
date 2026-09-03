// tests/r03_store_readers.rs
// R03/R13: what a store reader does with a file it cannot read. An absent file is an answer — the
// empty one the shell's `[ -f ] && awk … || true` also gives — and the two implementations have to
// agree on it. A file that is there but cannot be read is not an answer: D-12 makes it a
// cannot-decide (exit 2, `dstack: cannot read <path>: …` on stderr) instead of the empty result the
// shell continues with, and nothing in the store is written on the way out.
#![allow(non_snake_case)]

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The two-tool table both sandboxes read, so no machine-wide deps.tsv reaches a test.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
                    jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

fn shell_bin() -> String {
    shell_ref::dispatcher().to_string_lossy().into_owned()
}

fn rust_bin() -> String {
    env!("CARGO_BIN_EXE_dstack").to_string()
}

/// A repository with a store and one open run, built by the reference dispatcher the way the
/// parity harness builds its sandboxes.
fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p122-{tag}-{}", std::process::id()));
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

/// Every ledger this test file reads: an approved request with its cases, a question, a decision
/// and one plan. Written by the reference dispatcher, so the files are the shell's own.
fn populate(dir: &Path) {
    let sh = shell_bin();
    dstack(
        &sh,
        dir,
        &["request", "new", "--type", "cli", "--title", "the request"],
    );
    dstack(
        &sh,
        dir,
        &["req", "add", "the row", "--accept", "the criterion"],
    );
    dstack(
        &sh,
        dir,
        &["ask", "add", "the question", "--affects", "R01"],
    );
    // An open question refuses the approval below, so the ledger keeps an answered row.
    dstack(
        &sh,
        dir,
        &[
            "ask",
            "answer",
            "Q-01",
            "the answer",
            "--decision",
            "the decision the answer made",
        ],
    );
    dstack(
        &sh,
        dir,
        &["decision", "add", "the decision", "--affects", "R01"],
    );
    dstack(&sh, dir, &["request", "approve"]);
    dstack(&sh, dir, &["cases", "sync"]);
    dstack(&sh, dir, &["milestone", "add", "first"]);
    dstack(
        &sh,
        dir,
        &["plan", "add", "one", "--milestone", "M1", "--files", "src"],
    );
    dstack(
        &sh,
        dir,
        &[
            "task", "add", "one", "--plan", "P1", "--covers", "R01", "--files", "src",
        ],
    );
}

fn git(dir: &Path, args: &[&str]) {
    let done = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(done.status.success(), "git {args:?} failed in {dir:?}");
}

fn run_id(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(".dstack/local/CURRENT"))
        .unwrap_or_default()
        .trim_end_matches('\n')
        .to_string()
}

fn run_dir(dir: &Path) -> PathBuf {
    dir.join(".dstack/runs").join(run_id(dir))
}

/// One dstack call: stdout, stderr and the exit code, with the sandbox path, the run id and every
/// UTC stamp masked — the values D-02 lets the two implementations differ in.
fn dstack(bin: &str, dir: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    let mask = |bytes: Vec<u8>| {
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let text = text.replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>");
        let id = run_id(dir);
        let text = match id.is_empty() {
            true => text,
            false => text.replace(&id, "<RUNID>"),
        };
        mask_stamps(&text)
    };
    (
        mask(out.stdout),
        mask(out.stderr),
        out.status.code().expect("an exit code"),
    )
}

/// `2026-09-02T21:10:43Z` → `<UTC>`: the stamp is fixed width, so the scan needs no regex.
fn mask_stamps(text: &str) -> String {
    let bytes: Vec<char> = text.chars().collect();
    let shape = "0000-00-00T00:00:00Z";
    let mut out = String::new();
    let mut at = 0;
    while at < bytes.len() {
        let window: String = bytes.iter().skip(at).take(shape.len()).collect();
        let matches = window.chars().count() == shape.len()
            && window.chars().zip(shape.chars()).all(|(c, s)| match s {
                '0' => c.is_ascii_digit(),
                other => c == other,
            });
        if matches {
            out.push_str("<UTC>");
            at += shape.len();
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    out
}

/// Every readable file of the store as (path, bytes): what "nothing was written" compares.
///
/// meta.tsv is left out: both implementations stamp the owner heartbeat into it while they
/// resolve the target, before the verb reads a ledger at all, so it changes on every call.
fn store_snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    collect(&dir.join(".dstack"), dir, &mut files);
    files.sort();
    files
}

fn collect(at: &Path, root: &Path, files: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(at).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, root, files);
            continue;
        }
        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        if name.ends_with("meta.tsv") {
            continue;
        }
        // The file the test made unreadable is the one it cannot snapshot; its own permission is
        // what the assertions below are about.
        if let Ok(bytes) = std::fs::read(&path) {
            files.push((name, bytes));
        }
    }
}

fn chmod(path: &Path, mode: &str) {
    let done = Command::new("chmod")
        .args([mode, &path.to_string_lossy().into_owned()])
        .output()
        .expect("run chmod");
    assert!(done.status.success(), "chmod {mode} {path:?} failed");
}

/// The contract of D-12 for one store file: with the file unreadable, every one of these verbs
/// exits 2, names the path on stderr and leaves the store exactly as it found it.
fn assert_cannot_decide(tag: &str, file: &str, verbs: &[&[&str]]) {
    let dir = sandbox(tag);
    populate(&dir);
    let path = run_dir(&dir).join(file);
    assert!(path.is_file(), "{path:?} was not written by populate()");
    let before = store_snapshot(&dir);
    chmod(&path, "000");
    for args in verbs {
        let (out, err, code) = dstack(&rust_bin(), &dir, args);
        assert_eq!(
            code, 2,
            "dstack {args:?} with an unreadable {file}: {err}{out}"
        );
        let wanted = format!("dstack: cannot read <SANDBOX>/.dstack/runs/<RUNID>/{file}: ");
        assert!(
            err.starts_with(&wanted),
            "dstack {args:?} with an unreadable {file} said {err:?}, wanted {wanted:?}"
        );
    }
    chmod(&path, "644");
    assert_eq!(
        before,
        store_snapshot(&dir),
        "an unreadable {file} still let a verb write to the store"
    );
}

/// Case (a): the file is not there, and both implementations give the same empty answer.
#[test]
fn r03__an_absent_store_file_answers_the_way_the_shell_answers() {
    let shell = sandbox("absent-shell");
    let rust = sandbox("absent-rust");
    let verbs: &[&[&str]] = &[
        &["ask", "list"],
        &["decision", "list"],
        &["check", "decisions"],
        &["check", "request"],
        &["req", "status"],
        &["cases", "render"],
        &["next"],
        &["status"],
        &["status", "--oneline"],
        &["verify"],
        &["report"],
        &[
            "review", "close", "--scope", "plan", "--id", "P1", "--why", "x",
        ],
    ];
    for args in verbs {
        assert_eq!(
            dstack(&shell_bin(), &shell, args),
            dstack(&rust_bin(), &rust, args),
            "dstack {args:?} differs on a run whose ledgers are all absent"
        );
    }
}

/// Case (b), questions.md: `check request` counted an unreadable ledger as zero questions and let
/// the check pass (review round 040); `ask list` printed a ledger of no rows.
#[test]
fn r13__an_unreadable_questions_md_cannot_decide() {
    assert_cannot_decide(
        "questions",
        "questions.md",
        &[&["check", "request"], &["ask", "list"]],
    );
}

/// Case (b), decisions.md: the two readers of the decision ledger.
#[test]
fn r13__an_unreadable_decisions_md_cannot_decide() {
    assert_cannot_decide(
        "decisions",
        "decisions.md",
        &[&["decision", "list"], &["check", "decisions"]],
    );
}

/// Case (b), cases.tsv: the evidence ledger every verdict is computed from.
#[test]
fn r13__an_unreadable_cases_tsv_cannot_decide() {
    assert_cannot_decide(
        "cases",
        "cases.tsv",
        &[&["req", "status"], &["verify"], &["report"], &["status"]],
    );
}

/// Case (b), review/index.tsv: the sealed rounds a close and a bundle both count.
#[test]
fn r13__an_unreadable_review_index_cannot_decide() {
    let dir = sandbox("index");
    populate(&dir);
    let path = run_dir(&dir).join("review/index.tsv");
    std::fs::create_dir_all(path.parent().expect("review dir")).expect("review dir");
    std::fs::write(&path, "001\tplan\tP1\tcodex-review-001.md\t-\t0\t0\t1\n").expect("index row");
    let before = store_snapshot(&dir);
    chmod(&path, "000");
    for args in [
        vec![
            "review", "close", "--scope", "plan", "--id", "P1", "--why", "x",
        ],
        vec!["review", "--scope", "milestone", "--milestone", "M1"],
    ] {
        let (out, err, code) = dstack(&rust_bin(), &dir, &args);
        assert_eq!(code, 2, "dstack {args:?}: {err}{out}");
        assert!(
            err.starts_with(
                "dstack: cannot read <SANDBOX>/.dstack/runs/<RUNID>/review/index.tsv: "
            ),
            "dstack {args:?} said {err:?}"
        );
    }
    chmod(&path, "644");
    assert_eq!(before, store_snapshot(&dir), "the store was written");
}

/// Case (c), plan.json: a file that is there but is not the JSON the store writes. jq fails in the
/// shell, so nothing here is a divergence of behaviour — only of the message.
#[test]
fn r13__an_unparseable_plan_json_cannot_decide() {
    let dir = sandbox("plan-json");
    populate(&dir);
    let path = run_dir(&dir).join("plan.json");
    std::fs::write(&path, "{\"v\":2,\"milestones\":[\n").expect("a truncated plan.json");
    let before = store_snapshot(&dir);
    for args in [vec!["next"], vec!["status"], vec!["plan", "render"]] {
        let (out, err, code) = dstack(&rust_bin(), &dir, &args);
        assert_eq!(code, 2, "dstack {args:?}: {err}{out}");
        assert!(
            err.starts_with("dstack: cannot read <SANDBOX>/.dstack/runs/<RUNID>/plan.json: "),
            "dstack {args:?} said {err:?}"
        );
    }
    assert_eq!(before, store_snapshot(&dir), "the store was written");
}

/// Case (b), plan.json: unreadable, which the shell's jq also fails on.
#[test]
fn r13__an_unreadable_plan_json_cannot_decide() {
    assert_cannot_decide("plan-perm", "plan.json", &[&["next"], &["status"]]);
}

/// Case (b), request.md: the document every verb starts from.
#[test]
fn r13__an_unreadable_request_md_cannot_decide() {
    assert_cannot_decide(
        "request",
        "request.md",
        &[&["status"], &["req", "status"], &["check", "request"]],
    );
}

/// Row-level tolerance stays: awk skips a row it counts too few columns in and reads on, so a
/// half-written cases.tsv row is not a cannot-decide.
#[test]
fn r03__a_short_row_stays_tolerated() {
    let dir = sandbox("short-row");
    populate(&dir);
    let cases = run_dir(&dir).join("cases.tsv");
    let text = std::fs::read_to_string(&cases).expect("the ledger");
    std::fs::write(&cases, format!("{text}R99\tc1\n")).expect("a row of two cells");
    let (out, err, code) = dstack(&rust_bin(), &dir, &["cases", "render"]);
    assert_eq!(code, 0, "{err}");
    assert!(!out.contains("R99"), "the short row was not skipped: {out}");
}

// ── round 052: a verb that both reads and writes reads everything first ──────────────────────
// Four findings of one shape: the store was already mutated when the read that fails happened.

/// The store before and after, with the one call in between: nothing may change.
fn assert_no_write(tag: &str, unreadable: &str, args: &[&str], absent: Option<&str>) {
    let dir = sandbox(tag);
    populate(&dir);
    if let Some(gone) = absent {
        std::fs::remove_file(run_dir(&dir).join(gone)).expect("remove the ledger");
    }
    let path = run_dir(&dir).join(unreadable);
    let before = store_snapshot(&dir);
    chmod(&path, "000");
    let (out, err, code) = dstack(&rust_bin(), &dir, args);
    chmod(&path, "644");
    assert_eq!(code, 2, "dstack {args:?}: {err}{out}");
    let wanted = format!("cannot read <SANDBOX>/.dstack/runs/<RUNID>/{unreadable}: ");
    assert!(
        err.lines()
            .any(|line| line.starts_with("dstack: ") && line.contains(&wanted)),
        "dstack {args:?} said {err:?}, wanted a line naming {unreadable}"
    );
    assert_eq!(
        before,
        store_snapshot(&dir),
        "dstack {args:?} wrote to the store before the read of {unreadable} failed"
    );
    if let Some(gone) = absent {
        assert!(
            !run_dir(&dir).join(gone).exists(),
            "dstack {args:?} created {gone} before the read failed"
        );
    }
}

/// Round 052 (1): `ask add` warned about a request.md it could not read and appended anyway.
#[test]
fn r13__ask_add_stops_on_an_unreadable_request_md() {
    assert_no_write(
        "r2-ask-add",
        "request.md",
        &["ask", "add", "another question", "--affects", "R01"],
        None,
    );
}

/// Round 052 (2): `ask answer` moved the question to answered before decisions.md was read.
#[test]
fn r13__ask_answer_keeps_the_question_open_when_decisions_md_cannot_be_read() {
    let dir = sandbox("r2-ask-answer");
    populate(&dir);
    dstack(
        &shell_bin(),
        &dir,
        &["ask", "add", "the second question", "--affects", "R01"],
    );
    let path = run_dir(&dir).join("decisions.md");
    let before = store_snapshot(&dir);
    chmod(&path, "000");
    let args = [
        "ask",
        "answer",
        "Q-02",
        "the answer",
        "--decision",
        "the decision",
    ];
    let (out, err, code) = dstack(&rust_bin(), &dir, &args);
    chmod(&path, "644");
    assert_eq!(code, 2, "{err}{out}");
    assert!(
        err.starts_with("dstack: cannot read <SANDBOX>/.dstack/runs/<RUNID>/decisions.md: "),
        "said {err:?}"
    );
    assert_eq!(before, store_snapshot(&dir), "questions.md was rewritten");
}

/// Round 052 (3): `request approve` cleared the pending markers and stamped the approval before
/// the ledger `cases sync` expands was ever read.
#[test]
fn r13__request_approve_writes_nothing_when_cases_tsv_cannot_be_read() {
    assert_no_write("r2-approve", "cases.tsv", &["request", "approve"], None);
}

/// Round 052 (4): `cases sync` and `evidence add` created the ledger header before reading the
/// request they expand it from.
#[test]
fn r13__cases_sync_writes_no_header_when_request_md_cannot_be_read() {
    assert_no_write(
        "r2-sync",
        "request.md",
        &["cases", "sync"],
        Some("cases.tsv"),
    );
}

#[test]
fn r13__evidence_add_writes_no_header_when_request_md_cannot_be_read() {
    assert_no_write(
        "r2-evidence",
        "request.md",
        &[
            "evidence",
            "add",
            "--r",
            "R01",
            "--case",
            "c1",
            "--kind",
            "cli",
            "--artifact",
            "art.txt",
            "--produced-by",
            "the test",
        ],
        Some("cases.tsv"),
    );
}

/// Round 052 (sweep): `run new` counted the plans of the other open runs after minting its own.
#[test]
fn r13__run_new_mints_nothing_when_another_runs_plan_json_cannot_be_read() {
    let dir = sandbox("r2-run-new");
    populate(&dir);
    let path = run_dir(&dir).join("plan.json");
    let before = store_snapshot(&dir);
    let runs_before = std::fs::read_dir(dir.join(".dstack/runs"))
        .expect("runs")
        .count();
    chmod(&path, "000");
    // A second Goal needs its own worktree (R33); the count has to fail before either is made.
    let args = [
        "run",
        "new",
        "second",
        "--type",
        "cli",
        "--worktree",
        "../second",
    ];
    let (out, err, code) = dstack(&rust_bin(), &dir, &args);
    chmod(&path, "644");
    assert_eq!(code, 2, "{err}{out}");
    assert!(
        err.starts_with("dstack: cannot read <SANDBOX>/.dstack/runs/<RUNID>/plan.json: "),
        "said {err:?}"
    );
    assert_eq!(
        runs_before,
        std::fs::read_dir(dir.join(".dstack/runs"))
            .expect("runs")
            .count(),
        "a run directory was minted before the read failed"
    );
    assert_eq!(before, store_snapshot(&dir), "the store was written");
}

/// Round 053: `ask assume` moved the question and appended the decision before its `req add`
/// child ever loaded request.md, so an unreadable request left both writes behind and reported
/// the child's cannot-decide as this verb's checked failure.
#[test]
fn r13__ask_assume_writes_neither_ledger_when_request_md_cannot_be_read() {
    let dir = sandbox("r3-ask-assume");
    populate(&dir);
    dstack(
        &shell_bin(),
        &dir,
        &["ask", "add", "the open question", "--affects", "R01"],
    );
    let path = run_dir(&dir).join("request.md");
    let questions = std::fs::read(run_dir(&dir).join("questions.md")).expect("questions.md");
    let decisions = std::fs::read(run_dir(&dir).join("decisions.md")).expect("decisions.md");
    let before = store_snapshot(&dir);
    chmod(&path, "000");
    let args = [
        "ask",
        "assume",
        "Q-02",
        "the default",
        "--accept",
        "what is observed if it is wrong",
    ];
    let (out, err, code) = dstack(&rust_bin(), &dir, &args);
    chmod(&path, "644");
    assert_eq!(code, 2, "{err}{out}");
    assert!(
        err.starts_with("dstack: cannot read <SANDBOX>/.dstack/runs/<RUNID>/request.md: "),
        "said {err:?}"
    );
    assert_eq!(
        questions,
        std::fs::read(run_dir(&dir).join("questions.md")).expect("questions.md"),
        "the question was moved to assumed"
    );
    assert_eq!(
        decisions,
        std::fs::read(run_dir(&dir).join("decisions.md")).expect("decisions.md"),
        "a decision row was appended"
    );
    assert_eq!(before, store_snapshot(&dir), "the store was written");
}

/// Case (b), meta.tsv: the key/value table of the run itself. The shell's awk prints nothing for
/// a file it cannot open, so a run whose meta.tsv is unreadable read as a run with no status,
/// no owner and no branch — and every verdict computed from those fields was computed from air.
#[test]
fn r13__an_unreadable_meta_tsv_cannot_decide() {
    assert_cannot_decide(
        "r3-meta",
        "meta.tsv",
        &[
            &["status"],
            &["status", "--oneline"],
            &["run", "list"],
            &["next"],
            &["report"],
        ],
    );
}

/// Case (b), CURRENT: the file that names the run of this worktree. `cat` prints nothing for a
/// file it cannot open, so the port would have read "no current run" and gone on to answer about
/// a worktree it could not actually see.
#[test]
fn r13__an_unreadable_current_cannot_decide() {
    let dir = sandbox("r3-current");
    populate(&dir);
    let current = dir.join(".dstack/local/CURRENT");
    let before = store_snapshot(&dir);
    chmod(&current, "000");
    for args in [
        &["status"][..],
        &["status", "--oneline"],
        &["run", "verify"],
        &["gate"],
        &["req", "status"],
    ] {
        let (out, err, code) = dstack(&rust_bin(), &dir, args);
        assert_eq!(code, 2, "dstack {args:?} with an unreadable CURRENT: {err}{out}");
        assert!(
            err.starts_with("dstack: cannot read <SANDBOX>/.dstack/local/CURRENT: "),
            "dstack {args:?} said {err:?}"
        );
    }
    chmod(&current, "644");
    assert_eq!(
        before,
        store_snapshot(&dir),
        "an unreadable CURRENT still let a verb write to the store"
    );
}
