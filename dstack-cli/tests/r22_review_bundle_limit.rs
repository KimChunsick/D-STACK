#![allow(non_snake_case)]
#[path = "support/committed_review.rs"]
mod support;

use std::fs;
use std::process::Command;
use support::{Repo, RUN};

const CEILING: usize = 1_024_000;
// R22 acceptance is kept verbatim; installed activation is verified after integration.
const ROW: &str = "- [ ] **R22** 리뷰 묶음의 전체 크기 상한을 기존 512KB에서 1024KB로 올려요. 기존 십진 바이트 기준을 유지해 1,024,000바이트까지 허용하고 초과 시 전문 보존과 불완전한 묶음 생성 거부를 유지해요. R14와 R21의 512KB 유지 조건은 이 요구사항의 새 상한으로 대체해요. — accept: cargo test R22에서 이전 상한 초과 입력과 1,024,000바이트 경계는 통과하고 경계보다 1바이트 큰 입력은 거부돼요. 일반 및 작업 커밋 기준 리뷰의 기존 보호 동작과 관련 회귀 시험, bash dstack-cli/test.sh, dstack doctor --self, dstack lint-ko --changed가 통과하고 설치된 dstack에도 반영돼요.";

fn payload(size: usize) -> String {
    format!("FIRST\n{}\nLAST\n", "x".repeat(size))
}

fn replace(r: &Repo, size: usize, committed: bool) {
    r.s.write("allowed/a.txt", &payload(size));
    if committed {
        r.git(&["add", "allowed/a.txt"]);
        r.git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.test",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--amend",
            "--no-edit",
            "-q",
        ]);
        r.plan(&[&r.git(&["rev-parse", "HEAD"])]);
        r.edit_plan(|p| p["tasks"][0]["covers"] = serde_json::json!(["R22"]));
    }
}

fn checked(r: &Repo, committed: bool, size: usize) -> usize {
    assert!(r.review(committed).status.success());
    assert!(r
        .run(&["check", "review-bundle", "bundle.txt", "--run", "sample"])
        .status
        .success());
    let bundle = r.s.read("bundle.txt");
    assert!(bundle.starts_with(&format!("=== REQUEST (frozen) ===\n{ROW}\n")));
    assert!(bundle.contains(&format!("+FIRST\n+{}\n+LAST\n", "x".repeat(size))));
    assert_eq!(r.s.read("allowed/a.txt"), payload(size));
    let mut args = vec!["diff"];
    if committed {
        args.extend(["--binary", "--full-index"]);
    }
    args.push(&r.base);
    if committed {
        args.push("HEAD");
    }
    args.extend(["--", "allowed/a.txt"]);
    let diff = Command::new("git")
        .current_dir(&r.s.0)
        .args(args)
        .output()
        .unwrap();
    assert!(diff.status.success());
    assert!(!diff.stdout.is_empty());
    assert!(bundle.contains(std::str::from_utf8(&diff.stdout).unwrap()));
    eprintln!(
        "R22 committed={committed}: complete checked bundle {} bytes",
        bundle.len()
    );
    bundle.len()
}

fn boundary(committed: bool) {
    let r = Repo::new();
    r.s.write(
        &format!("{RUN}/request.md"),
        &format!("# 요청해요\n\n{ROW}\n"),
    );
    let head = r.task(&payload(1000));
    r.plan(&[&head]);
    r.edit_plan(|p| p["tasks"][0]["covers"] = serde_json::json!(["R22"]));
    let overhead = checked(&r, committed, 1000) - 1000;
    for total in [600_000, CEILING] {
        let size = total - overhead;
        replace(&r, size, committed);
        assert_eq!(checked(&r, committed, size), total);
    }
    replace(&r, CEILING + 1 - overhead, committed);
    let source = r.s.read("allowed/a.txt");
    let previous = r.s.read("bundle.txt");
    let result = r.review(committed);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        r.s.read("bundle.txt"),
        previous,
        "existing output is preserved"
    );
    fs::remove_file(r.s.0.join("bundle.txt")).unwrap();
    fs::remove_dir_all(r.s.0.join(format!("{RUN}/review"))).unwrap();
    for explicit in [false, true] {
        let mut args = vec![
            "review", "--run", "sample", "--scope", "plan", "--plan", "P1",
        ];
        if committed {
            args.push("--committed");
        }
        if explicit {
            args.extend(["--out", "not-created/bundle.txt"]);
        }
        let out = r.run(&args);
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stdout).contains("1024001 bytes (ceiling 1024000)"));
        assert!(String::from_utf8_lossy(&out.stderr).contains("bundle exceeds 1024KB"));
        assert!(!r.s.0.join("not-created").exists());
        assert!(!r.s.0.join(format!("{RUN}/review")).exists());
        assert_eq!(r.s.read("allowed/a.txt"), source);
    }
}

#[test]
fn R22__ordinary_review_accepts_1024000_bytes_and_refuses_one_more() {
    boundary(false);
}

#[test]
fn R22__committed_review_accepts_1024000_bytes_and_refuses_one_more() {
    boundary(true);
}
