// tests/r10_lint_speed.rs
// R10: the PreToolUse hook pipes every pending write through `lint-ko --stdin`, so the command
// runs on the hook path. The verdict here is relative — half of what the shell dispatcher takes
// in the same minute, measured call for call next to it — because this machine builds several
// ports at once and a millisecond count taken under load average 8 says nothing. The 30 ms R10
// asks for is still checked: hard on an idle machine, printed as a note on a busy one. The shell
// spends most of its time in 46 grep processes, one per regex row.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// The call under the clock.
const LINT: [&str; 4] = ["lint-ko", "--stdin", "--path", "README.md"];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// One implementation under the clock: the program to start and what comes before the dstack
/// arguments — the dispatcher path for the shell, nothing for the Rust binary.
struct Cli {
    program: PathBuf,
    lead: Vec<PathBuf>,
}

impl Cli {
    fn rust(bin: PathBuf) -> Cli {
        Cli {
            program: bin,
            lead: Vec::new(),
        }
    }

    fn shell() -> Cli {
        Cli {
            program: PathBuf::from("bash"),
            lead: vec![shell_ref::dispatcher()],
        }
    }

    /// One call with the prose on stdin: how long it took and what it exited with.
    fn call(&self, dir: &Path, stdin: &Path) -> (Duration, i32) {
        let file = File::open(stdin).expect("the input file");
        let started = Instant::now();
        let done = Command::new(&self.program)
            .args(&self.lead)
            .args(LINT)
            .current_dir(dir)
            .env("DSTACK_KO_RULES", repo().join("claude/lint/ko-rules.tsv"))
            .stdin(Stdio::from(file))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("run dstack");
        (started.elapsed(), done.code().expect("an exit code"))
    }
}

/// A scratch directory holding about 2 KB of Korean prose: what an Edit's new_string looks like
/// when the hook asks about it. lint-ko needs no store and no repository, so neither is built.
fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-p9-speed-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    let dir = std::fs::canonicalize(&dir).expect("the physical path of the scratch directory");
    let paragraph = "이 도구는 저장소 안에서 돌아요. 먼저 실행하면 저장 공간이 생겨요. \
                     실행 결과는 화면에 그대로 나와요. 값이 없으면 이유를 한 줄로 알려줘요.\n";
    let mut text = String::new();
    while text.len() < 2048 {
        text.push_str(paragraph);
    }
    std::fs::write(dir.join("input.md"), &text).expect("write the input");
    dir
}

/// The binary the hooks would call and the ceiling in milliseconds it is held to. The hooks call
/// the release build, so the release build is what is timed: when it is missing or older than
/// this test's own debug binary — the newest the sources can be — it is built again, so the
/// number is never for stale code. Only a failed build falls back to the debug binary, which
/// carries the same work plus its unoptimised overhead and gets 60 ms instead of 30.
fn binary() -> (PathBuf, u64) {
    let debug = PathBuf::from(env!("CARGO_BIN_EXE_dstack"));
    let release = debug
        .parent()
        .and_then(Path::parent)
        .expect("the target directory of this test binary")
        .join("release/dstack");
    if newer_than(&release, &debug) {
        return (release, 30);
    }
    println!("the release binary is missing or older than the sources, building it again");
    let build = Command::new(env!("CARGO"))
        .args(["build", "--release", "--manifest-path"])
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .output();
    let why = match build {
        Err(err) => err.to_string(),
        Ok(done) if !done.status.success() => {
            String::from_utf8_lossy(&done.stderr).trim().to_string()
        }
        Ok(_) if !release.is_file() => "cargo left no release binary".to_string(),
        Ok(_) => return (release, 30),
    };
    println!("the release build failed, timing the debug build instead: {why}");
    (debug, 60)
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

/// The 1-minute load average, the reason a millisecond count can be over its ceiling and still
/// say nothing. macOS answers `sysctl -n vm.loadavg` with "{ 9.50 8.49 8.50 }", Linux keeps the
/// three numbers in /proc/loadavg; in both the first number of the line is the one wanted.
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

fn load_text(load: Option<f64>) -> String {
    load.map_or_else(|| "unknown".to_string(), |load| format!("{load:.2}"))
}

/// The two sweeps of one run: five timings of each implementation, taken next to each other.
struct Measurement {
    ceiling: u64,
    rust: Vec<Duration>,
    shell: Vec<Duration>,
}

/// Both tests read one measurement: cargo runs the tests of a file in parallel, and two sweeps
/// running side by side would each inflate the other's numbers.
fn measured() -> &'static Measurement {
    static ONCE: OnceLock<Measurement> = OnceLock::new();
    ONCE.get_or_init(measure)
}

fn measure() -> Measurement {
    let (bin, ceiling) = binary();
    let rust = Cli::rust(bin);
    let shell = Cli::shell();
    let rust_dir = sandbox("rust");
    let shell_dir = sandbox("shell");
    // One warm-up call each, so the page cache of a binary is not part of a measurement.
    rust.call(&rust_dir, &rust_dir.join("input.md"));
    shell.call(&shell_dir, &shell_dir.join("input.md"));
    let mut rust_times: Vec<Duration> = Vec::new();
    let mut shell_times: Vec<Duration> = Vec::new();
    for _ in 0..5 {
        // Interleaved, so a burst of load lands on both sides of the comparison.
        shell_times.push(timed(&shell, &shell_dir));
        rust_times.push(timed(&rust, &rust_dir));
    }
    std::fs::remove_dir_all(&rust_dir).expect("clean up");
    std::fs::remove_dir_all(&shell_dir).expect("clean up");
    println!(
        "lint-ko --stdin ({}), ms: {}",
        rust.program.display(),
        list(&rust_times)
    );
    println!(
        "lint-ko --stdin ({}), ms: {}",
        shell.lead[0].display(),
        list(&shell_times)
    );
    println!(
        "median: rust {:.1} ms, shell {:.1} ms, ceiling {ceiling} ms, load average {}",
        ms(median(&rust_times)),
        ms(median(&shell_times)),
        load_text(load_average())
    );
    Measurement {
        ceiling,
        rust: rust_times,
        shell: shell_times,
    }
}

fn timed(cli: &Cli, dir: &Path) -> Duration {
    let (took, code) = cli.call(dir, &dir.join("input.md"));
    assert_eq!(code, 0, "the input is clean prose in an unclassified path");
    took
}

fn median(times: &[Duration]) -> Duration {
    let mut sorted = times.to_vec();
    sorted.sort();
    sorted[sorted.len() / 2]
}

fn ms(took: Duration) -> f64 {
    took.as_secs_f64() * 1000.0
}

fn list(times: &[Duration]) -> String {
    times
        .iter()
        .map(|took| format!("{:.1}", ms(*took)))
        .collect::<Vec<String>>()
        .join(", ")
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r10_lint_ko_stdin_is_fast_enough_for_the_hook_path() {
    let done = measured();
    let median = median(&done.rust);
    let load = load_average();
    let over = median >= Duration::from_millis(done.ceiling);
    if over {
        println!(
            "median {:.1} ms is over the absolute ceiling under load average {}",
            ms(median),
            load_text(load)
        );
    }
    // An idle machine has no excuse: there the ceiling R10 names is the verdict. On a busy one
    // the number is only printed, and the relative bound below carries the requirement.
    assert!(
        !over || !load.is_some_and(|load| load < 2.0),
        "median {:.1} ms is over the {} ms this build is held to on an idle machine",
        ms(median),
        done.ceiling
    );
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r10_lint_ko_stdin_beats_the_shell_by_half() {
    let done = measured();
    let (rust, shell) = (median(&done.rust), median(&done.shell));
    assert!(
        rust * 2 < shell,
        "the rust median {:.1} ms is not below half the shell's {:.1} ms",
        ms(rust),
        ms(shell)
    );
}
