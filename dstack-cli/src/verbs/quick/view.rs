// verbs/quick/view.rs
// dstack quick list|status|resume: what the quick track holds and what one task still needs.

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::store::cases;
use crate::store::request::RequestDoc;

use super::{require_dir, state};

/// The statuses that count as recorded evidence, as check coverage counts them.
const RECORDED: [&str; 3] = ["met", "abstain", "blocked"];

quick_verb!(QuickList, "quick list", list);
quick_verb!(QuickStatus, "quick status", status);
quick_verb!(QuickResume, "quick resume", resume);

fn list(ctx: &mut Context, _args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let rows = state::rows(&roots.quick)?;
    say!(ctx, "quick tasks in {}", roots.quick.display());
    ctx.out.say("slug | status | opened | closed");
    let count = |status: &str| rows.iter().filter(|row| row.status == status).count();
    let (open, done, abandoned) = (count("open"), count("done"), count("abandoned"));
    for row in &rows {
        say!(
            ctx,
            "{} | {} | {} | {}",
            row.slug,
            row.status,
            row.opened,
            row.closed
        );
    }
    say!(
        ctx,
        "quick tasks {} — open {open}, done {done}, abandoned {abandoned}",
        rows.len()
    );
    Ok(())
}

fn status(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let slug = args.first().cloned().unwrap_or_default();
    let dir = require_dir(&roots.quick, &slug, "status")?;
    let state_row: String = state::rows(&roots.quick)?
        .iter()
        .filter(|row| row.slug == slug)
        .map(|row| {
            let closed = match row.closed.is_empty() {
                true => "-",
                false => &row.closed,
            };
            format!(
                "status {}, opened {}, closed {closed}",
                row.status, row.opened
            )
        })
        .collect();
    say!(ctx, "quick: {slug}");
    say!(ctx, "  dir: {}", dir.display());
    say!(ctx, "  state row: {state_row}");
    let request = dir.join("request.md");
    if !request.is_file() {
        say!(
            ctx,
            "  request.md: missing (dstack request new --quick {slug} --type <work_type>)"
        );
        ctx.out.say("  R rows 0, cases 0");
        return Ok(());
    }
    let doc = RequestDoc::load(&request)?;
    let rows = doc.rows();
    let pending = rows.iter().filter(|row| is_pending(row)).count();
    let approved = match dir.join("request.approved").is_file() {
        true => "yes",
        false => "no",
    };
    let field = |key: &str| doc.field(key).unwrap_or_default();
    say!(ctx, "  request.md: present, approved {approved}");
    say!(
        ctx,
        "  fields: review={} e2e={} research={} effort={}",
        field("review"),
        field("e2e"),
        field("external_research"),
        field("codex_effort")
    );
    say!(
        ctx,
        "  R rows {} — live {}, pending {pending}",
        rows.len(),
        doc.live_ids().len()
    );
    if !dir.join("cases.tsv").is_file() {
        ctx.out
            .say("  cases 0 (no cases.tsv yet — dstack cases sync runs at approval)");
        return Ok(());
    }
    let ledger = cases::rows(&dir)?;
    let count = |status: &str| ledger.iter().filter(|row| row.status == status).count();
    say!(
        ctx,
        "  cases {} — met {}, open {}",
        ledger.len(),
        count("met"),
        count("open")
    );
    Ok(())
}

/// What a fresh session must do next. Everything printed here is a command, not a description:
/// a resumed quick task is resumed by a context that has read nothing.
fn resume(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let slug = args.first().cloned().unwrap_or_default();
    let dir = require_dir(&roots.quick, &slug, "resume")?;
    let request = dir.join("request.md");
    let (mut checked, mut missing) = (1, 0);
    say!(ctx, "quick: {slug} — what is still missing");
    if !request.is_file() {
        say!(
            ctx,
            "  MISSING request.md — dstack request new --quick {slug} --type cli"
        );
        say!(ctx, "checked {checked} items, missing 1");
        return Err(Error::Exit(1));
    }
    let doc = RequestDoc::load(&request)?;
    checked += 1;
    if doc.rows().is_empty() {
        missing += 1;
        say!(
            ctx,
            "  MISSING R rows — dstack req add \"<line>\" --accept \"<criterion>\" --quick {slug}"
        );
    }
    let pending = doc.rows().iter().filter(|row| is_pending(row)).count();
    checked += 1;
    if pending > 0 {
        missing += 1;
        say!(
            ctx,
            "  MISSING approval of {pending} pending row(s) — dstack request approve --quick {slug}"
        );
    }
    checked += 1;
    if !dir.join("request.approved").is_file() {
        missing += 1;
        say!(
            ctx,
            "  MISSING request.approved — dstack request approve --quick {slug}"
        );
    }
    for r in doc.live_ids() {
        checked += 1;
        let recorded = cases::for_r(&dir, &r)?
            .iter()
            .filter(|row| RECORDED.contains(&row.status.as_str()))
            .count();
        if recorded == 0 {
            missing += 1;
            say!(ctx, "  MISSING evidence for {r} — dstack evidence add --quick {slug} --r {r} --case c-1 --kind cli --artifact <path> --produced-by \"<cmd>\"");
        }
    }
    say!(ctx, "checked {checked} items, missing {missing}");
    if missing > 0 {
        return Err(Error::Exit(1));
    }
    say!(ctx, "  ready to close: dstack quick close {slug}");
    Ok(())
}

/// The shell searches the joined markers for the text, without the `;` guard req_live_ids uses,
/// so a marker whose value ends in it counts here too.
fn is_pending(row: &crate::store::rows::Row) -> bool {
    row.markers_string().contains("status=pending-approval")
}
