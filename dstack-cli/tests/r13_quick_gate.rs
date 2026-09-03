// tests/r13_quick_gate.rs
// R13/R11/R04/R05: dstack quick new|list|status|resume|close and the Stop gate answer exactly as
// the shell answers — through the parity harness, through the refusal wording R11 pins, and
// through the fixture runner of the one checker the quick track owns.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::verbs::{self, quick};

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
    let dir = std::env::temp_dir().join(format!("dstack-p13-{tag}-{}", std::process::id()));
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

/// One dstack call: its stdout and stderr with the sandbox path and the run id masked, and its
/// exit code. An unreadable CURRENT leaves no run id to mask, and an empty pattern would be
/// replaced between every character, so the id is only masked when there is one.
fn dstack_streams(bin: &str, dir: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    let id = run_id(dir);
    let mask = |text: Vec<u8>| {
        let text = String::from_utf8(text)
            .expect("utf-8")
            .replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>");
        match id.is_empty() {
            true => text,
            false => text.replace(&id, "<RUNID>"),
        }
    };
    (
        mask(out.stdout),
        mask(out.stderr),
        out.status.code().expect("an exit code"),
    )
}

/// The refusal of one call: its masked stderr and its exit code.
fn dstack(bin: &str, dir: &Path, args: &[&str]) -> (String, i32) {
    let (_, stderr, code) = dstack_streams(bin, dir, args);
    (stderr, code)
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
    let selftests = quick::selftests();
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
fn r13_the_quick_and_gate_verbs_reach_parity() {
    assert_parity("33-quick-gate");
}

#[test]
fn r11_quick_refuses_with_the_shell_wording() {
    assert_same_refusal(
        "quick",
        &[
            &["quick"],
            &["quick", "bogus"],
            &["quick", "new"],
            &["quick", "new", "--bogus"],
            &["quick", "new", "one", "two"],
            &["quick", "new", "Bad_Slug"],
            &["quick", "new", "-lead"],
            &["quick", "new", "okslug", "--type", "bogus"],
            &["quick", "new", "okslug", "--type"],
            &["quick", "status"],
            &["quick", "status", "nosuch"],
            &["quick", "resume"],
            &["quick", "resume", "nosuch"],
            &["quick", "close"],
            &["quick", "close", "nosuch"],
            &["quick", "close", "--bogus"],
            &["quick", "close", "one", "two"],
            &["quick", "close", "nosuch", "--abandon"],
        ],
    );
}

#[test]
fn r05_the_quick_new_checker_judges_its_fixtures() {
    assert_fixtures("quick-new");
}

#[test]
fn r11_the_gate_reads_no_argument_at_all() {
    assert_same_refusal("gate", &[&["gate", "--bogus"], &["gate", "extra"]]);
}

/// D-12 in the one place R101 cares about: a store file that is there and cannot be read leaves
/// the gate unable to compute, and a gate that cannot compute blocks (exit 2) instead of passing
/// the turn. The shell reads such a file as empty, so this contract is the port's alone.
#[test]
fn r13_the_gate_blocks_when_a_quick_request_cannot_be_read() {
    let dir = sandbox("gate-unreadable");
    let shell = shell_bin();
    dstack(&shell, &dir, &["quick", "new", "unreadable"]);
    dstack(
        &shell,
        &dir,
        &[
            "req",
            "add",
            "the row",
            "--accept",
            "the criterion",
            "--quick",
            "unreadable",
        ],
    );
    dstack(
        &shell,
        &dir,
        &["request", "approve", "--quick", "unreadable"],
    );
    let request = dir.join(".dstack/quick/unreadable/request.md");
    let locked = std::fs::Permissions::from_mode(0o000);
    std::fs::set_permissions(&request, locked).expect("take the read bit away");
    let (stderr, code) = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, &["gate"]);
    std::fs::set_permissions(&request, std::fs::Permissions::from_mode(0o644))
        .expect("put the read bit back");
    assert_eq!(code, 2, "the gate blocks instead of passing: {stderr}");
    assert!(
        stderr.starts_with(
            "dstack: quick unreadable: check coverage cannot decide: dstack: cannot read \
             <SANDBOX>/.dstack/quick/unreadable/request.md"
        ),
        "{stderr}"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// D-12 where R101 needs it most: CURRENT and the run's meta.tsv are what the gate reads to find
/// out whether an open run exists at all. current_run_id() and meta_get() both answer "absent"
/// for a file that cannot be read, and a gate that took that answer would clear a worktree whose
/// open run it never saw. Both reads therefore end the command with exit 2 and nothing on stdout.
fn assert_gate_blocks_on(dir: &Path, store_file: &str) {
    let path = dir.join(store_file);
    // The path in the message carries the run id, which dstack_streams masks; the expected one
    // is masked the same way, and the id is read while the file behind it is still readable.
    let id = run_id(dir);
    let shown = match id.is_empty() {
        true => store_file.to_string(),
        false => store_file.replace(&id, "<RUNID>"),
    };
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000))
        .expect("take the read bit away");
    let (stdout, stderr, code) = dstack_streams(env!("CARGO_BIN_EXE_dstack"), dir, &["gate"]);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
        .expect("put the read bit back");
    assert_eq!(code, 2, "the gate blocks instead of clearing: {stderr}");
    assert_eq!(stdout, "", "a blocked gate says nothing on stdout");
    assert!(
        stderr.starts_with(&format!("dstack: cannot read <SANDBOX>/{shown}")),
        "{stderr}"
    );
}

#[test]
fn r13_the_gate_blocks_when_current_cannot_be_read() {
    let dir = sandbox("gate-current");
    assert_gate_blocks_on(&dir, ".dstack/local/CURRENT");
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r13_the_gate_blocks_when_the_run_meta_cannot_be_read() {
    let dir = sandbox("gate-meta");
    let meta = format!(".dstack/runs/{}/meta.tsv", run_id(&dir));
    assert_gate_blocks_on(&dir, &meta);
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// The read-before-write rule (P12.2) on the one verb of this Plan that creates a directory: a
/// state table that cannot be read stops `quick new` before anything exists, so the retry after
/// the file is readable again is not refused as an already-open task.
#[test]
fn r13_quick_new_leaves_nothing_behind_when_the_state_table_cannot_be_read() {
    let dir = sandbox("quick-new-partial");
    let shell = shell_bin();
    dstack(&shell, &dir, &["quick", "new", "first"]);
    let table = dir.join(".dstack/quick/STATE.md");
    std::fs::set_permissions(&table, std::fs::Permissions::from_mode(0o000))
        .expect("take the read bit away");
    let (_, stderr, code) = dstack_streams(
        env!("CARGO_BIN_EXE_dstack"),
        &dir,
        &["quick", "new", "second"],
    );
    std::fs::set_permissions(&table, std::fs::Permissions::from_mode(0o644))
        .expect("put the read bit back");
    assert_eq!(code, 2, "an unreadable table is a cannot-decide: {stderr}");
    assert!(
        stderr.starts_with("dstack: cannot read <SANDBOX>/.dstack/quick/STATE.md"),
        "{stderr}"
    );
    assert!(
        !dir.join(".dstack/quick/second").exists(),
        "nothing of the refused task was created"
    );
    // With the table readable again the same call opens the task, so nothing was left half done.
    let (_, code) = dstack(
        env!("CARGO_BIN_EXE_dstack"),
        &dir,
        &["quick", "new", "second"],
    );
    assert_eq!(code, 0, "the retry opens the task");
    assert!(dir.join(".dstack/quick/second/request.md").is_file());
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// D-10 for the positional slug of the quick verbs: the shell joins it into `$QUICK` unchecked,
/// so an absolute slug replaces the quick root and `.` or `..` reads a directory outside the
/// quick tree and reports success. The port refuses anything that is not a plain name with the
/// wording resolve_target already uses for `--run` and `--quick`, before it touches the
/// filesystem — a documented divergence, declared in step 33, not a wording change.
#[test]
fn r11_a_path_like_quick_slug_is_refused_the_way_a_run_id_is() {
    let dir = sandbox("quick-slug");
    let table = dir.join(".dstack/quick/STATE.md");
    dstack(&shell_bin(), &dir, &["quick", "new", "real"]);
    let before = std::fs::read(&table).expect("the state table");
    for slug in ["/tmp", "../x", "a/b", ".", "..", "/"] {
        for verb in ["status", "resume", "close"] {
            let (stdout, stderr, code) =
                dstack_streams(env!("CARGO_BIN_EXE_dstack"), &dir, &["quick", verb, slug]);
            assert_eq!(code, 1, "dstack quick {verb} {slug}");
            assert_eq!(
                stdout, "",
                "dstack quick {verb} {slug} says nothing on stdout"
            );
            assert_eq!(
                stderr,
                format!("dstack: quick slug must be a plain name (got '{slug}')\n"),
                "dstack quick {verb} {slug}"
            );
        }
        // --abandon skips the report, so it is the one path that would have written a row.
        let (_, stderr, code) = dstack_streams(
            env!("CARGO_BIN_EXE_dstack"),
            &dir,
            &["quick", "close", slug, "--abandon", "nothing to abandon"],
        );
        assert_eq!(code, 1, "dstack quick close {slug} --abandon");
        assert_eq!(
            stderr,
            format!("dstack: quick slug must be a plain name (got '{slug}')\n")
        );
    }
    assert_eq!(
        std::fs::read(&table).expect("the state table"),
        before,
        "no refused slug rewrote the state table"
    );
    assert!(
        !std::path::Path::new("/tmp/.dstack").exists(),
        "nothing was written outside the store"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}
