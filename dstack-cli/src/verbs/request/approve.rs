// verbs/request/approve.rs
// dstack request approve: validate, clear pending, stamp the sha256, diff, sync cases (R46, R48, R94, R105).

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{sha256_file, utc_now};
use crate::core::mode::Mode;
use crate::core::target::{resolve_target, TargetKind};
use crate::core::tools::tool_check_for_mode;
use crate::store::cases;
use crate::store::request::write_approval;

use super::{
    check, counts, draft_file, load, require_file, rowfile, stamp_file, target_flags, udiff,
};

pub fn approve(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    let file = require_file(&target)?;

    // Approval is the last moment before korean_polish runs on the prose (R94) and the last
    // moment the file may change at all: validate first, on the file as it stands.
    ctx.out.say("== validating (pending rows and the approval hash are what this command is about, so both are skipped here)");
    let bad = check::core(ctx, &target, check::Mode::Approve)?;
    if bad > 0 {
        ctx.out.err_line(&format!(
            "dstack: check request failed: {bad} condition(s) above"
        ));
        fail!("request does not validate; fix the lines above and approve again");
    }

    // R105 again with the REAL fields, not the work_type defaults `run new` guessed from: the
    // user may have turned e2e to capture or review on during the approval loop.
    ctx.out.say("== tools (R105, from the approved fields)");
    let doc = load(&target)?;
    let fields: Vec<String> = ["e2e", "review", "visual", "unit_tests"]
        .iter()
        .map(|field| format!("{field}={}", doc.field(field).unwrap_or_default()))
        .collect();
    let mode = Mode::for_run(&ctx.roots()?, &target.dir)?;
    let need_sub = target.kind == TargetKind::Run
        || doc.field("review").as_deref() == Some("on")
        || doc.field("external_research").as_deref() == Some("one-pass");
    if tool_check_for_mode(ctx, &fields, &mode, need_sub)? != 0 {
        fail!("a goal-closing tool required by these fields is missing (install it, or change the field and approve again)");
    }

    // The draft is read before anything is written: an unreadable snapshot must not leave the
    // request stamped as approved with no diff and no cases sync behind it (D-12). A snapshot
    // that is simply not there is the prose line the shell's `[ -f ]` prints.
    let draft = draft_file(&target);
    let before = match std::fs::read_to_string(&draft) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(Error::cannot_decide(format!(
                "cannot read {}: {error}",
                draft.display()
            )))
        }
    };

    // The ledger `cases sync` will expand is read here, before the first write of this command:
    // an unreadable cases.tsv would otherwise leave the request cleared and stamped with the sync
    // refusing afterwards (D-12), and the run would be approved with no ledger behind it.
    let _ledger = cases::rows(&target.dir)?;

    // Clearing pending is a write, so it happens before the hash is taken.
    let mut text = doc.text().to_string();
    let mut cleared = 0;
    while let Some(lineno) = pending_lineno(&text) {
        let line = rowfile::lines(&text)[lineno - 1].to_string();
        text = rowfile::set_line(&text, lineno, &rowfile::drop_segment(&line, "status"));
        cleared += 1;
    }
    if cleared > 0 {
        rowfile::write(&file, &text)?;
    }

    let hash = sha256_file(&file)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", file.display())))?;
    write_approval(&target.dir, &hash, &utc_now())?;

    ctx.out.say("== diff against the agent draft");
    match &before {
        Some(before) => ctx.out.raw(&udiff::unified(&draft, before, &file, &text)),
        None => ctx
            .out
            .say("no draft snapshot (dstack request open takes one before the human edits)"),
    }

    // The shell spawns `dstack cases sync` and prints its merged stdout and stderr through one
    // `printf '%s\n'`, which drops the trailing newlines and adds exactly one back.
    ctx.out.say("== cases ledger (R73)");
    let called = ctx.call("cases sync", &target_flags(&target));
    let merged = format!("{}{}", called.stdout, called.stderr);
    ctx.out.say(merged.trim_end_matches('\n'));
    // A child that could not decide is not a checked failure of this command: its own
    // `dstack: cannot read …` line is in the block just printed, and the code travels out
    // unchanged (D-12) instead of being turned into this verb's exit 1.
    if called.code == 2 {
        return Err(Error::Exit(2));
    }
    if called.code != 0 {
        fail!(
            "cases sync failed after approval (the request is approved; rerun: dstack cases sync)"
        );
    }

    let doc = load(&target)?;
    let (rows, live, _pending) = counts(&doc);
    ctx.out.say("== approved");
    say!(ctx, "  file:    {}", file.display());
    say!(ctx, "  sha256:  {hash}");
    say!(ctx, "  stamp:   {}", stamp_file(&target).display());
    say!(
        ctx,
        "  rows {rows}, pending cleared {cleared}, live rows {live}"
    );
    Ok(())
}

/// The first row line still carrying the marker `request approve` wrote itself, by line number:
/// the shell searches the raw line, not the parsed markers, and rewrites it in place.
fn pending_lineno(text: &str) -> Option<usize> {
    rowfile::lines(text)
        .iter()
        .position(|line| rowfile::is_row_line(line) && line.contains(" — status: pending-approval"))
        .map(|index| index + 1)
}
