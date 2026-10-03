// store/qa_retire.rs
// Retiring a recorded Goal QA result (D-46, D-53, D-57): the scenario opens again for a new run,
// the retired result and its reason stay in <run>/qa-history.tsv, written whole for each new row,
// and an interrupted retire is finished.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::{read_text, utc_now};
use crate::store::qa::{self, QaEntry, QaRow};
use crate::store::request_part3;
use crate::store::tsv;

pub const HISTORY_HEADER: &str = "qa\tretired_at\twas_status\tartifact\tartifact_sha256\tproduced_by\trecorded_at\tresult_note\twhy\tattempt";

/// The history of retired results, in the run directory beside qa.tsv; qa/ holds only the texts.
pub fn history(dir: &Path) -> PathBuf {
    dir.join("qa-history.tsv")
}

/// The history as it stands, its header alone when there is none yet. A row is added by writing
/// the whole file again (D-57), so a file whose last row lacks its newline, one an append cut
/// short, cannot decide: a row added after it would merge into it.
fn history_text(dir: &Path) -> Result<String> {
    let file = history(dir);
    let text = read_text(&file)?.filter(|text| !text.is_empty());
    let text = text.unwrap_or_else(|| format!("{HISTORY_HEADER}\n"));
    if !text.ends_with('\n') {
        return Err(Error::cannot_decide(format!(
            "{} does not end with a newline — its last row is cut short, and a row added after it would merge into it; complete or remove that row, then run the command again",
            file.display()
        )));
    }
    Ok(text)
}

/// The rows of a history text after its header, split into cells.
fn history_rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines().skip(1).map(|line| line.split('\t').collect())
}

/// The file beside qa.tsv that holds the history row of a retire that has not finished every step
/// yet; while it exists the Goal QA check refuses (D-56).
pub const PENDING: &str = "qa-retire.pending";

fn pending(dir: &Path) -> PathBuf {
    dir.join(PENDING)
}

/// The pending history row, None when no retire is pending; a file that holds no history row
/// cannot decide.
fn pending_row(dir: &Path) -> Result<Option<Vec<String>>> {
    let file = pending(dir);
    let Some(text) = read_text(&file)? else {
        return Ok(None);
    };
    let cells: Vec<String> = text.trim_end_matches('\n').split('\t').map(String::from).collect();
    if cells.len() != HISTORY_HEADER.split('\t').count() {
        return Err(Error::cannot_decide(format!(
            "{} holds no qa-history.tsv row: {text}",
            file.display()
        )));
    }
    Ok(Some(cells))
}

/// The QA whose retire is unfinished, None when none is; verify and report refuse while one is
/// (D-56), so they read it here rather than the file's format.
pub fn pending_qa(dir: &Path) -> Result<Option<String>> {
    Ok(pending_row(dir)?.map(|cells| cells[0].clone()))
}

/// Retire the recorded result of `qa` with the reason `why`: the scenario reads again as qa add
/// wrote it, its id, usage scenario and text kept and every result cell reset. A pending retire is
/// finished first. Part 3 is rendered and the history read before the first write, so a refusal
/// writes nothing; then the history row, numbered 1 + the rows `qa` already has, is written to
/// qa-retire.pending and the steps `complete` names follow. An open scenario holds no result to
/// retire, unless the retire just finished was its own: then that one is reported. The caller holds
/// its worktree lock and then the run lock (`qa::lock`), and has checked the reason. Returns the
/// retired row as it was.
pub fn retire(dir: &Path, run: &str, qa: &str, why: &str) -> Result<QaRow> {
    let finished = finish(dir, run)?;
    let mut ledger = qa::entries(dir)?;
    let Some(at) = ledger.iter().position(|(row, _)| row.qa == qa) else {
        return Err(Error::failed(format!("QA scenario not found: {qa}")));
    };
    let held = ledger[at].0.clone();
    if held.status == "open" {
        return match finished {
            Some(cells) if cells[0] == qa => Ok(QaRow {
                status: cells[2].clone(),
                artifact: cells[3].clone(),
                artifact_sha256: cells[4].clone(),
                produced_by: cells[5].clone(),
                recorded_at: cells[6].clone(),
                note: cells[7].clone(),
                ..held
            }),
            _ => Err(Error::failed(format!(
                "{qa} is open — no recorded result; nothing to retire"
            ))),
        };
    }
    let index = reopen(dir, &mut ledger, at)?;
    let part3 = request_part3::preview(dir, run, &ledger)?;
    let text = history_text(dir)?;
    let attempt = history_rows(&text).filter(|row| row[0] == qa).count() + 1;
    let mut cells = vec![held.qa.clone(), utc_now()];
    let was = [&held.status, &held.artifact, &held.artifact_sha256, &held.produced_by, &held.recorded_at, &held.note];
    cells.extend(was.iter().map(|cell| tsv::dash(cell)));
    cells.push(tsv::dash(&tsv::tsv_clean(why)));
    cells.push(attempt.to_string());
    qa::write(&pending(dir), &format!("{}\n", cells.join("\t")))?;
    complete(dir, &cells, Some(text), Some(index), part3)?;
    Ok(held)
}

/// Finish the retire qa-retire.pending holds, if any; QA retire and QA record call it first, under
/// the same locks, so no other writer changes the pending scenario meanwhile. Its row is added to
/// the history unless it holds one for the same QA and attempt, the scenario reopened unless it is
/// open already, part 3 published and the pending file removed. Returns the pending row.
pub fn finish(dir: &Path, run: &str) -> Result<Option<Vec<String>>> {
    let Some(cells) = pending_row(dir)? else {
        return Ok(None);
    };
    let (qa, attempt) = (&cells[0], &cells[9]);
    let text = history_text(dir)?;
    let appended = history_rows(&text).any(|row| row[0] == qa.as_str() && row.get(9) == Some(&attempt.as_str()));
    let mut ledger = qa::entries(dir)?;
    let index = match ledger.iter().position(|(row, _)| row.qa == *qa && row.status != "open") {
        Some(at) => Some(reopen(dir, &mut ledger, at)?),
        None => None,
    };
    let part3 = request_part3::preview(dir, run, &ledger)?;
    complete(dir, &cells, (!appended).then_some(text), index, part3)?;
    Ok(Some(cells))
}

/// The scenario at `at` in `ledger` reopened as qa add wrote it; returns the qa.tsv text with it.
fn reopen(dir: &Path, ledger: &mut [QaEntry], at: usize) -> Result<String> {
    let held = &ledger[at].0;
    let row = QaRow::open(held.qa.clone(), held.scenario.clone(), held.body.clone(), held.body_sha256.clone());
    let index = qa::replaced(dir, &row)?;
    ledger[at].0 = row;
    Ok(index)
}

/// The steps after the pending row `cells` is written: add it to the history text `before`,
/// unless that is None, and write the whole file atomically, rewrite the qa.tsv line, publish
/// part 3, remove the pending file. A failed step leaves the pending file, so the same retire run
/// again, or any QA record, finishes it with one history row; a failed history write leaves the
/// file as it was.
fn complete(dir: &Path, cells: &[String], before: Option<String>, index: Option<String>, part3: Option<String>) -> Result<()> {
    let qa = &cells[0];
    let again = format!(
        "run the same dstack evidence retire --qa {qa} --why … again — it finishes the retire without a second history row"
    );
    let failed = |state: &str, e: Error| Error::cannot_decide(format!("{qa}: {state} ({}); {again}", e.message()));
    if let Some(text) = before {
        qa::write(&history(dir), &format!("{text}{}\n", cells.join("\t")))
            .map_err(|e| failed("the retire is pending, but its row is not appended to qa-history.tsv", e))?;
    }
    if let Some(index) = index {
        qa::write(&qa::index(dir), &index)
            .map_err(|e| failed("the history row is written to qa-history.tsv, but qa.tsv still holds the result", e))?;
    }
    if let Some(text) = part3 {
        request_part3::publish(dir, &text)
            .map_err(|e| failed("the retire is written to qa.tsv, but part 3 of request.md is not refreshed", e))?;
    }
    std::fs::remove_file(pending(dir)).map_err(|e| {
        let gone = Error::cannot_decide(format!("cannot remove {}: {e}", pending(dir).display()));
        failed("the retire is written, but it still reads as pending", gone)
    })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::store::qa::QaResult;
    use std::os::unix::fs::PermissionsExt;

    const AGAIN: &str = "run the same dstack evidence retire --qa QA1 --why … again";

    /// The history rows after the header.
    fn retired_rows(dir: &Path) -> usize {
        tsv::read_rows(&history(dir), 1, true).expect("history").len()
    }

    fn mode(dir: &Path, bits: u32) {
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(bits)).expect("mode");
    }

    /// QA1 recorded as failed beside a history of its header alone, in a temp dir of its own.
    fn failed_qa1(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dstack-qa-retire-{name}-{}", std::process::id()));
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
        dir
    }

    /// The row a retire of QA1 writes to qa-retire.pending before its history write, staged there.
    fn staged(dir: &Path) -> String {
        let held = &qa::rows(dir).expect("rows")[0];
        let was = [&held.status, &held.artifact, &held.artifact_sha256, &held.produced_by, &held.recorded_at, &held.note];
        let mut cells = vec!["QA1".to_string(), utc_now()];
        cells.extend(was.iter().map(|cell| cell.to_string()));
        cells.extend(["목록을 고쳤어요".to_string(), "1".to_string()]);
        let row = cells.join("\t");
        qa::write(&pending(dir), &format!("{row}\n")).expect("pending");
        row
    }

    #[test]
    fn R08_qa_rerun_a_failed_history_write_keeps_the_history_the_pending_row_and_the_result() {
        let dir = failed_qa1("history");
        let file = history(&dir);
        let row = staged(&dir);
        let before = std::fs::read(&file).expect("history");
        // A read-only run directory: no temporary file beside the history, so it cannot be written.
        mode(&dir, 0o555);
        let failed = retire(&dir, "run", "QA1", "목록을 고쳤어요");
        mode(&dir, 0o755);
        let message = failed.expect_err("the history write fails").message().to_string();
        assert!(message.contains("its row is not appended to qa-history.tsv"), "{message}");
        assert!(message.contains(AGAIN), "{message}");
        assert_eq!(std::fs::read(&file).expect("history"), before, "the history is byte-identical");
        assert!(pending(&dir).is_file());
        assert_eq!(qa::rows(&dir).expect("rows")[0].status, "failed");

        let held = retire(&dir, "run", "QA1", "목록을 고쳤어요").expect("finished");
        assert_eq!(held.status, "failed");
        assert_eq!(qa::rows(&dir).expect("rows")[0].status, "open");
        let text = read_text(&file).expect("history");
        assert_eq!(text, Some(format!("{HISTORY_HEADER}\n{row}\n")), "one row, every column in place");
        assert!(!pending(&dir).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn R08_qa_rerun_a_failed_qa_tsv_rewrite_is_finished_by_the_same_retire() {
        let dir = failed_qa1("qa-tsv");
        let file = history(&dir);
        // A retire stopped after its history write: the pending row and the history row written.
        let row = staged(&dir);
        qa::write(&file, &format!("{HISTORY_HEADER}\n{row}\n")).expect("history");
        // A read-only run directory: the qa.tsv rewrite, a temporary file renamed over it, fails.
        mode(&dir, 0o555);
        let failed = retire(&dir, "run", "QA1", "목록을 고쳤어요");
        mode(&dir, 0o755);
        let message = failed.expect_err("the qa.tsv rewrite fails").message().to_string();
        assert!(message.contains("history row is written to qa-history.tsv"), "{message}");
        assert!(message.contains(AGAIN), "{message}");
        assert_eq!((qa::rows(&dir).expect("rows")[0].status.as_str(), retired_rows(&dir)), ("failed", 1));
        assert!(pending(&dir).is_file());

        let held = retire(&dir, "run", "QA1", "목록을 고쳤어요").expect("finished");
        assert_eq!(held.status, "failed");
        assert_eq!((qa::rows(&dir).expect("rows")[0].status.as_str(), retired_rows(&dir)), ("open", 1));
        assert_eq!(tsv::read_rows(&file, 10, true).expect("history")[0][9], "1");
        assert!(!pending(&dir).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
