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
/// line is rewritten, then part 3. A history row already holding this very result is the append of
/// a retire whose qa.tsv rewrite failed, so the same retire run again finishes it without a second
/// row. An open scenario holds no result to retire. The caller holds its worktree lock and then the
/// run lock (`qa::lock`), and has checked the reason. Returns the retired row as it was.
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
    let was: Vec<String> = [
        &held.status,
        &held.artifact,
        &held.artifact_sha256,
        &held.produced_by,
        &held.recorded_at,
        &held.note,
    ]
    .iter()
    .map(|cell| tsv::dash(cell))
    .collect();
    if !retired(&file, qa, &was)? {
        let mut cells = vec![held.qa.clone(), utc_now()];
        cells.extend(was);
        cells.push(tsv::dash(&tsv::tsv_clean(why)));
        tsv::append_line(&file, &cells)?;
    }
    qa::write(&qa::index(dir), &index).map_err(|e| {
        Error::cannot_decide(format!(
            "{qa}: the history row is written to qa-history.tsv, but qa.tsv still holds the result ({}); \
             run the same dstack evidence retire --qa {qa} --why … again — it finishes the retire without a second history row",
            e.message()
        ))
    })?;
    qa::publish(dir, qa, part3, ["retired", "retire"])?;
    Ok(held)
}

/// Whether the history already holds `qa`'s recorded result `was`, every result cell alike: one
/// stamp and one artifact hash alone repeat when the same file is recorded again within a second.
fn retired(file: &Path, qa: &str, was: &[String]) -> Result<bool> {
    Ok(tsv::read_rows(file, 8, true)?.iter().any(|row| row[0] == qa && row[2..8] == *was))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::store::qa::QaResult;
    use std::os::unix::fs::PermissionsExt;

    /// The history rows after the header.
    fn retired_rows(dir: &Path) -> usize {
        tsv::read_rows(&history(dir), 1, true).expect("history").len()
    }

    #[test]
    fn R08_qa_rerun_a_failed_qa_tsv_rewrite_is_finished_by_the_same_retire() {
        let dir = std::env::temp_dir().join(format!("dstack-qa-retire-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        qa::add(&dir, "run", "S1", "준비: 저장소예요.\n단계: 실행해요.\n기대 결과: 0이에요.\n").expect("added");
        let result = QaResult {
            status: "failed".into(),
            artifact: "artifacts/qa1.txt".into(),
            artifact_sha256: "abc".into(),
            produced_by: "dstack qa".into(),
            note: "목록이 비어 있어요".into(),
        };
        qa::record(&dir, "run", "QA1", &result).expect("recorded");
        qa::write(&history(&dir), &format!("{HISTORY_HEADER}\n")).expect("history");
        // The history file is there, so in a read-only run directory its append still lands while
        // the qa.tsv rewrite, a temporary file renamed over it, cannot.
        let mode = |bits: u32| std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(bits)).expect("mode");
        mode(0o555);
        let failed = retire(&dir, "run", "QA1", "목록을 고쳤어요");
        mode(0o755);
        let message = failed.expect_err("the qa.tsv rewrite fails").message().to_string();
        assert!(message.contains("history row is written to qa-history.tsv"), "{message}");
        assert!(message.contains("run the same dstack evidence retire --qa QA1 --why … again"), "{message}");
        assert_eq!((qa::rows(&dir).expect("rows")[0].status.as_str(), retired_rows(&dir)), ("failed", 1));

        let held = retire(&dir, "run", "QA1", "목록을 고쳤어요").expect("finished");
        assert_eq!(held.status, "failed");
        assert_eq!((qa::rows(&dir).expect("rows")[0].status.as_str(), retired_rows(&dir)), ("open", 1));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
