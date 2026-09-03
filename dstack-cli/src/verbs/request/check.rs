// verbs/request/check.rs
// dstack check request: frontmatter, row grammar, ledger counts and the approval hash (R41–R46, R51).
//
// `request approve` calls the same core in approve mode, so approval and check can never
// disagree about what a valid request is.

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::sha256_file;
use std::path::Path;

use crate::core::paths::{fmt_rid, is_plain_name, parse_rid};
use crate::core::target::{resolve_target, Target};
use crate::store::request::{approval_matches, req_enum, RequestDoc, REQ_FIELDS};
use crate::store::tables::{q_count, questions};

use super::{is_approved, load, request_file, rowfile};

/// full = every condition; approve = skip the two conditions `request approve` exists to resolve
/// (the pending rows it clears and the hash of a file it is about to re-stamp).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Full,
    Approve,
}

pub fn check(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    match core(ctx, &target, Mode::Full)? {
        0 => Ok(()),
        bad => Err(Error::failed(format!(
            "check request failed: {bad} condition(s) above"
        ))),
    }
}

/// The number of conditions that did not hold; the caller turns it into the exit code and the
/// stderr line, because `request approve` adds a second line of its own after it.
pub fn core(ctx: &mut Context, target: &Target, mode: Mode) -> Result<usize> {
    let roots = ctx.roots()?;
    let file = request_file(target);
    say!(ctx, "check request: {}", file.display());
    if !file.is_file() {
        say!(ctx, "  no request.md in {}", target.dir.display());
        return Err(Error::failed(
            "no request.md (dstack request new --type <work_type>)",
        ));
    }
    let doc = load(target)?;
    let mut bad = 0;

    let mut field_bad = 0;
    for key in doc.declared_keys() {
        if !REQ_FIELDS.contains(&key.as_str()) {
            say!(
                ctx,
                "  field {key}: unknown key (allowed: {})",
                REQ_FIELDS.join(" ")
            );
            field_bad += 1;
        }
    }
    for key in REQ_FIELDS {
        let value = doc.field(key).unwrap_or_default();
        if value.is_empty() {
            say!(ctx, "  field {key}: missing from the frontmatter");
            field_bad += 1;
            continue;
        }
        field_bad += match key {
            "route" => route(ctx, &value, &roots.runs),
            "risk_axes" => risk_axes(ctx, &value),
            _ => enum_value(ctx, key, &value),
        };
    }
    bad += field_bad;
    say!(
        ctx,
        "  fields: checked {}, bad {field_bad}",
        REQ_FIELDS.len()
    );

    let grammar = grammar(&doc);
    for line in &grammar {
        say!(ctx, "  {line}");
    }
    bad += grammar.len();

    let rows = doc.rows();
    let (mut pend, mut wdn, mut dfr, mut sup, mut asm, mut row_bad) = (0, 0, 0, 0, 0, 0);
    let (mut previous, mut seen) = (0, Vec::new());
    for row in &rows {
        let id = &row.id;
        let number = parse_rid(id).unwrap_or_default();
        if seen.contains(id) {
            say!(ctx, "  row {id}: duplicate id");
            row_bad += 1;
        }
        seen.push(id.clone());
        // Ids never get renumbered (R42), so the file order must be the mint order; anything else
        // means a row was inserted by hand and the numbering is no longer a history.
        if number <= previous {
            say!(
                ctx,
                "  row {id}: id does not increase (previous {})",
                fmt_rid(previous)
            );
            row_bad += 1;
        }
        previous = number;
        if row.accept.is_empty() {
            say!(
                ctx,
                "  row {id}: empty accept — every row needs an observable criterion"
            );
            row_bad += 1;
        }
        if row.accept.starts_with("pending:") {
            pend += 1;
            say!(
                ctx,
                "  row {id}: accept is still '{}' (dstack req accept {id} \"<criterion>\")",
                row.accept
            );
            row_bad += 1;
        }
        let markers = format!(";{};", row.markers_string());
        if markers.contains(";status=pending-approval;") {
            pend += 1;
        }
        wdn += usize::from(markers.contains(";withdrawn="));
        dfr += usize::from(markers.contains(";deferred="));
        sup += usize::from(markers.contains(";superseded-by="));
        asm += usize::from(markers.contains(";from=Q-"));
    }
    bad += row_bad;
    let live = doc.live_ids().len();
    say!(ctx, "  rows: {} (live {live}, pending {pend}, withdrawn {wdn}, deferred {dfr}, superseded {sup}, assumed-from-Q {asm}), grammar errors {}, row errors {row_bad}",
         rows.len(), grammar.len());

    let ledger = target.dir.join("questions.md");
    let (open, answered, assumed) = (
        q_count(&ledger, "open")?,
        q_count(&ledger, "answered")?,
        q_count(&ledger, "assumed")?,
    );
    say!(
        ctx,
        "  questions: open {open}, answered {answered}, assumed {assumed}"
    );
    // An assumed question must have left an R row behind, or a default the user never read would
    // be part of the approved request with nothing to observe if it is wrong (R51).
    for question in questions(&ledger)?.iter().filter(|q| q.status == "assumed") {
        let needle = format!("from={}", question.id);
        if !rows
            .iter()
            .any(|row| row.markers_string().contains(&needle))
        {
            say!(ctx, "  question {}: assumed but no R row carries 'from: {}' (dstack req add --assumption --from {})",
                 question.id, question.id, question.id);
            bad += 1;
        }
    }

    let lines = doc.line_count();
    say!(
        ctx,
        "  size: live rows {live} (max 12), lines {lines} (max 60)"
    );
    if live > 12 {
        ctx.out.warn(&format!("live rows {live} > 12 (R43): split a Milestone (dstack milestone add) or route the rest to a new Goal"));
    }
    if lines > 60 {
        ctx.out.warn(&format!("lines {lines} > 60 (R43): split a Milestone (dstack milestone add) or route the rest to a new Goal"));
    }

    let mut approved = "no";
    if is_approved(target) {
        approved = "yes";
        if mode == Mode::Full {
            let hash = sha256_file(&file).map_err(|e| {
                Error::cannot_decide(format!("cannot read {}: {e}", file.display()))
            })?;
            if !approval_matches(&target.dir, &hash)? {
                ctx.out
                    .say("  hash mismatch (edited after approval): dstack request approve");
                bad += 1;
            }
        }
    }
    say!(ctx, "  approved: {approved}");

    if mode == Mode::Full && pend > 0 {
        say!(
            ctx,
            "  pending rows: {pend} (dstack request approve clears them)"
        );
        bad += 1;
    }
    if open > 0 {
        say!(
            ctx,
            "  open questions: {open} (dstack ask answer Q-NN … or dstack ask assume Q-NN …)"
        );
        bad += 1;
    }
    say!(
        ctx,
        "check request: fields {}, rows {}, questions {}, failures {bad}",
        REQ_FIELDS.len(),
        rows.len(),
        open + answered + assumed
    );
    Ok(bad)
}

/// `merge <run-id>` is the R48 route and the run it names must exist, or the pending rows would
/// be appended to nothing.
fn route(ctx: &mut Context, value: &str, runs: &Path) -> usize {
    match value {
        "new-goal" | "quick" => 0,
        _ => match value.strip_prefix("merge ") {
            Some(id) if names_a_run(runs, id) => 0,
            Some(id) => {
                say!(
                    ctx,
                    "  field route: 'merge {id}' names no run under {}",
                    runs.display()
                );
                1
            }
            None => {
                say!(
                    ctx,
                    "  field route: '{value}' is not one of: new-goal, quick, merge <run-id>"
                );
                1
            }
        },
    }
}

/// The run a `merge <id>` route names. D-10: an id that is not a plain name never reaches the
/// join — `merge /tmp` would otherwise replace the store path entirely and `merge ../x` would
/// point outside it. Such an id names no run, and says so in the wording a missing run uses.
fn names_a_run(runs: &Path, id: &str) -> bool {
    is_plain_name(id) && runs.join(id).is_dir()
}

fn risk_axes(ctx: &mut Context, value: &str) -> usize {
    let allowed = req_enum("risk_axes").join(" ");
    let (mut bad, mut axes, mut has_none) = (0, 0, false);
    for axis in value
        .split([',', ' ', '\t', '\n'])
        .filter(|a| !a.is_empty())
    {
        axes += 1;
        if !format!(" {allowed} ").contains(&format!(" {axis} ")) {
            say!(ctx, "  field risk_axes: '{axis}' is not one of: {allowed}");
            bad += 1;
        }
        has_none = has_none || axis == "none";
    }
    if has_none && axes > 1 {
        say!(
            ctx,
            "  field risk_axes: 'none' cannot be combined with another axis (got '{value}')"
        );
        bad += 1;
    }
    bad
}

/// The shell's `case " $(req_enum $k) " in *" $v "*)`: a substring test with the spaces kept.
fn enum_value(ctx: &mut Context, key: &str, value: &str) -> usize {
    let allowed = req_enum(key).join(" ");
    if format!(" {allowed} ").contains(&format!(" {value} ")) {
        return 0;
    }
    say!(ctx, "  field {key}: '{value}' is not one of: {allowed}");
    1
}

/// Row grammar (R42). Only list items are checked: design.md §4.2 leaves other prose free, but a
/// line that looks like a row and is not one is the failure mode this catches.
fn grammar(doc: &RequestDoc) -> Vec<String> {
    let mut found = Vec::new();
    for (index, line) in rowfile::lines(doc.text()).iter().enumerate() {
        let number = index + 1;
        if !line.starts_with("- ") {
            continue;
        }
        if rowfile::is_row_line(line) {
            if !line.starts_with("- [ ]") {
                found.push(format!(
                    "line {number}: box is ticked; boxes are computed, never hand-ticked: {line}"
                ));
            } else if !line.contains(" — accept:") {
                found.push(format!(
                    "line {number}: no \" — accept: <criterion>\" segment: {line}"
                ));
            }
            continue;
        }
        if mentions_row_id(line) || line.starts_with("- [") {
            found.push(format!("line {number}: not a row (want \"- [ ] **R<NN>** <text> — accept: <criterion>\"): {line}"));
        }
    }
    found
}

/// The awk test `/\*\*R[0-9]+\*\*/`: an R id in bold anywhere on the line.
fn mentions_row_id(line: &str) -> bool {
    line.match_indices("**R").any(|(at, _)| {
        let rest = &line[at + "**R".len()..];
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        digits > 0 && rest[digits..].starts_with("**")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_a_merge_route_only_names_a_plain_directory_of_the_store() {
        let base = std::env::temp_dir().join(format!("dstack-route-{}", std::process::id()));
        let runs = base.join("runs");
        std::fs::create_dir_all(runs.join("20260101T000000Z_real")).expect("a scratch store");
        std::fs::create_dir_all(base.join("outside")).expect("a directory next to the store");
        assert!(names_a_run(&runs, "20260101T000000Z_real"));
        assert!(!names_a_run(&runs, "nosuch"));
        // D-10: a path that leaves the store, an absolute path that replaces it, and the two
        // names that mean a directory of their own.
        assert!(!names_a_run(&runs, "../outside"));
        assert!(!names_a_run(
            &runs,
            base.join("outside").to_str().expect("utf-8")
        ));
        assert!(!names_a_run(&runs, "/tmp"));
        assert!(!names_a_run(&runs, ".."));
        assert!(!names_a_run(&runs, "."));
        assert!(!names_a_run(&runs, ""));
        let _ = std::fs::remove_dir_all(&base);
    }
}
