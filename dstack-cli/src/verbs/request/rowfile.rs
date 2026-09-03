// verbs/request/rowfile.rs
// The line primitives request.md is edited with: the record split, head/tail slicing, row lines.
//
// _line_set and _line_insert_after are `head`/`tail` in the shell, chosen there so a row holding
// a backslash or an `&` is copied byte for byte. The same is done here on the document text,
// which the caller writes back atomically.

use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::atomic_write;
use crate::store::rows::REQ_SEP;

/// The lines of the text, as awk counts records: a trailing newline ends the last line and does
/// not start an empty one.
pub fn lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&str> = text.split('\n').collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// `head -n <n>`: the first n lines with the newlines they carry in the source.
fn head(text: &str, n: usize) -> &str {
    if n == 0 {
        return "";
    }
    let mut seen = 0;
    for (at, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            seen += 1;
            if seen == n {
                return &text[..at + 1];
            }
        }
    }
    text
}

/// `tail -n +<k>`: everything from line k on, empty when the text has fewer lines.
fn tail_from(text: &str, k: usize) -> &str {
    if k <= 1 {
        return text;
    }
    let mut seen = 0;
    for (at, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            seen += 1;
            if seen == k - 1 {
                return &text[at + 1..];
            }
        }
    }
    ""
}

/// _line_set(): line <lineno> becomes <line>, every other byte is copied.
pub fn set_line(text: &str, lineno: usize, line: &str) -> String {
    format!(
        "{}{line}\n{}",
        head(text, lineno - 1),
        tail_from(text, lineno + 1)
    )
}

/// The `drop` half of _req_row_edit: every segment opening with `<key>:` leaves the row.
pub fn drop_segment(line: &str, key: &str) -> String {
    let prefix = format!("{key}:");
    let mut out = String::new();
    for segment in line.split(REQ_SEP) {
        if segment.starts_with(&prefix) || segment.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push_str(REQ_SEP);
        }
        out.push_str(segment);
    }
    out
}

/// _line_insert_after(): <line> after line <lineno>; lineno 0 appends at the end of the file.
pub fn insert_after(text: &str, lineno: usize, line: &str) -> String {
    if lineno == 0 {
        return format!("{text}{line}\n");
    }
    format!(
        "{}{line}\n{}",
        head(text, lineno),
        tail_from(text, lineno + 1)
    )
}

/// req_last_row_lineno(): the line number of the last R row, 0 when there is none.
pub fn last_row_lineno(text: &str) -> usize {
    let mut last = 0;
    for (index, line) in lines(text).iter().enumerate() {
        if is_row_line(line) {
            last = index + 1;
        }
    }
    last
}

/// The awk match `/^- \[[ xX]\] \*\*R[0-9]+\*\* /` every row reader starts from.
pub fn is_row_line(line: &str) -> bool {
    let rest = match line.strip_prefix("- [") {
        Some(rest) => rest,
        None => return false,
    };
    let rest = match rest.chars().next() {
        Some(box_char @ (' ' | 'x' | 'X')) => &rest[box_char.len_utf8()..],
        _ => return false,
    };
    let rest = match rest.strip_prefix("] **R") {
        Some(rest) => rest,
        None => return false,
    };
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && rest[digits..].starts_with("** ")
}

pub fn write(path: &Path, text: &str) -> Result<()> {
    atomic_write(path, text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_lines_count_records_the_way_awk_counts_them() {
        assert_eq!(lines("one\ntwo\n"), vec!["one", "two"]);
        assert_eq!(lines("one\ntwo"), vec!["one", "two"]);
        assert_eq!(lines("\n"), vec![""]);
        assert!(lines("").is_empty());
    }

    #[test]
    fn r13_head_and_tail_cut_where_the_shell_cuts() {
        let text = "one\ntwo\nthree\n";
        assert_eq!(head(text, 0), "");
        assert_eq!(head(text, 2), "one\ntwo\n");
        assert_eq!(head(text, 9), text);
        assert_eq!(tail_from(text, 1), text);
        assert_eq!(tail_from(text, 3), "three\n");
        assert_eq!(tail_from(text, 9), "");
        // `head -n 2` of a file whose last line has no newline keeps it without one.
        assert_eq!(head("one\ntwo", 2), "one\ntwo");
    }

    #[test]
    fn r13_insert_after_puts_the_row_where_the_shell_puts_it() {
        let text = "one\ntwo\nthree\n";
        assert_eq!(insert_after(text, 2, "x"), "one\ntwo\nx\nthree\n");
        assert_eq!(insert_after(text, 0, "x"), "one\ntwo\nthree\nx\n");
        assert_eq!(insert_after(text, 3, "x"), "one\ntwo\nthree\nx\n");
    }

    #[test]
    fn r13_row_lines_are_the_awk_match() {
        assert!(is_row_line("- [ ] **R01** text — accept: c"));
        assert!(is_row_line("- [x] **R123** text — accept: c"));
        assert!(!is_row_line("- [ ] **R01**text"));
        assert!(!is_row_line("- [ ] **R** text"));
        assert!(!is_row_line("  - [ ] **R01** text"));
        assert!(!is_row_line("- [y] **R01** text"));
        assert_eq!(last_row_lineno("a\n- [ ] **R01** t — accept: c\nb\n"), 2);
        assert_eq!(last_row_lineno("a\nb\n"), 0);
    }

    #[test]
    fn r13_set_line_and_drop_segment_edit_one_row() {
        assert_eq!(set_line("one\ntwo\nthree\n", 2, "TWO"), "one\nTWO\nthree\n");
        let row = "- [ ] **R01** t — accept: c — from: Q-01 — status: pending-approval";
        assert_eq!(
            drop_segment(row, "status"),
            "- [ ] **R01** t — accept: c — from: Q-01"
        );
        assert_eq!(drop_segment(row, "nothing"), row);
    }
}
