// store/tables.rs
// The two markdown ledgers: questions.md and decisions.md.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text};

pub const ASK_HEADER: &str = "| Q | Question | Affects | Status |\n|---|---|---|---|";
pub const DEC_HEADER: &str = "| D | Decision | Affects | Status |\n|---|---|---|---|";

/// One row of questions.md. The text is what was asked and never changes; the status moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub affects: String,
    pub status: String,
}

/// One row of decisions.md — `D-NN` and `D-DESIGN-NN` alike.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub id: String,
    pub text: String,
    pub affects: String,
    pub status: String,
}

/// ask_q_rows(): every row whose first cell is a Q id.
pub fn questions(path: &Path) -> Result<Vec<Question>> {
    Ok(table_cells(path, |cell| {
        let id = cell.trim_matches(' ');
        match id.strip_prefix("Q-") {
            Some(digits) => !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
            None => false,
        }
    })?
    .into_iter()
    .map(|cells| Question {
        id: cells[0].clone(),
        question: cells[1].clone(),
        affects: cells[2].clone(),
        status: cells[3].clone(),
    })
    .collect())
}

pub fn q_count(path: &Path, status: &str) -> Result<usize> {
    Ok(questions(path)?
        .iter()
        .filter(|row| row.status == status)
        .count())
}

/// ask_q_field(): column 2 is the question, 3 the affects, 4 the status — the shell's numbering.
pub fn q_field(path: &Path, id: &str, column: usize) -> Result<Option<String>> {
    let row = match questions(path)?.into_iter().find(|row| row.id == id) {
        Some(row) => row,
        None => return Ok(None),
    };
    Ok(match column {
        2 => Some(row.question),
        3 => Some(row.affects),
        4 => Some(row.status),
        _ => None,
    })
}

pub fn q_next_id(path: &Path) -> Result<String> {
    let highest = questions(path)?
        .iter()
        .map(|row| leading_number(row.id.trim_start_matches("Q-")))
        .max()
        .unwrap_or(0);
    Ok(format!("Q-{:02}", highest + 1))
}

/// The file is created with its preamble when it does not exist yet (_ask_init).
pub fn q_append(path: &Path, id: &str, question: &str, affects: &str, status: &str) -> Result<()> {
    init(
        path,
        &format!("# Questions (R51)\n\nWritten only by `dstack ask`.\n\n{ASK_HEADER}\n"),
    )?;
    append_line(
        path,
        &format!("| {id} | {question} | {affects} | {status} |"),
    )
}

/// The one in-place edit the question ledger allows: the row is rewritten from its parsed cells,
/// so a hand-spaced row comes back in the shape `dstack ask` writes.
pub fn q_set_status(path: &Path, id: &str, status: &str) -> Result<()> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", path.display())))?;
    let needle = format!("| {id} |");
    let at = text
        .lines()
        .position(|line| line.contains(&needle))
        .ok_or_else(|| Error::failed(format!("unknown question: {id}")))?;
    let question = q_field(path, id, 2)?.unwrap_or_default();
    let affects = q_field(path, id, 3)?.unwrap_or_default();
    set_line(
        path,
        &text,
        at,
        &format!("| {id} | {question} | {affects} | {status} |"),
    )
}

/// _ledger_text_ok(): a `|` would split a markdown cell into two, so it is refused at the door.
/// decision.sh keeps this one function for both ledgers.
pub fn q_text_ok(what: &str, text: &str) -> Result<()> {
    if text.contains('|') {
        return Err(Error::failed(format!(
            "{what} must not contain '|': the ledger is a markdown table ({text})"
        )));
    }
    if text.is_empty() {
        return Err(Error::failed(format!("{what} must not be empty")));
    }
    Ok(())
}

/// dec_rows(): every row whose first cell starts with `D-`.
pub fn decisions(path: &Path) -> Result<Vec<Decision>> {
    Ok(
        table_cells(path, |cell| cell.trim_start_matches(' ').starts_with("D-"))?
            .into_iter()
            .map(|cells| Decision {
                id: cells[0].clone(),
                text: cells[1].clone(),
                affects: cells[2].clone(),
                status: cells[3].clone(),
            })
            .collect(),
    )
}

pub fn d_design_rows(path: &Path) -> Result<Vec<Decision>> {
    Ok(decisions(path)?
        .into_iter()
        .filter(|row| row.id.starts_with("D-DESIGN-"))
        .collect())
}

/// The two counters run independently: mixing them would make "design round 3" stop meaning
/// the third design round.
pub fn d_next_id(path: &Path, design: bool) -> Result<String> {
    if design {
        let highest = d_design_rows(path)?
            .iter()
            .map(|row| leading_number(row.id.trim_start_matches("D-DESIGN-")))
            .max()
            .unwrap_or(0);
        Ok(format!("D-DESIGN-{:02}", highest + 1))
    } else {
        let highest = decisions(path)?
            .iter()
            .filter(|row| is_plain_id(&row.id))
            .map(|row| leading_number(row.id.trim_start_matches("D-")))
            .max()
            .unwrap_or(0);
        Ok(format!("D-{:02}", highest + 1))
    }
}

pub fn d_append(path: &Path, id: &str, text: &str, affects: &str, status: &str) -> Result<()> {
    init(
        path,
        &format!(
            "# Decisions (R51)\n\nWritten only by `dstack decision` and `dstack ask`.\n\n{DEC_HEADER}\n"
        ),
    )?;
    append_line(path, &format!("| {id} | {text} | {affects} | {status} |"))
}

/// dec_has_for_q(): the decision row that already records this question, if there is one.
pub fn d_has_for_q(path: &Path, qid: &str) -> Result<Option<String>> {
    let needle = format!("(from {qid}");
    Ok(decisions(path)?
        .into_iter()
        .find(|row| row.text.contains(&needle))
        .map(|row| row.id))
}

/// dec_design_reason(): what stands between `design round N: ` and the ` — ` of the decision.
/// Empty for a row that is not a design round — R55 needs the reason separable.
pub fn d_design_reason(text: &str) -> String {
    if !text.starts_with("design round ") {
        return String::new();
    }
    let rest = match text.find(": ") {
        Some(at) => &text[at + 2..],
        None => text,
    };
    let rest = match rest.find(" — ") {
        Some(at) => &rest[..at],
        None => rest,
    };
    rest.trim_matches(' ').to_string()
}

/// The four cells of every table row whose first cell passes the test, trimmed of spaces and
/// tabs. A row carrying a `|` inside a cell was refused when it was written, so the split is
/// the whole grammar. A ledger that is not there has no rows; one that cannot be read is a
/// cannot-decide (D-12), because "no rows" is an answer nobody may give for it.
fn table_cells<F: Fn(&str) -> bool>(path: &Path, is_id: F) -> Result<Vec<[String; 4]>> {
    let text = match read_text(path)? {
        Some(text) => text,
        None => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 5 || !is_id(parts[1]) {
            continue;
        }
        let cell = |index: usize| parts[index].trim_matches([' ', '\t']).to_string();
        rows.push([cell(1), cell(2), cell(3), cell(4)]);
    }
    Ok(rows)
}

fn is_plain_id(id: &str) -> bool {
    match id.strip_prefix("D-") {
        Some(digits) => !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
        None => false,
    }
}

/// awk reads `Q-1x` as 1: the leading digits decide, and anything else is 0.
fn leading_number(text: &str) -> u32 {
    let digits: String = text
        .trim_start_matches(' ')
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().unwrap_or(0)
}

fn init(path: &Path, preamble: &str) -> Result<()> {
    if path.is_file() {
        return Ok(());
    }
    atomic_write(path, preamble.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

fn append_line(path: &Path, line: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))?;
    writeln!(file, "{line}")
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

/// _line_set(): one line replaced, every other byte copied.
fn set_line(path: &Path, text: &str, index: usize, line: &str) -> Result<()> {
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    lines[index] = line.to_string();
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    atomic_write(path, out.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}
