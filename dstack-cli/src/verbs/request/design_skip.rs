// verbs/request/design_skip.rs
// dstack request design-skip --why: record why a run skips its design round, before approval (R13).

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::target::{resolve_target, TargetKind};
use crate::store::tables::{d_append, d_next_id, q_text_ok};

use super::design_gate::{dec_file, SKIP_PREFIX};
use super::{is_approved, load, require_file, take};

/// The reason is a decision row (D-DESIGN-01), so `check decisions` and the brief read it where
/// every other decision lives. It affects the request's live R rows: a skipped run has no design
/// round to cover an `--affects design` row, while a task covering any of its rows covers this.
pub fn design_skip(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let mut why: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        if let Some((value, eaten)) = take(&rest, i, "why")? {
            why = Some(value);
            i += eaten;
        } else if rest[i].starts_with('-') {
            fail!("unknown option: {}", rest[i]);
        } else {
            fail!("unexpected argument: {}", rest[i]);
        }
    }
    let Some(why) = why else {
        fail!("usage: dstack request design-skip --why \"<reason>\" [--run <id>]");
    };
    if target.kind == TargetKind::Quick {
        fail!("a quick request has no part 2 and no design round to skip");
    }
    let file = require_file(&target)?;
    if is_approved(&target) {
        fail!(
            "{} is approved; the design-skip reason is recorded only before approval",
            file.display()
        );
    }
    let why = why.trim();
    if why.is_empty() {
        fail!("--why must not be empty: the reason is what approval prints");
    }
    if why.contains(['\n', '\r']) {
        fail!("--why must be one line: a line break would split the decision row, and approval could not read the reason back");
    }
    q_text_ok("--why", why)?;
    let doc = load(&target)?;
    if doc.field("design_review").as_deref() == Some("required") {
        fail!("design_review: required cannot be skipped; fill the five part-2 sections (dstack request section <key> --from <file>)");
    }
    let live = doc.live_ids();
    if live.is_empty() {
        fail!("no live R row yet; add the rows first (the skip decision affects them)");
    }
    let decisions = dec_file(&target.dir);
    let id = d_next_id(&decisions, false)?;
    let text = format!("{SKIP_PREFIX} {why}");
    let affects = live.join(",");
    d_append(&decisions, &id, &text, &affects, "answered")?;
    say!(ctx, "decisions: {}", decisions.display());
    say!(ctx, "  {id} | {text} | {affects} | answered");
    say!(ctx, "  request approve prints this reason when the skip applies");
    Ok(())
}
