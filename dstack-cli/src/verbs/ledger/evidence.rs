// verbs/ledger/evidence.rs
// dstack evidence add: the only writer of recorded ledger rows, with its eight checks (R104).

use std::path::Path;

use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{file_mtime, file_size, read_text, sha256_file, utc_now, utc_to_epoch};
use crate::core::meta::meta_get;
use crate::core::roots::Roots;
use crate::core::target::{resolve_target, Target, TargetKind};
use crate::store::cases::{self, CaseRow, CASES_EVIDENCE_STATUSES, CASES_KINDS};
use crate::store::request::RequestDoc;
use crate::store::tsv;

use super::artifact::{names_word, resolve_artifact};
use super::kind_word;

const USAGE: &str = "usage: dstack evidence add --r R<NN> --case <id> --kind test|capture|transcript|cli|visual|review --artifact <path> --produced-by \"<cmd>\" [--shared <why>] [--status met|abstain|blocked|skipped] [--note <n>] [--run <id>|--quick <slug>]";

/// `date -u -r <mtime> +%Y-%m-%dT%H:%M:%SZ`: the only place an mtime is printed.
const STAMP: &[BorrowedFormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

ledger_verb!(EvidenceAdd, "evidence add", add);

fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut r, mut case_id, mut kind) = (String::new(), String::new(), String::new());
    let (mut artifact, mut produced, mut shared) = (String::new(), String::new(), String::new());
    let (mut status, mut note) = ("met".to_string(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        // `--R` is the two-word alias of `--r`; the shell's loop has no `--R=` arm.
        if arg == "--R" {
            r = next.ok_or(Error::Exit(1))?.to_string();
            i += 2;
        } else if let Some((value, eaten)) = opt(arg, next, "r")? {
            r = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "case")? {
            case_id = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "kind")? {
            kind = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "artifact")? {
            artifact = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "produced-by")? {
            produced = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "shared")? {
            shared = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "status")? {
            status = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "note")? {
            note = value;
            i += eaten;
        } else {
            fail!("unknown argument: {arg} — {USAGE}")
        }
    }
    // The shell writes the case id raw, so a tab or a newline in it invents a column or a whole
    // row; D-09 says a store-corrupting defect is not reproduced, so the id is cleaned like
    // every other free-text cell before anything reads or writes it.
    let case_id = tsv::tsv_clean(&case_id);
    let given = [&r, &case_id, &kind, &artifact, &produced];
    if given.iter().any(|value| value.is_empty()) {
        fail!("missing required option — {USAGE}")
    }

    // A tab or a newline in the path would invent a column or a whole row in cases.tsv, which
    // the shell writes raw; D-09 refuses the input instead of corrupting the ledger, so this
    // line has no shell wording to match and is checked before anything reads the path.
    if artifact.contains('\t') || artifact.contains('\n') {
        fail!("artifact path must not contain tabs or newlines")
    }
    let dir = &target.dir;
    let request = dir.join("request.md");
    if !request.is_file() {
        fail!(
            "no request.md in {} — nothing to record evidence against",
            dir.display()
        )
    }
    alive(&request, &r)?;

    // (2) kind and status
    if !CASES_KINDS.contains(&kind.as_str()) {
        fail!("unknown kind '{kind}' — one of: {}", CASES_KINDS.join(" "))
    }
    if !CASES_EVIDENCE_STATUSES.contains(&status.as_str()) {
        fail!(
            "unknown status '{status}' — one of: {}",
            CASES_EVIDENCE_STATUSES.join(" ")
        )
    }

    let abs = resolve_artifact(&artifact)?;
    let base = abs
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let size = file_size(&abs).unwrap_or(0);
    if size == 0 {
        fail!(
            "artifact is zero bytes: {} — an empty file proves nothing",
            abs.display()
        )
    }

    // (5) an artifact older than the target cannot be evidence produced by it.
    let (start, source) = start_epoch(&roots, &target)?.ok_or_else(|| {
        Error::cannot_decide(format!(
            "cannot read when this {} started (no started_at / no state row)",
            kind_word(target.kind)
        ))
    })?;
    let mtime = file_mtime(&abs).unwrap_or(0);
    if mtime < start {
        fail!(
            "artifact mtime {} is earlier than this {} started ({source}) — re-run the command that produces {base}",
            stamp(mtime),
            kind_word(target.kind)
        )
    }

    // Store paths relative to the store's repository so a ledger stays readable from any worktree.
    let rel = match abs.strip_prefix(&roots.main_root) {
        Ok(rel) => rel.to_string_lossy().into_owned(),
        Err(_) => abs.to_string_lossy().into_owned(),
    };

    // (6) one artifact, one R — unless the owner says why it covers two.
    let other = cases::rows(dir)?
        .into_iter()
        .find(|row| row.artifact == rel && row.r != r)
        .map(|row| format!("{}/{}", row.r, row.case_id));
    if let (Some(other), true) = (other, shared.is_empty()) {
        fail!("artifact already recorded under {other}: {rel} — pass --shared \"<why one artifact proves both>\"")
    }

    // (7) a test/cli artifact that never names the R is evidence for something else.
    if (kind == "test" || kind == "cli") && !names_word(&abs, &r) {
        fail!("kind {kind} requires the artifact to name {r} as a whole word; {rel} does not — record the run that mentions it")
    }

    // (8) an already-recorded case is never overwritten: that is the edit the sha256 recheck
    // exists to catch, so the CLI refuses it instead of letting it look legitimate.
    let held = cases::status_of(dir, &r, &case_id)?.unwrap_or_default();
    match held.as_str() {
        "" | "open" | "unreported" => {}
        status => {
            fail!("{r} case {case_id} is already recorded (status {status}) — use a new case id")
        }
    }

    let sha = sha256_file(&abs)
        .map_err(|e| Error::cannot_decide(format!("cannot hash {}: {e}", abs.display())))?;
    if !shared.is_empty() {
        note = match note.is_empty() {
            true => format!("shared: {shared}"),
            false => format!("{note}; shared: {shared}"),
        };
    }
    let row = CaseRow {
        r: r.clone(),
        case_id: case_id.clone(),
        kind,
        status,
        artifact: rel.clone(),
        sha256: sha.clone(),
        produced_by: tsv::tsv_clean(&produced),
        recorded_at: utc_now(),
        note: tsv::dash(&tsv::tsv_clean(&note)),
    };
    // The header is written once every read has succeeded, so a refusal leaves no ledger behind.
    cases::ensure(dir)?;
    let action = match held.is_empty() {
        true => {
            cases::append(dir, &row)?;
            "appended a new row".to_string()
        }
        false => {
            cases::replace(dir, &r, &case_id, &row)?;
            format!("filled the {held} row")
        }
    };
    let rows = cases::rows(dir)?;
    say!(
        ctx,
        "evidence add: {} {} — {action}",
        kind_word(target.kind),
        target.id
    );
    ctx.out.say(&row.to_line());
    say!(
        ctx,
        "  artifact {rel} ({size} bytes, sha256 {}…), ledger rows {}, met {}",
        &sha[..8.min(sha.len())],
        rows.len(),
        rows.iter().filter(|row| row.status == "met").count()
    );
    Ok(())
}

/// (1) the R must exist in the request and still be alive: recording evidence for a row the
/// owner withdrew is how a ledger starts lying about what the Goal delivered.
fn alive(request: &Path, r: &str) -> Result<()> {
    let doc = RequestDoc::load(request)?;
    let row = match doc.row(r) {
        Some(row) => row,
        None => fail!(
            "{r} is not a row of {} — dstack req add first",
            request.display()
        ),
    };
    for marker in ["withdrawn", "deferred"] {
        if let Some(why) = row.marker(marker).filter(|why| !why.is_empty()) {
            fail!("{r} is {marker} ({why}) — a {marker} row takes no evidence (R79 reports it as {marker}; use its replacement)")
        }
    }
    if let Some(into) = row.marker("superseded-by").filter(|into| !into.is_empty()) {
        fail!("{r} is superseded by {into} — record the evidence on the child rows (R103)")
    }
    Ok(())
}

/// The instant before which an artifact cannot be evidence for this target (R104), and what was
/// read to find it. Runs carry started_at in meta.tsv; a quick task has no meta, so its opened
/// column in the worktree's STATE.md is the start, and its request.md mtime is the fallback.
fn start_epoch(roots: &Roots, target: &Target) -> Result<Option<(i64, String)>> {
    if target.kind == TargetKind::Run {
        return Ok(meta_get(&target.dir, "started_at")?
            .and_then(|at| utc_to_epoch(&at))
            .map(|epoch| {
                (
                    epoch,
                    format!("{}/meta.tsv started_at", target.dir.display()),
                )
            }));
    }
    let state = roots.quick.join("STATE.md");
    let opened = opened_column(&state, &target.id)?;
    if let Some(epoch) = opened.and_then(|at| utc_to_epoch(&at)) {
        return Ok(Some((epoch, format!("{} opened column", state.display()))));
    }
    let request = target.dir.join("request.md");
    Ok(file_mtime(&request).map(|epoch| (epoch, format!("{} mtime", request.display()))))
}

/// The opened cell of the STATE.md row whose slug column is this quick task. A STATE.md that is
/// not there has no row; one that cannot be read is a cannot-decide (D-12), because the fallback
/// below would date the task by a file mtime instead.
fn opened_column(state: &Path, slug: &str) -> Result<Option<String>> {
    let text = match read_text(state)? {
        Some(text) => text,
        None => return Ok(None),
    };
    for line in text.lines() {
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() < 5 {
            continue;
        }
        let trim = |cell: &str| cell.trim_matches([' ', '\t']).to_string();
        if trim(cells[1]) == slug {
            return Ok(Some(trim(cells[3])).filter(|opened| !opened.is_empty()));
        }
    }
    Ok(None)
}

fn stamp(epoch: i64) -> String {
    OffsetDateTime::from_unix_timestamp(epoch)
        .ok()
        .and_then(|at| at.format(&STAMP).ok())
        .unwrap_or_else(|| epoch.to_string())
}
