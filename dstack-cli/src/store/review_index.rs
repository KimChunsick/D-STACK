// store/review_index.rs
// The review index: sealed rounds, their per-R verdict counts and the closed reviews.

use std::path::{Path, PathBuf};

use regex::Regex;

use crate::core::error::Result;
use crate::core::fsx::read_text;
use crate::store::tsv;

/// One sealed round, as review/index.tsv records it. The file carries no header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexRow {
    pub round: String,
    pub scope: String,
    pub id: String,
    pub filename: String,
    pub timestamp: String,
    pub absent: String,
    pub partial: String,
    pub covered: String,
}

/// One deliberate stop, as review/closed.tsv records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedRow {
    pub scope: String,
    pub id: String,
    pub round: String,
    pub ids: String,
    pub timestamp: String,
    pub why: String,
}

fn index_path(dir: &Path) -> PathBuf {
    dir.join("review/index.tsv")
}

fn closed_path(dir: &Path) -> PathBuf {
    dir.join("review/closed.tsv")
}

/// A short row reads as awk sees it: the missing columns are empty, never absent. An index that
/// cannot be read is a cannot-decide (D-12), never an index of no rounds.
pub fn index_rows(dir: &Path) -> Result<Vec<IndexRow>> {
    Ok(tsv::read_rows(&index_path(dir), 1, false)?
        .iter()
        .map(|cells| {
            let cell = |index: usize| cells.get(index).cloned().unwrap_or_default();
            IndexRow {
                round: cell(0),
                scope: cell(1),
                id: cell(2),
                filename: cell(3),
                timestamp: cell(4),
                absent: cell(5),
                partial: cell(6),
                covered: cell(7),
            }
        })
        .collect())
}

pub fn index_append(dir: &Path, row: &IndexRow) -> Result<()> {
    let path = index_path(dir);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    tsv::append_line(
        &path,
        &[
            &row.round,
            &row.scope,
            &row.id,
            &row.filename,
            &row.timestamp,
            &row.absent,
            &row.partial,
            &row.covered,
        ],
    )
}

/// _next_seq(): one past the highest three-digit sequence among `<prefix>NNN.*` in this
/// directory. `dir` is the review directory itself, as the shell passes it.
pub fn next_seq(dir: &Path, prefix: &str) -> String {
    let mut highest = 0u32;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if !entry.path().is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with(prefix) {
                continue;
            }
            let tail = name.rsplit('-').next().unwrap_or_default();
            let digits = tail.split('.').next().unwrap_or_default();
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let seq: u32 = digits.parse().unwrap_or(0);
            if seq > highest {
                highest = seq;
            }
        }
    }
    format!("{:03}", highest + 1)
}

/// _sealed_counts(): how many rounds judged this plan and what the latest one found. An
/// older round's "absent" is not news once a later round covered it, so the last row wins.
pub fn sealed_counts(dir: &Path, id: &str) -> Result<(usize, String, String, String)> {
    let mut rounds = 0;
    let mut latest = None;
    for row in index_rows(dir)? {
        if row.id == id {
            rounds += 1;
            latest = Some(row);
        }
    }
    Ok(match latest {
        Some(row) => (rounds, row.absent, row.partial, row.covered),
        None => (0, "-".to_string(), "-".to_string(), "-".to_string()),
    })
}

/// _verdict_count(): the `| R01 | covered | … |` rows of a sealed round carrying this verdict.
/// A file that is not there has no verdict rows, which is what `review seal` refuses on; a file
/// that is there and cannot be read is a cannot-decide (D-12).
pub fn verdict_count(file: &Path, verdict: &str) -> Result<usize> {
    let text = match read_text(file)? {
        Some(text) => text,
        None => return Ok(0),
    };
    let pattern = format!(
        r"^[ \t]*\|[ \t]*R[0-9][0-9]*[ \t]*\|[ \t]*{}[ \t]*\|",
        regex::escape(verdict)
    );
    let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => return Ok(0),
    };
    Ok(text.lines().filter(|line| re.is_match(line)).count())
}

pub fn closed_rows(dir: &Path) -> Result<Vec<ClosedRow>> {
    Ok(tsv::read_rows(&closed_path(dir), 1, false)?
        .iter()
        .map(|cells| {
            let cell = |index: usize| cells.get(index).cloned().unwrap_or_default();
            ClosedRow {
                scope: cell(0),
                id: cell(1),
                round: cell(2),
                ids: cell(3),
                timestamp: cell(4),
                why: cell(5),
            }
        })
        .collect())
}

/// The reason loses its tabs and newlines, as the row is one line of a TSV.
pub fn closed_append(
    dir: &Path,
    scope: &str,
    id: &str,
    round: &str,
    ids: &str,
    timestamp: &str,
    why: &str,
) -> Result<()> {
    let path = closed_path(dir);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    tsv::append_line(
        &path,
        &[scope, id, round, ids, timestamp, &tsv::tsv_clean(why)],
    )
}

pub fn is_closed(dir: &Path, scope: &str, id: &str, round: &str) -> Result<bool> {
    Ok(closed_rows(dir)?
        .iter()
        .any(|row| row.scope == scope && row.id == id && row.round == round))
}

/// The round a close comes after: the latest sealed round for this scope and id, else `000`.
pub fn latest_round(dir: &Path, scope: &str, id: &str) -> Result<String> {
    Ok(index_rows(dir)?
        .into_iter()
        .filter(|row| row.scope == scope && row.id == id)
        .next_back()
        .map(|row| row.round)
        .unwrap_or_else(|| "000".to_string()))
}
