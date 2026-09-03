// verbs/request/marks.rs
// dstack req accept|split|withdraw|defer|status: the markers a row grows and the per-row view.

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::target::resolve_target;
use crate::store::request::{req_text_ok, RequestDoc};
use crate::store::{cases, rows::Row};

use super::{counts, is_approved, load, require_file, row_line, take};

/// req accept (R45): the criterion of a row that was born as `pending: agent to propose`.
pub fn accept(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut id, mut criterion) = (String::new(), String::new());
    for arg in &rest {
        if arg.starts_with('-') {
            fail!("unknown option: {arg}");
        } else if id.is_empty() {
            id = arg.clone();
        } else if criterion.is_empty() {
            criterion = arg.clone();
        } else {
            fail!("unexpected argument: {arg}");
        }
    }
    if id.is_empty() || criterion.is_empty() {
        fail!("usage: dstack req accept R<NN> \"<observable criterion>\"");
    }
    let file = require_file(&target)?;
    let mut doc = load(&target)?;
    let row = require_row(&doc, &id, &file)?;
    req_text_ok("criterion", &criterion)?;
    if !row.accept.starts_with("pending:") {
        fail!("{id} already has an accept ('{}'); a row's criterion is not rewritten once it is real (R45)", row.accept);
    }
    doc.replace_accept(&id, &criterion)?;
    doc.save()?;
    let doc = load(&target)?;
    let left = doc
        .rows()
        .iter()
        .filter(|row| row.accept.starts_with("pending:"))
        .count();
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  {}", row_line(&doc, &id));
    say!(ctx, "  pending accepts left: {left}");
    Ok(())
}

/// req split (R103): the parent keeps its number and its text, so a report can read it as MET
/// only when every child is MET.
pub fn split(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut id, mut into) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if let Some((value, eaten)) = take(&rest, i, "into")? {
            into = value;
            i += eaten;
        } else if arg.starts_with('-') {
            fail!("unknown option: {arg}");
        } else if id.is_empty() {
            id = arg;
            i += 1;
        } else {
            fail!("unexpected argument: {arg}");
        }
    }
    if id.is_empty() || into.is_empty() {
        fail!("usage: dstack req split R<NN> --into R<a>,R<b>");
    }
    let file = require_file(&target)?;
    let mut doc = load(&target)?;
    let row = require_row(&doc, &id, &file)?;
    if let Some(parents) = marker(&row, "superseded-by") {
        fail!("{id} is already superseded by {parents}");
    }
    if row.marker("withdrawn").is_some() {
        fail!("{id} is withdrawn; a withdrawn row is not split");
    }
    if row.marker("deferred").is_some() {
        fail!("{id} is deferred; a deferred row is not split");
    }
    let children = words(&into);
    for child in &children {
        if *child == id {
            fail!("a row cannot supersede itself ({id})");
        }
        if doc.row_lineno(child).is_none() {
            fail!("child row {child} does not exist yet (dstack req add … first)");
        }
    }
    if children.len() < 2 {
        fail!("--into needs at least two children (got '{into}')");
    }
    doc.append_marker(&id, &format!("superseded-by: {}", children.join(", ")))?;
    doc.save()?;
    let doc = load(&target)?;
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  {}", row_line(&doc, &id));
    for child in &children {
        say!(ctx, "  child: {}", row_line(&doc, child));
    }
    say!(
        ctx,
        "  parent 1, children {}, superseded rows now {}",
        children.len(),
        count_marker(&doc, "superseded-by=")
    );
    Ok(())
}

pub fn withdraw(ctx: &mut Context, args: &[String]) -> Result<()> {
    mark_with_why(ctx, args, "withdrawn")
}

pub fn defer(ctx: &mut Context, args: &[String]) -> Result<()> {
    mark_with_why(ctx, args, "deferred")
}

/// _req_mark_with_why(): withdraw and defer differ only in the marker key they append.
fn mark_with_why(ctx: &mut Context, args: &[String], key: &str) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut id, mut why) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if let Some((value, eaten)) = take(&rest, i, "why")? {
            why = value;
            i += eaten;
        } else if arg.starts_with('-') {
            fail!("unknown option: {arg}");
        } else if id.is_empty() {
            id = arg;
            i += 1;
        } else {
            fail!("unexpected argument: {arg}");
        }
    }
    if id.is_empty() || why.is_empty() {
        fail!("usage: dstack req {key} R<NN> --why \"<reason>\"");
    }
    let file = require_file(&target)?;
    let mut doc = load(&target)?;
    let row = require_row(&doc, &id, &file)?;
    req_text_ok("--why", &why)?;
    if let Some(before) = marker(&row, key) {
        fail!("{id} is already {key}: {before}");
    }
    doc.append_marker(&id, &format!("{key}: {why}"))?;
    doc.save()?;
    let doc = load(&target)?;
    let (rows, live, _pending) = counts(&doc);
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  {}", row_line(&doc, &id));
    say!(
        ctx,
        "  rows {rows}, live {live}, {key} {}",
        count_marker(&doc, &format!("{key}="))
    );
    Ok(())
}

/// req status: one line per row with its computed state, plus the ledger column when there is a
/// cases.tsv to read.
pub fn status(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    let file = require_file(&target)?;
    let doc = load(&target)?;
    let has_cases = target.dir.join("cases.tsv").is_file();
    // The ledger is read once: the shell re-runs its awk per row, which is the same numbers at
    // the cost of one file read per R id.
    let ledger = match has_cases {
        true => cases::rows(&target.dir)?,
        false => Vec::new(),
    };
    let approved = if is_approved(&target) { "yes" } else { "no" };
    say!(ctx, "request: {} (approved: {approved})", file.display());
    ctx.out.say(match has_cases {
        true => "R | state | cases met/total | text",
        false => "R | state | text",
    });
    let (mut pend, mut wdn, mut dfr, mut sup, mut asm, mut pacc) = (0, 0, 0, 0, 0, 0);
    for row in doc.rows() {
        let mut state = "live".to_string();
        if row.is_pending() {
            state = "pending-approval".to_string();
            pend += 1;
        }
        if let Some(why) = marker(&row, "withdrawn") {
            state = format!("withdrawn ({why})");
            wdn += 1;
        }
        if let Some(why) = marker(&row, "deferred") {
            state = format!("deferred ({why})");
            dfr += 1;
        }
        if let Some(kids) = marker(&row, "superseded-by") {
            state = format!("superseded ({} → {kids})", row.id);
            sup += 1;
        }
        if format!(";{};", row.markers_string()).contains(";from=Q-") {
            state = format!(
                "{state}, assumed-from-{}",
                row.marker("from").unwrap_or_default()
            );
            asm += 1;
        }
        if row.accept.starts_with("pending:") {
            state = format!("{state}, accept {}", row.accept);
            pacc += 1;
        }
        if has_cases {
            let mine = ledger.iter().filter(|case| case.r == row.id);
            let total = mine.clone().count();
            let met = mine.filter(|case| case.status == "met").count();
            say!(ctx, "{} | {state} | {met}/{total} | {}", row.id, row.text);
        } else {
            say!(ctx, "{} | {state} | {}", row.id, row.text);
        }
    }
    let (rows, live, _pending) = counts(&doc);
    say!(ctx, "rows {rows}, live {live}, pending {pend}, withdrawn {wdn}, deferred {dfr}, superseded {sup}, assumed-from-Q {asm}, pending accepts {pacc}");
    if has_cases {
        let met = ledger.iter().filter(|case| case.status == "met").count();
        say!(ctx, "cases: {met} met of {}", ledger.len());
    } else {
        ctx.out.say("cases: no cases.tsv yet (dstack cases sync)");
    }
    Ok(())
}

fn require_row(doc: &RequestDoc, id: &str, file: &std::path::Path) -> Result<Row> {
    match doc.row(id).filter(|_| doc.row_lineno(id).is_some()) {
        Some(row) => Ok(row),
        None => Err(Error::failed(format!("no row {id} in {}", file.display()))),
    }
}

/// req_marker() through the `;key=` test the verbs guard with: a marker whose value is empty is
/// still a marker, which `Row::marker` alone cannot tell from a missing one.
fn marker(row: &Row, key: &str) -> Option<String> {
    match format!(";{};", row.markers_string()).contains(&format!(";{key}=")) {
        true => Some(row.marker(key).unwrap_or_default()),
        false => None,
    }
}

/// The count of rows whose marker string holds this text, as the shell's `index($4, k)` counts.
fn count_marker(doc: &RequestDoc, needle: &str) -> usize {
    doc.rows()
        .iter()
        .filter(|row| row.markers_string().contains(needle))
        .count()
}

/// The shell's `$(printf '%s' "$into" | tr ',' ' ')`: commas and blanks split, empties vanish.
fn words(list: &str) -> Vec<String> {
    list.split([',', ' ', '\t', '\n'])
        .filter(|word| !word.is_empty())
        .map(|word| word.to_string())
        .collect()
}
