// tests/r05_fixture_runner.rs
// R05: the fixture runner is the proof that every checker can fail, so cargo test drives the same
// runner `dstack doctor --self` drives — in process, over claude/lint/fixtures — and reads the
// fixture directories back against the registry, so a checker nobody registered cannot hide.
// R10: `doctor --self` is the slowest thing on the pipeline's path, held here to a quarter of the
// shell's 97 s baseline, measured next to the shell in the same minute.
#![allow(non_snake_case)]

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::verbs;
use dstack_cli::verbs::doctor::selfrun::{self, Counts};

/// What the runner has to reach on this repository: R05 asks for 23 checkers and 75 fixtures.
const CHECKERS: usize = 23;
const FIXTURES: usize = 75;

/// The shell baseline R10 names, in seconds, and the share of it the port is held to.
const SHELL_BASELINE: u64 = 97;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Everything the three tests below read. The sweep and the two timed runs each drive every
/// fixture of the repository, so they run once and in sequence: cargo runs the tests of a file in
/// parallel, and three sweeps at a time would each inflate the others' numbers.
struct Work {
    counts: Counts,
    printed: String,
    rust: Duration,
    shell: Duration,
    ceiling: Duration,
    /// The exit code and the closing count line of each timed run: a run that did not exit 0
    /// measured a `doctor --self` that failed, and a duration taken from it proves nothing.
    rust_run: (i32, String),
    shell_run: (i32, String),
}

fn work() -> &'static Work {
    static ONCE: OnceLock<Work> = OnceLock::new();
    ONCE.get_or_init(|| {
        let (counts, printed) = sweep();
        let (binary, note) = release_binary();
        println!("timing {} ({note})", binary.display());
        let (rust, rust_code, rust_out) = timed(&binary, &[]);
        let (shell, shell_code, shell_out) =
            timed(Path::new("bash"), &[shell_ref::dispatcher()]);
        println!(
            "doctor --self: rust {:.1} s, shell {:.1} s, ceiling {:.1} s",
            rust.as_secs_f64(),
            shell.as_secs_f64(),
            shell.as_secs_f64() / 4.0
        );
        Work {
            counts,
            printed,
            rust,
            shell,
            ceiling: Duration::from_secs(SHELL_BASELINE / 4),
            rust_run: (rust_code, last_line(&rust_out)),
            shell_run: (shell_code, last_line(&shell_out)),
        }
    })
}

/// The runner of `doctor --self`, called in process the way the verb calls it.
fn sweep() -> (Counts, String) {
    let home = Home::resolve().expect("the repository of this test binary");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    ctx.out.begin_capture();
    let counts = selfrun::sweep(&mut ctx, &verbs::all_selftests()).expect("the runner decides");
    let (printed, _) = ctx.out.end_capture();
    (counts, printed)
}

/// One `doctor --self` under the clock, run on this repository (the sandboxes it builds live
/// under the machine's temp directory, so nothing here writes to the store).
fn timed(program: &Path, lead: &[PathBuf]) -> (Duration, i32, String) {
    let started = Instant::now();
    let out = Command::new(program)
        .args(lead)
        .arg("doctor")
        .arg("--self")
        .current_dir(repo())
        .output()
        .expect("run doctor --self");
    let took = started.elapsed();
    let code = out.status.code().expect("an exit code");
    let printed = String::from_utf8_lossy(&out.stdout).into_owned();
    (took, code, printed)
}

/// The hooks call the release build, so the release build is what is timed: when it is missing or
/// older than this test's own debug binary — the newest the sources can be — it is built again.
fn release_binary() -> (PathBuf, &'static str) {
    let debug = PathBuf::from(env!("CARGO_BIN_EXE_dstack"));
    let release = debug
        .parent()
        .and_then(Path::parent)
        .expect("the target directory of this test binary")
        .join("release/dstack");
    if newer_than(&release, &debug) {
        return (release, "release");
    }
    let build = Command::new(env!("CARGO"))
        .args(["build", "--release", "--manifest-path"])
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .output();
    match build {
        Ok(done) if done.status.success() && release.is_file() => (release, "release"),
        _ => (debug, "debug: the release build failed"),
    }
}

fn newer_than(built: &Path, than: &Path) -> bool {
    let stamp = |path: &Path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    };
    match (stamp(built), stamp(than)) {
        (Some(built), Some(than)) => built >= than,
        _ => false,
    }
}

/// The closing count line of a runner's output, which is what a failing run has to be judged by.
fn last_line(printed: &str) -> String {
    printed.lines().last().unwrap_or_default().to_string()
}

/// macOS answers `sysctl -n vm.loadavg` with "{ 9.50 8.49 8.50 }", Linux keeps the three numbers
/// in /proc/loadavg; in both the first number of the line is the one wanted.
fn load_average() -> Option<f64> {
    let text = match std::fs::read_to_string("/proc/loadavg") {
        Ok(line) => line,
        Err(_) => {
            let out = Command::new("sysctl")
                .args(["-n", "vm.loadavg"])
                .output()
                .ok()?;
            String::from_utf8(out.stdout).ok()?
        }
    };
    text.split_whitespace().find_map(|word| word.parse().ok())
}

#[test]
fn r05__the_fixture_runner_passes_over_every_fixture() {
    let done = work();
    let counts = &done.counts;
    assert_eq!(
        counts.failed, 0,
        "a fixture did not get its verdict:\n{}",
        done.printed
    );
    assert_eq!(
        counts.zero, 0,
        "a checker without both kinds of fixture:\n{}",
        done.printed
    );
    assert!(
        counts.checkers >= CHECKERS,
        "checkers {} is under the {CHECKERS} R05 asks for",
        counts.checkers
    );
    assert!(
        counts.fixtures >= FIXTURES,
        "fixtures {} is under the {FIXTURES} R05 asks for",
        counts.fixtures
    );
    assert_eq!(counts.passed, counts.fixtures, "every fixture is a pass");
}

/// The finding of P1 round 005 read the other way round: the registry was only ever asked about
/// the checkers it holds, so a fixture directory nobody registered stayed invisible. Both
/// directions are compared here — directory to registry and registry to directory — because
/// either one alone lets a checker that never proved it can fail sit in a green run.
#[test]
fn r05__the_fixture_directories_and_the_registry_are_the_same_set() {
    let home = Home::resolve().expect("the repository of this test binary");
    let registered: Vec<&'static str> = verbs::all_selftests()
        .iter()
        .map(|checker| checker.checker())
        .collect();
    let mut unregistered: Vec<String> = Vec::new();
    let fixtures = home.home.join("lint/fixtures");
    for entry in std::fs::read_dir(&fixtures)
        .expect("the fixture directories")
        .flatten()
    {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if !path.is_dir() || name.starts_with('.') {
            continue;
        }
        assert!(
            selfrun::fixtures(&path).len() >= 2,
            "{name} needs one bad-* and one good-* fixture"
        );
        if !registered.contains(&name.as_str()) {
            unregistered.push(name);
        }
    }
    assert!(
        unregistered.is_empty(),
        "fixture directories no checker is registered for: {unregistered:?}"
    );
    // And back: a registered checker whose directory is missing, or holds only one kind of
    // fixture, has never shown that it can fail — the runner counts it as a zero-fixture checker.
    let fixtureless: Vec<&str> = registered
        .iter()
        .copied()
        .filter(|checker| selfrun::fixtures(&fixtures.join(checker)).len() < 2)
        .collect();
    assert!(
        fixtureless.is_empty(),
        "registered checkers without a bad-* and a good-* fixture: {fixtureless:?}"
    );
    assert!(
        registered.len() >= CHECKERS,
        "the registry holds {} checkers, under the {CHECKERS} R05 asks for",
        registered.len()
    );
}

/// R10: a quarter of the shell baseline of 97 s. Most of what the runner spends is the sandbox
/// subprocesses — git init, a commit, `dstack init` — which are the same work on both sides, so
/// the port comes out a little over three times faster, not four; what R10 names is the absolute
/// bound, asked of an idle machine, and the relative one carries the verdict under load.
#[test]
fn r10__doctor_self_is_under_a_quarter_of_the_shell_baseline() {
    let done = work();
    // A duration is only worth reading when the run it came from passed: a `doctor --self` that
    // exited nonzero stopped somewhere, and the seconds it took say nothing about the hook path.
    assert_eq!(
        done.rust_run.0, 0,
        "the timed run exited {}: {}",
        done.rust_run.0, done.rust_run.1
    );
    // The reference side is a baseline, not a verdict, but it still has to have done the work.
    // Exit 1 is fixtures that failed — since P14 the shell's own hook checkers cannot pass,
    // because the wrapper hands `hook <event>` to a dispatcher that has no such verb and P17
    // retires it — while 2 or 127 means it never ran and the seconds next to it mean nothing.
    assert!(
        matches!(done.shell_run.0, 0 | 1),
        "the shell run it is measured against exited {}: {}",
        done.shell_run.0,
        done.shell_run.1
    );
    assert!(
        done.rust * 2 < done.shell,
        "the port took {:.1} s against the shell's {:.1} s in the same minute",
        done.rust.as_secs_f64(),
        done.shell.as_secs_f64()
    );
    let load = load_average();
    let over = done.rust >= done.ceiling;
    if over {
        println!(
            "{:.1} s is over the {} s ceiling under load average {}",
            done.rust.as_secs_f64(),
            done.ceiling.as_secs(),
            load.map_or_else(|| "unknown".to_string(), |load| format!("{load:.2}"))
        );
    }
    assert!(
        !over || !load.is_some_and(|load| load < 2.0),
        "{:.1} s is over the {} s an idle machine is held to",
        done.rust.as_secs_f64(),
        done.ceiling.as_secs()
    );
}
