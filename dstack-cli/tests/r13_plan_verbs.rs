// tests/r13_plan_verbs.rs
// R13/R11/R04/R05: milestone, plan, task and next answer exactly as the shell answers — through
// the parity harness, through the refusal wording R11 pins, and through their two checkers.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::verbs::{self, plan};

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
    let dir = std::env::temp_dir().join(format!("dstack-p10-{tag}-{}", std::process::id()));
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

/// One dstack call: both streams with the sandbox path and the run id masked, and its exit code.
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
            .replace(&run_id(dir), "<RUNID>")
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

/// The fixture runner of one checker, the way doctor --self drives it: every bad-* fixture has
/// to be rejected and every good-* fixture has to pass.
fn assert_fixtures(checker: &str) {
    let home = Home::resolve().expect("the repository");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let selftests = plan::selftests();
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
fn r13_the_roadmap_verbs_reach_parity() {
    assert_parity("30-plan");
}

/// The refusals a caller can provoke with no plan.json at all: the wording skills and hooks
/// quote is the shell's, down to the usage line and the exit code.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_the_roadmap_verbs_refuse_with_the_shell_wording() {
    assert_same(
        "usage",
        &[
            &["milestone", "add"],
            &["milestone", "add", "Bad_Slug"],
            &["milestone", "add", "core", "--bogus"],
            &["plan", "add"],
            &["plan", "add", "first", "--milestone", "M1"],
            &["plan", "add", "first", "--nope", "x"],
            &["plan", "insert", "first", "--milestone", "M1"],
            &["plan", "remove"],
            &["plan", "remove", "P1"],
            &["plan", "edit"],
            &["plan", "edit", "P1", "--slug", "other"],
            &["plan", "render"],
            &["plan", "start"],
            &["plan", "start", "P1"],
            &["plan", "done"],
            &["plan", "done", "P1"],
            &["task", "add"],
            &["task", "add", "write", "--plan", "P1"],
            &["task", "done"],
            &["task", "done", "T1", "--commit", "HEAD"],
            &["next"],
            &["next", "--max", "0"],
            &["next", "--bogus"],
        ],
    );
}

/// A digit-only --max that i64 cannot hold is where bash's own `[ "$max" -ge 1 ]` fails, so the
/// shell lands on the "at least 1" refusal — plus one diagnostic line of its own naming
/// the reference's next.sh, which D-11 says the port does not reproduce.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_an_overflowing_max_refuses_the_way_the_shell_refuses() {
    let shell_dir = sandbox("overflow-shell");
    let rust_dir = sandbox("overflow-rust");
    // next reads plan.json before it parses its options, so the run needs a milestone first.
    for dir in [&shell_dir, &rust_dir] {
        assert_eq!(
            dstack(&shell_bin(), dir, &["milestone", "add", "core"]).2,
            0
        );
    }
    for value in ["9223372036854775808", "99999999999999999999"] {
        let args = ["next", "--max", value];
        let shell = dstack(&shell_bin(), &shell_dir, &args);
        let ported = dstack(env!("CARGO_BIN_EXE_dstack"), &rust_dir, &args);
        assert_eq!(
            ported,
            (
                String::new(),
                "dstack: --max must be at least 1\n".to_string(),
                1
            ),
            "dstack next --max {value}"
        );
        assert_eq!(
            (shell.0.as_str(), shell.2),
            ("", 1),
            "the shell refuses {value}"
        );
        assert!(
            shell.1.ends_with("dstack: --max must be at least 1\n"),
            "the shell's own refusal line: {}",
            shell.1
        );
        assert!(
            shell.1.contains("integer expression expected"),
            "the bash diagnostic is the only line the port drops: {}",
            shell.1
        );
    }
    // The largest value the test builtin can hold is still a cap on both sides.
    let args = ["next", "--max", "9223372036854775807"];
    assert_eq!(
        dstack(env!("CARGO_BIN_EXE_dstack"), &rust_dir, &args),
        dstack(&shell_bin(), &shell_dir, &args)
    );
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
}

/// A --worktree path that exists but is not a directory: the shell's `cd "$wt"` fails and
/// `set -e` ends the run with cd's status, so the plan never starts and plan.json is untouched.
/// bash prints one diagnostic of its own naming the reference's plan.sh, which D-11 does not reproduce.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11_a_worktree_that_is_not_a_directory_starts_nothing() {
    let shell_dir = sandbox("wtfile-shell");
    let rust_dir = sandbox("wtfile-rust");
    let mut before = Vec::new();
    for dir in [&shell_dir, &rust_dir] {
        assert_eq!(
            dstack(&shell_bin(), dir, &["milestone", "add", "core"]).2,
            0
        );
        assert_eq!(
            dstack(
                &shell_bin(),
                dir,
                &[
                    "plan",
                    "add",
                    "first",
                    "--milestone",
                    "M1",
                    "--files",
                    "src/a.rs"
                ]
            )
            .2,
            0
        );
        std::fs::write(dir.join("afile"), "a regular file, not a directory\n").expect("the file");
        before.push(std::fs::read(plan_json(dir)).expect("plan.json"));
    }
    let shell = dstack(
        &shell_bin(),
        &shell_dir,
        &[
            "plan",
            "start",
            "P1",
            "--worktree",
            &path_of(&shell_dir, "afile"),
        ],
    );
    let ported = dstack(
        env!("CARGO_BIN_EXE_dstack"),
        &rust_dir,
        &[
            "plan",
            "start",
            "P1",
            "--worktree",
            &path_of(&rust_dir, "afile"),
        ],
    );
    assert_eq!(
        ported.2, shell.2,
        "the exit code of a non-directory worktree"
    );
    assert_eq!(ported.2, 1, "cd's status ends the run");
    assert_eq!(ported.0, "", "the port prints no stdout");
    assert_eq!(
        ported.1, "",
        "the port prints nothing where bash printed its own diagnostic"
    );
    assert_eq!(shell.0, "", "the shell prints no stdout either");
    assert!(
        shell.1.contains("cd:") && shell.1.contains("Not a directory"),
        "the bash diagnostic is the only line the port drops: {}",
        shell.1
    );
    for (dir, was) in [&shell_dir, &rust_dir].iter().zip(before) {
        assert_eq!(
            std::fs::read(plan_json(dir)).expect("plan.json"),
            was,
            "nothing of the store was written in {dir:?}"
        );
    }
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
}

/// The plan.json of the sandbox's current run.
fn plan_json(dir: &Path) -> PathBuf {
    dir.join(".dstack/runs").join(run_id(dir)).join("plan.json")
}

fn path_of(dir: &Path, name: &str) -> String {
    dir.join(name).to_string_lossy().into_owned()
}

/// `_csv_list` turns every comma into a newline and then reads one item per line, so a literal
/// newline in a list option separates items exactly as a comma does — in every verb that takes
/// one, and in the plan.json those verbs write.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r13_a_newline_separates_a_list_option_the_way_a_comma_does() {
    let shell_dir = sandbox("csv-shell");
    let rust_dir = sandbox("csv-rust");
    for dir in [&shell_dir, &rust_dir] {
        for args in [
            &["request", "new", "--type", "cli", "--title", "csv"][..],
            &[
                "req",
                "add",
                "the first row",
                "--accept",
                "the first criterion",
            ][..],
            &[
                "req",
                "add",
                "the second row",
                "--accept",
                "the second criterion",
            ][..],
            &[
                "req",
                "add",
                "the third row",
                "--accept",
                "the third criterion",
            ][..],
            &["request", "approve"][..],
            &["milestone", "add", "core"][..],
            &[
                "plan",
                "add",
                "first",
                "--milestone",
                "M1",
                "--files",
                "src/a.rs,src/b.rs",
            ][..],
        ] {
            assert_eq!(dstack(&shell_bin(), dir, args).2, 0, "seeding {args:?}");
        }
    }
    let calls: &[&[&str]] = &[
        &[
            "task", "add", "one", "--plan", "P1", "--covers", "R01\nR03", "--files", "src/a.rs",
        ],
        &[
            "task",
            "add",
            "two",
            "--plan",
            "P1",
            "--covers",
            "R01",
            "--files",
            "src/a.rs\nsrc/b.rs",
        ],
        &[
            "task",
            "add",
            "three",
            "--plan",
            "P1",
            "--covers",
            "R01,,R03",
            "--files",
            "src/a.rs,",
        ],
        &[
            "task",
            "add",
            "four",
            "--plan",
            "P1",
            "--covers",
            "R01\n  R03  ",
            "--files",
            "src/a.rs\n",
        ],
        &[
            "plan",
            "add",
            "second",
            "--milestone",
            "M1",
            "--files",
            "src/c.rs\nsrc/d.rs",
        ],
        &[
            "plan",
            "add",
            "third",
            "--milestone",
            "M1",
            "--files",
            "src/e.rs",
            "--deps",
            "P1\nP2",
        ],
        &["plan", "edit", "P2", "--files", "src/c.rs\nsrc/f.rs"],
        &[
            "plan",
            "add",
            "empty",
            "--milestone",
            "M1",
            "--files",
            "\n  \n",
        ],
    ];
    for args in calls {
        let shell = dstack(&shell_bin(), &shell_dir, args);
        let ported = dstack(env!("CARGO_BIN_EXE_dstack"), &rust_dir, args);
        assert_eq!(ported, shell, "dstack {args:?}");
        assert_eq!(
            std::fs::read(plan_json(&rust_dir)).expect("plan.json"),
            std::fs::read(plan_json(&shell_dir)).expect("plan.json"),
            "the plan.json the two wrote after {args:?}"
        );
    }
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
}

#[test]
fn r05_the_plan_add_and_task_add_checkers_judge_their_fixtures() {
    assert_fixtures("plan-add");
    assert_fixtures("task-add");
}
