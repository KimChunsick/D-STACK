// tests/r13_verify_report.rs
// R13/R11/R04/R05: verify and report answer exactly as the shell answers — through the parity
// harness, through the refusal wording R11 pins, and through the fixture runner of the verify
// checker.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::verbs::{self, verify};

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
        .args(["--shell-ref", "shell-final"])
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
    let dir = std::env::temp_dir().join(format!("dstack-p12-{tag}-{}", std::process::id()));
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

/// The fixture runner of one checker, the way doctor --self drives it: every bad-* fixture has
/// to be rejected and every good-* fixture has to pass.
fn assert_fixtures(checker: &str) {
    let home = Home::resolve().expect("the repository");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let selftests = verify::selftests();
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
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r13_verify_and_report_reach_parity() {
    assert_parity("32-verify-report");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_verify_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "verify",
        &[
            &["verify", "--bogus"],
            &["verify", "extra"],
            &["verify", "--run", "nosuch"],
            &["verify", "--quick", "nosuch"],
            &["verify"],
            &["verify", "--accept-abstain", "R01"],
            &["verify", "--accept-abstain"],
            &["verify", "--why"],
        ],
    );
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_report_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "report",
        &[
            &["report", "--bogus"],
            &["report", "extra"],
            &["report", "--run", "nosuch"],
            &["report", "--quick", "nosuch"],
            &["report"],
            &["report", "--metrics"],
        ],
    );
}

/// A request the report can read: report needs the file, not the ledger.
fn write_request(run_dir: &Path) {
    std::fs::write(
        run_dir.join("request.md"),
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: skip\nreview: off\ncodex_effort: high\ne2e: cli\nunit_tests: off\n\
         visual: none\nkorean_polish: off\n---\n# the transcript request\n\n\
         - [ ] **R01** the metrics name what they read — accept: the table says which file\n",
    )
    .expect("write the request");
    std::fs::write(
        run_dir.join("request.approved"),
        "sha256 -  approved_at 2026-01-01T00:00:00Z\n",
    )
    .expect("write the approval");
}

/// meta_set() from the outside: the Stop hook writes transcript_path, and this test has no hook.
fn meta_set(run_dir: &Path, key: &str, value: &str) {
    let path = run_dir.join("meta.tsv");
    let text = std::fs::read_to_string(&path).expect("the meta table");
    let mut out = String::new();
    for line in text.lines() {
        match line.split('\t').next() == Some(key) {
            true => out.push_str(&format!("{key}\t{value}\n")),
            false => out.push_str(&format!("{line}\n")),
        }
    }
    std::fs::write(&path, out).expect("write the meta table");
}

/// One call's stdout, with the sandbox path and the run id masked.
fn stdout_of(bin: &str, dir: &Path, args: &[&str]) -> String {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    String::from_utf8(out.stdout)
        .expect("utf-8")
        .replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>")
        .replace(&run_id(dir), "<RUNID>")
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the directory of the file");
    }
    std::fs::write(path, text).expect("write");
}

/// R01: the token sums walk the transcript subtree the way `find` walks it — following no
/// symlink. A link planted under `subagents` must not pull an unrelated tree into the sum, and a
/// cycle must not hang the walk; both implementations have to answer the same numbers.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r01_a_symlinked_transcript_tree_sums_as_find_sums() {
    let dir = sandbox("symlink");
    let run = run_id(&dir);
    let run_dir = dir.join(".dstack/runs").join(&run);
    write_request(&run_dir);

    let outside = dir.join("outside");
    write(&outside.join("leak.jsonl"), "{\"message\":{\"usage\":{\"input_tokens\":9000}}}\n");
    write(&outside.join("deep/deeper.jsonl"), "{\"message\":{\"usage\":{\"input_tokens\":900}}}\n");
    let transcript = dir.join("transcript/session.jsonl");
    write(&transcript, "{\"message\":{\"usage\":{\"input_tokens\":10}}}\n");
    let subagents = dir.join("transcript/session/subagents");
    write(&subagents.join("real.jsonl"), "{\"message\":{\"usage\":{\"input_tokens\":7,\"output_tokens\":3}}}\n");
    std::os::unix::fs::symlink(&outside, subagents.join("link-dir")).expect("a directory symlink out of the subtree");
    std::os::unix::fs::symlink(&subagents, subagents.join("loop")).expect("a cyclic symlink");
    std::os::unix::fs::symlink(outside.join("leak.jsonl"), subagents.join("link.jsonl"))
        .expect("a symlink to a transcript outside the subtree");
    meta_set(&run_dir, "transcript_path", &transcript.to_string_lossy());
    // The wall clock of an open run is read at the moment of the call, so the two calls below
    // would straddle a second under load; the stamps are pinned and every metric is a constant.
    meta_set(&run_dir, "started_at", "2026-01-01T00:00:00Z");
    meta_set(&run_dir, "closed_at", "2026-01-01T01:02:03Z");

    let args = ["report", "--metrics", "--run", run.as_str()];
    let shell = stdout_of(&shell_bin(), &dir, &args);
    let ported = stdout_of(env!("CARGO_BIN_EXE_dstack"), &dir, &args);
    assert_eq!(ported, shell, "a planted symlink is counted differently");
    assert!(
        ported.contains("| subagent-tokens | 10 | ") && ported.contains("(1 file(s))"),
        "only the real transcript is summed:\n{ported}"
    );

    // The whole directory as a symlink: `test -d` follows it, so the row reads present, and find
    // never descends a symlinked start point, so it holds nothing to sum.
    std::fs::remove_dir_all(&subagents).expect("drop the real directory");
    std::os::unix::fs::symlink(&outside, &subagents).expect("a symlinked subagents directory");
    let shell = stdout_of(&shell_bin(), &dir, &args);
    let ported = stdout_of(env!("CARGO_BIN_EXE_dstack"), &dir, &args);
    assert_eq!(ported, shell, "a symlinked subagents directory is read differently");
    assert!(
        ported.contains("| subagent-tokens | 0 | ") && ported.contains("(no transcripts)"),
        "a symlinked directory holds no transcript find would list:\n{ported}"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r05_the_verify_checker_judges_its_fixtures() {
    assert_fixtures("verify");
}
