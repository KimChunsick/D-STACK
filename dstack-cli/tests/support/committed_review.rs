#![allow(dead_code)]
#[path = "mode_settings.rs"]
mod settings;

use serde_json::{json, Value};
pub use settings::Scratch;
use std::fs;
use std::process::{Command, Output};

pub const RUN: &str = ".dstack/runs/sample";
pub const ROW: &str = "- [ ] **R21** 완료 기록의 변경 전문을 리뷰해요. — accept: 기록된 작업 커밋의 변경 전문을 포함해요.";

pub struct Repo {
    pub s: Scratch,
    pub base: String,
}

pub fn log(program: &str, args: &[&str], output: &Output) {
    eprintln!(
        "R21 argv: {program} {args:?}\nstdout:\n{}stderr:\n{}exit: {:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        output.status.code()
    );
}

impl Repo {
    pub fn new() -> Self {
        let mut r = Self {
            s: Scratch::new(),
            base: String::new(),
        };
        r.git(&["init", "-q"]);
        r.s.write(".gitignore", ".dstack/\ndeps.tsv\nbundle*.txt\n");
        r.s.write("allowed/a.txt", "initial\n");
        r.s.write("outside.txt", "initial\n");
        r.base = r.commit();
        r.s.write(".dstack/version", "2\n");
        r.s.write(
            &format!("{RUN}/meta.tsv"),
            &format!("status\topen\nbase_head\t{}\n", r.base),
        );
        r.s.write(
            &format!("{RUN}/request.md"),
            &format!("# 요청해요\n\n{ROW}\n"),
        );
        r.s.write(&format!("{RUN}/request.approved"), "fixture\n");
        r
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .current_dir(&self.s.0)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .args(args)
            .output()
            .unwrap();
        log("git", args, &out);
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap().trim_end().to_owned()
    }

    pub fn commit(&self) -> String {
        self.git(&["add", "."]);
        self.git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.test",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-qm",
            "fixture",
        ]);
        self.git(&["rev-parse", "HEAD"])
    }

    pub fn task(&self, text: &str) -> String {
        self.s.write("allowed/a.txt", text);
        self.commit()
    }

    pub fn plan(&self, commits: &[&str]) {
        let tasks: Vec<Value> = commits
            .iter()
            .enumerate()
            .map(|(i, sha)| {
                json!({
                    "id":format!("T{}", i + 1), "slug":"fixture", "covers":["R21"],
                    "files":["allowed"], "deps":[], "commit":sha, "done_at":"2026-09-08T00:00:00Z"
                })
            })
            .collect();
        self.save(
            &json!({"v":2, "milestones":[{"id":"M1","slug":"fixture","order":1}],
            "plans":[{"id":"P1","milestone":"M1","slug":"fixture","files":["allowed"],
                "deps":[],"status":"in-progress","worktree":"","started_at":"","done_at":"",
                "tasks":tasks}]}),
        );
    }

    pub fn edit_plan(&self, edit: impl FnOnce(&mut Value)) {
        let mut doc: Value =
            serde_json::from_str(&self.s.read(&format!("{RUN}/plan.json"))).unwrap();
        edit(&mut doc["plans"][0]);
        self.save(&doc);
    }

    fn save(&self, doc: &Value) {
        self.s.write(&format!("{RUN}/plan.json"), &doc.to_string());
    }

    pub fn run(&self, args: &[&str]) -> Output {
        let out = self.s.run(args);
        log(env!("CARGO_BIN_EXE_dstack"), args, &out);
        out
    }

    pub fn review(&self, committed: bool) -> Output {
        let before: Vec<_> = ["request.md", "request.approved", "plan.json", "meta.tsv"]
            .iter()
            .map(|name| (name, fs::read(self.s.0.join(format!("{RUN}/{name}"))).ok()))
            .collect();
        let mut args = vec![
            "review",
            "--run",
            "sample",
            "--scope",
            "plan",
            "--plan",
            "P1",
            "--out",
            "bundle.txt",
        ];
        if committed {
            args.push("--committed");
        }
        let out = self.run(&args);
        for (name, bytes) in before {
            assert_eq!(fs::read(self.s.0.join(format!("{RUN}/{name}"))).ok(), bytes);
        }
        out
    }

    pub fn checked(&self) -> String {
        assert!(self.review(true).status.success());
        assert!(self
            .run(&["check", "review-bundle", "bundle.txt", "--run", "sample"])
            .status
            .success());
        self.s.read("bundle.txt")
    }

    pub fn refused(&self, reason: &str) {
        let out = self.review(true);
        assert_eq!(out.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(reason),
            "expected {reason}"
        );
        assert!(!self.s.0.join("bundle.txt").exists());
        assert!(!self.s.0.join(format!("{RUN}/review")).exists());
    }
}
