// verbs/ledger/retire.rs
// dstack evidence retire: a recorded row ends with a reason, and its history stays (R104).

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::utc_now;
use crate::core::target::resolve_target;
use crate::store::cases;
use crate::store::tsv;

const RETIRE_USAGE: &str = "usage: dstack evidence retire --r R<NN> --case <id> --why \"<reason>\"";

ledger_verb!(EvidenceRetire, "evidence retire", retire);

/// A recorded row whose artifact was overwritten, or that proved the wrong thing, cannot be
/// edited by hand (R74) and must not be deleted (the ledger is the history): it is retired, and
/// the R needs a fresh row to be proven again.
fn retire(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut r, mut case_id, mut why) = (String::new(), String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        if arg == "--R" {
            r = next.ok_or(Error::Exit(1))?.to_string();
            i += 2;
        } else if let Some((value, eaten)) = opt(arg, next, "r")? {
            r = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "case")? {
            case_id = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "why")? {
            why = value;
            i += eaten;
        } else {
            fail!("unknown argument: {arg} — {RETIRE_USAGE}")
        }
    }
    if r.is_empty() || case_id.is_empty() || why.is_empty() {
        fail!("{RETIRE_USAGE}")
    }
    // add cleans the id before it writes the row, so retire looks it up the same way.
    let case_id = tsv::tsv_clean(&case_id);
    let dir = &target.dir;
    if !dir.join("cases.tsv").is_file() {
        fail!("no cases.tsv in {}", dir.display())
    }
    let held = cases::status_of(dir, &r, &case_id)?.unwrap_or_default();
    match held.as_str() {
        "" => fail!("{r} case {case_id} is not in the ledger — nothing to retire"),
        "open" | "unreported" => fail!(
            "{r} case {case_id} is {held}: it holds no evidence yet — fill it with dstack evidence add instead of retiring it"
        ),
        "retired" => fail!("{r} case {case_id} is already retired"),
        _ => {}
    }
    let old: String = cases::for_r(dir, &r)?
        .into_iter()
        .find(|row| row.case_id == case_id)
        .map(|row| row.sha256)
        .unwrap_or_default()
        .chars()
        .take(8)
        .collect();
    let note = format!(
        "retired {}: {} (was {held}, sha {old})",
        utc_now(),
        tsv::tsv_clean(&why)
    );
    cases::retire(dir, &r, &case_id, &note)?;
    say!(ctx, "retired {r} case {case_id} (was {held}): {why}");
    ctx.out.say("  the artifact and its old sha stay in the row as history; the R needs a new row to be proven again:");
    say!(
        ctx,
        "  dstack evidence add --r {r} --case <new-id> --kind <kind> --artifact <path> --produced-by \"<cmd>\""
    );
    let rows = cases::for_r(dir, &r)?;
    let counting = rows
        .iter()
        .filter(|row| ["met", "abstain", "blocked"].contains(&row.status.as_str()))
        .count();
    let retired = rows.iter().filter(|row| row.status == "retired").count();
    say!(
        ctx,
        "  {r}: recorded rows still counting {counting}, retired {retired}"
    );
    Ok(())
}
