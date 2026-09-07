#![allow(non_snake_case)]
#[path = "support/mode_settings.rs"]
mod support;

use dstack_cli::core::fsx::sha256_bytes;
use serde_json::json;
use std::fs;
use std::process::{Command, Output};
use support::Scratch;

const RUN: &str = ".dstack/runs/sample";
const CEILING: usize = 512000;
// R14 acceptance is kept verbatim; the fixture covers the CLI behavior, not the live Goal.
const ROW: &str = "- [ ] **R14** 리뷰 묶음이 큰 선언 파일의 변경 내용을 조용히 누락하지 않게 해요. 전체 크기 상한 안에서는 변경 전문을 포함하고 상한을 넘으면 불완전한 묶음을 내보내지 않고 거부해요. — accept: 64KB를 넘는 선언 파일의 처음과 마지막 변경이 모두 포함된 묶음이 검사 종료 코드 0으로 생성되고, 512KB 전체 상한과 선언 파일 경계는 유지돼요. 상한을 넘는 경우 종료 코드가 0이 아니고 불완전한 묶음이 생기지 않아요. 실제 연구 Goal P11.1 묶음에 factory_review.py 변경 전문이 포함돼요.";

fn record(s: &Scratch, program: &str, args: &[&str], output: &Output) {
    eprintln!(
        "R14 cwd: {}\nargv: {program} {args:?}\nstdout:\n{}stderr:\n{}exit: {:?}",
        s.0.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        output.status.code()
    );
}

fn git(s: &Scratch, args: &[&str]) -> Output {
    let output = Command::new("git")
        .current_dir(&s.0)
        .args(args)
        .output()
        .unwrap();
    // Diff bytes are checked and hashed below; do not dump hundreds of KB into the transcript.
    eprintln!(
        "R14 git {args:?}: exit {:?}, stdout bytes {}, SHA256 {}, stderr {}",
        output.status.code(),
        output.stdout.len(),
        sha256_bytes(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn fixture(declared: &[&str], tracked: &[&str]) -> Scratch {
    let s = Scratch::new();
    assert!(git(&s, &["init", "-q"]).status.success());
    s.write(".gitignore", ".dstack/\n.deps.tsv\ndeps.tsv\nbundle*.txt\n");
    for path in tracked {
        s.write(path, "before\n");
    }
    s.write("outside.txt", "outside before\n");
    assert!(git(&s, &["add", "."]).status.success());
    assert!(git(
        &s,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.test",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "fixture base"
        ]
    )
    .status
    .success());
    let head = git(&s, &["rev-parse", "HEAD"]);
    assert!(head.status.success());
    s.write(".dstack/version", "2\n");
    s.write(
        &format!("{RUN}/meta.tsv"),
        &format!(
            "status\topen\nbase_head\t{}\n",
            String::from_utf8(head.stdout).unwrap().trim()
        ),
    );
    s.write(
        &format!("{RUN}/request.md"),
        &format!("# 요청해요\n\n{ROW}\n"),
    );
    s.write(&format!("{RUN}/request.approved"), "fixture\n");
    s.write(
        &format!("{RUN}/plan.json"),
        &json!({"v":2,
            "milestones":[{"id":"M1","slug":"fixture","order":1}],
            "plans":[{"id":"P1","milestone":"M1","slug":"fixture","files":declared,
                "deps":[],"status":"in-progress","worktree":"","started_at":"","done_at":"",
                "tasks":[{"id":"T1","slug":"fixture","covers":["R14"],"files":declared,
                    "deps":[],"commit":"","done_at":""}]}]
        })
        .to_string(),
    );
    s.write("outside.txt", "UNDECLARED_CHANGED_SENTINEL\n");
    s
}

fn large(tag: &str, size: usize) -> String {
    format!("{tag}_FIRST\n{}\n{tag}_LAST\n", "x".repeat(size))
}

fn snapshot(s: &Scratch, paths: &[&str]) -> Vec<(String, Vec<u8>)> {
    ["request.md", "request.approved", "plan.json", "meta.tsv"]
        .iter()
        .map(|name| format!("{RUN}/{name}"))
        .chain(paths.iter().map(|p| (*p).to_owned()))
        .map(|p| {
            let bytes = fs::read(s.0.join(&p)).unwrap();
            (p, bytes)
        })
        .collect()
}

fn generate(s: &Scratch, out: Option<&str>, paths: &[&str]) -> Output {
    let before = snapshot(s, paths);
    let mut args = vec![
        "review", "--run", "sample", "--scope", "plan", "--plan", "P1",
    ];
    if let Some(out) = out {
        args.extend(["--out", out]);
    }
    let result = s.run(&args);
    record(s, env!("CARGO_BIN_EXE_dstack"), &args, &result);
    for (path, bytes) in before {
        let after = fs::read(s.0.join(&path)).unwrap();
        eprintln!(
            "R14 unchanged {path}: {} -> {}",
            sha256_bytes(&bytes),
            sha256_bytes(&after)
        );
        assert_eq!(after, bytes, "source/frozen metadata changed: {path}");
    }
    result
}

fn expected_diff(s: &Scratch, path: &str, tracked: bool) -> Vec<u8> {
    let args = if tracked {
        vec!["diff", "HEAD", "--", path]
    } else {
        vec!["diff", "--no-index", "--", "/dev/null", path]
    };
    let diff = git(s, &args);
    assert_eq!(diff.status.code(), Some(if tracked { 0 } else { 1 }));
    assert!(!diff.stdout.is_empty());
    diff.stdout
}

fn checked(s: &Scratch, paths: &[&str]) -> Vec<u8> {
    assert!(generate(s, Some("bundle.txt"), paths).status.success());
    let args = ["check", "review-bundle", "bundle.txt", "--run", "sample"];
    let result = s.run(&args);
    record(s, env!("CARGO_BIN_EXE_dstack"), &args, &result);
    assert!(result.status.success());
    let bytes = fs::read(s.0.join("bundle.txt")).unwrap();
    assert!(bytes.len() <= CEILING);
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.starts_with(&format!(
        "=== REQUEST (frozen) ===\n{ROW}\n\n=== PLAN ===\n"
    )));
    assert!(!text.contains("UNDECLARED_CHANGED_SENTINEL"));
    eprintln!(
        "R14 bundle: bytes {}, SHA256 {}",
        bytes.len(),
        sha256_bytes(&bytes)
    );
    bytes
}

fn complete(bytes: &[u8], diff: &[u8]) {
    let start = bytes.windows(diff.len()).position(|window| window == diff);
    assert!(
        start.is_some(),
        "R14 full git diff ({} bytes, SHA256 {}) missing",
        diff.len(),
        sha256_bytes(diff)
    );
    let included = &bytes[start.unwrap()..start.unwrap() + diff.len()];
    assert_eq!(included, diff);
    assert_eq!(sha256_bytes(included), sha256_bytes(diff));
    eprintln!(
        "R14 complete first-to-last diff bytes and SHA256 match: {}",
        sha256_bytes(diff)
    );
}

#[test]
fn R14__tracked_large_diff_is_complete() {
    let s = fixture(&["large.txt"], &["large.txt"]);
    s.write("large.txt", &large("TRACKED", 70000));
    let diff = expected_diff(&s, "large.txt", true);
    assert!(diff.len() > 65536);
    complete(&checked(&s, &["large.txt", "outside.txt"]), &diff);
}

#[test]
fn R14__untracked_large_diff_is_complete() {
    let s = fixture(&["large.txt"], &[]);
    s.write("large.txt", &large("UNTRACKED", 70000));
    let diff = expected_diff(&s, "large.txt", false);
    assert!(diff.len() > 65536);
    complete(&checked(&s, &["large.txt", "outside.txt"]), &diff);
}

#[test]
fn R14__declared_directory_expands_complete_tracked_and_untracked_diffs() {
    let s = fixture(&["allowed"], &["allowed/tracked.txt"]);
    s.write("allowed/tracked.txt", &large("TRACKED", 80000));
    s.write("allowed/nested/untracked.txt", &large("UNTRACKED", 90000));
    s.write("allowed-neighbor.txt", "UNDECLARED_CHANGED_SENTINEL\n");
    let bytes = checked(
        &s,
        &[
            "allowed/tracked.txt",
            "allowed/nested/untracked.txt",
            "outside.txt",
            "allowed-neighbor.txt",
        ],
    );
    assert!(String::from_utf8_lossy(&bytes).contains("--- declared: allowed (2 changed file(s))"));
    for (path, tracked) in [
        ("allowed/tracked.txt", true),
        ("allowed/nested/untracked.txt", false),
    ] {
        let diff = expected_diff(&s, path, tracked);
        assert!(diff.len() > 65536);
        complete(&bytes, &diff);
    }
}

#[test]
fn R14__exact_total_ceiling_passes_and_one_extra_byte_publishes_nothing() {
    let s = fixture(&["large.txt"], &[]);
    s.write("large.txt", &large("BOUNDARY", 70000));
    let initial = checked(&s, &["large.txt", "outside.txt"]);
    let size = 70000 + CEILING - initial.len();
    s.write("large.txt", &large("BOUNDARY", size));
    let bytes = checked(&s, &["large.txt", "outside.txt"]);
    assert_eq!(
        bytes.len(),
        CEILING,
        "R14 the unchanged ceiling is inclusive"
    );
    complete(&bytes, &expected_diff(&s, "large.txt", false));
    s.write("large.txt", &large("BOUNDARY", size + 1));
    fs::remove_file(s.0.join("bundle.txt")).unwrap();
    fs::remove_dir_all(s.0.join(format!("{RUN}/review"))).unwrap();
    for out in [Some("bundle.txt"), None] {
        let result = generate(&s, out, &["large.txt", "outside.txt"]);
        assert_eq!(result.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&result.stdout).contains("512001 bytes (ceiling 512000)"));
        assert!(String::from_utf8_lossy(&result.stderr).contains("bundle exceeds 512KB"));
        assert!(!s.0.join("bundle.txt").exists());
        assert!(!s.0.join(format!("{RUN}/review")).exists());
    }
}

#[test]
fn R14__combined_small_diffs_still_obey_the_total_ceiling() {
    let s = fixture(&["allowed"], &[]);
    for n in 0..9 {
        let path = format!("allowed/file{n}.txt");
        s.write(&path, &large("COMBINED", 60000));
        assert!(expected_diff(&s, &path, false).len() < 65536);
    }
    let result = generate(&s, Some("bundle.txt"), &["outside.txt"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("bundle exceeds 512KB"));
    assert!(!s.0.join("bundle.txt").exists());
    assert!(!s.0.join(format!("{RUN}/review")).exists());
}
