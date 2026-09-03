// verbs/status.rs
// dstack status: the human view, and the ≤2KB one-liner the UserPromptSubmit hook injects (R24).

use std::fs;
use std::path::Path;

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::fsx::read_text;
use crate::core::meta::meta_get;
use crate::core::roots::Roots;
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::request::RequestDoc;
use crate::store::{cases, plan};

/// 2048 bytes is the ceiling R24 sets for what the hook may inject per turn.
const CEILING: usize = 2048;

struct Status;

impl Verb for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        status(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(Status)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![]
}

fn status(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    let id = roots.current_run_id()?.unwrap_or_default();
    if args.first().map(String::as_str) == Some("--oneline") {
        oneline(ctx, &roots, &id)?;
        return Ok(());
    }
    let current = match id.is_empty() {
        true => "(none)".to_string(),
        false => status_line(&roots, &id)?,
    };
    ctx.out.say(&format!("store:    {}", roots.store.display()));
    ctx.out
        .say(&format!("worktree: {}", roots.wt_root.display()));
    ctx.out.say(&format!("current:  {current}"));
    ctx.out
        .say(&format!("quick open: {}", quick_open(&roots)?));
    let mut open = 0;
    for entry in fs::read_dir(&roots.runs).into_iter().flatten().flatten() {
        let dir = entry.path();
        if dir.join("meta.tsv").is_file() && meta_get(&dir, "status")?.as_deref() == Some("open") {
            open += 1;
        }
    }
    ctx.out.say(&format!("open runs in store: {open}"));
    Ok(())
}

/// `printf '%s\n' "$line" | head -c 2048`: the newline is part of what is cut, and a line over
/// the ceiling gets one back so the hook still reads a whole line.
fn oneline(ctx: &mut Context, roots: &Roots, id: &str) -> Result<()> {
    let mut line = if id.is_empty() {
        "dstack: no current run in this worktree (dstack run new <slug> | dstack run adopt <id>)"
            .to_string()
    } else if !roots.runs.join(id).is_dir() {
        format!("dstack: CURRENT points at a missing run '{id}' (dstack run adopt <id> or dstack run pause)")
    } else {
        format!("dstack: {}", status_line(roots, id)?)
    };
    line.push_str(&format!("; quick open {}", quick_open(roots)?));
    let with_newline = format!("{line}\n");
    let cut = &with_newline.as_bytes()[..with_newline.len().min(CEILING)];
    ctx.out.raw(&String::from_utf8_lossy(cut));
    if line.len() > CEILING {
        ctx.out.raw("\n");
    }
    Ok(())
}

/// _status_line(): everything the hook needs about one run on a single line.
fn status_line(roots: &Roots, id: &str) -> Result<String> {
    let dir = roots.runs.join(id);
    let mut out = format!("run {id} [{}]", meta_get(&dir, "status")?.unwrap_or_default());
    let request = dir.join("request.md");
    // The shell only asks whether the file is there, so an absent one prints the hint below; a
    // request.md that is there and cannot be read is a cannot-decide (D-12), never empty fields.
    match request.is_file() {
        false => out.push_str(" no request.md yet (dstack request new --type <work_type>)"),
        true => {
            let doc = RequestDoc::load(&request)?;
            let rows = doc.rows();
            let pending = rows
                .iter()
                .filter(|row| row.markers_string().contains("status=pending-approval"))
                .count();
            let field = |key: &str| doc.field(key).unwrap_or_default();
            out.push_str(&format!(
                " type={} route={} research={} review={}/{} e2e={} tests={} visual={} polish={}",
                field("work_type"),
                field("route"),
                field("external_research"),
                field("review"),
                field("codex_effort"),
                field("e2e"),
                field("unit_tests"),
                field("visual"),
                field("korean_polish")
            ));
            let approved = match dir.join("request.approved").is_file() {
                true => "yes",
                false => "no",
            };
            out.push_str(&format!(
                "; R rows {}, pending {pending}, approved {approved}",
                rows.len()
            ));
            let questions = dir.join("questions.md");
            if questions.is_file() {
                out.push_str(&format!("; Q open {}", open_rows(&questions)?));
            }
            if dir.join("cases.tsv").is_file() {
                let ledger = cases::rows(&dir)?;
                let met = ledger.iter().filter(|row| row.status == "met").count();
                out.push_str(&format!("; cases met {met}/{}", ledger.len()));
            }
        }
    }
    if plan::exists(&dir) {
        let doc = plan::load(&dir)?;
        let ids = |status: &str| {
            doc.plans
                .iter()
                .filter(|plan| plan.status == status)
                .map(|plan| plan.id.clone())
                .collect::<Vec<String>>()
                .join(",")
        };
        let done = doc.plans.iter().filter(|plan| plan.status == "done").count();
        out.push_str(&format!(
            "; plans ready [{}] in-progress [{}] done {done}/{}",
            ids("ready"),
            ids("in-progress"),
            doc.plans.len()
        ));
    }
    Ok(out)
}

fn quick_open(roots: &Roots) -> Result<usize> {
    open_rows(&roots.quick.join("STATE.md"))
}

/// `grep -c '| open |'`: the number of lines carrying the marker, 0 when the file is not there.
/// A file that is there and cannot be read is a cannot-decide (D-12), not a count of 0.
fn open_rows(path: &Path) -> Result<usize> {
    Ok(read_text(path)?
        .unwrap_or_default()
        .lines()
        .filter(|line| line.contains("| open |"))
        .count())
}
