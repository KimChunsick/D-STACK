// verbs/ledger/evidence_selftests.rs
// The evidence-add and evidence-retire fixture checkers of the runner (R100).

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

use super::verdict;

/// A day is what `touch -t "$(_selftest_yesterday)"` puts between the fixture and the run.
const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

pub(super) struct EvidenceAdd;
pub(super) struct EvidenceRetire;

/// evidence-add: the fixture is the artifact itself. A zero-byte fixture cannot carry a
/// directive comment, so the two fixtures that need staging (an old mtime, a second R) are
/// recognised by name — the one place a name decides behaviour.
impl Selftest for EvidenceAdd {
    fn checker(&self) -> &'static str {
        "evidence-add"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let run_dir = sandbox.run_dir()?;
        sandbox.write_request(&run_dir)?;
        let _ = sandbox.dsx(ctx, &["cases", "sync"])?;
        let name = file_name(fixture);
        let artifact = sandbox.dir.join("artifacts").join(&name);
        copy(fixture, &artifact)?;
        // Repository fixtures are older than the sandbox run by definition; every fixture but
        // the mtime one is therefore re-stamped, otherwise all of them would fail the mtime rule.
        let now = SystemTime::now();
        let stamp = match name.starts_with("bad-old-mtime") {
            true => now - A_DAY,
            false => now,
        };
        set_mtime(&artifact, stamp)?;
        let path = artifact.to_string_lossy().into_owned();
        let mut code = add(&sandbox, ctx, "R01", &path)?;
        if name.starts_with("bad-shared-without-flag") {
            code = add(&sandbox, ctx, "R02", &path)?;
        }
        verdict(code, "dstack evidence add")
    }
}

/// The one call every evidence-add fixture is judged by.
fn add(sandbox: &Sandbox, ctx: &Context, r: &str, artifact: &str) -> Result<i32> {
    let (code, _) = sandbox.dsx(
        ctx,
        &[
            "evidence",
            "add",
            "--r",
            r,
            "--case",
            "c1",
            "--kind",
            "cli",
            "--artifact",
            artifact,
            "--produced-by",
            "selftest",
        ],
    )?;
    Ok(code)
}

/// evidence-retire: the fixture's first line is the status to retire from (met|open); "reject"
/// means the retire was refused.
impl Selftest for EvidenceRetire {
    fn checker(&self) -> &'static str {
        "evidence-retire"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let text = fs::read_to_string(fixture)
            .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", fixture.display())))?;
        let status: String = text
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let sandbox = Sandbox::new(ctx)?;
        let run_dir = sandbox.run_dir()?;
        sandbox.write_request(&run_dir)?;
        let _ = sandbox.dsx(ctx, &["cases", "sync"])?;
        if status == "met" {
            let artifact = sandbox.artifact("r01.txt", "R01 ok")?;
            add(&sandbox, ctx, "R01", &artifact.to_string_lossy())?;
        }
        let (code, _) = sandbox.dsx(
            ctx,
            &[
                "evidence", "retire", "--r", "R01", "--case", "c1", "--why", "fixture",
            ],
        )?;
        verdict(code, "dstack evidence retire")
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn copy(fixture: &Path, artifact: &Path) -> Result<()> {
    let dir = artifact.parent().unwrap_or(artifact);
    fs::create_dir_all(dir)
        .and_then(|()| fs::copy(fixture, artifact))
        .map_err(|e| {
            Error::cannot_decide(format!(
                "selftest: cannot stage {}: {e}",
                artifact.display()
            ))
        })?;
    Ok(())
}

/// `touch` and `touch -t`: the mtime is set through the open file, so the checker spawns nothing.
fn set_mtime(artifact: &Path, stamp: SystemTime) -> Result<()> {
    fs::File::options()
        .write(true)
        .open(artifact)
        .and_then(|file| file.set_modified(stamp))
        .map_err(|e| {
            Error::cannot_decide(format!(
                "selftest: cannot stamp {}: {e}",
                artifact.display()
            ))
        })
}
