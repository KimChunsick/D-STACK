// verbs/request/open_show.rs
// dstack request open and request show: the draft snapshot and the fresh read (R44, R46).

use std::process::{Command, Stdio};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::sha256_file;
use crate::core::target::resolve_target;
use crate::core::tools::tool_present;
use crate::store::request::{approval_matches, stamp_text};

use super::{counts, draft_file, is_approved, load, require_file};

pub fn open(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    let file = require_file(&target)?;
    let draft = draft_file(&target);
    // The snapshot is taken before the human edits, so `request approve` can show exactly what
    // the user changed about the agent's draft — the one diff that matters at approval (R46).
    std::fs::copy(&file, &draft).map_err(|e| {
        Error::cannot_decide(format!("cannot write {}: {e}", draft.display()))
    })?;
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  draft snapshot: {}", draft.display());
    // The optional editor of deps.tsv is still launched (D-14): R01's git-only clause names the
    // text-processing tools the port replaces, not this. -g and never -w: a blocking editor would
    // stall the session that must keep driving (R44). The harness and every sandbox run with a
    // PATH that has no `code` at all, so no editor opens there.
    if tool_present("command -v code")? {
        let spot = format!("{}:1", file.display());
        // `if code -g "$f:1"` is false for every failure of the invocation alike — a non-zero
        // exit, a file that cannot be executed, a shebang that names nothing — and the shell goes
        // on to the summary line with exit 0. A spawn error takes the same fallback here.
        let opened = Command::new("code")
            .args(["-g", &spot])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match opened {
            Ok(status) if status.success() => say!(ctx, "  opened: code -g {spot}"),
            _ => ctx.out.say("  code -g failed; open the path above by hand"),
        }
    } else {
        ctx.out.say("  code is not on PATH; open the path above by hand");
    }
    let doc = load(&target)?;
    let approved = if is_approved(&target) { "yes" } else { "no" };
    say!(ctx, "  rows {}, lines {}, approved: {approved}", doc.rows().len(), doc.line_count());
    Ok(())
}

pub fn show(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    let file = require_file(&target)?;
    let doc = load(&target)?;
    ctx.out.raw(doc.text());
    ctx.out.say("---");
    say!(ctx, "path: {}", file.display());
    if is_approved(&target) {
        let hash = sha256_file(&file)
            .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", file.display())))?;
        say!(ctx, "approved: yes ({})", stamp_text(&target.dir)?.unwrap_or_default());
        if approval_matches(&target.dir, &hash)? {
            ctx.out.say("hash: matches the approved file");
        } else {
            ctx.out
                .say("hash: MISMATCH — the file changed after approval (dstack request approve)");
        }
    } else {
        ctx.out.say("approved: no");
    }
    let (rows, _live, pending) = counts(&doc);
    say!(ctx, "rows {rows}, pending {pending}, lines {}", doc.line_count());
    Ok(())
}
