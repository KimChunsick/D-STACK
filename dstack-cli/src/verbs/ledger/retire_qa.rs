// verbs/ledger/retire_qa.rs
// dstack evidence retire --qa: a recorded Goal QA result ends with a reason, the scenario opens
// again for a new run, and the retired result stays as history (R08, D-46).

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::fsx::with_lock;
use crate::core::paths::base_name;
use crate::core::roots::Roots;
use crate::core::target::{Target, TargetKind};
use crate::store::{qa, qa_retire};

use super::retire::RETIRE_USAGE;

/// The options of an R row's retire; a QA retire names no R row and no case.
const R_OPTIONS: [&str; 3] = ["r", "R", "case"];

/// Every check but the store's own runs before the locks and the first write, so a refusal leaves
/// the run as it was. Whether the scenario holds a result to retire belongs to qa_retire::retire,
/// which reads the ledger under the run lock.
pub(super) fn retire(ctx: &mut Context, roots: &Roots, target: &Target, rest: &[String]) -> Result<()> {
    let (mut id, mut why) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        let r_option = |name: &&str| arg == format!("--{name}") || arg.starts_with(&format!("--{name}="));
        if R_OPTIONS.iter().any(r_option) {
            fail!("--qa does not combine with --r or --case — {RETIRE_USAGE}")
        } else if let Some((value, eaten)) = opt(arg, next, "qa")? {
            id = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "why")? {
            why = value;
            i += eaten;
        } else {
            fail!("unknown argument: {arg} — {RETIRE_USAGE}")
        }
    }
    if id.is_empty() {
        fail!("{RETIRE_USAGE}")
    }
    if why.trim().is_empty() {
        fail!("retiring a QA result needs a reason (--why) — {RETIRE_USAGE}")
    }
    if target.kind == TargetKind::Quick {
        fail!("quick tasks have no Goal QA scenarios; evidence retire --qa retires them for a run")
    }
    let dir = &target.dir;
    let known: Vec<String> = qa::rows(dir)?.into_iter().map(|row| row.qa).collect();
    if !known.contains(&id) {
        let known = match known.is_empty() {
            true => "none".to_string(),
            false => known.join(" "),
        };
        fail!("unknown QA scenario: {id} (known: {known})")
    }

    let run = base_name(dir);
    let held = {
        let _local = with_lock(&roots.local)?;
        let _run = qa::lock(dir)?;
        qa_retire::retire(dir, &run, &id, &why)?
    };
    say!(ctx, "retired {id} (was {}): {why}", held.status);
    ctx.out.say("  the old result and the reason stay in qa-history.tsv; the scenario is open again — record its new run:");
    say!(
        ctx,
        "  dstack evidence add --qa {id} --artifact <path> --produced-by \"<cmd>\" [--status met|failed|skipped|blocked] [--note <reason>]"
    );
    Ok(())
}
