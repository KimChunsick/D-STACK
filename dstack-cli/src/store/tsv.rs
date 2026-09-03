// store/tsv.rs
// The tab-separated reader and writer every table goes through.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text};

/// _tsv_clean(): tabs and newlines would invent columns and rows, so free text loses them.
pub fn tsv_clean(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\t' || c == '\n' { ' ' } else { c })
        .collect()
}

/// _dash(): an empty cell is written as `-`, because a trailing empty column does not survive
/// an editor or a pipe that trims whitespace.
pub fn dash(cell: &str) -> String {
    if cell.is_empty() {
        "-".to_string()
    } else {
        cell.to_string()
    }
}

/// _undash(): `-` reads back as the empty cell.
pub fn undash(cell: &str) -> String {
    if cell == "-" {
        String::new()
    } else {
        cell.to_string()
    }
}

/// The rows of a table, cells split on the tab.
///
/// `min_cols` is awk's `NF >= n` guard and `skip_header` is its `NR > 1`: the first line goes
/// unread whatever it holds. An absent file reads as no rows, as every shell reader does; a file
/// that is there and cannot be read is a cannot-decide (D-12), never no rows.
pub fn read_rows(path: &Path, min_cols: usize, skip_header: bool) -> Result<Vec<Vec<String>>> {
    let text = match read_text(path)? {
        Some(text) => text,
        None => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if skip_header && index == 0 {
            continue;
        }
        // awk counts no fields in an empty record; splitting would count one.
        if line.is_empty() && min_cols > 0 {
            continue;
        }
        let cells: Vec<String> = line.split('\t').map(|c| c.to_string()).collect();
        if cells.len() >= min_cols {
            rows.push(cells);
        }
    }
    Ok(rows)
}

/// One appended row, the file created when it is absent (the shell's `>>`).
pub fn append_line<S: AsRef<str>>(path: &Path, cells: &[S]) -> Result<()> {
    let mut line = String::new();
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            line.push('\t');
        }
        line.push_str(cell.as_ref());
    }
    line.push('\n');
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))?;
    file.write_all(line.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

/// The whole table rewritten atomically: the header, then every row.
pub fn rewrite<S: AsRef<str>>(path: &Path, header: Option<&str>, rows: &[Vec<S>]) -> Result<()> {
    let mut text = String::new();
    if let Some(header) = header {
        text.push_str(header);
        text.push('\n');
    }
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if index > 0 {
                text.push('\t');
            }
            text.push_str(cell.as_ref());
        }
        text.push('\n');
    }
    atomic_write(path, text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}
