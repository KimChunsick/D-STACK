// tests/r09_lib_size.rs
// R09: the per-file size rule of the repository carried over to Rust. `dstack doctor` prints the
// lib layout table over dstack-cli/src/**/*.rs — one row per file, its line count and the
// responsibility line 2 names — and the lib-size checker rejects a .rs fixture that is over it.
#![allow(non_snake_case)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::OnceLock;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::verbs;

const MAX: usize = 350;
const HEADER: &str = "lib layout (≤ 350 lines per file): file | lines | responsibility";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The whole sweep runs once: it reads the repository, which no test here writes to.
fn doctor() -> &'static String {
    static ONCE: OnceLock<String> = OnceLock::new();
    ONCE.get_or_init(|| {
        let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
            .arg("doctor")
            .current_dir(repo())
            .output()
            .expect("run dstack doctor");
        String::from_utf8(out.stdout).expect("utf-8")
    })
}

/// The lib layout section: the rows between its header and its count line.
fn rows() -> Vec<String> {
    doctor()
        .lines()
        .skip_while(|line| *line != HEADER)
        .skip(1)
        .take_while(|line| !line.starts_with("  files "))
        .map(String::from)
        .collect()
}

fn count_line() -> String {
    doctor()
        .lines()
        .skip_while(|line| *line != HEADER)
        .find(|line| line.starts_with("  files "))
        .expect("the count line of the lib layout section")
        .to_string()
}

/// Every .rs file under dstack-cli/src as its path below src, in name order.
fn source_files() -> Vec<String> {
    let src = repo().join("dstack-cli/src");
    let mut files = Vec::new();
    collect(&src, &src, &mut files);
    files.sort();
    files
}

fn collect(at: &Path, root: &Path, files: &mut Vec<String>) {
    for entry in std::fs::read_dir(at)
        .expect("read the source tree")
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, root, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(
                path.strip_prefix(root)
                    .expect("below src")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
}

#[test]
fn r09__the_layout_table_has_one_row_per_rust_source_file() {
    let files = source_files();
    let rows = rows();
    let named: Vec<String> = rows
        .iter()
        .map(|row| {
            row.trim_start()
                .split(" | ")
                .next()
                .expect("the file column")
                .to_string()
        })
        .collect();
    assert_eq!(named, files, "the table and the source tree disagree");
}

#[test]
fn r09__every_row_carries_the_line_count_and_the_responsibility() {
    for row in rows() {
        // The responsibility carries " | " of its own (core/target.rs does), so the row splits
        // into three columns at most.
        let column: Vec<&str> = row.trim_start().splitn(3, " | ").collect();
        assert_eq!(column.len(), 3, "unexpected row: {row}");
        let lines: usize = column[1].parse().expect("a line count");
        let path = repo().join("dstack-cli/src").join(column[0]);
        let text = std::fs::read_to_string(&path).expect("read the source");
        assert_eq!(lines, text.matches('\n').count(), "line count of {row}");
        assert!(lines <= MAX, "{row} is over the limit");
        let second = text.lines().nth(1).unwrap_or_default();
        assert_eq!(
            column[2],
            second.trim_start_matches('/').trim_start(),
            "the responsibility of {row} is not line 2 of the file"
        );
    }
}

#[test]
fn r09__the_count_line_reports_nothing_over_the_limit() {
    let files = source_files();
    let total: usize = files
        .iter()
        .map(|name| {
            std::fs::read_to_string(repo().join("dstack-cli/src").join(name))
                .expect("read the source")
                .matches('\n')
                .count()
        })
        .sum();
    assert_eq!(
        count_line(),
        format!("  files {}, lines {total}, over the limit 0", files.len())
    );
}

/// The fixtures R09 asks for: a .rs file over the limit is rejected, a short one passes.
#[test]
fn r09__the_lib_size_checker_judges_rust_fixtures() {
    let home = Home::resolve().expect("the repository of this test binary");
    let fixtures = home.home.join("lint/fixtures/lib-size");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let checkers = verbs::all_selftests();
    let checker = checkers
        .iter()
        .find(|checker| checker.checker() == "lib-size")
        .expect("a registered lib-size checker");
    for (fixture, wanted) in [
        ("bad-too-long.rs", Verdict::Reject),
        ("good-short.rs", Verdict::Pass),
    ] {
        let verdict = checker
            .run(&mut ctx, &fixtures.join(fixture))
            .expect("the checker decides");
        assert_eq!(verdict, wanted, "{fixture}");
    }
}
