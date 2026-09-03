// verbs/request/udiff.rs
// The `diff -u` request approve prints, rendered in process (R01 leaves git as the only child).
//
// An LCS walk with three lines of context, the headers and the hunk shape BSD diff writes. D-15
// records the two limits: the header stamp is the file's mtime in UTC where BSD diff prints it in
// the machine's local zone (the `time` crate reads no local offset in this build), and where
// several edit scripts are equally short the hunk grouping may differ from BSD diff's. A third
// limit is this file's own: past MAX_CELLS the quadratic table is not built at all and the whole
// differing region becomes one replacement hunk.

use std::path::Path;

use time::macros::format_description;
use time::OffsetDateTime;

use crate::core::fsx::file_mtime;

const CONTEXT: usize = 3;

/// The largest LCS table this renderer builds: four million cells is a 2000-line pair, far past
/// any request.md (R43 caps one at 60 lines). A larger pair is rendered as one replacement hunk
/// rather than as a table of usize that would grow with the product of the two lengths.
const MAX_CELLS: usize = 4_000_000;

/// The bound is on the allocation, not on the product of the two lengths: the table has one row
/// and one column more than the texts have lines, so an empty side still allocates the other
/// side's length. `0 * m` would pass a check on the product and allocate m + 1 cells anyway.
fn table_fits(n: usize, m: usize) -> bool {
    n.saturating_add(1).saturating_mul(m.saturating_add(1)) <= MAX_CELLS
}

const STAMP: &[time::format_description::BorrowedFormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

/// One line as diff compares them: a last line without a newline is not the same line as one
/// with, which is why the flag belongs to the key.
#[derive(PartialEq, Eq)]
struct Line {
    text: String,
    newline: bool,
}

enum Op {
    Same(usize, usize),
    Del(usize),
    Ins(usize),
}

/// The whole diff, empty when the two texts are equal.
pub fn unified(old_path: &Path, old_text: &str, new_path: &Path, new_text: &str) -> String {
    let (old, new) = (split(old_text), split(new_text));
    let ops = script(&old, &new);
    let changed: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, op)| !matches!(op, Op::Same(_, _)))
        .map(|(at, _)| at)
        .collect();
    if changed.is_empty() {
        return String::new();
    }
    let mut out = format!(
        "--- {}\t{}\n+++ {}\t{}\n",
        old_path.display(),
        stamp(old_path),
        new_path.display(),
        stamp(new_path)
    );
    for group in groups(&changed) {
        let from = group[0].saturating_sub(CONTEXT);
        let to = (group[group.len() - 1] + CONTEXT).min(ops.len() - 1);
        out.push_str(&hunk(&ops[from..=to], &old, &new));
    }
    out
}

/// Change positions that are closer than twice the context share one hunk, as diff joins them.
fn groups(changed: &[usize]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    for at in changed {
        match out.last_mut() {
            Some(group) if at - group[group.len() - 1] - 1 <= 2 * CONTEXT => group.push(*at),
            _ => out.push(vec![*at]),
        }
    }
    out
}

fn hunk(ops: &[Op], old: &[Line], new: &[Line]) -> String {
    let (mut old_at, mut new_at) = (usize::MAX, usize::MAX);
    let (mut old_len, mut new_len) = (0, 0);
    let mut body = String::new();
    for op in ops {
        let (mark, line, index) = match op {
            Op::Same(a, b) => {
                old_at = old_at.min(*a);
                new_at = new_at.min(*b);
                old_len += 1;
                new_len += 1;
                (' ', &old[*a], *a)
            }
            Op::Del(a) => {
                old_at = old_at.min(*a);
                old_len += 1;
                ('-', &old[*a], *a)
            }
            Op::Ins(b) => {
                new_at = new_at.min(*b);
                new_len += 1;
                ('+', &new[*b], *b)
            }
        };
        let _ = index;
        body.push(mark);
        body.push_str(&line.text);
        body.push('\n');
        if !line.newline {
            body.push_str("\\ No newline at end of file\n");
        }
    }
    format!(
        "@@ -{} +{} @@\n{body}",
        range(old_at, old_len),
        range(new_at, new_len)
    )
}

/// `-a,b`, or `-a` when the side holds one line, or `-0,0` when it holds none.
fn range(at: usize, len: usize) -> String {
    if len == 0 {
        return "0,0".to_string();
    }
    let start = at + 1;
    match len {
        1 => format!("{start}"),
        _ => format!("{start},{len}"),
    }
}

/// The lines of a text; a trailing newline ends the last line rather than starting an empty one.
fn split(text: &str) -> Vec<Line> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<Line> = text
        .split('\n')
        .map(|text| Line {
            text: text.to_string(),
            newline: true,
        })
        .collect();
    if text.ends_with('\n') {
        lines.pop();
    } else if let Some(last) = lines.last_mut() {
        last.newline = false;
    }
    lines
}

/// The edit script of the longest common subsequence: within one change, deletions come before
/// insertions, which is the order diff prints them in.
fn script(old: &[Line], new: &[Line]) -> Vec<Op> {
    let (n, m) = (old.len(), new.len());
    if !table_fits(n, m) {
        return replacement(old, new);
    }
    let mut common = vec![vec![0usize; m + 1]; n + 1];
    for a in (0..n).rev() {
        for b in (0..m).rev() {
            common[a][b] = match old[a] == new[b] {
                true => common[a + 1][b + 1] + 1,
                false => common[a + 1][b].max(common[a][b + 1]),
            };
        }
    }
    let (mut a, mut b) = (0, 0);
    let mut ops = Vec::new();
    while a < n && b < m {
        if old[a] == new[b] {
            ops.push(Op::Same(a, b));
            a += 1;
            b += 1;
        } else if common[a + 1][b] >= common[a][b + 1] {
            ops.push(Op::Del(a));
            a += 1;
        } else {
            ops.push(Op::Ins(b));
            b += 1;
        }
    }
    while a < n {
        ops.push(Op::Del(a));
        a += 1;
    }
    while b < m {
        ops.push(Op::Ins(b));
        b += 1;
    }
    ops
}

/// Past MAX_CELLS: the lines the two texts share at the head and at the tail are kept, and
/// everything between them is one deletion followed by one insertion.
fn replacement(old: &[Line], new: &[Line]) -> Vec<Op> {
    let (n, m) = (old.len(), new.len());
    let head = (0..n.min(m)).take_while(|at| old[*at] == new[*at]).count();
    let tail = (0..n.min(m) - head)
        .take_while(|at| old[n - 1 - at] == new[m - 1 - at])
        .count();
    let mut ops: Vec<Op> = (0..head).map(|at| Op::Same(at, at)).collect();
    ops.extend((head..n - tail).map(Op::Del));
    ops.extend((head..m - tail).map(Op::Ins));
    ops.extend((0..tail).map(|at| Op::Same(n - tail + at, m - tail + at)));
    ops
}

/// The mtime BSD diff puts after the path, in UTC (see the file header).
fn stamp(path: &Path) -> String {
    let seconds = file_mtime(path).unwrap_or_default();
    OffsetDateTime::from_unix_timestamp(seconds)
        .ok()
        .and_then(|at| at.format(&STAMP).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hunks only: the two header lines carry paths and mtimes, which the shell's diff prints
    /// from the file system and no test can pin. Every expected string below is what BSD diff -u
    /// prints for the same pair on macOS.
    fn hunks(old: &str, new: &str) -> String {
        let rendered = unified(Path::new("a"), old, Path::new("b"), new);
        match rendered.is_empty() {
            true => rendered,
            false => rendered
                .splitn(3, '\n')
                .nth(2)
                .unwrap_or_default()
                .to_string(),
        }
    }

    #[test]
    fn r13_equal_texts_print_nothing() {
        assert_eq!(hunks("a\nb\n", "a\nb\n"), "");
        assert_eq!(hunks("", ""), "");
    }

    #[test]
    fn r13_one_hunk_carries_three_lines_of_context() {
        assert_eq!(
            hunks("a\nb\nc\nd\ne\nf\ng\n", "a\nb\nX\nd\ne\nf\ng\nh\n"),
            "@@ -1,7 +1,8 @@\n a\n b\n-c\n+X\n d\n e\n f\n g\n+h\n"
        );
    }

    #[test]
    fn r13_distant_changes_are_two_hunks() {
        let old: String = (1..=20).map(|n| format!("l{n}\n")).collect();
        let new = old.replace("l3\n", "X3\n").replace("l17\n", "X17\n");
        assert_eq!(
            hunks(&old, &new),
            "@@ -1,6 +1,6 @@\n l1\n l2\n-l3\n+X3\n l4\n l5\n l6\n\
             @@ -14,7 +14,7 @@\n l14\n l15\n l16\n-l17\n+X17\n l18\n l19\n l20\n"
        );
    }

    #[test]
    fn r13_a_missing_final_newline_is_its_own_line() {
        assert_eq!(
            hunks("a\nb\nc\nd\ne\nf\ng\n", "a\nb"),
            "@@ -1,7 +1,2 @@\n a\n-b\n-c\n-d\n-e\n-f\n-g\n+b\n\\ No newline at end of file\n"
        );
    }

    /// Past the table bound the renderer stops looking for the shortest script: the shared head
    /// and tail stay and everything between them becomes one deletion and one insertion, so two
    /// distant edits come back as a single hunk instead of the two an LCS walk would find.
    #[test]
    fn r13_a_pair_past_the_table_bound_becomes_one_replacement_hunk() {
        let old: String = (1..=3000).map(|n| format!("l{n}\n")).collect();
        let new = old
            .replace("l500\n", "X500\n")
            .replace("l2500\n", "X2500\n");
        let rendered = hunks(&old, &new);
        let (header, body) = rendered.split_once('\n').expect("a hunk header");
        assert_eq!(rendered.matches("@@ -").count(), 1, "one hunk, not two");
        assert_eq!(header, "@@ -497,2007 +497,2007 @@");
        assert_eq!(body.lines().filter(|l| l.starts_with('-')).count(), 2001);
        assert_eq!(body.lines().filter(|l| l.starts_with('+')).count(), 2001);
        assert!(body.contains("-l500\n") && body.contains("+X500\n"));
        assert!(body.contains("-l2500\n") && body.contains("+X2500\n"));
    }

    /// The bound counts the cells the table allocates, so an empty side does not smuggle an
    /// arbitrarily long other side past it.
    #[test]
    fn r13_the_table_bound_counts_the_row_and_column_the_table_adds() {
        assert!(table_fits(1999, 1999));
        assert!(!table_fits(2000, 2000));
        assert!(table_fits(0, MAX_CELLS - 1));
        assert!(!table_fits(0, MAX_CELLS));
        assert!(!table_fits(0, usize::MAX));
        assert!(!table_fits(usize::MAX, 0));
    }

    /// One side empty and the other long: every line is an insertion, in one hunk that opens at
    /// the zero range, whichever path the bound sends it down.
    #[test]
    fn r13_an_empty_side_against_a_long_one_is_one_insertion_hunk() {
        let new: String = (1..=5000).map(|n| format!("l{n}\n")).collect();
        let rendered = hunks("", &new);
        let (header, body) = rendered.split_once('\n').expect("a hunk header");
        assert_eq!(header, "@@ -0,0 +1,5000 @@");
        assert_eq!(rendered.matches("@@ -").count(), 1);
        assert_eq!(body.lines().filter(|l| l.starts_with('+')).count(), 5000);
    }

    #[test]
    fn r13_an_empty_side_prints_the_zero_range() {
        assert_eq!(hunks("", "a\n"), "@@ -0,0 +1 @@\n+a\n");
        assert_eq!(hunks("a\n", ""), "@@ -1 +0,0 @@\n-a\n");
    }
}
