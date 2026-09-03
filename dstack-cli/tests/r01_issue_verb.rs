// tests/r01_issue_verb.rs
// R01/R02/R03/R04: dstack issue new writes one file per issue into $HOME/Documents/dstack-issues,
// a repeated title appends a sighting instead of a second file, an issue missing what makes it
// actionable is refused, and dstack issue list prints what has been filed.
//
// Every call runs against a scratch $HOME the way tests/r08_install.rs does (D-05): the folder is
// literally fixed at $HOME/Documents/dstack-issues, so the real ~/Documents is never written to.
#![allow(non_snake_case)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The two-tool table the sandbox reads through DSTACK_DEPS, so no machine-wide deps.tsv is read.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
                    jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

const TITLE: &str = "plan start refuses a file worktree";
const SLUG: &str = "plan-start-refuses-a-file-worktree";

/// A scratch $HOME. The issue folder is $HOME/Documents/dstack-issues and nothing else here is
/// read, so the whole test writes inside this directory.
fn scratch_home(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p1-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    std::fs::canonicalize(&dir).expect("the physical path of the scratch directory")
}

fn folder(home: &Path) -> PathBuf {
    home.join("Documents/dstack-issues")
}

/// One dstack call with a scratch $HOME → stdout, stderr, exit code.
fn dstack(home: &Path, cwd: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("DSTACK_DEPS", home.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "issue-test")
        .output()
        .expect("run dstack");
    (
        String::from_utf8(out.stdout).expect("utf-8"),
        String::from_utf8(out.stderr).expect("utf-8"),
        out.status.code().expect("an exit code"),
    )
}

/// dstack issue new with the three required fields filled and the title as its operand.
fn file_issue(home: &Path, cwd: &Path, title: &str) -> (String, String, i32) {
    dstack(
        home,
        cwd,
        &[
            "issue",
            "new",
            title,
            "--symptom",
            "dstack plan start --worktree <path> exits 1 and prints nothing",
            "--repro",
            "dstack plan start P4 --worktree ./notes.txt",
            "--source",
            "dstack-cli/src/verbs/plan/lifecycle.rs",
        ],
    )
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The files of the issue folder, in name order.
fn listing(home: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder(home))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// A git repository with a store and no run yet, at <home>/sandbox.
fn store(home: &Path) -> PathBuf {
    let dir = home.join("sandbox");
    std::fs::create_dir_all(&dir).expect("sandbox directory");
    std::fs::write(home.join(".deps.tsv"), DEPS).expect("write the deps table");
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
    ok(home, &dir, &["init"]);
    dir
}

/// One dstack call that has to succeed for the scenario to mean anything.
fn ok(home: &Path, cwd: &Path, args: &[&str]) {
    let (_out, err, code) = dstack(home, cwd, args);
    assert_eq!(code, 0, "dstack {args:?} in {}: {err}", cwd.display());
}

/// A run with a milestone and `plans` plans added, all still ready.
fn run_with_plans(home: &Path, work: &Path, slug: &str, plans: &[&str]) -> String {
    ok(home, work, &["run", "new", slug, "--type", "cli"]);
    ok(home, work, &["milestone", "add", "core"]);
    for (n, plan) in plans.iter().enumerate() {
        let files = format!("src/{n}.rs");
        ok(
            home,
            work,
            &["plan", "add", plan, "--milestone", "M1", "--files", &files],
        );
    }
    current_run(work)
}

fn current_run(work: &Path) -> String {
    read(&work.join(".dstack/local/CURRENT"))
        .trim_end()
        .to_string()
}

/// A repository with a store, an open run and TWO plans in progress, each in a worktree of its
/// own — the shape R30 allows, and the one that makes "which plan filed this" a real question. The
/// triple is (the checkout that opened the run, P1's worktree, P2's worktree).
fn sandbox(home: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let dir = store(home);
    run_with_plans(home, &dir, "sandbox", &["first", "second"]);
    ok(
        home,
        &dir,
        &["plan", "start", "P1", "--worktree", "../wt-P1"],
    );
    ok(
        home,
        &dir,
        &["plan", "start", "P2", "--worktree", "../wt-P2"],
    );
    (dir, home.join("wt-P1"), home.join("wt-P2"))
}

fn git(dir: &Path, args: &[&str]) {
    let done = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(done.status.success(), "git {args:?} failed in {dir:?}");
}

#[test]
fn r01__a_new_issue_is_one_file_named_after_the_slug_of_its_title() {
    let home = scratch_home("new");
    let (out, err, code) = file_issue(&home, &home, TITLE);
    assert_eq!(code, 0, "{err}");
    assert_eq!(err, "");

    let file = folder(&home).join(format!("{SLUG}.md"));
    assert!(
        out.contains(&file.display().to_string()),
        "the path it wrote is not on stdout: {out}"
    );
    assert!(out.contains("sighting 1"), "{out}");
    assert_eq!(listing(&home), vec![format!("{SLUG}.md")]);

    let text = read(&file);
    assert!(
        text.starts_with(&format!("---\ntitle: {TITLE}\nfirst_seen: 20")),
        "the frontmatter does not open with the title and first_seen:\n{text}"
    );
    assert!(text.contains("\nsightings: 1\n---\n"), "{text}");
    for section in ["## Symptom", "## Reproduction", "## Source", "## Sightings"] {
        assert!(
            text.contains(&format!("\n{section}\n")),
            "no {section}:\n{text}"
        );
    }
    assert!(
        !text.contains("## Proposal"),
        "an issue filed without --proposal has no Proposal section:\n{text}"
    );
    assert!(
        text.contains("\ndstack plan start P4 --worktree ./notes.txt\n"),
        "{text}"
    );
    assert_eq!(
        text.lines().filter(|line| line.starts_with("- 20")).count(),
        1,
        "one sighting line:\n{text}"
    );
}

#[test]
fn r01__a_proposal_is_written_when_it_is_given() {
    let home = scratch_home("proposal");
    let (out, err, code) = dstack(
        &home,
        &home,
        &[
            "issue",
            "new",
            TITLE,
            "--symptom",
            "exits 1 and prints nothing",
            "--repro",
            "dstack plan start P4 --worktree ./notes.txt",
            "--source",
            "dstack-cli/src/verbs/plan/lifecycle.rs",
            "--proposal",
            "say which path was refused and why",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("sighting 1"), "{out}");
    let text = read(&folder(&home).join(format!("{SLUG}.md")));
    assert!(
        text.contains("\n## Proposal\nsay which path was refused and why\n"),
        "{text}"
    );
}

#[test]
fn r01__the_sighting_carries_the_run_and_the_plan_it_was_filed_from() {
    let home = scratch_home("origin");
    let (work, _p1, _p2) = sandbox(&home);
    let run = current_run(&work);
    assert!(
        run.ends_with("_sandbox"),
        "the sandbox has a current run: {run}"
    );

    // The run is the one CURRENT names. Two plans are in progress and this checkout is the
    // worktree of neither, so naming either of them would be a guess: the plan reads "-".
    let (out, err, code) = file_issue(&home, &work, TITLE);
    assert_eq!(code, 0, "{err}");
    let expected = format!("run {run}  plan -");
    assert!(
        out.contains(&expected),
        "stdout has no '{expected}':\n{out}"
    );
    let text = read(&folder(&home).join(format!("{SLUG}.md")));
    let sighting = text
        .lines()
        .find(|line| line.starts_with("- 20"))
        .expect("a sighting line");
    assert!(sighting.ends_with(&expected), "{sighting}");
}

#[test]
fn r01__a_filing_from_a_worker_worktree_names_the_run_and_the_plan() {
    let home = scratch_home("worker");
    let (work, _p1, worker) = sandbox(&home);
    let run = current_run(&work);
    // R36: dstack made this worktree for P2, and a worktree it made carries no CURRENT of its own.
    assert!(
        !worker.join(".dstack/local/CURRENT").exists(),
        "the worker worktree has no CURRENT"
    );

    // Two plans are running; the one that recorded this worktree is the one that filed.
    let (out, err, code) = file_issue(&home, &worker, TITLE);
    assert_eq!(code, 0, "{err}");
    let expected = format!("run {run}  plan P2");
    assert!(
        out.contains(&expected),
        "stdout has no '{expected}':\n{out}"
    );
}

/// Shape (a) of review 003: finishing a plan leaves its worktree recorded, and the next plan may
/// start in that same directory. The sighting names the plan running there now, not the one that
/// used to run there.
#[test]
fn r01__a_reused_worker_checkout_names_the_plan_running_in_it_now() {
    let home = scratch_home("reused");
    let work = store(&home);
    let run = run_with_plans(&home, &work, "goal", &["first", "second"]);
    ok(
        &home,
        &work,
        &["plan", "start", "P1", "--worktree", "../wt"],
    );
    ok(&home, &work, &["plan", "done", "P1"]);
    // P2 starts in the directory P1 left behind: plan start takes an existing path as it is.
    ok(
        &home,
        &work,
        &["plan", "start", "P2", "--worktree", "../wt"],
    );

    let worker = home.join("wt");
    let (out, err, code) = file_issue(&home, &worker, TITLE);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains(&format!("run {run}  plan P2")),
        "the finished P1 still records this worktree, and must not take the sighting:\n{out}"
    );
}

/// Shape (b) of review 003: a store holds every run it ever opened, and read_dir walks them in no
/// order anyone chose. A closed run whose finished plan recorded this directory must not be able
/// to take a sighting from the plan that is running in it.
#[test]
fn r01__a_closed_run_that_recorded_the_worktree_does_not_take_the_sighting() {
    let home = scratch_home("older");
    let work = store(&home);
    run_with_plans(&home, &work, "older", &["first"]);
    ok(
        &home,
        &work,
        &["plan", "start", "P1", "--worktree", "../wt"],
    );
    ok(&home, &work, &["plan", "done", "P1"]);
    ok(
        &home,
        &work,
        &["run", "close", "--abandon", "the older Goal"],
    );

    // A filing while only the closed run records the directory: nothing is running there, so
    // neither the run nor the plan is something the sighting can claim.
    let worker = home.join("wt");
    let (out, err, code) = file_issue(&home, &worker, "an issue between two goals");
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("run -  plan -"),
        "a finished plan of a closed run is not where a filing came from:\n{out}"
    );

    // Now the open Goal starts its own plan in the same directory.
    let run = run_with_plans(&home, &work, "current", &["first", "second"]);
    ok(
        &home,
        &work,
        &["plan", "start", "P2", "--worktree", "../wt"],
    );
    let (out, err, code) = file_issue(&home, &worker, TITLE);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains(&format!("run {run}  plan P2")),
        "the open run's in-progress plan is the one that filed:\n{out}"
    );
}

/// The plan.json of one run of the store, as text.
fn plan_json(work: &Path, run: &str) -> String {
    read(&work.join(".dstack/runs").join(run).join("plan.json"))
}

/// Shape (a) of review 005: `run close --abandon` writes the run's status and never touches its
/// plans, so the abandoned Goal keeps a plan that still says in-progress and still records the
/// checkout. A filing from there did not come out of that Goal.
#[test]
fn r01__an_abandoned_run_does_not_take_a_filing_from_the_checkout_it_kept() {
    let home = scratch_home("abandoned");
    let work = store(&home);
    let run = run_with_plans(&home, &work, "abandoned", &["first"]);
    ok(
        &home,
        &work,
        &["plan", "start", "P1", "--worktree", "../wt"],
    );
    ok(
        &home,
        &work,
        &["run", "close", "--abandon", "the Goal was dropped"],
    );
    // The shape this test is about: the plan of the abandoned run is still in-progress in ../wt.
    let recorded = plan_json(&work, &run);
    assert!(
        recorded.contains("\"status\": \"in-progress\""),
        "{recorded}"
    );
    assert!(recorded.contains("/wt\""), "{recorded}");

    let worker = home.join("wt");
    let (out, err, code) = file_issue(&home, &worker, TITLE);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("run -  plan -"),
        "a Goal that was abandoned is not where this filing came from:\n{out}"
    );
}

/// Shape (b) of review 005: the same checkout, recorded by the abandoned run's stale plan and by
/// the live plan of the open Goal. The stale record must not make the live one ambiguous.
#[test]
fn r01__a_live_plan_wins_over_the_abandoned_run_that_kept_the_checkout() {
    let home = scratch_home("stale-and-live");
    let work = store(&home);
    run_with_plans(&home, &work, "abandoned", &["first"]);
    ok(
        &home,
        &work,
        &["plan", "start", "P1", "--worktree", "../wt"],
    );
    ok(
        &home,
        &work,
        &["run", "close", "--abandon", "the Goal was dropped"],
    );
    let run = run_with_plans(&home, &work, "current", &["first", "second"]);
    ok(
        &home,
        &work,
        &["plan", "start", "P2", "--worktree", "../wt"],
    );

    let worker = home.join("wt");
    let (out, err, code) = file_issue(&home, &worker, TITLE);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains(&format!("run {run}  plan P2")),
        "the plan running in the open Goal is the one that filed:\n{out}"
    );
}

/// Shape (c) of review 005: CURRENT names the run, and two plans of that run are in progress in
/// this one checkout. The run is known; which of the two filed is not, and is not guessed.
#[test]
fn r01__two_plans_running_in_one_checkout_are_not_guessed_between() {
    let home = scratch_home("two-here");
    let work = store(&home);
    let run = run_with_plans(&home, &work, "goal", &["first", "second"]);
    ok(&home, &work, &["plan", "start", "P1", "--worktree", "."]);
    ok(&home, &work, &["plan", "start", "P2", "--worktree", "."]);
    assert_eq!(current_run(&work), run, "CURRENT names the open run");

    let (out, err, code) = file_issue(&home, &work, TITLE);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains(&format!("run {run}  plan -")),
        "CURRENT still names the run, and neither plan may claim the filing:\n{out}"
    );
}

#[test]
fn r02__the_same_title_twice_appends_a_sighting_to_the_one_file() {
    let home = scratch_home("again");
    let (first, err, code) = file_issue(&home, &home, TITLE);
    assert_eq!(code, 0, "{err}");
    assert!(first.contains("sighting 1"), "{first}");

    // The second filing normalises to the same slug through the punctuation and the capitals.
    let (second, err, code) = file_issue(&home, &home, "Plan start, refuses a file worktree!");
    assert_eq!(code, 0, "{err}");
    assert!(second.contains("sighting 2"), "{second}");

    assert_eq!(listing(&home), vec![format!("{SLUG}.md")]);
    let text = read(&folder(&home).join(format!("{SLUG}.md")));
    assert!(text.contains("\nsightings: 2\n---\n"), "{text}");
    assert_eq!(
        text.lines().filter(|line| line.starts_with("- 20")).count(),
        2,
        "two sighting lines:\n{text}"
    );
    // The first filing's title is the one the file keeps.
    assert!(
        text.starts_with(&format!("---\ntitle: {TITLE}\n")),
        "{text}"
    );
}

/// R02: the wave this verb exists for — several workers hit the same friction at the same moment.
/// Reading the count and writing it back is one step under the folder lock, so every filing gets a
/// number of its own and none is overwritten by a neighbour that read the same count.
#[test]
fn r02__filings_that_race_each_other_all_reach_the_file() {
    const WORKERS: usize = 6;
    let home = scratch_home("race");
    let racing: Vec<_> = (0..WORKERS)
        .map(|_| {
            let home = home.clone();
            std::thread::spawn(move || file_issue(&home, &home, TITLE))
        })
        .collect();
    let mut numbered: Vec<usize> = Vec::new();
    for worker in racing {
        let (out, err, code) = worker.join().expect("the filing finished");
        assert_eq!(code, 0, "{err}");
        let line = out
            .lines()
            .find(|line| line.trim_start().starts_with("sighting "))
            .unwrap_or_else(|| panic!("no sighting line:\n{out}"));
        numbered.push(
            line.split_whitespace()
                .nth(1)
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or_else(|| panic!("no sighting number: {line}")),
        );
    }
    numbered.sort_unstable();
    assert_eq!(
        numbered,
        (1..=WORKERS).collect::<Vec<usize>>(),
        "every filing got a number of its own"
    );

    assert_eq!(listing(&home), vec![format!("{SLUG}.md")]);
    let text = read(&folder(&home).join(format!("{SLUG}.md")));
    assert!(
        text.contains(&format!("\nsightings: {WORKERS}\n---\n")),
        "{text}"
    );
    assert_eq!(
        text.lines().filter(|line| line.starts_with("- 20")).count(),
        WORKERS,
        "one sighting line per filing:\n{text}"
    );
    assert!(
        !folder(&home).join("lock").exists(),
        "the lock is released on the way out"
    );
}

#[test]
fn r03__an_issue_missing_a_required_field_is_refused_and_writes_nothing() {
    let home = scratch_home("refused");
    let full = [
        "issue",
        "new",
        TITLE,
        "--symptom",
        "exits 1 and prints nothing",
        "--repro",
        "dstack plan start P4 --worktree ./notes.txt",
        "--source",
        "dstack-cli/src/verbs/plan/lifecycle.rs",
    ];
    for (field, drop_at) in [("--symptom", 3), ("--repro", 5), ("--source", 7)] {
        // Absent: the option and its value are cut out of the full call.
        let mut absent: Vec<&str> = full.to_vec();
        absent.drain(drop_at..drop_at + 2);
        // Empty: the value is there and reads as nothing at all.
        let mut empty: Vec<&str> = full.to_vec();
        empty[drop_at + 1] = "";
        for args in [absent, empty] {
            let (out, err, code) = dstack(&home, &home, &args);
            assert_eq!(code, 1, "{field} missing must exit 1: {out}{err}");
            assert!(
                err.starts_with("dstack: ") && err.contains(field),
                "the refusal names {field}: {err}"
            );
            assert_eq!(out, "", "a refused issue prints nothing on stdout");
            assert!(
                !folder(&home).exists(),
                "a refused issue does not create the folder"
            );
        }
    }
}

#[test]
fn r03__an_issue_without_a_title_is_refused() {
    let home = scratch_home("no-title");
    for title in ["", "!!!"] {
        let (out, err, code) = file_issue(&home, &home, title);
        assert_eq!(code, 1, "{out}{err}");
        assert!(err.starts_with("dstack: "), "{err}");
        assert_eq!(out, "");
        assert!(!folder(&home).exists());
    }
}

#[test]
fn r04__list_prints_a_row_per_file_and_a_closing_count() {
    let home = scratch_home("list");
    // A folder that is not there is a count of 0, not a refusal.
    let (out, err, code) = dstack(&home, &home, &["issue", "list"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("issues 0"), "{out}");
    assert!(!folder(&home).exists(), "list creates nothing");

    file_issue(&home, &home, TITLE);
    file_issue(&home, &home, TITLE);
    file_issue(&home, &home, "evidence add refuses a stale artifact");

    let (out, err, code) = dstack(&home, &home, &["issue", "list"]);
    assert_eq!(code, 0, "{err}");
    let rows: Vec<&str> = out.lines().filter(|line| line.starts_with("  ")).collect();
    assert_eq!(rows.len(), 2, "one row per file:\n{out}");
    let filed = read(&folder(&home).join(format!("{SLUG}.md")));
    let last = filed
        .lines()
        .filter(|line| line.starts_with("- 20"))
        .last()
        .expect("a sighting line")
        .split("  ")
        .next()
        .expect("the stamp")
        .trim_start_matches("- ")
        .to_string();
    let row = rows
        .iter()
        .find(|row| row.contains(TITLE))
        .unwrap_or_else(|| panic!("no row for {TITLE}:\n{out}"));
    assert!(row.contains("sightings 2"), "{row}");
    assert!(
        row.contains(&last),
        "the row carries the last seen stamp: {row}"
    );
    assert!(
        out.lines().any(|line| line == "issues 2"),
        "the closing count line:\n{out}"
    );
}

/// D-12 of run 20260902T085818Z_dstack-rust: a folder that is not there is nothing filed yet, but
/// one that is there and cannot be read is a cannot-decide — a row dropped in silence would make
/// the closing count say the folder holds less than it does.
#[test]
fn r04__a_folder_that_cannot_be_read_is_a_cannot_decide() {
    let home = scratch_home("unreadable");
    file_issue(&home, &home, TITLE);
    let dir = folder(&home);
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o000)).expect("chmod 000");
    let (out, err, code) = dstack(&home, &home, &["issue", "list"]);
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).expect("chmod back");
    assert_eq!(code, 2, "an unreadable folder cannot be listed: {out}{err}");
    assert!(
        err.starts_with(&format!("dstack: cannot read {}", dir.display())),
        "{err}"
    );
    assert_eq!(out, "", "no count line to disbelieve");
}
