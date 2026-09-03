// verbs/review/emit_diff.rs
// The DIFF section: the plan's declared paths expanded into the files that actually changed.
//
// A file outside the plan cannot reach the reviewer even when it changed, because reviewing
// files the plan never claimed is how scope creep gets rubber-stamped (R69). A declared path is
// often a DIRECTORY, so it is first expanded into the files that changed under it: one 64KB cap
// over a whole directory would skip the directory's entire diff.

use std::path::Path;
use std::process::{Command, Stdio};

use super::MAX_FILE_DIFF;

/// What _emit_diff writes into its diffcount file: the files it framed and the ones it skipped.
#[derive(Default)]
pub struct Counts {
    pub files: usize,
    pub skipped: usize,
}

pub fn emit(out: &mut Vec<u8>, wt: &Path, base: &str, files: &[String]) -> Counts {
    let mut counts = Counts::default();
    for declared in files {
        let changed = changed_files(wt, base, declared);
        push(
            out,
            &format!(
                "--- declared: {declared} ({} changed file(s))\n",
                changed.len()
            ),
        );
        // Keep the declared path visible even with nothing under it: the reviewer must see that
        // the plan claimed a file it never touched.
        if changed.is_empty() {
            push(out, &format!("--- file: {declared}\n"));
            push(out, "(no changes against the base)\n");
            counts.files += 1;
            continue;
        }
        for one in &changed {
            counts.files += 1;
            push(out, &format!("--- file: {one}\n"));
            if !file_diff(out, wt, base, one) {
                counts.skipped += 1;
            }
        }
    }
    counts
}

/// One file's diff against the base, capped. False when the file was skipped for size, so the
/// caller can count it.
fn file_diff(out: &mut Vec<u8>, wt: &Path, base: &str, path: &str) -> bool {
    let untracked = wt.join(path).exists()
        && !git_ok(wt, &["ls-files", "--error-unmatch", "--", path]);
    let (bytes, _) = if untracked {
        // There is no blob to diff against, so show the whole file as an addition.
        git(wt, &["diff", "--no-index", "--", "/dev/null", path])
    } else {
        match against(base) {
            Some(base) => git(wt, &["diff", base, "--", path]),
            None => git(wt, &["diff", "--", path]),
        }
    };
    if bytes.len() > MAX_FILE_DIFF {
        push(out, "(SKIPPED: diff >64KB — split the plan)\n");
        return false;
    }
    if bytes.is_empty() {
        push(out, "(no changes against the base)\n");
    } else {
        out.extend_from_slice(&bytes);
    }
    true
}

/// The files that changed under one declared path: what git diff names against the base, plus
/// what git has never seen, sorted and unique in byte order.
fn changed_files(wt: &Path, base: &str, path: &str) -> Vec<String> {
    let (named, ok) = match against(base) {
        Some(base) => git(wt, &["diff", "--name-only", base, "--", path]),
        None => git(wt, &["diff", "--name-only", "--", path]),
    };
    let mut found = text_lines(&named);
    // The shell runs both readers in one group under `set -e`: a base git cannot resolve ends
    // the group, so the untracked half never runs either.
    if ok {
        let (others, _) = git(
            wt,
            &["ls-files", "--others", "--exclude-standard", "--", path],
        );
        found.extend(text_lines(&others));
    }
    // `grep -c .` counts the lines that carry something, and the read loop skips the others.
    found.retain(|line| !line.is_empty());
    found.sort();
    found.dedup();
    found
}

/// `[ -n "$base" ] && [ "$base" != none ]`: the base commit, or the working-tree diff.
fn against(base: &str) -> Option<&str> {
    match base.is_empty() || base == "none" {
        true => None,
        false => Some(base),
    }
}

/// git with its raw stdout, whatever it exited with: the shell reads the bytes git wrote and
/// drops the status (`> "$d" 2>/dev/null || true`), because `diff --no-index` reports a
/// difference as exit 1 and that is not a failure here.
fn git(wt: &Path, args: &[&str]) -> (Vec<u8>, bool) {
    let out = Command::new("git")
        .arg("-C")
        .arg(wt)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    match out {
        Ok(out) => (out.stdout, out.status.success()),
        Err(_) => (Vec::new(), false),
    }
}

fn git_ok(wt: &Path, args: &[&str]) -> bool {
    git(wt, args).1
}

/// git names paths in its own encoding; the frame around them is ASCII, so a path that is not
/// UTF-8 travels as the replacement character rather than dropping the file from the bundle.
fn text_lines(bytes: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(bytes);
    super::lines(&text).iter().map(|l| l.to_string()).collect()
}

fn push(out: &mut Vec<u8>, text: &str) {
    out.extend_from_slice(text.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_the_base_is_dropped_when_there_is_none() {
        assert_eq!(against("abc123"), Some("abc123"));
        assert_eq!(against(""), None);
        assert_eq!(against("none"), None);
    }

    #[test]
    fn r13_git_output_reads_as_lines_without_the_empty_tail() {
        assert_eq!(text_lines(b"a\nb\n"), vec!["a".to_string(), "b".to_string()]);
        assert!(text_lines(b"").is_empty());
    }
}
