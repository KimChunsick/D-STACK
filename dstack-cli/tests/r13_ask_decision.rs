// tests/r13_ask_decision.rs
// R13/R11/R04: the question ledger (ask) and the decision rows (decision, check decisions)
// answer exactly as the shell answers, through the parity harness and through the refusal
// wording R11 pins; R05 keeps the check-decisions fixtures able to fail.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;

/// The two-tool table the harness gives its sandboxes, so no machine-wide deps.tsv is read.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
                    jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A scratch directory this test created and nothing else. fs::create_dir is exclusive, so an
/// existing path — a directory, a file, or a symlink planted on shared temp storage — makes it
/// fail instead of being reused or followed; the name carries a nonce so a lost race retries.
/// Nothing that was already there is ever removed.
fn scratch_dir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir();
    for _ in 0..16 {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.subsec_nanos())
            .unwrap_or(0);
        let dir = base.join(format!(
            "dstack-p7-{tag}.{}.{}.{nonce}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => {
                // The store resolves physical paths, so the masking in dstack() needs one too.
                return std::fs::canonicalize(&dir).expect("the physical path of the scratch dir");
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("cannot create {}: {e}", dir.display()),
        }
    }
    panic!("no free scratch directory under {}", base.display());
}

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

/// A repository with a store, one open run and one open question — the state the wrong-usage
/// calls are made against. Both sandboxes are built by the shell dispatcher, so the calls under
/// test are the only thing that can differ.
fn sandbox(tag: &str) -> PathBuf {
    let dir = scratch_dir(tag);
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
        &["run", "new", "sandbox", "--type", "cli"],
        &["ask", "add", "which store format", "--affects", "R01"],
    ] {
        let (stderr, code) = dstack(&shell_bin(), &dir, args);
        assert_eq!(code, 0, "building the sandbox: dstack {args:?}: {stderr}");
    }
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

/// One dstack call: its stderr with the sandbox path masked, and its exit code.
fn dstack(bin: &str, dir: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    let stderr = String::from_utf8(out.stderr).expect("utf-8");
    (
        stderr.replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>"),
        out.status.code().expect("an exit code"),
    )
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

#[test]
fn r13_the_ledgers_reach_parity() {
    assert_parity("22-ask-decision");
}

#[test]
fn r11_ask_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "ask",
        &[
            &["ask", "add"],
            &["ask", "add", "which store format"],
            &["ask", "add", "which store format", "--bogus"],
            &["ask", "add", "one | two", "--affects", "R01"],
            &["ask", "add", "which store format", "--affects", "R01|R02"],
            &["ask", "add", "which store format", "--affects="],
            &[
                "ask",
                "add",
                "which store format",
                "extra",
                "--affects",
                "R01",
            ],
            &["ask", "add", "which store format", "--affects"],
            &["ask", "answer", "Q-01"],
            &["ask", "answer", "Q-99", "x", "--decision", "y"],
            &["ask", "answer", "Q-01", "an answer", "--decision"],
            &["ask", "assume", "Q-01"],
            &["ask", "assume", "Q-99", "x", "--accept", "y"],
            &["ask", "assume", "Q-01", "a default", "--accept"],
            &["ask", "list", "--run", "nosuch"],
        ],
    );
}

#[test]
fn r11_decision_and_check_refuse_with_the_shell_wording() {
    assert_same_refusal(
        "decision",
        &[
            &["decision", "add"],
            &["decision", "add", "the ledger stays a single tsv"],
            &[
                "decision",
                "add",
                "the ledger stays a single tsv",
                "--bogus",
            ],
            &["decision", "add", "one | two", "--affects", "R01"],
            &["decision", "add", "one", "--affects", "R01|R02"],
            &["decision", "add", "one", "--affects="],
            &["decision", "add", "one", "extra", "--affects", "R01"],
            &["decision", "add", "one", "--affects"],
            &[
                "decision",
                "add",
                "one",
                "--affects",
                "design",
                "--design",
                "why | not",
            ],
            &["decision", "list", "--run", "nosuch"],
            &["check", "decisions", "--run", "nosuch"],
        ],
    );
}

/// R05: the fixture runner stays the proof that the checker can fail — bad-* must be rejected
/// and good-* must pass, driven through the same Selftest the doctor's runner drives.
#[test]
fn r05_check_decisions_judges_its_fixtures() {
    let home = Home::resolve().expect("the D-STACK repository");
    let fixtures = home.home.join("lint/fixtures/check-decisions");
    let registry = Rc::new(Registry::new(dstack_cli::verbs::all_verbs()));
    let mut ctx = Context::new(
        Home::resolve().expect("the D-STACK repository"),
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        registry,
    );
    let checker = dstack_cli::verbs::decision::selftests()
        .into_iter()
        .find(|s| s.checker() == "check-decisions")
        .expect("the check-decisions selftest is registered");

    let mut checked = 0;
    let mut names: Vec<String> = std::fs::read_dir(&fixtures)
        .expect("the fixture directory")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in &names {
        let want = match name.starts_with("bad-") {
            true => Verdict::Reject,
            false => Verdict::Pass,
        };
        let got = checker
            .run(&mut ctx, &fixtures.join(name))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(got, want, "fixture {name}");
        checked += 1;
    }
    assert!(
        checked >= 4,
        "the checker needs a bad and a good fixture for the coverage rule and for the moot rule"
    );
}

/// The run directory CURRENT names in a sandbox.
fn current_run_dir(dir: &Path) -> PathBuf {
    let id = std::fs::read_to_string(dir.join(".dstack/local/CURRENT")).expect("CURRENT");
    dir.join(".dstack/runs").join(id.trim_end_matches('\n'))
}

/// R13: a missing plan.json is "no plan", but one that cannot be read is a cannot decide — a
/// broken file must not read as a run whose decisions reach no task.
#[test]
fn r13_check_decisions_cannot_decide_on_an_unreadable_plan() {
    let dir = sandbox("brokenplan");
    let run_dir = current_run_dir(&dir);
    std::fs::write(
        run_dir.join("decisions.md"),
        "# Decisions (R51)\n\n| D | Decision | Affects | Status |\n|---|---|---|---|\n\
         | D-01 | the ledger stays a single tsv | R01 | answered |\n",
    )
    .expect("write the ledger");
    std::fs::write(run_dir.join("plan.json"), "{ not json").expect("write a broken plan");
    let (stderr, code) = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, &["check", "decisions"]);
    assert_eq!(
        code, 2,
        "an unreadable plan.json cannot be decided: {stderr}"
    );
    assert!(
        stderr.starts_with("dstack: cannot read <SANDBOX>/.dstack/runs/"),
        "{stderr}"
    );
    std::fs::remove_file(run_dir.join("plan.json")).expect("remove the broken plan");
    let (stderr, code) = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, &["check", "decisions"]);
    assert_eq!(
        code, 1,
        "a missing plan.json leaves the row uncovered: {stderr}"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// One dstack call with both streams, the sandbox path masked in each.
fn dstack_full(bin: &str, dir: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    // The run id carries the UTC second it was minted in, so two sandboxes built a second apart
    // differ in it; the harness masks the same value.
    let run_id = std::fs::read_to_string(dir.join(".dstack/local/CURRENT")).unwrap_or_default();
    let run_id = run_id.trim_end_matches('\n').to_string();
    let mask = |bytes: Vec<u8>| {
        let text = String::from_utf8(bytes)
            .expect("utf-8")
            .replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>");
        match run_id.is_empty() {
            true => text,
            false => text.replace(&run_id, "<RUN>"),
        }
    };
    (
        mask(out.stdout),
        mask(out.stderr),
        out.status.code().expect("an exit code"),
    )
}

/// A sandbox whose run already carries the request row R99 and a task covering it, written by
/// hand and byte for byte the same on both sides: what a token has to resolve to for the split
/// of the affects column to be observable at all.
fn seeded(tag: &str) -> PathBuf {
    let dir = sandbox(tag);
    let id = std::fs::read_to_string(dir.join(".dstack/local/CURRENT")).expect("CURRENT");
    let run_dir = dir.join(".dstack/runs").join(id.trim_end_matches('\n'));
    std::fs::write(
        run_dir.join("request.md"),
        "---\nwork_type: cli\n---\n# seeded request\n\n\
         - [ ] **R01** the command prints what it counted — accept: stdout carries \"checked N\"\n\
         - [ ] **R99** the command refuses bad input — accept: exit code 1 with a reason\n",
    )
    .expect("write the request");
    std::fs::write(
        run_dir.join("plan.json"),
        "{ \"v\": 2,\n  \"milestones\": [ {\"id\":\"M1\",\"slug\":\"only\",\"order\":1} ],\n\
         \x20 \"plans\": [ {\"id\":\"P1\",\"milestone\":\"M1\",\"slug\":\"only\",\"files\":[\"a.sh\"],\"deps\":[],\n\
         \x20              \"status\":\"pending\",\"worktree\":\"\",\"started_at\":\"\",\"done_at\":\"\",\n\
         \x20              \"tasks\":[ {\"id\":\"T1\",\"slug\":\"one\",\"covers\":[\"R99\"],\"files\":[\"a.sh\"],\
         \"deps\":[],\"commit\":\"\",\"done_at\":\"\"} ] } ] }\n",
    )
    .expect("write the plan");
    dir
}

/// R13: the shell splits the affects column with the default IFS after `tr ',' ' '` — ASCII
/// space, tab and newline and nothing else. `R01<NBSP>R99` is therefore one name nothing knows,
/// where a Unicode-aware split would find two known rows: no warning from ask add, and a
/// decision row that reads as covered by the task on R99.
#[test]
fn r13_affects_splits_on_ascii_whitespace_only() {
    let hard = "R01\u{a0}R99";
    let cases: &[&[&str]] = &[
        &["ask", "add", "which cap", "--affects", hard],
        &["decision", "add", "the cap stays three", "--affects", hard],
        &["check", "decisions"],
    ];
    let shell_dir = seeded("nbsp-shell");
    let rust_dir = seeded("nbsp-rust");
    for args in cases {
        let shell = dstack_full(&shell_bin(), &shell_dir, args);
        let ported = dstack_full(env!("CARGO_BIN_EXE_dstack"), &rust_dir, args);
        assert_eq!(ported, shell, "dstack {args:?}");
        // The token is one name, so it reaches neither the request row nor the task.
        assert!(
            !shell.0.contains("covered   — task"),
            "the shell reads the hard space as part of the token:\n{}",
            shell.0
        );
    }
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
}

/// A sandbox whose run carries the plain R01/R02 request, written by hand so both sides start
/// from the same bytes: `request new` belongs to another Plan's step.
fn with_request(tag: &str) -> PathBuf {
    let dir = sandbox(tag);
    std::fs::write(
        current_run_dir(&dir).join("request.md"),
        "---\nwork_type: cli\n---\n# moot request\n\n\
         - [ ] **R01** the command prints what it counted — accept: stdout carries \"checked N\"\n\
         - [ ] **R02** the command refuses bad input — accept: exit code 1 with a reason\n",
    )
    .expect("write the request");
    dir
}

/// Both implementations through the same calls, each in its own sandbox, asserting byte equality
/// on every one; the shell's captures come back for the assertions on the wording.
fn both(tag: &str, calls: &[&[&str]]) -> Vec<(String, String, i32)> {
    let shell_dir = with_request(&format!("{tag}-shell"));
    let rust_dir = with_request(&format!("{tag}-rust"));
    let mut captured = Vec::new();
    for args in calls {
        let shell = dstack_full(&shell_bin(), &shell_dir, args);
        let ported = dstack_full(env!("CARGO_BIN_EXE_dstack"), &rust_dir, args);
        assert_eq!(ported, shell, "dstack {args:?}");
        captured.push(shell);
    }
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
    captured
}

/// R14: a withdrawn row takes no task and no evidence by design, so a decision that affects only
/// withdrawn rows has nothing left to reach — it is moot, and moot is covered.
#[test]
fn r14_a_row_whose_every_r_is_marked_is_moot() {
    let calls: &[&[&str]] = &[
        &["req", "withdraw", "R02", "--why", "the interview dropped it"],
        &[
            "decision",
            "add",
            "bad input is refused with a reason on stderr",
            "--affects",
            "R02",
        ],
        &["check", "decisions"],
    ];
    let captured = both("moot", calls);
    let (stdout, _stderr, code) = &captured[2];
    assert_eq!(*code, 0, "the moot row is covered:\n{stdout}");
    assert!(
        stdout.contains("D-01           covered   — moot — every affected row is marked: R02 (withdrawn)\n"),
        "{stdout}"
    );
}

/// R14: one live R id that is neither tasked nor evidenced still leaves the row UNCOVERED — the
/// moot rule forgives the marked ids, never the row.
#[test]
fn r14_a_live_id_keeps_a_mixed_row_uncovered() {
    let calls: &[&[&str]] = &[
        &["req", "withdraw", "R02", "--why", "the interview dropped it"],
        &[
            "decision",
            "add",
            "retries use a fixed backoff of two seconds",
            "--affects",
            "R02,R07",
        ],
        &["check", "decisions"],
    ];
    let captured = both("mixed", calls);
    let (stdout, _stderr, code) = &captured[2];
    assert_eq!(*code, 1, "the live R07 is uncovered:\n{stdout}");
    assert!(
        stdout.contains("D-01           UNCOVERED — no task and no evidence on: R02,R07\n"),
        "{stdout}"
    );
}
