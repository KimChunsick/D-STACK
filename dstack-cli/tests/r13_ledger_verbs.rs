// tests/r13_ledger_verbs.rs
// R13/R11/R04/R05: cases, evidence, check coverage and worker report answer exactly as the shell
// answers — through the parity harness, through the refusal wording R11 pins, and through the
// fixture runner of the four checkers they own.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::verbs::{self, ledger};

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

/// A repository with a store and one open run, built by the reference dispatcher the way the
/// harness builds its sandboxes.
fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p8-{tag}-{}", std::process::id()));
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

/// The fixture runner of one checker, the way doctor --self will drive it: every bad-* fixture
/// has to be rejected and every good-* fixture has to pass.
fn assert_fixtures(checker: &str) {
    let home = Home::resolve().expect("the repository");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let selftests = ledger::selftests();
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
fn r13_the_ledger_verbs_reach_parity() {
    assert_parity("23-ledger");
}

#[test]
fn r11_cases_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "cases",
        &[
            &["cases", "sync", "--bogus"],
            &["cases", "sync", "nosuch"],
            &["cases", "sync"],
            &["cases", "render", "extra"],
            &["cases", "render", "--bogus"],
            &["cases", "render"],
        ],
    );
}

#[test]
fn r11_evidence_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "evidence",
        &[
            &["evidence", "add", "--bogus"],
            &["evidence", "add", "--r", "R01"],
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
                "nosuch.txt",
                "--produced-by",
                "cmd",
            ],
            &["evidence", "retire", "--bogus"],
            &["evidence", "retire", "--r", "R01"],
            &[
                "evidence",
                "retire",
                "--r",
                "R01",
                "--case",
                "c1",
                "--why",
                "no ledger",
            ],
        ],
    );
}

#[test]
fn r05_the_evidence_checkers_judge_their_fixtures() {
    assert_fixtures("evidence-add");
    assert_fixtures("evidence-retire");
}

/// A request with one live row, approved the way `request approve` approves it (cases sync only
/// asks whether the stamp file is there), so the ledger can be filled by the port alone.
fn approved_run(dir: &Path) -> PathBuf {
    let run_dir = dir.join(".dstack/runs").join(run_id(dir));
    std::fs::write(
        run_dir.join("request.md"),
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: skip\nreview: off\ncodex_effort: high\ne2e: cli\nunit_tests: off\n\
         visual: none\nkorean_polish: off\n---\n# injection\n\n\
         - [ ] **R01** the ledger keeps one row per case — accept: cases.tsv has one row\n",
    )
    .expect("write the request");
    std::fs::write(
        run_dir.join("request.approved"),
        "sha256 -  approved_at -\n",
    )
    .expect("write the approval");
    run_dir
}

/// D-09: the shell writes the case id raw, so a tab or a newline in it invents a column or a
/// whole row. The port cleans it, and the ledger keeps exactly one row per recorded case.
#[test]
fn r13_a_case_id_with_a_tab_or_a_newline_writes_one_row() {
    let dir = sandbox("injection");
    let run_dir = approved_run(&dir);
    let port = env!("CARGO_BIN_EXE_dstack");
    assert_eq!(dstack(port, &dir, &["cases", "sync"]).1, 0);
    std::fs::write(dir.join("proof.md"), "the reviewer read it\n").expect("the artifact");
    for (case_id, rows) in [("c1\tc2", 2), ("c3\nc4", 3)] {
        let (stderr, code) = dstack(
            port,
            &dir,
            &[
                "evidence",
                "add",
                "--r",
                "R01",
                "--case",
                case_id,
                "--kind",
                "review",
                "--artifact",
                "proof.md",
                "--produced-by",
                "the reviewer",
                "--shared",
                "one note for both",
            ],
        );
        assert_eq!(code, 0, "{stderr}");
        let ledger = std::fs::read_to_string(run_dir.join("cases.tsv")).expect("the ledger");
        assert_eq!(
            ledger.lines().count(),
            rows + 1,
            "the header plus {rows} row(s):\n{ledger}"
        );
        for line in ledger.lines() {
            assert_eq!(line.split('\t').count(), 9, "a row grew a column: {line}");
        }
    }
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// D-12: a plan.json that cannot be parsed is a store the command cannot decide on, so
/// check coverage exits 2 with a reason instead of counting zero covering tasks and exiting 1.
#[test]
fn r13_a_plan_json_that_cannot_be_read_cannot_be_decided() {
    let dir = sandbox("badplan");
    let run_dir = approved_run(&dir);
    let port = env!("CARGO_BIN_EXE_dstack");
    assert_eq!(dstack(port, &dir, &["cases", "sync"]).1, 0);
    std::fs::write(run_dir.join("plan.json"), "{ \"plans\": [ oops").expect("a broken plan");
    let (stderr, code) = dstack(port, &dir, &["check", "coverage"]);
    assert_eq!(code, 2, "{stderr}");
    assert!(
        stderr.starts_with("dstack: cannot read ") && stderr.contains("plan.json"),
        "unexpected refusal: {stderr}"
    );
    let (stderr, code) = dstack(
        port,
        &dir,
        &["worker", "report", "--plan", "P1", "--from", ".deps.tsv"],
    );
    assert_eq!(code, 2, "{stderr}");
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// D-09 again, on the other cell `evidence add` writes without cleaning: the path itself.
#[test]
fn r13_an_artifact_path_with_a_tab_or_a_newline_is_refused() {
    let dir = sandbox("badpath");
    approved_run(&dir);
    let port = env!("CARGO_BIN_EXE_dstack");
    assert_eq!(dstack(port, &dir, &["cases", "sync"]).1, 0);
    for path in ["proof\tR02\tc1.md", "proof\nR02.md"] {
        let (stderr, code) = dstack(
            port,
            &dir,
            &[
                "evidence",
                "add",
                "--r",
                "R01",
                "--case",
                "c1",
                "--kind",
                "review",
                "--artifact",
                path,
                "--produced-by",
                "the reviewer",
            ],
        );
        assert_eq!(code, 1, "{stderr}");
        assert_eq!(
            stderr, "dstack: artifact path must not contain tabs or newlines\n",
            "{path:?}"
        );
    }
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r11_coverage_and_worker_refuse_with_the_shell_wording() {
    assert_same_refusal(
        "coverage",
        &[
            &["check", "coverage", "--bogus"],
            &["check", "coverage", "extra"],
            &["check", "coverage"],
            &["check", "coverage", "--run", "nosuch"],
            &["worker", "report", "--bogus"],
            &["worker", "report", "--plan", "P1"],
            &["worker", "report", "--plan", "P1", "--from", "nosuch.txt"],
        ],
    );
}

#[test]
fn r05_the_coverage_and_worker_checkers_judge_their_fixtures() {
    assert_fixtures("check-coverage");
    assert_fixtures("worker-report");
}
