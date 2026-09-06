// verbs/ledger/cases.rs
// dstack cases sync and cases render: approved R rows become ledger rows, and the table (R73).

use crate::core::args::is_option;
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::fsx::utc_now;
use crate::core::target::{resolve_target, Target, TargetKind};
use crate::store::cases::{self, CaseRow};
use crate::store::request::RequestDoc;
use crate::store::tsv;

use super::kind_word;

/// The statuses cases render counts, in the order it prints them.
const RENDERED_STATUSES: [&str; 7] = [
    "open",
    "met",
    "abstain",
    "blocked",
    "skipped",
    "unreported",
    "retired",
];

ledger_verb!(CasesSync, "cases sync", sync_verb);
ledger_verb!(CasesRender, "cases render", render);

fn sync_verb(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (mut target, rest) = resolve_target(ctx, args)?;
    // `dstack cases sync <run-id>` (R48) names the run positionally; --run/--quick already went
    // to resolve_target, so anything left must be that id.
    for arg in &rest {
        if is_option(arg) {
            fail!("unknown option: {arg} (usage: dstack cases sync [<run-id>] [--run <id>|--quick <slug>])")
        }
        if target.kind == TargetKind::Quick {
            fail!("give either <run-id> or --quick <slug>, not both")
        }
        // The id names one directory under RUNS and nothing else, as resolve_target requires of
        // the option form (D-10); the shell joined it unchecked.
        if arg.contains('/') || arg == "." || arg == ".." {
            fail!("run id must be a plain name (got '{arg}')")
        }
        target = Target {
            kind: TargetKind::Run,
            id: arg.clone(),
            dir: roots.runs.join(arg),
        };
        if !target.dir.is_dir() {
            fail!("run not found: {arg}")
        }
    }
    sync(ctx, &target)
}

/// cases sync itself: `request approve` calls it in process once the approval is written.
pub fn sync(ctx: &mut Context, target: &Target) -> Result<()> {
    let dir = &target.dir;
    let path = dir.join("request.md");
    if !path.is_file() {
        fail!(
            "no request.md in {} (dstack request new --type <work_type>)",
            dir.display()
        )
    }
    // An unapproved request still moves: expanding it would freeze rows the owner has not seen
    // (R48 makes approval the moment the ledger grows).
    if !dir.join("request.approved").is_file() {
        fail!(
            "request not approved: {}/request.approved is missing — dstack request approve",
            dir.display()
        )
    }
    // Every read first: an unreadable request.md or ledger must not leave a header behind in a
    // file this command then refuses to fill (D-12).
    let doc = RequestDoc::load(&path)?;
    let before = cases::rows(dir)?;
    let default_kind = cases::default_kind(&doc);
    let tests = doc.field("unit_tests").unwrap_or_default();
    cases::ensure(dir)?;
    let mut added: Vec<CaseRow> = Vec::new();
    for row in doc.rows() {
        if row.is_pending() || before.iter().any(|held| held.r == row.id) {
            continue;
        }
        match skip_note(&row) {
            // A row that will never be worked on still needs a ledger row: R79 counts SKIPPED
            // separately, and "no row at all" is indistinguishable from "forgotten".
            Some(note) => added.push(CaseRow {
                r: row.id.clone(),
                case_id: "c1".to_string(),
                kind: default_kind.to_string(),
                status: "skipped".to_string(),
                artifact: "-".to_string(),
                sha256: "-".to_string(),
                produced_by: "dstack cases sync".to_string(),
                recorded_at: utc_now(),
                note: tsv::tsv_clean(&note),
            }),
            None => {
                added.push(open_row(&row.id, "c1", default_kind));
                if tests == "on" {
                    added.push(open_row(&row.id, "c-test", "test"));
                }
            }
        }
    }
    for row in &added {
        cases::append(dir, row)?;
    }
    say!(
        ctx,
        "cases sync: {} {} — {}/cases.tsv",
        kind_word(target.kind),
        target.id,
        dir.display()
    );
    say!(
        ctx,
        "  default kind {default_kind} (e2e: {}), unit_tests {tests}",
        doc.field("e2e").unwrap_or_default()
    );
    say!(
        ctx,
        "  added {}, kept {}, total {}",
        added.len(),
        before.len(),
        cases::rows(dir)?.len()
    );
    Ok(())
}

/// The note a row that takes no work carries, in the order the shell asks for the markers.
fn skip_note(row: &crate::store::rows::Row) -> Option<String> {
    for key in ["withdrawn", "deferred", "superseded-by"] {
        if let Some(value) = row.marker(key).filter(|value| !value.is_empty()) {
            return Some(format!("{key}: {value}"));
        }
    }
    None
}

fn open_row(r: &str, case_id: &str, kind: &str) -> CaseRow {
    CaseRow {
        r: r.to_string(),
        case_id: case_id.to_string(),
        kind: kind.to_string(),
        status: "open".to_string(),
        artifact: "-".to_string(),
        sha256: "-".to_string(),
        produced_by: "-".to_string(),
        recorded_at: "-".to_string(),
        note: "-".to_string(),
    }
}

fn render(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    if let Some(first) = rest.first() {
        fail!(
            "unexpected argument: {first} (usage: dstack cases render [--run <id>|--quick <slug>])"
        )
    }
    let path = target.dir.join("cases.tsv");
    if !path.is_file() {
        fail!("no ledger at {} — dstack cases sync", path.display())
    }
    say!(
        ctx,
        "cases: {} {} — {}",
        kind_word(target.kind),
        target.id,
        path.display()
    );
    ctx.out
        .say("| R | case | kind | status | artifact | produced_by | recorded_at | note |");
    ctx.out.say("|---|---|---|---|---|---|---|---|");
    let rows = cases::rows(&target.dir)?;
    for row in &rows {
        // The sha256 column is the one the table leaves out.
        say!(
            ctx,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            row.r,
            row.case_id,
            row.kind,
            row.status,
            row.artifact,
            row.produced_by,
            row.recorded_at,
            row.note
        );
    }
    let mut counts = String::new();
    for status in RENDERED_STATUSES {
        let held = rows.iter().filter(|row| row.status == status).count();
        counts.push_str(&format!("{status} {held}, "));
    }
    say!(
        ctx,
        "rows {}: {}",
        rows.len(),
        counts.strip_suffix(", ").unwrap_or(&counts)
    );
    Ok(())
}
