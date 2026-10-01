// verbs/ledger/evidence_qa.rs
// dstack evidence add --qa: record the result of one Goal QA scenario in the run's QA ledger (R08, D-40).

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{sha256_file, with_lock};
use crate::core::paths::base_name;
use crate::core::roots::Roots;
use crate::core::target::{Target, TargetKind};
use crate::store::qa::{self, QaResult};

use super::artifact::names_word;
use super::evidence::{checked_artifact, Artifact, USAGE};

/// The options of an R case; a QA result has no R row, no case id and no kind.
const R_OPTIONS: [&str; 4] = ["r", "R", "case", "kind"];

/// Every check but the store's own runs before the lock and the first write, so a refusal leaves
/// the run as it was. The status, its reason and the never-overwrite rule belong to qa::record,
/// which checks them again under the lock.
pub(super) fn add(ctx: &mut Context, roots: &Roots, target: &Target, rest: &[String]) -> Result<()> {
    let (mut id, mut artifact, mut produced) = (String::new(), String::new(), String::new());
    let (mut status, mut note) = ("met".to_string(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        let r_option = |name: &&str| arg == format!("--{name}") || arg.starts_with(&format!("--{name}="));
        if R_OPTIONS.iter().any(r_option) {
            fail!("--qa does not combine with --r, --case or --kind — {USAGE}")
        } else if let Some((value, eaten)) = opt(arg, next, "qa")? {
            id = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "artifact")? {
            artifact = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "produced-by")? {
            produced = value;
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
    if [&id, &artifact, &produced].iter().any(|value| value.is_empty()) {
        fail!("missing required option — {USAGE}")
    }
    if target.kind == TargetKind::Quick {
        fail!("quick tasks have no Goal QA scenarios; evidence add --qa records them for a run")
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

    let Artifact { abs, rel, size } = checked_artifact(roots, target, &artifact)?;
    // The QA analogue of check (7): an artifact that never names the scenario proves another one.
    if !names_word(&abs, &id) {
        fail!("the artifact must name {id} as a whole word; {rel} does not — record the run that mentions it")
    }
    let sha = sha256_file(&abs)
        .map_err(|e| Error::cannot_decide(format!("cannot hash {}: {e}", abs.display())))?;
    let result = QaResult {
        status,
        artifact: rel.clone(),
        artifact_sha256: sha.clone(),
        produced_by: produced,
        note,
    };
    let run = base_name(dir);
    let row = {
        let _lock = with_lock(&roots.local)?;
        qa::record(dir, &run, &id, &result)?
    };
    let rows = qa::rows(dir)?;
    let open = rows.iter().filter(|row| row.status == "open").count();
    say!(ctx, "evidence add: run {run} — {id} {}", row.status);
    ctx.out.say(&row.to_line());
    say!(
        ctx,
        "  artifact {rel} ({size} bytes, sha256 {}…), qa.tsv scenarios {}, open {open}",
        &sha[..8.min(sha.len())],
        rows.len()
    );
    Ok(())
}
