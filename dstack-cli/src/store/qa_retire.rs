// store/qa_retire.rs
// Retiring a recorded Goal QA result (D-46): the scenario opens again for a new run, and the
// retired result with the reason stays as history in the append-only <run>/qa-history.tsv.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::utc_now;
use crate::store::qa::{self, QaRow};
use crate::store::request_part3;
use crate::store::tsv;

pub const HISTORY_HEADER: &str = "qa\tretired_at\twas_status\tartifact\tartifact_sha256\tproduced_by\trecorded_at\tresult_note\twhy";

/// The history of retired results, in the run directory beside qa.tsv; qa/ holds only the texts.
pub fn history(dir: &Path) -> PathBuf {
    dir.join("qa-history.tsv")
}

/// Retire the recorded result of `qa` with the reason `why`: the scenario reads again as qa add
/// wrote it, its id, usage scenario and text kept and every result cell reset. Part 3 is rendered
/// first, so a refusal writes nothing; then the history row is appended, then only that qa.tsv
/// line is rewritten, then part 3. An open scenario holds no result to retire. The caller holds its
/// worktree lock and then the run lock (`qa::lock`), and has checked the reason. Returns the
/// retired row as it was.
pub fn retire(dir: &Path, run: &str, qa: &str, why: &str) -> Result<QaRow> {
    let mut ledger = qa::entries(dir)?;
    let Some(at) = ledger.iter().position(|(row, _)| row.qa == qa) else {
        return Err(Error::failed(format!("QA scenario not found: {qa}")));
    };
    let held = ledger[at].0.clone();
    if held.status == "open" {
        return Err(Error::failed(format!(
            "{qa} is open — no recorded result; nothing to retire"
        )));
    }
    let row = QaRow::open(
        held.qa.clone(),
        held.scenario.clone(),
        held.body.clone(),
        held.body_sha256.clone(),
    );
    let index = qa::replaced(dir, &row)?;
    ledger[at].0 = row;
    let part3 = request_part3::preview(dir, run, &ledger)?;
    let file = history(dir);
    if !file.is_file() {
        qa::write(&file, &format!("{HISTORY_HEADER}\n"))?;
    }
    let was = [
        &held.status,
        &held.artifact,
        &held.artifact_sha256,
        &held.produced_by,
        &held.recorded_at,
        &held.note,
    ];
    let mut cells = vec![held.qa.clone(), utc_now()];
    cells.extend(was.iter().map(|cell| tsv::dash(cell)));
    cells.push(tsv::dash(&tsv::tsv_clean(why)));
    tsv::append_line(&file, &cells)?;
    qa::write(&qa::index(dir), &index)?;
    qa::publish(dir, qa, part3, ["retired", "retire"])?;
    Ok(held)
}
