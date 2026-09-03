// verbs/run_view.rs
// run list and run verify: the read-only views of the run store.

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::roots::git_out;
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::cases;
use crate::store::request::RequestDoc;
use crate::core::paths::base_name;
use crate::verbs::run::{cannot, field, run_dirs};

struct RunList;

impl Verb for RunList {
    fn name(&self) -> &'static str {
        "run list"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        list(ctx, args)
    }
}

struct RunVerify;

impl Verb for RunVerify {
    fn name(&self) -> &'static str {
        "run verify"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        verify(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(RunList), Box::new(RunVerify)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![]
}

fn list(ctx: &mut Context, _args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let mut runs = 0;
    ctx.out.say("id | status | worktree | branch | opened | closed | R | met");
    for dir in run_dirs(&roots) {
        if !dir.join("meta.tsv").is_file() { continue }
        runs += 1;
        // A run without a request.md counts no rows; one whose request.md or ledger cannot be
        // read is a cannot-decide (D-12), not a run of zero rows and zero met cases.
        let request = dir.join("request.md");
        let rows = match request.is_file() {
            true => RequestDoc::load(&request)?.rows().len(),
            false => 0,
        };
        let met = cases::rows(&dir)?.iter().filter(|c| c.status == "met").count();
        let cells: Vec<String> = ["status", "worktree", "branch", "started_at", "closed_at"]
            .iter().map(|key| field(&dir, key)).collect::<Result<Vec<String>>>()?;
        ctx.out.say(&format!("{} | {} | {rows} | {met}", base_name(&dir), cells.join(" | ")));
    }
    ctx.out.say(&format!("runs: {runs}"));
    Ok(())
}

fn verify(ctx: &mut Context, _args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    let cwd = std::env::current_dir().map_err(|e| cannot("read the cwd", &e))?;
    let git = |args: &[&str], or: &str| git_out(None, args).unwrap_or(or.to_string());
    for (label, value) in [
        ("pwd:", cwd.to_string_lossy().into_owned()),
        ("common-dir:", git(&["rev-parse", "--git-common-dir"], "none")),
        ("main root:", roots.main_root.to_string_lossy().into_owned()),
        ("store:", roots.store.to_string_lossy().into_owned()),
        ("worktree:", roots.wt_root.to_string_lossy().into_owned()),
        ("branch:", git(&["rev-parse", "--abbrev-ref", "HEAD"], "detached")),
        ("HEAD:", git(&["rev-parse", "HEAD"], "none")),
        ("CURRENT:", roots.current_run_id()?.unwrap_or_default()),
    ] {
        ctx.out.say(&format!("{label:<11} {value}"));
    }
    Ok(())
}
