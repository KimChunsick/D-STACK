// store/cases.rs
// cases.tsv: the requirement ledger with its evidence columns, plus accepts.tsv and metrics.tsv.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::atomic_write;
use crate::store::request::RequestDoc;
use crate::store::tsv;

pub const CASES_HEADER: &str =
    "R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote";
pub const CASES_KINDS: [&str; 6] = ["test", "capture", "transcript", "cli", "visual", "review"];
pub const CASES_EVIDENCE_STATUSES: [&str; 4] = ["met", "abstain", "blocked", "skipped"];
pub const ACCEPTS_HEADER: &str = "R\twhy\taccepted_at";
pub const METRICS_HEADER: &str = "metric\tvalue\tsource";

/// The statuses that count as recorded evidence; skipped, open, unreported and retired do not.
const RECORDED: [&str; 3] = ["met", "abstain", "blocked"];

/// One ledger row. `case` is a keyword in Rust, so the column keeps its name only in the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseRow {
    pub r: String,
    pub case_id: String,
    pub kind: String,
    pub status: String,
    pub artifact: String,
    pub sha256: String,
    pub produced_by: String,
    pub recorded_at: String,
    pub note: String,
}

impl CaseRow {
    /// A row read from the file: a short row pads with the empty cells awk would report.
    pub fn parse(cells: &[String]) -> CaseRow {
        let cell = |index: usize| cells.get(index).cloned().unwrap_or_default();
        CaseRow {
            r: cell(0),
            case_id: cell(1),
            kind: cell(2),
            status: cell(3),
            artifact: cell(4),
            sha256: cell(5),
            produced_by: cell(6),
            recorded_at: cell(7),
            note: cell(8),
        }
    }

    pub fn cells(&self) -> Vec<String> {
        vec![
            self.r.clone(),
            self.case_id.clone(),
            self.kind.clone(),
            self.status.clone(),
            self.artifact.clone(),
            self.sha256.clone(),
            self.produced_by.clone(),
            self.recorded_at.clone(),
            self.note.clone(),
        ]
    }

    pub fn to_line(&self) -> String {
        self.cells().join("\t")
    }
}

fn path(dir: &Path) -> PathBuf {
    dir.join("cases.tsv")
}

/// _cases_ensure(): the header exists before anything appends to it.
pub fn ensure(dir: &Path) -> Result<()> {
    let file = path(dir);
    if file.is_file() {
        return Ok(());
    }
    atomic_write(&file, format!("{CASES_HEADER}\n").as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))
}

/// cases_rows(): every row after the header that awk counts four or more columns in. An absent
/// ledger has no rows; one that cannot be read is a cannot-decide (D-12).
pub fn rows(dir: &Path) -> Result<Vec<CaseRow>> {
    Ok(tsv::read_rows(&path(dir), 4, true)?
        .iter()
        .map(|cells| CaseRow::parse(cells))
        .collect())
}

pub fn for_r(dir: &Path, r: &str) -> Result<Vec<CaseRow>> {
    Ok(rows(dir)?.into_iter().filter(|row| row.r == r).collect())
}

/// _cases_status_of(): the status of one case, or None when the ledger has no such row.
pub fn status_of(dir: &Path, r: &str, case_id: &str) -> Result<Option<String>> {
    Ok(rows(dir)?
        .into_iter()
        .find(|row| row.r == r && row.case_id == case_id)
        .map(|row| row.status))
}

/// _cases_has_kind(): true when R has a row of one of these kinds holding real evidence.
pub fn has_kind(dir: &Path, r: &str, kinds: &[&str]) -> Result<bool> {
    Ok(for_r(dir, r)?
        .iter()
        .any(|row| kinds.contains(&row.kind.as_str()) && RECORDED.contains(&row.status.as_str())))
}

/// _cases_default_kind(): what "verified" means for this Goal is the e2e field (R71).
pub fn default_kind(request: &RequestDoc) -> &'static str {
    match request.field("e2e").unwrap_or_default().as_str() {
        "capture" => "capture",
        "cli" => "cli",
        _ => "review",
    }
}

pub fn count_status(dir: &Path, status: &str) -> Result<usize> {
    Ok(rows(dir)?.iter().filter(|row| row.status == status).count())
}

pub fn append(dir: &Path, row: &CaseRow) -> Result<()> {
    tsv::append_line(&path(dir), &row.cells())
}

/// `evidence add` filling an open or unreported row: the matched line is replaced where it
/// stands, every other line is copied verbatim so a row awk cannot parse still survives.
pub fn replace(dir: &Path, r: &str, case_id: &str, row: &CaseRow) -> Result<()> {
    rewrite_matching(dir, r, case_id, |_| row.to_line())
}

/// `evidence retire`: only the status and the note change; the artifact and its old sha stay in
/// the row as history.
///
/// The awk of evidence.sh assigns `$4` and `$9` and prints the whole record, so a row carrying
/// columns past the ninth (a produced_by written before _tsv_clean existed holds tabs) keeps
/// them, and a shorter row is extended with empty columns. Parsing into CaseRow here would
/// truncate both shapes, so the raw cells are edited.
pub fn retire(dir: &Path, r: &str, case_id: &str, note: &str) -> Result<()> {
    rewrite_matching(dir, r, case_id, |cells| {
        let mut cells = cells.to_vec();
        set_cell(&mut cells, 3, "retired");
        set_cell(&mut cells, 8, note);
        cells.join("\t")
    })
}

/// Assigning a column past NF extends the awk record; the columns in between become empty.
fn set_cell(cells: &mut Vec<String>, index: usize, value: &str) {
    if cells.len() <= index {
        cells.resize(index + 1, String::new());
    }
    cells[index] = value.to_string();
}

fn rewrite_matching<F: Fn(&[String]) -> String>(
    dir: &Path,
    r: &str,
    case_id: &str,
    edit: F,
) -> Result<()> {
    let file = path(dir);
    let text = std::fs::read_to_string(&file)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", file.display())))?;
    let mut out = String::new();
    for (index, line) in text.lines().enumerate() {
        let cells: Vec<String> = line.split('\t').map(|cell| cell.to_string()).collect();
        let cell = |at: usize| cells.get(at).map(String::as_str).unwrap_or("");
        if index > 0 && cell(0) == r && cell(1) == case_id {
            out.push_str(&edit(&cells));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    atomic_write(&file, out.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))
}

/// One accepted ABSTAIN or BLOCKED (§4.9): the reason the report prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptRow {
    pub r: String,
    pub why: String,
    pub accepted_at: String,
}

pub fn accepts_rows(dir: &Path) -> Result<Vec<AcceptRow>> {
    Ok(tsv::read_rows(&dir.join("accepts.tsv"), 1, true)?
        .iter()
        .map(|cells| AcceptRow {
            r: cells.first().cloned().unwrap_or_default(),
            why: cells.get(1).cloned().unwrap_or_default(),
            accepted_at: cells.get(2).cloned().unwrap_or_default(),
        })
        .collect())
}

/// accepts_why(): the reason of the first accepted row for this R, empty when not accepted.
pub fn accepts_why(dir: &Path, r: &str) -> Result<Option<String>> {
    Ok(accepts_rows(dir)?
        .into_iter()
        .find(|row| row.r == r)
        .map(|row| row.why))
}

/// The header is written once, then one row per accepted R; the reason loses tabs and newlines.
pub fn accepts_append(dir: &Path, r: &str, why: &str, accepted_at: &str) -> Result<()> {
    let file = dir.join("accepts.tsv");
    if !file.is_file() {
        atomic_write(&file, format!("{ACCEPTS_HEADER}\n").as_bytes())
            .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))?;
    }
    tsv::append_line(&file, &[r, &tsv::tsv_clean(why), accepted_at])
}

/// One row of the R01 metrics table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRow {
    pub metric: String,
    pub value: String,
    pub source: String,
}

pub fn metrics_rows(dir: &Path) -> Result<Vec<MetricRow>> {
    Ok(tsv::read_rows(&dir.join("metrics.tsv"), 3, true)?
        .iter()
        .map(|cells| MetricRow {
            metric: cells[0].clone(),
            value: cells[1].clone(),
            source: cells[2].clone(),
        })
        .collect())
}

/// `report --metrics` rewrites the whole table every time it runs.
pub fn metrics_write(dir: &Path, rows: &[MetricRow]) -> Result<()> {
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| vec![row.metric.clone(), row.value.clone(), row.source.clone()])
        .collect();
    tsv::rewrite(&dir.join("metrics.tsv"), Some(METRICS_HEADER), &cells)
}
