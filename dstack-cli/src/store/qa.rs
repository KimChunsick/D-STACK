// store/qa.rs
// The run-local QA ledger (D-40): one text per Goal QA scenario in qa/QA<n>.md and the index
// qa.tsv, plus the usage scenario ids of a request that a QA scenario may be tied to.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text, sha256_bytes, utc_now, with_lock, LockGuard};
use crate::store::plan;
use crate::store::request_part3;
use crate::store::request_sections::{body, Place};
use crate::store::tsv;
use crate::store::visible::{heading, lines};

pub const QA_HEADER: &str = "qa\tscenario\tstatus\tbody\tbody_sha256\tartifact\tartifact_sha256\tproduced_by\trecorded_at\tnote";

/// The scenario cell of a check tied to no usage scenario (D-12's QA6).
pub const NO_SCENARIO: &str = "none";

/// The statuses a recorded result carries; a scenario starts `open` and is recorded once.
pub const QA_RESULTS: [&str; 4] = ["met", "failed", "skipped", "blocked"];

/// The `##` section whose `### S<n>` headings name the usage scenarios.
const SCENARIOS: Place = Place::Section("사용 시나리오");

/// One index row; every cell the file holds, `-` for an empty one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QaRow {
    pub qa: String,
    pub scenario: String,
    pub status: String,
    pub body: String,
    pub body_sha256: String,
    pub artifact: String,
    pub artifact_sha256: String,
    pub produced_by: String,
    pub recorded_at: String,
    pub note: String,
}

impl QaRow {
    /// A row read from the file: a short row pads with empty cells.
    pub fn parse(cells: &[String]) -> QaRow {
        let cell = |index: usize| cells.get(index).cloned().unwrap_or_default();
        QaRow {
            qa: cell(0),
            scenario: cell(1),
            status: cell(2),
            body: cell(3),
            body_sha256: cell(4),
            artifact: cell(5),
            artifact_sha256: cell(6),
            produced_by: cell(7),
            recorded_at: cell(8),
            note: cell(9),
        }
    }

    /// A row as qa add writes it: open, every result cell `-`.
    pub fn open(qa: String, scenario: String, body: String, body_sha256: String) -> QaRow {
        let dash = || "-".to_string();
        QaRow {
            qa,
            scenario,
            status: "open".to_string(),
            body,
            body_sha256,
            artifact: dash(),
            artifact_sha256: dash(),
            produced_by: dash(),
            recorded_at: dash(),
            note: dash(),
        }
    }

    pub fn cells(&self) -> Vec<String> {
        vec![
            self.qa.clone(),
            self.scenario.clone(),
            self.status.clone(),
            self.body.clone(),
            self.body_sha256.clone(),
            self.artifact.clone(),
            self.artifact_sha256.clone(),
            self.produced_by.clone(),
            self.recorded_at.clone(),
            self.note.clone(),
        ]
    }

    pub fn to_line(&self) -> String {
        self.cells().join("\t")
    }
}

/// A ledger row with its text, None when the text is gone or cannot be read.
pub type QaEntry = (QaRow, Option<String>);

/// What a result records about one open QA scenario. The artifact checks belong to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QaResult {
    pub status: String,
    pub artifact: String,
    pub artifact_sha256: String,
    pub produced_by: String,
    pub note: String,
}

pub(crate) fn index(dir: &Path) -> PathBuf {
    dir.join("qa.tsv")
}

/// Where the text of one QA scenario lives, relative to the run directory as the index names it.
fn body_rel(qa: &str) -> String {
    format!("qa/{qa}.md")
}

/// The run's lock, `<run>/lock`: every worktree reaches the same run directory, so both QA writers
/// hold it, after their worktree lock, from the ledger read to the last write. Nothing lists a run
/// directory, so the lock directory is never read as a run file.
pub fn lock(dir: &Path) -> Result<LockGuard> {
    with_lock(dir)
}

/// Every index row after the header; an absent ledger has none, an unreadable one cannot decide.
pub fn rows(dir: &Path) -> Result<Vec<QaRow>> {
    Ok(tsv::read_rows(&index(dir), 3, true)?
        .iter()
        .map(|cells| QaRow::parse(cells))
        .collect())
}

/// The text of a QA scenario as its body cell names it, None when the file is gone.
pub fn body_text(dir: &Path, row: &QaRow) -> Result<Option<String>> {
    read_text(&dir.join(&row.body))
}

/// Every row with its text as part 3 shows it: a text that is gone or cannot be read is None, so
/// one bad file never stops part 3, or any plan write that renders it.
pub fn entries(dir: &Path) -> Result<Vec<QaEntry>> {
    let entry = |row: QaRow| {
        let text = body_text(dir, &row).ok().flatten();
        (row, text)
    };
    Ok(rows(dir)?.into_iter().map(entry).collect())
}

/// The usage scenarios of a request: the `S<n>` that starts each `### S<n> <title>` heading in
/// `## 사용 시나리오` with the title after it, in order and once each. A heading a comment or a
/// code fence holds shows no heading, so it names none; a request whose outline cannot be read
/// refuses.
pub fn scenarios(request: &str) -> Result<Vec<(String, String)>> {
    let mut found: Vec<(String, String)> = Vec::new();
    for line in lines(body(request, &SCENARIOS)?) {
        let text = match heading(line.raw.trim_end_matches('\r')) {
            Some((3, text)) if line.heading => text.trim_start(),
            _ => continue,
        };
        let (id, title) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
        let digits = id.strip_prefix('S').unwrap_or("");
        let numbered = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
        if numbered && !found.iter().any(|(held, _)| held == id) {
            found.push((id.to_string(), title.trim().to_string()));
        }
    }
    Ok(found)
}

/// The usage scenario ids alone, as `scenarios` finds them.
pub fn scenario_ids(request: &str) -> Result<Vec<String>> {
    Ok(scenarios(request)?.into_iter().map(|(id, _)| id).collect())
}

/// Record a new open QA scenario of the run `run` in `dir` as the next QA<n>: part 3 rendered with
/// it first, so a refusal writes nothing; then its text written as given, then its index row, so
/// no row names a missing text, then part 3 of the request. The caller holds its worktree lock and
/// then the run lock (`lock`), and has checked the scenario and the text.
pub fn add(dir: &Path, run: &str, scenario: &str, text: &str) -> Result<QaRow> {
    let mut ledger = entries(dir)?;
    let next = ledger
        .iter()
        .filter_map(|(row, _)| row.qa.strip_prefix("QA")?.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let qa = format!("QA{next}");
    let rel = body_rel(&qa);
    let file = dir.join(&rel);
    let row = QaRow::open(qa, scenario.to_string(), rel, sha256_bytes(text.as_bytes()));
    ledger.push((row.clone(), Some(text.to_string())));
    let part3 = request_part3::preview(dir, run, &ledger)?;
    let parent = file.parent().unwrap_or(dir);
    std::fs::create_dir_all(parent)
        .map_err(|e| Error::cannot_decide(format!("cannot create {}: {e}", parent.display())))?;
    write(&file, text)?;
    if !index(dir).is_file() {
        write(&index(dir), &format!("{QA_HEADER}\n"))?;
    }
    tsv::append_line(&index(dir), &row.cells())?;
    publish(dir, &row.qa, part3, ["recorded", "record"])?;
    Ok(row)
}

/// Record the result of the open QA scenario `qa` in place, then part 3, which is rendered before
/// the first write so a refusal writes nothing. A recorded result is never overwritten; a status
/// that is no result and a failed, skipped or blocked one without a reason refuse. Every other
/// line is copied verbatim. The caller holds its worktree lock and then the run lock (`lock`).
pub fn record(dir: &Path, run: &str, qa: &str, result: &QaResult) -> Result<QaRow> {
    if !QA_RESULTS.contains(&result.status.as_str()) {
        return Err(Error::failed(format!(
            "unknown QA result: {} (one of {})",
            result.status,
            QA_RESULTS.join(", ")
        )));
    }
    if result.status != "met" && result.note.trim().is_empty() {
        return Err(Error::failed(format!(
            "a {} QA result needs a reason (--note)",
            result.status
        )));
    }
    let mut ledger = entries(dir)?;
    let Some(at) = ledger.iter().position(|(row, _)| row.qa == qa) else {
        return Err(Error::failed(format!("QA scenario not found: {qa}")));
    };
    let held = ledger[at].0.clone();
    if held.status != "open" {
        return Err(Error::failed(format!(
            "{qa} is already recorded (status {}); a recorded QA result is never overwritten",
            held.status
        )));
    }
    let row = QaRow {
        status: result.status.clone(),
        artifact: tsv::dash(&tsv::tsv_clean(&result.artifact)),
        artifact_sha256: tsv::dash(&result.artifact_sha256),
        produced_by: tsv::dash(&tsv::tsv_clean(&result.produced_by)),
        recorded_at: utc_now(),
        note: tsv::dash(&tsv::tsv_clean(&result.note)),
        ..held
    };
    let out = replaced(dir, &row)?;
    ledger[at].0 = row.clone();
    let part3 = request_part3::preview(dir, run, &ledger)?;
    write(&index(dir), &out)?;
    publish(dir, qa, part3, ["recorded", "record"])?;
    Ok(row)
}

/// The index text with the line of `row`'s QA id replaced by `row`; every other line verbatim.
pub(crate) fn replaced(dir: &Path, row: &QaRow) -> Result<String> {
    let text = read_text(&index(dir))?.unwrap_or_default();
    let mut out = String::new();
    for (at, line) in text.lines().enumerate() {
        match at > 0 && line.split('\t').next() == Some(row.qa.as_str()) {
            true => out.push_str(&row.to_line()),
            false => out.push_str(line),
        }
        out.push('\n');
    }
    Ok(out)
}

/// Part 3 written after the ledger. The QA write (`done` and `deed` name it: recorded and record,
/// retired and retire) stands even when this one fails, so the refusal says so and names what
/// writes part 3 again; repeating the QA write would not.
pub(crate) fn publish(dir: &Path, qa: &str, part3: Option<String>, [done, deed]: [&str; 2]) -> Result<()> {
    let Some(text) = part3 else {
        return Ok(());
    };
    request_part3::publish(dir, &text).map_err(|e| {
        let again = match plan::exists(dir) {
            true => "dstack plan render writes it again",
            false => "the next plan or QA write (dstack milestone add) writes it again",
        };
        Error::cannot_decide(format!(
            "{qa} is {done} in qa.tsv, but part 3 of request.md is not refreshed ({}); {again} — do not {deed} {qa} again",
            e.message()
        ))
    })
}

pub(crate) fn write(file: &Path, text: &str) -> Result<()> {
    atomic_write(file, text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn R08_qa_record_scenario_ids_read_visible_level_three_headings() {
        let text = "# 제목\n\n요약이에요.\n\n## 사용 시나리오\n\n### S1 하나\n\n#### S5 깊어요\n\
                    <!--\n### S2 숨겨요\n-->\n~~~\n### S3 코드예요\n~~~\n### S4\n### S1 다시\n\
                    ### Sx 아니에요\n\n## 요구사항\n\n### S6 밖이에요\n";
        assert_eq!(scenario_ids(text).expect("reads"), ["S1", "S4"]);
        let titles = scenarios(text).expect("reads");
        assert_eq!(titles, [("S1".into(), "하나".into()), ("S4".into(), String::new())]);
        assert!(scenario_ids("# 제목\n\n## 목표\n").is_err());
    }

    #[test]
    fn R08_qa_record_a_recorded_result_is_never_overwritten() {
        let dir = std::env::temp_dir().join(format!("dstack-qa-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let result = |status: &str, note: &str| QaResult {
            status: status.into(),
            artifact: "artifacts/qa1.txt".into(),
            artifact_sha256: "abc".into(),
            produced_by: "dstack qa".into(),
            note: note.into(),
        };
        add(&dir, "run", "S1", "준비: 저장소예요.\n단계: 실행해요.\n기대 결과: 0이에요.\n").expect("added");
        add(&dir, "run", NO_SCENARIO, "준비: 없어요.\n단계: 확인해요.\n기대 결과: 0이에요.\n").expect("added");
        assert!(record(&dir, "run", "QA1", &result("failed", " ")).is_err());
        assert!(record(&dir, "run", "QA1", &result("open", "")).is_err());
        assert!(record(&dir, "run", "QA9", &result("met", "")).is_err());
        let row = record(&dir, "run", "QA1", &result("met", "")).expect("recorded");
        assert_eq!((row.status.as_str(), row.note.as_str()), ("met", "-"));
        let before = std::fs::read(dir.join("qa.tsv")).expect("index");
        let again = record(&dir, "run", "QA1", &result("skipped", "다시 해요")).expect_err("kept");
        assert!(again.message().contains("already recorded (status met)"), "{}", again.message());
        assert_eq!(std::fs::read(dir.join("qa.tsv")).expect("index"), before);
        let held = rows(&dir).expect("rows");
        assert_eq!((held[1].qa.as_str(), held[1].status.as_str()), ("QA2", "open"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
