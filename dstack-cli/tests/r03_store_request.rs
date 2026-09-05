// tests/r03_store_request.rs
// R03: the store layer reads and writes the v2 formats exactly as the shell library does.

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;

use dstack_cli::store::cases;
use dstack_cli::store::request::{self, RequestDoc};
use dstack_cli::store::review_index;
use dstack_cli::store::rows;
use dstack_cli::store::tables;
use dstack_cli::store::tsv;

/// A request carrying every marker kind the shell can write, plus a ticked box and a
/// `pending:` accept, so one fixture exercises the whole row grammar.
const REQUEST: &str = "---\n\
work_type: cli\n\
route: merge 20260902T052531Z_dstack-v2\n\
external_research: none\n\
risk_axes: none\n\
design_review: auto\n\
review: off\n\
codex_effort: high\n\
e2e: cli\n\
unit_tests: on\n\
visual: none\n\
korean_polish: on\n\
---\n\
# fixture\n\
\n\
## Requirements\n\
\n\
- [ ] **R01** the first row — accept: the command exits 0\n\
- [ ] **R02** a withdrawn row — accept: nothing — withdrawn: the owner dropped it\n\
- [ ] **R03** a deferred row — accept: nothing — deferred: the next Goal\n\
- [ ] **R04** a split parent — accept: nothing — superseded-by: R06,R07\n\
- [ ] **R05** a row nobody approved yet — accept: it prints — status: pending-approval\n\
- [ ] **R06** an assumption row — accept: the log names it — from: Q-01\n\
- [x] **R07** a ticked row — accept: the box is computed\n\
- [ ] **R08** a row born incomplete — accept: pending: agent to propose\n\
\n\
Prose that mentions **R01** without being a row.\n";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The run R03's acceptance clause names, which only this machine's store holds.
const LIVE_RUN: &str = "20260902T052531Z_dstack-v2";

/// The live store of this repository: it lives in the main worktree (the git common dir's
/// parent, as resolve_roots computes it) and is machine-local, so a test that reads it skips
/// loudly when it is absent.
fn live_run() -> Option<PathBuf> {
    let common = Command::new("git")
        .args([
            "-C",
            &repo().to_string_lossy(),
            "rev-parse",
            "--git-common-dir",
        ])
        .output()
        .expect("git rev-parse");
    let common = PathBuf::from(String::from_utf8_lossy(&common.stdout).trim().to_string());
    let common = if common.is_absolute() {
        common
    } else {
        repo().join(common)
    };
    let dir = common
        .parent()
        .expect("the git common dir has a parent")
        .join(".dstack/runs")
        .join(LIVE_RUN);
    if dir.is_dir() {
        Some(dir)
    } else {
        println!("skipped: live store absent ({})", dir.display());
        None
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-r03-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write(path: &Path, text: &str) {
    std::fs::write(path, text).expect("write fixture");
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).expect("read back")
}

/// Runs one shell-library expression with `$1` bound to a path — the parity oracle: what the
/// port must produce is whatever the shell reference produces for the same file.
fn shell(script: &str, arg: &Path) -> String {
    let out = Command::new("bash")
        .arg("-c")
        .arg(format!(". \"$DSTACK_LIB/common.sh\"; {script}"))
        .arg("_")
        .arg(arg)
        .env("DSTACK_LIB", shell_ref::lib())
        .env("DSTACK_SELF", shell_ref::dispatcher())
        .output()
        .expect("run the shell library");
    assert!(
        out.status.success(),
        "shell library failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("shell output is utf-8")
}

/// The port's rows in the exact shape `req_rows` prints them.
fn rows_as_shell_prints(doc: &RequestDoc) -> String {
    let mut out = String::new();
    for row in doc.rows() {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            row.id,
            row.text,
            row.accept,
            row.markers_string()
        ));
    }
    out
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_rows_parse_as_the_shell_prints_them() {
    let dir = scratch("rows");
    let file = dir.join("request.md");
    write(&file, REQUEST);
    let doc = RequestDoc::load(&file).expect("load");
    assert_eq!(rows_as_shell_prints(&doc), shell("req_rows \"$1\"", &file));
    assert_eq!(
        doc.live_ids().join("\n") + "\n",
        shell("req_live_ids \"$1\"", &file)
    );
    assert_eq!(doc.max_id(), 8);
    assert_eq!(
        shell("req_max_id \"$1\"", &file).trim_end(),
        doc.max_id().to_string()
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r03_every_marker_kind_is_readable() {
    let dir = scratch("markers");
    let file = dir.join("request.md");
    write(&file, REQUEST);
    let doc = RequestDoc::load(&file).expect("load");

    let r02 = doc.row("R02").expect("R02");
    assert_eq!(
        r02.marker("withdrawn").as_deref(),
        Some("the owner dropped it")
    );
    assert!(!r02.is_live());
    assert_eq!(
        doc.row("R03").expect("R03").marker("deferred").as_deref(),
        Some("the next Goal")
    );
    assert_eq!(
        doc.row("R04")
            .expect("R04")
            .marker("superseded-by")
            .as_deref(),
        Some("R06,R07")
    );
    let r05 = doc.row("R05").expect("R05");
    assert!(r05.is_pending() && !r05.is_live());
    let r06 = doc.row("R06").expect("R06");
    assert_eq!(r06.marker("from").as_deref(), Some("Q-01"));
    assert!(r06.is_live());
    let r07 = doc.row("R07").expect("R07");
    assert!(r07.ticked && r07.is_live());
    assert_eq!(r07.markers_string(), "ticked=yes");
    assert_eq!(
        doc.row("R08").expect("R08").accept,
        "pending: agent to propose"
    );
    assert_eq!(doc.row("R09"), None);

    // The row's line number is the file line; row_lineno is the grep the verbs use, and the
    // prose mention of **R01** at the end of the fixture must not win over the row.
    assert_eq!(
        doc.row("R01").expect("R01").lineno,
        doc.row_lineno("R01").expect("R01 line")
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// req_marker() itself, out of the reference: `printf '%s\n' "$markers" | tr ';' '\n' | awk -F=`.
fn shell_req_marker(markers: &str, key: &str) -> String {
    let out = Command::new("bash")
        .arg("-c")
        .arg(". \"$DSTACK_LIB/common.sh\"; req_marker \"$1\" \"$2\"")
        .arg("_")
        .arg(markers)
        .arg(key)
        .env("DSTACK_LIB", shell_ref::lib())
        .env("DSTACK_SELF", shell_ref::dispatcher())
        .output()
        .expect("run req_marker");
    assert!(
        out.status.success(),
        "req_marker failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("req_marker output is utf-8")
}

/// The lookup is not "find the segment named key": the shell joins every marker into one string
/// and tokenizes that, so a value carrying a `;` ends there and whatever follows is a marker of
/// its own. A value carrying an `=` keeps everything after the first one.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_marker_tokenizes_like_req_marker() {
    let lines = [
        "- [ ] **R10** a row — accept: it prints — deferred: wait;withdrawn=why — from: Q-02",
        "- [x] **R11** another row — accept: it prints — deferred: a=b;c — status: pending-approval",
    ];
    for line in lines {
        let row = rows::parse_line(1, line).expect("a row");
        for key in [
            "deferred",
            "withdrawn",
            "from",
            "status",
            "c",
            "ticked",
            "missing",
        ] {
            let produced = match row.marker(key) {
                Some(value) => format!("{value}\n"),
                None => String::new(),
            };
            assert_eq!(
                produced,
                shell_req_marker(&row.markers_string(), key),
                "marker({key}) of {line}"
            );
        }
    }

    let row = rows::parse_line(1, lines[0]).expect("a row");
    assert_eq!(
        row.markers_string(),
        "deferred=wait;withdrawn=why;from=Q-02"
    );
    assert_eq!(row.marker("deferred").as_deref(), Some("wait"));
    assert_eq!(row.marker("withdrawn").as_deref(), Some("why"));
    assert_eq!(row.marker("missing"), None);
    assert!(!row.is_live(), "the hidden withdrawn kills the row");

    let row = rows::parse_line(1, lines[1]).expect("a row");
    assert_eq!(row.marker("deferred").as_deref(), Some("a=b"));
    assert_eq!(row.marker("c").as_deref(), Some("c"));
    assert_eq!(row.marker("status").as_deref(), Some("pending-approval"));
    assert_eq!(row.marker("ticked").as_deref(), Some("yes"));
}

#[test]
fn r03_rows_render_back_byte_for_byte() {
    let dir = scratch("render");
    let file = dir.join("request.md");
    write(&file, REQUEST);
    let doc = RequestDoc::load(&file).expect("load");
    let lines: Vec<&str> = REQUEST.lines().collect();
    for row in doc.rows() {
        assert_eq!(
            row.render(),
            lines[row.lineno - 1],
            "row {} re-renders",
            row.id
        );
    }
    assert_eq!(rows::parse_line(1, "not a row"), None);
    assert_eq!(rows::parse_line(1, "- [ ] **R1x** bad — accept: a"), None);
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// A request whose last id is past u32: `req add --id R4294967296` mints such a row, and awk's
/// `\*\*R[0-9]+\*\*` reads it back as a row like any other.
const WIDE_REQUEST: &str = "# fixture\n\
\n\
- [ ] **R01** the first row — accept: it prints\n\
- [ ] **R02** the second row — accept: it prints\n\
- [ ] **R4294967296** a row past u32 — accept: it prints\n";

/// `$((10#$n))`, the arithmetic that mints an id: the oracle for what a number of any width is.
fn bash_arith(digits: &str) -> String {
    let out = Command::new("bash")
        .arg("-c")
        .arg("echo $((10#$1))")
        .arg("_")
        .arg(digits)
        .output()
        .expect("bash arithmetic");
    String::from_utf8(out.stdout)
        .expect("bash output is utf-8")
        .trim_end()
        .to_string()
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_an_id_past_u32_stays_a_row() {
    let dir = scratch("wide-ids");
    let file = dir.join("request.md");
    write(&file, WIDE_REQUEST);
    let doc = RequestDoc::load(&file).expect("load");

    assert_eq!(doc.rows().len(), 3);
    assert_eq!(rows_as_shell_prints(&doc), shell("req_rows \"$1\"", &file));
    assert_eq!(doc.live_ids(), ["R01", "R02", "R4294967296"]);
    assert_eq!(
        doc.max_id().to_string(),
        shell("req_max_id \"$1\"", &file).trim_end()
    );
    let row = doc.row("R4294967296").expect("R4294967296");
    assert_eq!(
        row.render(),
        WIDE_REQUEST.lines().nth(row.lineno - 1).unwrap()
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// Past 2^63 the two oracles part: bash's `$((10#$n))` wraps to a negative, while req_max_id's
/// awk holds the digits in a double and prints them back positive. The store follows the mint,
/// so an id it reports is the one `req add` would compare against.
#[test]
fn r03_an_id_past_i64_wraps_as_the_mint_does() {
    let dir = scratch("wrapped-ids");
    let file = dir.join("request.md");
    write(
        &file,
        "# fixture\n\n- [ ] **R9223372036854775808** past i64 — accept: it prints\n",
    );
    let doc = RequestDoc::load(&file).expect("load");

    assert_eq!(doc.rows().len(), 1);
    assert!(doc.row("R9223372036854775808").is_some());
    assert_eq!(doc.live_ids(), ["R9223372036854775808"]);
    assert_eq!(doc.max_id().to_string(), bash_arith("9223372036854775808"),);
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_frontmatter_is_read_as_the_shell_reads_it() {
    let dir = scratch("frontmatter");
    let file = dir.join("request.md");
    write(&file, REQUEST);
    let doc = RequestDoc::load(&file).expect("load");
    for key in request::REQ_FIELDS {
        assert_eq!(
            doc.field(key).unwrap_or_default(),
            shell(&format!("req_field \"$1\" {key}"), &file).trim_end_matches('\n'),
            "field {key}"
        );
    }
    assert_eq!(
        doc.field("route").as_deref(),
        Some("merge 20260902T052531Z_dstack-v2")
    );
    assert_eq!(doc.field("nothing"), None);
    assert_eq!(
        doc.declared_keys(),
        request::REQ_FIELDS.map(|k| k.to_string()).to_vec()
    );
    assert_eq!(doc.line_count(), REQUEST.lines().count());

    assert_eq!(request::req_enum("route"), ["new-goal", "quick", "merge"]);
    assert_eq!(request::req_enum("nothing").len(), 0);
    assert_eq!(request::field_default("web-ui", "e2e"), "capture");
    assert_eq!(request::field_default("docs-writing", "unit_tests"), "off");
    assert_eq!(request::field_default("cli", "e2e"), "cli");
    for wtype in request::req_enum("work_type") {
        for field in request::REQ_FIELDS {
            assert_eq!(
                request::field_default(wtype, field),
                shell(&format!("field_default {wtype} {field}"), &file).trim_end_matches('\n'),
                "default {wtype}/{field}"
            );
        }
    }
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r03_row_edits_touch_one_line_only() {
    let dir = scratch("edits");
    let file = dir.join("request.md");
    write(&file, REQUEST);
    let mut doc = RequestDoc::load(&file).expect("load");
    doc.replace_accept("R08", "the ledger names it")
        .expect("accept");
    doc.append_marker("R01", "withdrawn: replaced by R09")
        .expect("marker");
    doc.drop_marker("R05", "status").expect("drop");
    doc.save().expect("save");

    let after = read(&file);
    let expected = REQUEST
        .replace(
            "- [ ] **R08** a row born incomplete — accept: pending: agent to propose",
            "- [ ] **R08** a row born incomplete — accept: the ledger names it",
        )
        .replace(
            "- [ ] **R01** the first row — accept: the command exits 0",
            "- [ ] **R01** the first row — accept: the command exits 0 — withdrawn: replaced by R09",
        )
        .replace(
            "- [ ] **R05** a row nobody approved yet — accept: it prints — status: pending-approval",
            "- [ ] **R05** a row nobody approved yet — accept: it prints",
        );
    assert_eq!(after, expected);

    let reloaded = RequestDoc::load(&file).expect("reload");
    assert!(!reloaded.row("R01").expect("R01").is_live());
    assert!(reloaded.row("R05").expect("R05").is_live());
    assert!(doc.replace_accept("R99", "nothing").is_err());
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r03_req_text_ok_refuses_what_would_corrupt_a_row() {
    let err = request::req_text_ok("row text", "a — b").expect_err("separator");
    assert_eq!(
        err.message(),
        "row text must not contain ' — ' (it separates row segments): a — b"
    );
    assert_eq!(err.code(), 1);
    assert_eq!(
        request::req_text_ok("--accept", "")
            .expect_err("empty")
            .message(),
        "--accept must not be empty"
    );
    assert!(request::req_text_ok("row text", "an ordinary line").is_ok());
}

#[test]
fn r03_approval_stamp_keeps_its_two_spaces() {
    let dir = scratch("approval");
    write(&dir.join("request.md"), REQUEST);
    assert!(request::read_approval(&dir).expect("no stamp").is_none());
    assert!(!request::approval_matches(&dir, "beef").expect("no stamp"));
    request::write_approval(&dir, "beef", "2026-09-02T05:26:06Z").expect("stamp");
    assert_eq!(
        read(&dir.join("request.approved")),
        "sha256 beef  approved_at 2026-09-02T05:26:06Z\n"
    );
    let stamp = request::read_approval(&dir).expect("read").expect("stamp");
    assert_eq!(stamp.sha256, "beef");
    assert_eq!(stamp.approved_at, "2026-09-02T05:26:06Z");
    assert!(request::approval_matches(&dir, "beef").expect("read"));
    assert!(!request::approval_matches(&dir, "dead").expect("read"));
    assert_eq!(
        request::stamp_text(&dir).expect("read").as_deref(),
        Some("sha256 beef  approved_at 2026-09-02T05:26:06Z")
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// The two-tool table the harness gives its sandboxes, so no machine-wide deps.tsv is read.
const DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
                    git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
                    jq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n";

/// A repository with a store and one run carrying the wide-id request, built the way the parity
/// harness builds its sandboxes and driven by the reference dispatcher, so the fixture the two
/// binaries are asked about is the shell's own work.
fn run_sandbox(tag: &str) -> PathBuf {
    run_sandbox_minted_by(tag, &shell_bin())
}

/// The same sandbox, with the wide row minted by the binary named here: `req add --id` is the
/// one command of the fixture that has to parse the id, so the port has to be asked it too.
fn run_sandbox_minted_by(tag: &str, minter: &str) -> PathBuf {
    let dir = scratch(tag);
    let dir = std::fs::canonicalize(&dir).expect("the physical path of the scratch directory");
    write(&dir.join(".deps.tsv"), DEPS);
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
    let shell = shell_bin();
    for args in [
        &["init"][..],
        &["run", "new", "wide-ids", "--type", "cli"],
        &["request", "new", "--type", "cli", "--title", "wide ids"],
        &["req", "add", "the first row", "--accept", "it prints"],
    ] {
        let (_out, err, code) = dstack(&shell, &dir, args);
        assert_eq!(code, 0, "shell dstack {args:?} failed: {err}");
    }
    let wide = [
        "req",
        "add",
        "a row past u32",
        "--id",
        "R4294967296",
        "--accept",
        "it prints",
    ];
    let (_out, err, code) = dstack(minter, &dir, &wide);
    assert_eq!(code, 0, "{minter} dstack {wide:?} failed: {err}");
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

/// One dstack call: both streams and the exit code. PATH carries no `code`, so nothing opens an
/// editor on the machine running the tests.
fn dstack(bin: &str, dir: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("PATH", "/usr/bin:/bin")
        .env("DSTACK_DEPS", dir.join(".deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "parity")
        .output()
        .expect("run dstack");
    (
        String::from_utf8(out.stdout).expect("utf-8"),
        String::from_utf8(out.stderr).expect("utf-8"),
        out.status.code().expect("an exit code"),
    )
}

/// R03 asks for the acceptance commands themselves, not only the readers behind them: both
/// binaries answer about the same run in the same sandbox, so the run id and every path in the
/// output are the same string and nothing has to be masked. `check request` and `req status`
/// only read, which is what lets one sandbox serve both.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_check_request_and_req_status_agree_on_a_wide_id_run() {
    let dir = run_sandbox("wide-id-verbs");
    let shell = shell_bin();
    let both_answer = |args: &[&str]| -> String {
        let (rust_out, rust_err, rust_code) = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, args);
        let (shell_out, shell_err, shell_code) = dstack(&shell, &dir, args);
        assert_eq!(rust_out, shell_out, "stdout of dstack {args:?}");
        assert_eq!(rust_err, shell_err, "stderr of dstack {args:?}");
        assert_eq!(rust_code, shell_code, "exit code of dstack {args:?}");
        rust_out
    };
    let checked = both_answer(&["check", "request"]);
    assert!(checked.contains("failures 0"), "{checked}");
    let status = both_answer(&["req", "status"]);
    assert!(status.contains("R4294967296"), "{status}");
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// The wide id itself: `req add --id R4294967296` is the call that has to parse an id past u32
/// (the shell folds it with `$((10#…))`), and until now only the shell was ever asked to mint it,
/// so the port could have mishandled the option and the fixture would still have looked right.
/// Both binaries mint the row into their own sandbox and the two stores have to agree.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_both_binaries_mint_the_same_wide_id() {
    let by_shell = run_sandbox_minted_by("wide-id-shell", &shell_bin());
    let by_rust = run_sandbox_minted_by("wide-id-rust", env!("CARGO_BIN_EXE_dstack"));
    for file in ["request.md", "cases.tsv"] {
        let shell_file = glob_run(&by_shell).join(file);
        let rust_file = glob_run(&by_rust).join(file);
        assert_eq!(
            std::fs::read(&shell_file).ok(),
            std::fs::read(&rust_file).ok(),
            "{file} differs after the wide row was minted"
        );
    }
    // And the two answer about their own store the same way, wide row and all. Each sandbox has
    // its own path and its own run id — the two values D-02 lets the implementations differ in.
    for args in [&["req", "status"][..], &["check", "request"]] {
        let (shell_out, _, shell_code) = dstack(&shell_bin(), &by_shell, args);
        let (rust_out, _, rust_code) = dstack(env!("CARGO_BIN_EXE_dstack"), &by_rust, args);
        assert_eq!(
            masked(&shell_out, &by_shell),
            masked(&rust_out, &by_rust),
            "stdout of dstack {args:?}"
        );
        assert_eq!(shell_code, rust_code, "exit code of dstack {args:?}");
    }
    // `req status` is the one of the two that prints the id itself.
    let (listed, _, _) = dstack(env!("CARGO_BIN_EXE_dstack"), &by_rust, &["req", "status"]);
    assert!(listed.contains("R4294967296"), "{listed}");
    std::fs::remove_dir_all(&by_shell).expect("clean up");
    std::fs::remove_dir_all(&by_rust).expect("clean up");
}

/// R03's acceptance clause names the commands themselves: status, run list, report, cases render
/// and plan render have to print the same thing from both binaries on the closed run
/// 20260902T052531Z_dstack-v2, and check request has to pass on it. The run is copied into a
/// scratch store per binary — plan render regenerates ROADMAP.md and STATE.md, so neither binary
/// is ever pointed at the repository's own store — and both copies start from the same bytes.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_the_acceptance_commands_agree_on_the_live_closed_run() {
    let run = match live_run() {
        Some(run) => run,
        None => {
            println!("skipped: the closed run {LIVE_RUN} is not in this machine's store");
            return;
        }
    };
    let by_shell = live_store("live-accept-shell", &run);
    let by_rust = live_store("live-accept-rust", &run);
    let mut checked = 0;
    for args in [
        &["status"][..],
        &["run", "list"],
        &["report", "--run", LIVE_RUN],
        &["cases", "render", "--run", LIVE_RUN],
        &["plan", "render", "--run", LIVE_RUN],
        &["check", "request", "--run", LIVE_RUN],
    ] {
        let (shell_out, shell_err, shell_code) = dstack(&shell_bin(), &by_shell, args);
        let (rust_out, rust_err, rust_code) = dstack(env!("CARGO_BIN_EXE_dstack"), &by_rust, args);
        assert_eq!(
            live_mask(&shell_out, &by_shell),
            live_mask(&rust_out, &by_rust),
            "stdout of dstack {args:?}"
        );
        assert_eq!(
            live_mask(&shell_err, &by_shell),
            live_mask(&rust_err, &by_rust),
            "stderr of dstack {args:?}"
        );
        assert_eq!(shell_code, rust_code, "exit code of dstack {args:?}");
        // Every one of these prints a table or a verdict; an empty pair would compare nothing.
        assert!(!rust_out.is_empty(), "dstack {args:?} printed nothing");
        checked += 1;
    }
    assert_eq!(checked, 6, "every command of the accept clause ran");
    // "and dstack check request passes on it".
    let (_out, err, code) = dstack(
        env!("CARGO_BIN_EXE_dstack"),
        &by_rust,
        &["check", "request", "--run", LIVE_RUN],
    );
    assert_eq!(code, 0, "check request on the live run: {err}");
    std::fs::remove_dir_all(&by_shell).expect("clean up");
    std::fs::remove_dir_all(&by_rust).expect("clean up");
}

/// A scratch store built by the reference dispatcher with the live run copied into it, so a verb
/// that writes (plan render) writes to the copy and both binaries read the same starting bytes.
fn live_store(tag: &str, run: &Path) -> PathBuf {
    let dir = scratch(tag);
    let dir = std::fs::canonicalize(&dir).expect("the physical path of the scratch directory");
    write(&dir.join(".deps.tsv"), DEPS);
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
    let (_out, err, code) = dstack(&shell_bin(), &dir, &["init"]);
    assert_eq!(code, 0, "init the {tag} store: {err}");
    let copied = Command::new("cp")
        .arg("-R")
        .arg(run)
        .arg(dir.join(".dstack/runs").join(LIVE_RUN))
        .status()
        .expect("run cp");
    assert!(copied.success(), "copy the live run into {tag}");
    dir
}

/// The scratch path and the values a copy of a run cannot share with its twin: the owner heartbeat
/// each binary stamps into meta.tsv while it resolves the target, and the pid behind it.
fn live_mask(text: &str, dir: &Path) -> String {
    let masked = text.replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>");
    masked
        .split_inclusive('\n')
        .map(|line| match line.len() >= 20 {
            true => mask_stamps(line),
            false => line.to_string(),
        })
        .collect()
}

/// `2026-09-02T21:10:43Z` → `<UTC>`: the stamp is fixed width, so the scan needs no regex.
fn mask_stamps(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < bytes.len() {
        let stamp = at + 20 <= bytes.len()
            && bytes[at].is_ascii_digit()
            && bytes[at + 4] == b'-'
            && bytes[at + 7] == b'-'
            && bytes[at + 10] == b'T'
            && bytes[at + 13] == b':'
            && bytes[at + 16] == b':'
            && bytes[at + 19] == b'Z';
        if stamp {
            out.push_str("<UTC>");
            at += 20;
            continue;
        }
        out.push(text[at..].chars().next().expect("a character"));
        at += text[at..].chars().next().expect("a character").len_utf8();
    }
    out
}

/// The sandbox path and the stamp of the run id, the two values the two sandboxes cannot share.
fn masked(text: &str, dir: &Path) -> String {
    let masked = text.replace(&dir.to_string_lossy().into_owned(), "<SANDBOX>");
    masked
        .split_inclusive('\n')
        .map(|line| match line.find("Z_wide-ids") {
            Some(at) if at >= 15 => format!("{}<RUNID>{}", &line[..at - 15], &line[at + 1..]),
            _ => line.to_string(),
        })
        .collect()
}

/// D-12: a stamp the verb cannot read is a cannot-decide, never a verdict. The shell diverges
/// here — `cat` prints "Permission denied" and the comparison against an empty stamp becomes a
/// hash mismatch with exit 1 — so this is one more reference defect the port does not reproduce,
/// like D-09. An unreadable file is used rather than a directory: `[ -f ]` and `is_file()` both
/// answer "not approved" for a directory, so that shape never reaches the stamp reader.
#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_an_unreadable_stamp_cannot_decide() {
    let dir = run_sandbox("unreadable-stamp");
    let shell = shell_bin();
    let (_out, err, code) = dstack(&shell, &dir, &["request", "approve"]);
    assert_eq!(code, 0, "the shell approves the fixture: {err}");
    let stamp = glob_run(&dir).join("request.approved");
    chmod(&stamp, "000");

    let (out, err, code) = dstack(env!("CARGO_BIN_EXE_dstack"), &dir, &["check", "request"]);
    chmod(&stamp, "600");
    assert_eq!(code, 2, "an unreadable stamp cannot be judged:\n{out}{err}");
    assert!(
        err.contains(&format!("cannot read {}", stamp.display())),
        "the refusal names the file: {err}"
    );
    assert!(
        !out.contains("hash mismatch"),
        "an unreadable stamp is not a mismatch: {out}"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// The one run directory of a sandbox.
fn glob_run(dir: &Path) -> PathBuf {
    let runs = dir.join(".dstack/runs");
    let entry = std::fs::read_dir(&runs)
        .expect("read the runs directory")
        .flatten()
        .next()
        .expect("one run");
    entry.path()
}

fn chmod(path: &Path, mode: &str) {
    let done = Command::new("chmod")
        .args([mode, &path.to_string_lossy()])
        .status()
        .expect("run chmod");
    assert!(done.success(), "chmod {mode} {path:?}");
}

#[test]
fn r03_tsv_cells_and_files_survive_a_round_trip() {
    let dir = scratch("tsv");
    let file = dir.join("table.tsv");
    assert_eq!(tsv::tsv_clean("a\tb\nc"), "a b c");
    assert_eq!(tsv::dash(""), "-");
    assert_eq!(tsv::dash("note"), "note");
    assert_eq!(tsv::undash("-"), "");
    assert_eq!(tsv::undash("note"), "note");

    assert_eq!(
        tsv::read_rows(&file, 1, false)
            .expect("the store file reads")
            .len(),
        0
    );
    tsv::append_line(&file, &["R01", "why", "2026-09-02T05:58:00Z"]).expect("append");
    tsv::append_line(&file, &["short"]).expect("append");
    assert_eq!(read(&file), "R01\twhy\t2026-09-02T05:58:00Z\nshort\n");
    assert_eq!(
        tsv::read_rows(&file, 3, false)
            .expect("the store file reads")
            .len(),
        1
    );
    assert_eq!(
        tsv::read_rows(&file, 1, true).expect("the store file reads"),
        vec![vec!["short".to_string()]]
    );

    tsv::rewrite(
        &file,
        Some("R\twhy\taccepted_at"),
        &[vec!["R02".to_string(), "-".to_string(), "-".to_string()]],
    )
    .expect("rewrite");
    assert_eq!(read(&file), "R\twhy\taccepted_at\nR02\t-\t-\n");
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_live_store_request_round_trips() {
    let run = match live_run() {
        Some(dir) => dir,
        None => return,
    };
    let source = run.join("request.md");
    let dir = scratch("live-request");
    let file = dir.join("request.md");
    std::fs::copy(&source, &file).expect("copy the live request");

    let doc = RequestDoc::load(&file).expect("load");
    assert_eq!(rows_as_shell_prints(&doc), shell("req_rows \"$1\"", &file));
    assert_eq!(
        doc.live_ids().join("\n") + "\n",
        shell("req_live_ids \"$1\"", &file)
    );
    let text = read(&file);
    let lines: Vec<&str> = text.lines().collect();
    for row in doc.rows() {
        assert_eq!(row.render(), lines[row.lineno - 1], "live row {}", row.id);
    }

    // Loading and saving without an edit changes nothing; one edit changes one line.
    let mut doc = RequestDoc::load(&file).expect("load");
    doc.save().expect("save");
    assert_eq!(read(&file), text);
    let id = doc.live_ids().first().expect("a live row").clone();
    doc.append_marker(&id, "deferred: parity check")
        .expect("marker");
    doc.save().expect("save");
    let edited = read(&file);
    assert_eq!(edited.lines().count(), text.lines().count());
    let changed: Vec<usize> = edited
        .lines()
        .zip(text.lines())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(changed.len(), 1, "exactly one line changed");
    assert!(!RequestDoc::load(&file)
        .expect("reload")
        .live_ids()
        .contains(&id));

    // The approval stamp of the live run is the shape request show matches against.
    let stamp = request::read_approval(&run)
        .expect("read the stamp")
        .expect("the live run is approved");
    assert_eq!(stamp.sha256.len(), 64);
    assert!(request::approval_matches(&run, &stamp.sha256).expect("read the stamp"));
    std::fs::remove_dir_all(&dir).expect("clean up");
}

// ── the ledgers: cases.tsv, accepts.tsv, metrics.tsv, the two tables, the review index ──────

/// A ledger holding every kind and every status the shell can write.
const CASES: &str = "R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote\n\
R01\tc1\tcli\tmet\t.dstack/local/artifacts/R01.txt\t9dff\tdstack report --metrics\t2026-09-02T05:57:49Z\t-\n\
R01\tc-test\ttest\topen\t-\t-\t-\t-\t-\n\
R02\tc1\tcapture\tabstain\tshots/R02.png\tab12\tdstack exec\t2026-09-02T05:58:00Z\towner decision\n\
R03\tc1\ttranscript\tblocked\tlog.jsonl\tcd34\tdstack exec\t2026-09-02T05:58:01Z\tblocked on a missing tool\n\
R04\tc1\tvisual\tskipped\t-\t-\tdstack cases sync\t2026-09-02T05:58:02Z\twithdrawn: the owner dropped it\n\
R05\tc-worker-P1\treview\tunreported\t-\t-\tdstack worker report --plan P1\t2026-09-02T05:58:03Z\tnot mentioned in the worker report for P1\n\
R06\tc1\tcli\tretired\told.txt\tef56\tdstack exec\t2026-09-02T05:58:04Z\tretired 2026-09-02T06:00:00Z: the artifact changed (was met, sha ef56)\n";

const QUESTIONS: &str = "# Questions (R51)\n\
\n\
Written only by `dstack ask`.\n\
\n\
| Q | Question | Affects | Status |\n\
|---|---|---|---|\n\
| Q-01 | Which repository is the benchmark? | R03 | answered |\n\
| Q-02 | Does the port keep the shell wording? | R04,design | assumed |\n\
| Q-03 | Who runs the harness? | R11 | open |\n";

const DECISIONS: &str = "# Decisions (R51)\n\
\n\
Written only by `dstack decision` and `dstack ask`.\n\
\n\
| D | Decision | Affects | Status |\n\
|---|---|---|---|\n\
| D-01 | The store layer owns one reader per format | R03 | answered |\n\
| D-02 | the comparison table carries the v1 column only (from Q-01: no benchmark repository) | R03 | answered |\n\
| D-18 | adopted default: the port keeps the shell wording (from Q-02) | R04,design | assumed |\n\
| D-DESIGN-01 | design round 1: cli work type: module boundaries (R55) — Design confirmed once for the cli work type | design | answered |\n";

const REVIEW_ROUND: &str = "# Codex review 001\n\
\n\
| R | verdict | why |\n\
|---|---|---|\n\
| R01 | covered | the test names it |\n\
| R02 | partial | the artifact proves half of it |\n\
| R03 | covered | the ledger row is real |\n\
\n\
VERDICT: approve\n";

/// The ledger as `cases_rows` prints it, padded to the nine columns awk counts.
const CASES_ROWS_SCRIPT: &str = r#"cases_rows "$1" | awk -F"\t" '{ printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n", $1,$2,$3,$4,$5,$6,$7,$8,$9 }'"#;

fn case_lines(rows: &[cases::CaseRow]) -> String {
    let mut out = String::new();
    for row in rows {
        out.push_str(&row.to_line());
        out.push('\n');
    }
    out
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_cases_ledger_reads_as_the_shell_reads_it() {
    let dir = scratch("cases-read");
    write(&dir.join("cases.tsv"), CASES);
    assert_eq!(
        case_lines(&cases::rows(&dir).expect("the store file reads")),
        shell(CASES_ROWS_SCRIPT, &dir)
    );
    assert_eq!(cases::rows(&dir).expect("the store file reads").len(), 7);
    assert_eq!(
        cases::for_r(&dir, "R01")
            .expect("the store file reads")
            .len(),
        2
    );
    assert_eq!(
        cases::status_of(&dir, "R01", "c1")
            .expect("the store file reads")
            .as_deref(),
        Some("met")
    );
    assert_eq!(
        cases::status_of(&dir, "R09", "c1").expect("the store file reads"),
        None
    );
    assert_eq!(
        shell(
            r#". "$DSTACK_LIB/cases.sh"; _cases_status_of "$1" R01 c1"#,
            &dir
        ),
        "met\n"
    );

    // Only met, abstain and blocked count as recorded evidence.
    assert!(cases::has_kind(&dir, "R01", &["cli"]).expect("the store file reads"));
    assert!(!cases::has_kind(&dir, "R01", &["test"]).expect("the store file reads"));
    assert!(cases::has_kind(&dir, "R03", &["capture", "transcript"]).expect("the store file reads"));
    assert!(!cases::has_kind(&dir, "R04", &["visual"]).expect("the store file reads"));
    assert!(!cases::has_kind(&dir, "R06", &["cli"]).expect("the store file reads"));
    assert_eq!(
        cases::count_status(&dir, "met").expect("the store file reads"),
        1
    );
    assert_eq!(
        cases::count_status(&dir, "open").expect("the store file reads"),
        1
    );
    assert_eq!(
        cases::count_status(&dir, "retired").expect("the store file reads"),
        1
    );
    assert_eq!(cases::CASES_KINDS.len(), 6);
    assert_eq!(cases::CASES_EVIDENCE_STATUSES.len(), 4);

    let request = dir.join("request.md");
    write(&request, REQUEST);
    let doc = RequestDoc::load(&request).expect("load");
    assert_eq!(cases::default_kind(&doc), "cli");
    write(&request, &REQUEST.replace("e2e: cli", "e2e: capture"));
    assert_eq!(
        cases::default_kind(&RequestDoc::load(&request).expect("load")),
        "capture"
    );
    write(&request, &REQUEST.replace("e2e: cli", "e2e: none"));
    assert_eq!(
        cases::default_kind(&RequestDoc::load(&request).expect("load")),
        "review"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_cases_writers_keep_the_ledger_format() {
    let dir = scratch("cases-write");
    let file = dir.join("cases.tsv");
    cases::ensure(&dir).expect("ensure");
    assert_eq!(read(&file), format!("{}\n", cases::CASES_HEADER));

    let open = cases::CaseRow {
        r: "R01".to_string(),
        case_id: "c1".to_string(),
        kind: "cli".to_string(),
        status: "open".to_string(),
        artifact: "-".to_string(),
        sha256: "-".to_string(),
        produced_by: "-".to_string(),
        recorded_at: "-".to_string(),
        note: "-".to_string(),
    };
    cases::append(&dir, &open).expect("append");
    cases::ensure(&dir).expect("the header is written once");
    let filled = cases::CaseRow {
        status: "met".to_string(),
        artifact: "out/R01.txt".to_string(),
        sha256: "ef56".to_string(),
        produced_by: tsv::tsv_clean("dstack exec\tR01"),
        recorded_at: "2026-09-02T06:00:00Z".to_string(),
        note: tsv::dash(""),
        ..open.clone()
    };
    cases::replace(&dir, "R01", "c1", &filled).expect("replace");
    assert_eq!(
        read(&file),
        format!(
            "{}\nR01\tc1\tcli\tmet\tout/R01.txt\tef56\tdstack exec R01\t2026-09-02T06:00:00Z\t-\n",
            cases::CASES_HEADER
        )
    );

    cases::retire(
        &dir,
        "R01",
        "c1",
        "retired 2026-09-02T07:00:00Z: wrong artifact (was met, sha ef56)",
    )
    .expect("retire");
    assert_eq!(
        read(&file),
        format!(
            "{}\nR01\tc1\tcli\tretired\tout/R01.txt\tef56\tdstack exec R01\t2026-09-02T06:00:00Z\tretired 2026-09-02T07:00:00Z: wrong artifact (was met, sha ef56)\n",
            cases::CASES_HEADER
        )
    );
    assert_eq!(
        case_lines(&cases::rows(&dir).expect("the store file reads")),
        shell(CASES_ROWS_SCRIPT, &dir)
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r03_accepts_and_metrics_keep_their_headers() {
    let dir = scratch("accepts");
    cases::accepts_append(
        &dir,
        "R01",
        "the owner accepted\tthe abstain",
        "2026-09-02T05:58:00Z",
    )
    .expect("accept");
    cases::accepts_append(&dir, "R02", "another reason", "2026-09-02T05:58:01Z").expect("accept");
    assert_eq!(
        read(&dir.join("accepts.tsv")),
        "R\twhy\taccepted_at\n\
R01\tthe owner accepted the abstain\t2026-09-02T05:58:00Z\n\
R02\tanother reason\t2026-09-02T05:58:01Z\n"
    );
    assert_eq!(
        cases::accepts_rows(&dir)
            .expect("the store file reads")
            .len(),
        2
    );
    assert_eq!(
        cases::accepts_why(&dir, "R01")
            .expect("the store file reads")
            .as_deref(),
        Some("the owner accepted the abstain")
    );
    assert_eq!(
        cases::accepts_why(&dir, "R09").expect("the store file reads"),
        None
    );

    let metrics = vec![
        cases::MetricRow {
            metric: "wall-clock".to_string(),
            value: "0h 33m 52s".to_string(),
            source: "meta.tsv".to_string(),
        },
        cases::MetricRow {
            metric: "coverage-rate".to_string(),
            value: "0/75 (0.0%)".to_string(),
            source: "dstack report (R79 table above)".to_string(),
        },
    ];
    cases::metrics_write(&dir, &metrics).expect("metrics");
    assert_eq!(
        read(&dir.join("metrics.tsv")),
        "metric\tvalue\tsource\n\
wall-clock\t0h 33m 52s\tmeta.tsv\n\
coverage-rate\t0/75 (0.0%)\tdstack report (R79 table above)\n"
    );
    assert_eq!(
        cases::metrics_rows(&dir)
            .expect("the store file reads")
            .len(),
        2
    );
    assert_eq!(
        cases::metrics_rows(&dir).expect("the store file reads")[1].value,
        "0/75 (0.0%)"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_question_ledger_round_trips() {
    let dir = scratch("questions");
    let file = dir.join("questions.md");
    write(&file, QUESTIONS);
    let asked = tables::questions(&file).expect("the store file reads");
    let mut printed = String::new();
    for q in &asked {
        printed.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            q.id, q.question, q.affects, q.status
        ));
    }
    assert_eq!(
        printed,
        shell(r#". "$DSTACK_LIB/ask.sh"; ask_q_rows "$1""#, &file)
    );
    assert_eq!(
        tables::q_count(&file, "open").expect("the store file reads"),
        1
    );
    assert_eq!(
        tables::q_count(&file, "answered").expect("the store file reads"),
        1
    );
    assert_eq!(
        tables::q_field(&file, "Q-02", 3)
            .expect("the store file reads")
            .as_deref(),
        Some("R04,design")
    );
    assert_eq!(
        tables::q_field(&file, "Q-02", 4)
            .expect("the store file reads")
            .as_deref(),
        Some("assumed")
    );
    assert_eq!(
        tables::q_field(&file, "Q-09", 2).expect("the store file reads"),
        None
    );
    assert_eq!(
        tables::q_next_id(&file).expect("the store file reads"),
        "Q-04"
    );

    tables::q_set_status(&file, "Q-03", "answered").expect("status");
    assert_eq!(
        read(&file),
        QUESTIONS.replace(
            "| Q-03 | Who runs the harness? | R11 | open |",
            "| Q-03 | Who runs the harness? | R11 | answered |"
        )
    );
    assert!(tables::q_set_status(&file, "Q-09", "answered").is_err());

    let fresh = dir.join("fresh.md");
    tables::q_append(&fresh, "Q-01", "The first question?", "R01", "open").expect("append");
    assert_eq!(
        read(&fresh),
        format!(
            "# Questions (R51)\n\nWritten only by `dstack ask`.\n\n{}\n| Q-01 | The first question? | R01 | open |\n",
            tables::ASK_HEADER
        )
    );
    assert_eq!(
        tables::q_next_id(&fresh).expect("the store file reads"),
        "Q-02"
    );
    assert_eq!(
        tables::q_text_ok("question", "a | b")
            .expect_err("pipe")
            .message(),
        "question must not contain '|': the ledger is a markdown table (a | b)"
    );
    assert_eq!(
        tables::q_text_ok("--affects", "")
            .expect_err("empty")
            .message(),
        "--affects must not be empty"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_decision_ledger_round_trips() {
    let dir = scratch("decisions");
    let file = dir.join("decisions.md");
    write(&file, DECISIONS);
    let decided = tables::decisions(&file).expect("the store file reads");
    let mut printed = String::new();
    for d in &decided {
        printed.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            d.id, d.text, d.affects, d.status
        ));
    }
    assert_eq!(
        printed,
        shell(r#". "$DSTACK_LIB/decision.sh"; dec_rows "$1""#, &file)
    );
    assert_eq!(decided.len(), 4);
    assert_eq!(
        tables::d_design_rows(&file)
            .expect("the store file reads")
            .len(),
        1
    );
    assert_eq!(
        tables::d_next_id(&file, false).expect("the store file reads"),
        "D-19"
    );
    assert_eq!(
        tables::d_next_id(&file, true).expect("the store file reads"),
        "D-DESIGN-02"
    );
    assert_eq!(
        tables::d_has_for_q(&file, "Q-01")
            .expect("the store file reads")
            .as_deref(),
        Some("D-02")
    );
    assert_eq!(
        tables::d_has_for_q(&file, "Q-09").expect("the store file reads"),
        None
    );
    assert_eq!(
        tables::d_design_reason(&decided[3].text),
        "cli work type: module boundaries (R55)"
    );
    assert_eq!(tables::d_design_reason("a plain decision"), "");

    let fresh = dir.join("fresh.md");
    tables::d_append(&fresh, "D-01", "The first decision", "R01", "answered").expect("append");
    assert_eq!(
        read(&fresh),
        format!(
            "# Decisions (R51)\n\nWritten only by `dstack decision` and `dstack ask`.\n\n{}\n| D-01 | The first decision | R01 | answered |\n",
            tables::DEC_HEADER
        )
    );
    assert_eq!(
        tables::d_next_id(&fresh, false).expect("the store file reads"),
        "D-02"
    );
    assert_eq!(
        tables::d_next_id(&fresh, true).expect("the store file reads"),
        "D-DESIGN-01"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_review_index_counts_the_sealed_rounds() {
    let dir = scratch("review");
    let review = dir.join("review");
    std::fs::create_dir_all(&review).expect("review dir");
    for round in ["001", "002", "003"] {
        write(
            &review.join(format!("codex-review-{round}.md")),
            REVIEW_ROUND,
        );
    }
    write(
        &review.join("index.tsv"),
        "001\tplan\tP1\tcodex-review-001.md\t2026-09-02T05:00:00Z\t0\t1\t4\n\
002\tplan\tP1\tcodex-review-002.md\t2026-09-02T06:00:00Z\t0\t0\t5\n\
003\tmilestone\tM1\tcodex-review-003.md\t2026-09-02T07:00:00Z\t1\t0\t3\n",
    );

    let rows = review_index::index_rows(&dir).expect("the store file reads");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1].filename, "codex-review-002.md");
    assert_eq!(rows[2].absent, "1");
    assert_eq!(review_index::next_seq(&review, "codex-review-"), "004");
    assert_eq!(
        shell(
            r#". "$DSTACK_LIB/rounds.sh"; _next_seq "$1/review" 'codex-review-'"#,
            &dir
        ),
        "004\n"
    );
    let (rounds, absent, partial, covered) =
        review_index::sealed_counts(&dir, "P1").expect("the store file reads");
    assert_eq!(
        (rounds, absent.as_str(), partial.as_str(), covered.as_str()),
        (2, "0", "0", "5")
    );
    assert_eq!(
        format!("{rounds} {absent} {partial} {covered}\n"),
        shell(r#". "$DSTACK_LIB/rounds.sh"; _sealed_counts "$1" P1"#, &dir)
    );
    let (rounds, absent, partial, covered) =
        review_index::sealed_counts(&dir, "P9").expect("the store file reads");
    assert_eq!(
        (rounds, absent.as_str(), partial.as_str(), covered.as_str()),
        (0, "-", "-", "-")
    );

    let round_file = review.join("codex-review-001.md");
    assert_eq!(
        review_index::verdict_count(&round_file, "covered").expect("the store file reads"),
        2
    );
    assert_eq!(
        review_index::verdict_count(&round_file, "partial").expect("the store file reads"),
        1
    );
    assert_eq!(
        review_index::verdict_count(&round_file, "absent").expect("the store file reads"),
        0
    );
    assert_eq!(
        shell(
            r#". "$DSTACK_LIB/rounds.sh"; _verdict_count "$1/review/codex-review-001.md" covered"#,
            &dir
        ),
        "2\n"
    );

    review_index::index_append(
        &dir,
        &review_index::IndexRow {
            round: "004".to_string(),
            scope: "plan".to_string(),
            id: "P2".to_string(),
            filename: "codex-review-004.md".to_string(),
            timestamp: "2026-09-02T08:00:00Z".to_string(),
            absent: "0".to_string(),
            partial: "0".to_string(),
            covered: "2".to_string(),
        },
    )
    .expect("index append");
    assert_eq!(
        review_index::index_rows(&dir)
            .expect("the store file reads")
            .len(),
        4
    );
    assert_eq!(
        review_index::latest_round(&dir, "plan", "P1").expect("the store file reads"),
        "002"
    );
    assert_eq!(
        review_index::latest_round(&dir, "plan", "P9").expect("the store file reads"),
        "000"
    );

    assert!(!review_index::is_closed(&dir, "plan", "P1", "002").expect("the store file reads"));
    review_index::closed_append(
        &dir,
        "plan",
        "P1",
        "002",
        "R01,R02",
        "2026-09-02T09:00:00Z",
        "the owner\tstopped the review",
    )
    .expect("close");
    assert_eq!(
        read(&review.join("closed.tsv")),
        "plan\tP1\t002\tR01,R02\t2026-09-02T09:00:00Z\tthe owner stopped the review\n"
    );
    assert!(review_index::is_closed(&dir, "plan", "P1", "002").expect("the store file reads"));
    assert!(!review_index::is_closed(&dir, "plan", "P1", "003").expect("the store file reads"));
    assert_eq!(
        review_index::closed_rows(&dir).expect("the store file reads")[0].ids,
        "R01,R02"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03_live_store_tables_round_trip() {
    let run = match live_run() {
        Some(dir) => dir,
        None => return,
    };
    let dir = scratch("live-tables");

    // cases.tsv of this run carries rows whose produced_by held a newline before _tsv_clean was
    // applied, so the shell reader drops the continuation lines; what must survive a rewrite is
    // the ledger as that reader sees it.
    assert_eq!(
        case_lines(&cases::rows(&run).expect("the store file reads")),
        shell(CASES_ROWS_SCRIPT, &run)
    );
    // Retiring a row of the live ledger has to produce the bytes the awk of evidence.sh
    // produces, extra columns and all: line 90 (R105) carries tabs inside produced_by and the
    // deps row of R105's fixture is eight columns long, so both shapes are real here.
    let source = run.join("cases.tsv");
    for (r, case_id) in [
        ("R105", "c1"),
        ("nope", "command -v definitely-missing-xyz"),
    ] {
        let file = dir.join("cases.tsv");
        std::fs::copy(&source, &file).expect("copy the live ledger");
        let note = "retired 2026-09-02T09:00:00Z: parity (was met, sha 37a6ace6)";
        cases::retire(&dir, r, case_id, note).expect("retire");
        let produced = std::fs::read(&file).expect("read the rewrite");
        let expected = awk_retire(&source, r, case_id, note);
        assert_eq!(
            String::from_utf8_lossy(&produced),
            String::from_utf8_lossy(&expected),
            "retiring {r}/{case_id} of the live ledger"
        );
        assert_eq!(produced, expected, "byte for byte");
        assert_ne!(
            produced,
            std::fs::read(&source).expect("read the live ledger"),
            "the retire found {r}/{case_id} and changed the file"
        );
    }
    std::fs::remove_file(dir.join("cases.tsv")).expect("clean up the ledger copy");

    // accepts.tsv and metrics.tsv are three clean columns: byte for byte.
    let accepts = cases::accepts_rows(&run).expect("the store file reads");
    assert!(!accepts.is_empty());
    for row in &accepts {
        cases::accepts_append(&dir, &row.r, &row.why, &row.accepted_at).expect("accept");
    }
    assert_eq!(
        read(&dir.join("accepts.tsv")),
        read(&run.join("accepts.tsv"))
    );

    let metrics = cases::metrics_rows(&run).expect("the store file reads");
    assert_eq!(metrics.len(), 6);
    cases::metrics_write(&dir, &metrics).expect("metrics");
    assert_eq!(
        read(&dir.join("metrics.tsv")),
        read(&run.join("metrics.tsv"))
    );

    // decisions.md: every table row renders back exactly as it stands in the file.
    let file = run.join("decisions.md");
    let decided = tables::decisions(&file).expect("the store file reads");
    assert!(decided.len() > 40);
    let table: Vec<String> = read(&file)
        .lines()
        .filter(|line| line.starts_with("| D-"))
        .map(|line| line.to_string())
        .collect();
    assert_eq!(decided.len(), table.len());
    for (row, line) in decided.iter().zip(table.iter()) {
        assert_eq!(
            format!(
                "| {} | {} | {} | {} |",
                row.id, row.text, row.affects, row.status
            ),
            *line
        );
    }
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// The awk program evidence.sh runs to retire a row (`$4` and `$9` on the record, everything
/// else printed as it stands). It is the reference the Rust writer is compared against.
fn awk_retire(file: &Path, r: &str, case_id: &str, note: &str) -> Vec<u8> {
    let out = Command::new("awk")
        .arg("-F\t")
        .args([
            "-v",
            &format!("r={r}"),
            "-v",
            &format!("c={case_id}"),
            "-v",
            &format!("n={note}"),
        ])
        .arg(
            "BEGIN{OFS=\"\\t\"}\n\
             NR==1 { print; next }\n\
             $1==r && $2==c { $4=\"retired\"; $9=n; print; next }\n\
             { print }",
        )
        .arg(file)
        .output()
        .expect("run awk");
    assert!(out.status.success(), "awk failed");
    out.stdout
}

/// A ledger with a twelve-column row (tabs inside produced_by) and a seven-column row: the two
/// shapes an in-place rewrite must not reshape.
const RAGGED: &str = "R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote\n\
R01\tc1\tcli\tmet\tout/R01.txt\tef56\tfor t in a b; do echo $t\tdone\t2026-09-02T06:00:00Z\t-\textra1\textra2\n\
R02\tc1\tcli\tmet\tout/R02.txt\tab12\tdstack exec\n\
R03\tc1\tcli\topen\t-\t-\t-\t-\t-\n";

#[test]
fn r03_retire_rewrites_the_record_as_awk_does() {
    for (r, case_id) in [("R01", "c1"), ("R02", "c1")] {
        let dir = scratch(&format!("retire-{r}"));
        let file = dir.join("cases.tsv");
        write(&file, RAGGED);
        let note = "retired 2026-09-02T09:00:00Z: the artifact changed (was met, sha ef56ab12)";
        cases::retire(&dir, r, case_id, note).expect("retire");
        let reference = dir.join("reference.tsv");
        write(&reference, RAGGED);
        let produced = std::fs::read(&file).expect("read the rewrite");
        let expected = awk_retire(&reference, r, case_id, note);
        assert_eq!(
            String::from_utf8_lossy(&produced),
            String::from_utf8_lossy(&expected),
            "retiring {r} against the awk of evidence.sh"
        );
        assert_eq!(produced, expected, "byte for byte");
        std::fs::remove_dir_all(&dir).expect("clean up");
    }
}

/// The awk program evidence.sh runs when `evidence add` fills an open row: the whole record is
/// replaced by the new nine-column row, so columns past the ninth go away — on purpose.
fn awk_replace(file: &Path, r: &str, case_id: &str, row: &str) -> Vec<u8> {
    let out = Command::new("awk")
        .arg("-F\t")
        .args([
            "-v",
            &format!("r={r}"),
            "-v",
            &format!("c={case_id}"),
            "-v",
            &format!("new={row}"),
        ])
        .arg(
            "NR==1 { print; next }\n\
             $1==r && $2==c { print new; next }\n\
             { print }",
        )
        .arg(file)
        .output()
        .expect("run awk");
    assert!(out.status.success(), "awk failed");
    out.stdout
}

#[test]
fn r03_replace_rewrites_the_record_as_awk_does() {
    let dir = scratch("replace-ragged");
    let file = dir.join("cases.tsv");
    let reference = dir.join("reference.tsv");
    write(&file, RAGGED);
    write(&reference, RAGGED);
    let row = cases::CaseRow {
        r: "R01".to_string(),
        case_id: "c1".to_string(),
        kind: "cli".to_string(),
        status: "met".to_string(),
        artifact: "out/R01.txt".to_string(),
        sha256: "ef56".to_string(),
        produced_by: "dstack exec".to_string(),
        recorded_at: "2026-09-02T06:00:00Z".to_string(),
        note: "-".to_string(),
    };
    cases::replace(&dir, "R01", "c1", &row).expect("replace");
    let produced = std::fs::read(&file).expect("read the rewrite");
    let expected = awk_replace(&reference, "R01", "c1", &row.to_line());
    assert_eq!(
        String::from_utf8_lossy(&produced),
        String::from_utf8_lossy(&expected)
    );
    assert_eq!(produced, expected, "byte for byte");
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// A ledger whose produced_by holds the two-character sequences backslash-t and backslash-n.
const BACKSLASH_LEDGER: &str =
    "R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote\n\
R01\tc1\tcli\tmet\tout/R01.txt\tef56\tprintf 'a\\tb\\n'\t2026-09-02T06:00:00Z\t-\n\
R02\tc1\tcli\topen\t-\t-\t-\t-\t-\n";

// D-09: strict parity stops at a reference defect. The shell hands the replacement row and the
// retire note to `awk -v`, which expands backslash-n and backslash-t into a real newline and tab
// — the very expansion that left multi-line produced_by cells in the v2 ledger — so the Rust
// writers keep both characters literally and this test pins the divergence on both sides.
#[test]
fn r03_backslash_sequences_are_written_literally_by_decision_d09() {
    let dir = scratch("d09");
    let file = dir.join("cases.tsv");
    let reference = dir.join("reference.tsv");
    write(&file, BACKSLASH_LEDGER);
    write(&reference, BACKSLASH_LEDGER);

    // retire: the note travels through -v n in evidence.sh:225.
    let note = "retired 2026-09-02T09:00:00Z: printf 'a\\tb\\n' (was met, sha ef56)";
    cases::retire(&dir, "R01", "c1", note).expect("retire");
    let produced = read(&file);
    assert_eq!(
        produced.lines().count(),
        BACKSLASH_LEDGER.lines().count(),
        "no row grew a line"
    );
    let cells: Vec<&str> = produced
        .lines()
        .nth(1)
        .expect("the row")
        .split('\t')
        .collect();
    assert_eq!(cells[8], note, "the note is stored as it was given");
    assert_eq!(
        cells[6], "printf 'a\\tb\\n'",
        "a cell that already held the sequences is untouched"
    );

    let expanded = String::from_utf8(awk_retire(&reference, "R01", "c1", note)).expect("utf-8");
    assert_eq!(
        expanded.lines().count(),
        produced.lines().count() + 1,
        "awk -v turned backslash-n into a real newline and split the record"
    );
    assert!(
        expanded.contains("printf 'a\tb"),
        "awk -v turned backslash-t into a real tab"
    );
    assert_ne!(
        expanded, produced,
        "the two writers disagree, by decision D-09"
    );

    // replace: the whole row travels through -v new in evidence.sh:151.
    write(&file, BACKSLASH_LEDGER);
    let row = cases::CaseRow {
        r: "R02".to_string(),
        case_id: "c1".to_string(),
        kind: "cli".to_string(),
        status: "met".to_string(),
        artifact: "out/R02.txt".to_string(),
        sha256: "ab12".to_string(),
        produced_by: "printf 'c\\td\\n'".to_string(),
        recorded_at: "2026-09-02T07:00:00Z".to_string(),
        note: "-".to_string(),
    };
    cases::replace(&dir, "R02", "c1", &row).expect("replace");
    let produced = read(&file);
    assert_eq!(produced.lines().count(), BACKSLASH_LEDGER.lines().count());
    let cells: Vec<&str> = produced
        .lines()
        .nth(2)
        .expect("the row")
        .split('\t')
        .collect();
    assert_eq!(cells[6], "printf 'c\\td\\n'");

    let expanded =
        String::from_utf8(awk_replace(&reference, "R02", "c1", &row.to_line())).expect("utf-8");
    assert_eq!(
        expanded.lines().count(),
        produced.lines().count() + 1,
        "awk -v split the replacement row too"
    );
    assert!(expanded.contains("printf 'c\td"));
    assert_ne!(expanded, produced);
    std::fs::remove_dir_all(&dir).expect("clean up");
}
