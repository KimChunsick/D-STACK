// store/request.rs
// The request file: frontmatter, R rows and the approval stamp.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text};
use crate::core::paths::parse_rid;
use crate::store::rows::{self, Row, REQ_SEP};

/// The frontmatter keys, in the order `request new` writes and `check request` reports them.
pub const REQ_FIELDS: [&str; 11] = [
    "work_type",
    "route",
    "external_research",
    "risk_axes",
    "design_review",
    "review",
    "codex_effort",
    "e2e",
    "unit_tests",
    "visual",
    "korean_polish",
];

/// req_enum(): the allowed values of a field. `route` also takes `merge <run-id>`, which only
/// `check request` can validate, so it is listed here as the bare word.
pub fn req_enum(field: &str) -> &'static [&'static str] {
    match field {
        "work_type" => &["web-ui", "http-api", "cli", "library", "docs-writing"],
        "external_research" => &["none", "one-pass"],
        "risk_axes" => &["none", "ux", "perf", "security"],
        "design_review" => &["required", "auto", "skip"],
        "review" | "unit_tests" | "korean_polish" => &["on", "off"],
        "codex_effort" => &["medium", "high", "xhigh"],
        "e2e" => &["capture", "cli", "none"],
        "visual" => &["design", "regression", "none"],
        "route" => &["new-goal", "quick", "merge"],
        _ => &[],
    }
}

/// field_default(): the R41 table. `work_type` has no default — it is what the table keys on.
pub fn field_default(work_type: &str, field: &str) -> &'static str {
    match field {
        "external_research" => "none",
        "risk_axes" => match work_type {
            "web-ui" => "ux",
            "http-api" => "security",
            _ => "none",
        },
        "design_review" => match work_type {
            "docs-writing" => "skip",
            _ => "auto",
        },
        "review" => "on",
        "codex_effort" => match work_type {
            "docs-writing" => "medium",
            _ => "high",
        },
        "e2e" => match work_type {
            "web-ui" => "capture",
            "docs-writing" => "none",
            _ => "cli",
        },
        "unit_tests" => match work_type {
            "docs-writing" => "off",
            _ => "on",
        },
        "visual" => "none",
        "korean_polish" => "on",
        "route" => "new-goal",
        _ => "",
    }
}

/// The request document in memory. Edits rewrite one line and `save` puts the whole text back
/// atomically, which is what the shell's `_line_set` does one edit at a time.
pub struct RequestDoc {
    pub path: PathBuf,
    text: String,
}

impl RequestDoc {
    pub fn load(path: &Path) -> Result<RequestDoc> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", path.display())))?;
        Ok(RequestDoc {
            path: path.to_path_buf(),
            text,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// `wc -l`: the number of newlines, not of lines.
    pub fn line_count(&self) -> usize {
        self.text.matches('\n').count()
    }

    /// req_field(): the value of a frontmatter key, trimmed of spaces and tabs.
    pub fn field(&self, key: &str) -> Option<String> {
        for line in frontmatter(&self.text) {
            let at = match line.find(':') {
                Some(at) => at,
                None => continue,
            };
            if &line[..at] == key {
                return Some(line[at + 1..].trim_matches([' ', '\t']).to_string());
            }
        }
        None
    }

    /// Every key the frontmatter declares, whether or not it is a known field. check_request.sh
    /// reads the same block with its own awk and then word-splits the result, so a key carrying
    /// a space arrives here as two keys — the behaviour its "unknown key" lines depend on.
    pub fn declared_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for line in frontmatter(&self.text) {
            if let Some(at) = line.find(':') {
                keys.extend(line[..at].split_whitespace().map(|k| k.to_string()));
            }
        }
        keys
    }

    pub fn rows(&self) -> Vec<Row> {
        lines_of(&self.text)
            .iter()
            .enumerate()
            .filter_map(|(index, line)| rows::parse_line(index + 1, line))
            .collect()
    }

    pub fn row(&self, id: &str) -> Option<Row> {
        self.rows().into_iter().find(|row| row.id == id)
    }

    /// req_max_id(): the highest id in the file, or 0 when there is none. The shell's awk holds
    /// `substr($1,2)+0` in a double, so an id past 2^53 loses digits there and one past 2^63
    /// still prints positive; the number kept here is the one `$((10#$n))` mints, which is what
    /// `request add` compares a `--id` against.
    pub fn max_id(&self) -> i64 {
        self.rows()
            .iter()
            .filter_map(|row| parse_rid(&row.id))
            .max()
            .unwrap_or(0)
    }

    pub fn live_ids(&self) -> Vec<String> {
        self.rows()
            .into_iter()
            .filter(|row| row.is_live())
            .map(|row| row.id)
            .collect()
    }

    /// req_row_lineno(): the first line holding `] **R<NN>** `, which is a substring search and
    /// not a regex — `**R01**` is all metacharacters, and the `] ` keeps a prose mention out.
    pub fn row_lineno(&self, id: &str) -> Option<usize> {
        let needle = format!("] **{id}** ");
        lines_of(&self.text)
            .iter()
            .position(|line| line.contains(&needle))
            .map(|index| index + 1)
    }

    /// req_row_replace_accept(): the criterion of a row born incomplete.
    pub fn replace_accept(&mut self, id: &str, criterion: &str) -> Result<()> {
        self.edit_row(id, |line| {
            join_segments(line, |segment| {
                if segment.starts_with("accept:") {
                    format!("accept: {criterion}")
                } else {
                    segment.to_string()
                }
            })
        })
    }

    /// req_row_append_marker(): a marker is added at the end of the row, never inside it.
    pub fn append_marker(&mut self, id: &str, marker: &str) -> Result<()> {
        self.edit_row(id, |line| format!("{line}{REQ_SEP}{marker}"))
    }

    /// The `drop` half of _req_row_edit: `request approve` clears the marker it wrote itself.
    pub fn drop_marker(&mut self, id: &str, key: &str) -> Result<()> {
        let prefix = format!("{key}:");
        self.edit_row(id, |line| {
            join_segments(line, |segment| {
                if segment.starts_with(&prefix) {
                    String::new()
                } else {
                    segment.to_string()
                }
            })
        })
    }

    fn edit_row<F: Fn(&str) -> String>(&mut self, id: &str, edit: F) -> Result<()> {
        let lineno = self
            .row_lineno(id)
            .ok_or_else(|| Error::failed(format!("no row {id} in {}", self.path.display())))?;
        let mut lines: Vec<String> = lines_of(&self.text)
            .iter()
            .map(|line| line.to_string())
            .collect();
        lines[lineno - 1] = edit(&lines[lineno - 1]);
        let mut text = lines.join("\n");
        if self.text.ends_with('\n') {
            text.push('\n');
        }
        self.text = text;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        atomic_write(&self.path, self.text.as_bytes())
            .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", self.path.display())))
    }
}

/// req_text_ok(): a segment carrying the separator would grow a marker nobody wrote.
pub fn req_text_ok(what: &str, text: &str) -> Result<()> {
    if text.contains(REQ_SEP) {
        return Err(Error::failed(format!(
            "{what} must not contain '{REQ_SEP}' (it separates row segments): {text}"
        )));
    }
    if text.is_empty() {
        return Err(Error::failed(format!("{what} must not be empty")));
    }
    Ok(())
}

/// The stamp request.approved carries: `sha256 <hash>  approved_at <utc>` (two spaces).
pub struct Approval {
    pub sha256: String,
    pub approved_at: String,
}

/// None is "no stamp of this shape", which is an answer; a stamp that cannot be read is not.
pub fn read_approval(dir: &Path) -> Result<Option<Approval>> {
    let text = match stamp_text(dir)? {
        Some(text) => text,
        None => return Ok(None),
    };
    let fields: Vec<&str> = text.split_whitespace().collect();
    if fields.len() < 4 || fields[0] != "sha256" || fields[2] != "approved_at" {
        return Ok(None);
    }
    Ok(Some(Approval {
        sha256: fields[1].to_string(),
        approved_at: fields[3].to_string(),
    }))
}

/// The stamp file as `cat` gives it to `request show`: trailing newlines dropped. A missing file
/// is None — the run is simply not approved. Every other read error is a cannot-decide (D-12):
/// the shell lets `cat` fail and then judges the empty output a hash mismatch, and that verdict
/// on an unreadable store file is the defect the port does not reproduce.
pub fn stamp_text(dir: &Path) -> Result<Option<String>> {
    let path = dir.join("request.approved");
    Ok(read_text(&path)?.map(|text| text.trim_end_matches('\n').to_string()))
}

pub fn write_approval(dir: &Path, sha256: &str, approved_at: &str) -> Result<()> {
    let path = dir.join("request.approved");
    let line = format!("sha256 {sha256}  approved_at {approved_at}\n");
    atomic_write(&path, line.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

/// How `request show` and `check request` compare: the hash has to appear in the stamp, so a
/// stamp of another shape still answers the one question asked of it.
pub fn approval_matches(dir: &Path, sha256: &str) -> Result<bool> {
    Ok(match stamp_text(dir)? {
        Some(text) => text.contains(sha256),
        None => false,
    })
}

/// The lines of a file, keeping a carriage return and knowing nothing about a trailing newline.
fn lines_of(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// The first `---` block, which is where every frontmatter reader stops.
fn frontmatter(text: &str) -> Vec<&str> {
    let lines = lines_of(text);
    if lines.first() != Some(&"---") {
        return Vec::new();
    }
    lines[1..]
        .iter()
        .take_while(|line| **line != "---")
        .copied()
        .collect()
}

/// _req_row_edit(): split the row on the separator, map each segment, drop the empty ones and
/// join what is left. Done on strings rather than with a regex so `&` and `\1` in a user's text
/// survive the rewrite.
fn join_segments<F: Fn(&str) -> String>(line: &str, edit: F) -> String {
    let mut out = String::new();
    for segment in line.split(REQ_SEP) {
        let segment = edit(segment);
        if segment.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push_str(REQ_SEP);
        }
        out.push_str(&segment);
    }
    out
}
