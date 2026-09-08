// Actual Git fixtures for the committed-range refusal boundary, also run by doctor --self.
use std::path::Path;
use std::process::Command;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};
use crate::store::plan::{Plan, Task};

use super::committed_range::CommittedRange;

pub struct CommittedReview;

impl Selftest for CommittedReview {
    fn checker(&self) -> &'static str {
        "review-bundle"
    }

    fn run(&self, _ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let case = std::fs::read_to_string(fixture).map_err(io_error)?;
        let case = case.lines().next().unwrap_or_default();
        if !["complete", "gap", "dirty", "undeclared-reverted"].contains(&case) {
            return Err(Error::cannot_decide("unknown committed review fixture"));
        }
        let sandbox = Sandbox::scratch()?;
        let wt = &sandbox.dir;
        std::fs::write(wt.join(".git/info/exclude"), ".deps.tsv\n").map_err(io_error)?;
        std::fs::write(wt.join("allowed.txt"), "first\n").map_err(io_error)?;
        let first = commit(wt)?;
        if case == "gap" {
            std::fs::write(wt.join("allowed.txt"), "unrecorded\n").map_err(io_error)?;
            commit(wt)?;
        }
        let mut commits = vec![first];
        if case == "undeclared-reverted" {
            std::fs::write(wt.join("outside.txt"), "outside\n").map_err(io_error)?;
            commits.push(commit(wt)?);
            std::fs::remove_file(wt.join("outside.txt")).map_err(io_error)?;
        }
        std::fs::write(wt.join("allowed.txt"), "last\n").map_err(io_error)?;
        commits.push(commit(wt)?);
        if case == "dirty" {
            std::fs::write(wt.join("untracked.txt"), "dirty\n").map_err(io_error)?;
        }
        let plan = Plan {
            id: "P1".into(),
            milestone: "M1".into(),
            slug: "fixture".into(),
            files: vec!["allowed.txt".into()],
            deps: vec![],
            status: "in-progress".into(),
            worktree: String::new(),
            started_at: String::new(),
            done_at: String::new(),
            tasks: commits
                .into_iter()
                .enumerate()
                .map(|(i, commit)| Task {
                    id: format!("T{i}"),
                    slug: "fixture".into(),
                    covers: vec!["R21".into()],
                    files: vec!["allowed.txt".into()],
                    deps: vec![],
                    commit,
                    done_at: "fixture".into(),
                })
                .collect(),
        };
        match CommittedRange::derive(wt, &plan).and_then(|r| r.emit(&mut Vec::new(), wt)) {
            Ok(_) => Ok(Verdict::Pass),
            Err(Error::Failed(_)) => Ok(Verdict::Reject),
            Err(error) => Err(error),
        }
    }
}

fn commit(wt: &Path) -> Result<String> {
    git(wt, &["add", "."])?;
    git(
        wt,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.test",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "fixture",
        ],
    )?;
    git(wt, &["rev-parse", "HEAD"])
}

fn git(wt: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(wt)
        .args(args)
        .output()
        .map_err(io_error)?;
    if !out.status.success() {
        return Err(Error::cannot_decide(format!(
            "committed fixture Git failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
}

fn io_error(error: std::io::Error) -> Error {
    Error::cannot_decide(format!("committed review fixture: {error}"))
}
