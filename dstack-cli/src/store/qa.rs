// store/qa.rs
// The run-local QA ledger (D-40): one text per Goal QA scenario in qa/QA<n>.md and the index
// qa.tsv, plus the usage scenario ids of a request that a QA scenario may be tied to.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text, sha256_bytes, utc_now};
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

/// What a result records about one open QA scenario. The artifact checks belong to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QaResult {
    pub status: String,
    pub artifact: String,
    pub artifact_sha256: String,
    pub produced_by: String,
    pub note: String,
}

fn index(dir: &Path) -> PathBuf {
    dir.join("qa.tsv")
}

/// Where the text of one QA scenario lives, relative to the run directory as the index names it.
fn body_rel(qa: &str) -> String {
    format!("qa/{qa}.md")
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

/// The usage scenario ids of a request: the `S<n>` that starts each `### S<n> <title>` heading in
/// `## 사용 시나리오`, in order and once each. A heading a comment or a code fence holds shows
/// no heading, so it names none; a request whose outline cannot be read refuses.
pub fn scenario_ids(request: &str) -> Result<Vec<String>> {
    let mut ids: Vec<String> = Vec::new();
    for line in lines(body(request, &SCENARIOS)?) {
        let id = match heading(line.raw.trim_end_matches('\r')) {
            Some((3, title)) if line.heading => title.split_whitespace().next().unwrap_or(""),
            _ => continue,
        };
        let digits = id.strip_prefix('S').unwrap_or("");
        let numbered = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
        if numbered && !ids.iter().any(|held| held == id) {
            ids.push(id.to_string());
        }
    }
    Ok(ids)
}

/// Record a new open QA scenario of the run `run` in `dir`: the next QA<n>, its text written as
/// given, then its index row, so no row names a missing text, then part 3 of the request. The
/// caller holds the store lock and has checked the scenario and the text.
pub fn add(dir: &Path, run: &str, scenario: &str, text: &str) -> Result<QaRow> {
    let held = rows(dir)?;
    let next = held
        .iter()
        .filter_map(|row| row.qa.strip_prefix("QA")?.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let qa = format!("QA{next}");
    let rel = body_rel(&qa);
    let file = dir.join(&rel);
    let parent = file.parent().unwrap_or(dir);
    std::fs::create_dir_all(parent)
        .map_err(|e| Error::cannot_decide(format!("cannot create {}: {e}", parent.display())))?;
    write(&file, text)?;
    let row = QaRow {
        qa,
        scenario: scenario.to_string(),
        status: "open".to_string(),
        body: rel,
        body_sha256: sha256_bytes(text.as_bytes()),
        artifact: "-".to_string(),
        artifact_sha256: "-".to_string(),
        produced_by: "-".to_string(),
        recorded_at: "-".to_string(),
        note: "-".to_string(),
    };
    if !index(dir).is_file() {
        write(&index(dir), &format!("{QA_HEADER}\n"))?;
    }
    tsv::append_line(&index(dir), &row.cells())?;
    request_part3::refresh(dir, run)?;
    Ok(row)
}

/// Record the result of the open QA scenario `qa` in place, then part 3. A recorded result is
/// never overwritten; a status that is no result and a failed, skipped or blocked one without a
/// reason refuse. Every other line is copied verbatim. The caller holds the store lock.
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
    let held = match rows(dir)?.into_iter().find(|row| row.qa == qa) {
        Some(row) => row,
        None => return Err(Error::failed(format!("QA scenario not found: {qa}"))),
    };
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
    let text = read_text(&index(dir))?.unwrap_or_default();
    let mut out = String::new();
    for (at, line) in text.lines().enumerate() {
        match at > 0 && line.split('\t').next() == Some(qa) {
            true => out.push_str(&row.to_line()),
            false => out.push_str(line),
        }
        out.push('\n');
    }
    write(&index(dir), &out)?;
    request_part3::refresh(dir, run)?;
    Ok(row)
}

fn write(file: &Path, text: &str) -> Result<()> {
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
        add(&dir, "run", "S1", "준비해요.\n").expect("added");
        add(&dir, "run", NO_SCENARIO, "확인해요.\n").expect("added");
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
