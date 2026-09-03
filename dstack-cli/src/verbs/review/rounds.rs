// verbs/review/rounds.rs
// dstack review seal and review close: a sealed round and a deliberate stop (R69, R96).
//
// Sealed rounds are written once and indexed in review/index.tsv; review close records an
// intentional stop in review/closed.tsv. verify reads both; nothing here edits a sealed file.

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{read_text, utc_now};
use crate::core::target::{resolve_target, TargetKind};
use crate::store::plan;
use crate::store::plan_graph::{milestone_covers, plan_covers};
use crate::store::request::RequestDoc;
use crate::store::review_index::{
    closed_append, index_append, is_closed, latest_round, next_seq, verdict_count, IndexRow,
};

use super::{lines, take};

pub fn seal(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut from, mut scope, mut id) = (String::new(), String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        if let Some((value, eaten)) = take(&rest, i, "from")? {
            from = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "scope")? {
            scope = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "id")? {
            id = value;
            i += eaten;
        } else {
            fail!("unexpected argument: {} (dstack review seal --from <file> --scope plan --id P1)", rest[i]);
        }
    }
    if from.is_empty() {
        fail!("usage: dstack review seal --from <file> --scope plan|milestone|quick --id P<n>|M<n>|<slug>");
    }
    let source = Path::new(&from);
    if !source.is_file() {
        fail!("review output not found: {from}");
    }
    match scope.as_str() {
        "plan" | "milestone" if id.is_empty() => {
            fail!("--id is required (the plan or milestone this round judged)")
        }
        "plan" | "milestone" => {}
        "quick" => {
            if target.kind != TargetKind::Quick {
                fail!("--scope quick needs a quick target (--quick <slug>)");
            }
            if id.is_empty() {
                id = target.id.clone();
            }
        }
        _ => fail!("--scope must be plan, milestone or quick (got '{scope}')"),
    }

    // A round without a verdict table is not a round: R69's per-R verdict is the one thing
    // `review: off` cannot turn off, so sealing a file that lacks it would launder a skipped
    // review.
    let covered = verdict_count(source, "covered")?;
    let partial = verdict_count(source, "partial")?;
    let absent = verdict_count(source, "absent")?;
    let rows = covered + partial + absent;
    if rows == 0 {
        fail!("no per-R verdict rows (| R01 | covered | … |) in {from} — nothing to seal");
    }
    if !has_verdict_line(source) {
        fail!("no 'VERDICT: approve|reject' line in {from} — nothing to seal");
    }

    let dir = target.dir.clone();
    let review_dir = dir.join("review");
    let _ = std::fs::create_dir_all(&review_dir);
    let round = next_seq(&review_dir, "codex-review-");
    let file = review_dir.join(format!("codex-review-{round}.md"));
    if file.exists() {
        fail!("sealed round already exists: {} (sealed rounds are never rewritten)", file.display());
    }
    let text = std::fs::read(source)
        .map_err(|e| Error::cannot_decide(format!("cannot read {from}: {e}")))?;
    std::fs::write(&file, &text)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))?;
    index_append(
        &dir,
        &IndexRow {
            round: round.clone(),
            scope: scope.clone(),
            id: id.clone(),
            filename: format!("codex-review-{round}.md"),
            timestamp: utc_now(),
            absent: absent.to_string(),
            partial: partial.to_string(),
            covered: covered.to_string(),
        },
    )?;

    say!(ctx, "sealed round {round}: {}", file.display());
    say!(ctx, "  scope: {scope} {id}");
    say!(ctx, "  verdict rows {rows} — covered {covered}, partial {partial}, absent {absent}");
    say!(ctx, "  index: {}", review_dir.join("index.tsv").display());
    if absent > 0 {
        say!(ctx, "  NOTE: {absent} absent verdict(s) — this round cannot seal positively (R69)");
    }
    Ok(())
}

/// Ends the review of one scope on purpose, with the reason written where the checks read it.
/// Nothing sealed is touched and no round is invented: review/closed.tsv gains one row and the R
/// ids the scope covers read ABSTAIN (owner-accepted through verify --accept-abstain) until a
/// later round seals a verdict for them. A partial/absent verdict already sealed stays a failure.
pub fn close(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut scope, mut id, mut why) = (String::new(), String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        if let Some((value, eaten)) = take(&rest, i, "scope")? {
            scope = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "id")? {
            id = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "why")? {
            why = value;
            i += eaten;
        } else {
            fail!("unexpected argument: {} — usage: dstack review close --scope plan|milestone|quick --id <id> --why \"<reason>\"", rest[i]);
        }
    }
    if why.is_empty() {
        fail!("usage: dstack review close --scope plan|milestone|quick --id <id> --why \"<reason>\" — the reason is what the report will print");
    }
    let dir = target.dir.clone();
    let ids = match scope.as_str() {
        "plan" => {
            if target.kind != TargetKind::Run {
                fail!("--scope plan needs a run target (quick tasks use --scope quick)");
            }
            if id.is_empty() {
                fail!("--scope plan needs --id P<n>");
            }
            if !plan::exists(&dir) {
                fail!("no plan.json in {}", dir.display());
            }
            let doc = plan::load(&dir)?;
            if doc.plan(&id).is_none() {
                fail!("plan not found: {id}");
            }
            plan_covers(&doc, &id).join(",")
        }
        "milestone" => {
            if target.kind != TargetKind::Run {
                fail!("--scope milestone needs a run target");
            }
            if id.is_empty() {
                fail!("--scope milestone needs --id M<n>");
            }
            if !plan::exists(&dir) {
                fail!("no plan.json in {}", dir.display());
            }
            let doc = plan::load(&dir)?;
            if !doc.milestones.iter().any(|m| m.id == id) {
                fail!("milestone not found: {id}");
            }
            milestone_covers(&doc, &id).join(",")
        }
        "quick" => {
            if target.kind != TargetKind::Quick {
                fail!("--scope quick needs a quick target (--quick <slug>)");
            }
            if id.is_empty() {
                id = target.id.clone();
            }
            live_ids(&dir.join("request.md"))?.join(",")
        }
        _ => fail!("--scope must be plan, milestone or quick (got '{scope}')"),
    };
    if ids.is_empty() {
        fail!("{scope} {id} covers no live R id — nothing to close");
    }
    let review_dir = dir.join("review");
    let _ = std::fs::create_dir_all(&review_dir);
    // The round the close comes after: the latest sealed round for this scope/id, else 000.
    let round = latest_round(&dir, &scope, &id)?;
    if is_closed(&dir, &scope, &id, &round)? {
        fail!("{scope} {id} is already closed after round {round} — seal a newer round before closing again");
    }
    let open = open_findings(&dir)?;
    let now = utc_now();
    closed_append(&dir, &scope, &id, &round, &ids, &now, &why)?;
    let record = review_dir.join(format!("closed-{scope}-{id}-{round}.md"));
    let mut text = format!(
        "# Review closed — {scope} {id}\n\nclosed_at: {now}\nafter round: {round}\nR ids: {ids}\nopen findings at close: {open}\n\nwhy: {why}\n"
    )
    .into_bytes();
    let findings = dir.join("findings.md");
    if findings.is_file() {
        text.extend_from_slice(b"\n## findings.md at close\n\n");
        text.extend_from_slice(&std::fs::read(&findings).unwrap_or_default());
    }
    std::fs::write(&record, &text)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", record.display())))?;

    let n = ids.split(',').filter(|part| !part.is_empty()).count();
    say!(ctx, "review closed: {scope} {id} after round {round}");
    say!(ctx, "  R ids now ABSTAIN until a newer round seals them: {n} ({ids})");
    say!(ctx, "  open findings copied: {open}; record: {}", record.display());
    say!(ctx, "  next: dstack verify --accept-abstain <R,…> --why \"<reason>\" (R79: the owner accepts each), then dstack report");
    Ok(())
}

/// `^[ \t]*VERDICT:[ \t]*\(approve\|reject\)`: the one line a round cannot be sealed without.
fn has_verdict_line(file: &Path) -> bool {
    let text = String::from_utf8_lossy(&std::fs::read(file).unwrap_or_default()).into_owned();
    lines(&text).iter().any(|line| {
        match line.trim_start_matches([' ', '\t']).strip_prefix("VERDICT:") {
            Some(rest) => {
                let rest = rest.trim_start_matches([' ', '\t']);
                rest.starts_with("approve") || rest.starts_with("reject")
            }
            None => false,
        }
    })
}

/// req_live_ids(): a quick task closes on the rows it still owes. A request file that is not
/// there reads as no rows, which is the refusal below and not a crash.
fn live_ids(request: &Path) -> Result<Vec<String>> {
    if !request.is_file() {
        return Ok(Vec::new());
    }
    Ok(RequestDoc::load(request)?.live_ids())
}

/// The list items of findings.md that no line marked resolved. A findings.md that is there and
/// cannot be read is a cannot-decide (D-12): "0 open findings" is the record a close writes down.
fn open_findings(dir: &Path) -> Result<usize> {
    let findings = dir.join("findings.md");
    let text = match read_text(&findings)? {
        Some(text) => text,
        None => return Ok(0),
    };
    Ok(lines(&text)
        .iter()
        .filter(|line| line.starts_with("- ") && !line.contains("resolved"))
        .count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_the_verdict_line_is_read_with_its_leading_blanks() {
        let dir = std::env::temp_dir().join(format!("dstack-rounds-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("round.md");
        for (text, wanted) in [
            ("VERDICT: approve\n", true),
            ("\t VERDICT:\treject now\n", true),
            ("VERDICT: maybe\n", false),
            ("verdict: approve\n", false),
            ("the round says VERDICT: approve\n", false),
        ] {
            std::fs::write(&file, text).expect("round");
            assert_eq!(has_verdict_line(&file), wanted, "{text:?}");
        }
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13_open_findings_are_the_items_no_line_resolved() {
        let dir = std::env::temp_dir().join(format!("dstack-findings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        assert_eq!(open_findings(&dir).expect("no findings.md"), 0);
        std::fs::write(
            dir.join("findings.md"),
            "# findings\n\n- one still open\n- another — resolved in P1\n  - an indented item\n",
        )
        .expect("findings");
        assert_eq!(open_findings(&dir).expect("the findings"), 1);
        std::fs::remove_dir_all(&dir).expect("clean up");
    }
}
