// verbs/quick/state.rs
// .dstack/quick/STATE.md: the one table of the quick tasks this worktree opened (R33, R99).

use std::io::Write;
use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text, utc_now};

pub const HEADER: &str = "| slug | status | opened | closed |";
pub const RULE: &str = "|---|---|---|---|";

/// The statuses a row may carry. A line is a row only when its second cell holds one of them,
/// so the header and the rule line drop out and a slug literally called "status" cannot fake a
/// row.
const STATUSES: [&str; 3] = ["open", "done", "abandoned"];

pub struct QuickRow {
    pub slug: String,
    pub status: String,
    pub opened: String,
    pub closed: String,
}

/// _quick_rows(): the rows of the table in file order. An absent table has no rows; one that is
/// there and cannot be read is a cannot-decide (D-12), never an empty quick track.
pub fn rows(quick: &Path) -> Result<Vec<QuickRow>> {
    let text = match read_text(&quick.join("STATE.md"))? {
        Some(text) => text,
        None => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for line in text.lines() {
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() < 5 {
            continue;
        }
        let cell = |at: usize| cells[at].trim_matches([' ', '\t']).to_string();
        let status = cell(2);
        if !STATUSES.contains(&status.as_str()) {
            continue;
        }
        rows.push(QuickRow {
            slug: cell(1),
            status,
            opened: cell(3),
            closed: cell(4),
        });
    }
    Ok(rows)
}

/// quick_open_slugs(): the open quick items of THIS worktree — what the Stop gate checks (R33).
pub fn open_slugs(quick: &Path) -> Result<Vec<String>> {
    Ok(rows(quick)?
        .into_iter()
        .filter(|row| row.status == "open")
        .map(|row| row.slug)
        .collect())
}

/// The read ensure() will do, done on its own so a caller can find out whether the table can be
/// read before it writes anything of its own (the read-before-write rule).
pub fn readable(quick: &Path) -> Result<()> {
    read_text(&quick.join("STATE.md")).map(|_| ())
}

/// _quick_state_ensure(): the file, its heading and its table header, each written only where it
/// is missing — an existing table is never rewritten, so rows nobody touched keep their bytes.
pub fn ensure(quick: &Path) -> Result<()> {
    std::fs::create_dir_all(quick).map_err(|e| cannot("create", &quick.join(""), &e))?;
    let path = quick.join("STATE.md");
    if !path.is_file() {
        let fresh = format!("# Quick tasks state\n\n## Quick tasks\n\n{HEADER}\n{RULE}\n");
        return std::fs::write(&path, fresh).map_err(|e| cannot("write", &path, &e));
    }
    if !has_line_starting(&path, "## Quick tasks")? {
        append(&path, "\n## Quick tasks\n\n")?;
    }
    if !has_line_starting(&path, "|---")? {
        append(&path, &format!("{HEADER}\n{RULE}\n"))?;
    }
    Ok(())
}

/// _quick_state_add(): one open row, appended.
pub fn add(quick: &Path, slug: &str) -> Result<()> {
    append(
        &quick.join("STATE.md"),
        &format!("| {slug} | open | {} | |\n", utc_now()),
    )
}

/// _quick_state_close(): the status and the closing stamp of one slug's row, in place. Only a
/// row that already carries a known status is rewritten, so the header survives a slug of its
/// own name, and every other line is put back byte for byte.
pub fn close(quick: &Path, slug: &str, status: &str, when: &str) -> Result<()> {
    let path = quick.join("STATE.md");
    let text = read_text(&path)?.unwrap_or_default();
    let mut out = String::new();
    for line in records(&text) {
        out.push_str(&rewrite(line, slug, status, when));
        out.push('\n');
    }
    atomic_write(&path, out.as_bytes()).map_err(|e| cannot("write", &path, &e))
}

fn rewrite(line: &str, slug: &str, status: &str, when: &str) -> String {
    let mut cells: Vec<&str> = line.split('|').collect();
    if cells.len() < 5 {
        return line.to_string();
    }
    let trimmed = |cell: &str| cell.trim_matches([' ', '\t']).to_string();
    if trimmed(cells[1]) != slug || !STATUSES.contains(&trimmed(cells[2]).as_str()) {
        return line.to_string();
    }
    let (status, when) = (format!(" {status} "), format!(" {when} "));
    cells[2] = &status;
    cells[4] = &when;
    cells.join("|")
}

/// The records awk reads: the lines of the text, without an empty one for the final newline.
fn records(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last().map(|last| last.is_empty()).unwrap_or(false) {
        lines.pop();
    }
    lines
}

/// `grep -q '^<prefix>'`: does any line of the file start with this text?
fn has_line_starting(path: &Path, prefix: &str) -> Result<bool> {
    Ok(read_text(path)?
        .unwrap_or_default()
        .lines()
        .any(|line| line.starts_with(prefix)))
}

fn append(path: &Path, text: &str) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| cannot("write", path, &e))?;
    file.write_all(text.as_bytes())
        .map_err(|e| cannot("write", path, &e))
}

fn cannot(what: &str, path: &Path, error: &std::io::Error) -> Error {
    Error::cannot_decide(format!("cannot {what} {}: {error}", path.display()))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__only_a_known_status_makes_a_row() {
        let dir = std::env::temp_dir().join(format!("dstack-quick-rows-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        std::fs::write(
            dir.join("STATE.md"),
            format!("{HEADER}\n{RULE}\n| one | open | a | |\n| two | done | b | c |\n| bad | x | d | |\nprose\n"),
        )
        .expect("write");
        let rows = rows(&dir).expect("read");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].slug, "one");
        assert_eq!(rows[0].closed, "");
        assert_eq!(rows[1].closed, "c");
        assert_eq!(open_slugs(&dir).expect("read"), vec!["one".to_string()]);
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13__close_rewrites_one_row_and_leaves_the_rest() {
        let dir = std::env::temp_dir().join(format!("dstack-quick-close-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        ensure(&dir).expect("the fresh table");
        add(&dir, "one").expect("a row");
        add(&dir, "two").expect("a row");
        close(&dir, "two", "done", "2026-01-01T00:00:00Z").expect("close");
        let text = std::fs::read_to_string(dir.join("STATE.md")).expect("read");
        assert!(text.starts_with(&format!(
            "# Quick tasks state\n\n## Quick tasks\n\n{HEADER}\n{RULE}\n"
        )));
        assert!(text.contains("| two | done | "));
        assert!(text.ends_with(" | 2026-01-01T00:00:00Z |\n"));
        assert_eq!(open_slugs(&dir).expect("read"), vec!["one".to_string()]);
        // A second ensure() on a table that already has both markers writes nothing at all.
        let before = text.clone();
        ensure(&dir).expect("idempotent");
        assert_eq!(
            std::fs::read_to_string(dir.join("STATE.md")).expect("read"),
            before
        );
        std::fs::remove_dir_all(&dir).expect("clean up");
    }
}
