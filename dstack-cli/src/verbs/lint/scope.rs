// verbs/lint/scope.rs
// The R93 scope table: where it lives, the bash case-glob that reads it, and relative paths.

use std::fs;
use std::path::{Path, PathBuf};

/// exempt, en and ko-data are not blocking: ko-data is product copy and built-in examples, where
/// the tone rules must not fire (R93), so the scan returns before it reads a single line.
pub fn blocks(scope: &str) -> bool {
    matches!(scope, "ko-haeyo" | "ko-plain" | "commit-msg")
}

/// _ko_rel(): the repository-relative path of an argument. The file need not exist and nothing is
/// normalised — the PreToolUse hook asks about a path that is about to be created.
pub fn rel(wt_root: &Path, arg: &str) -> String {
    let mut path = arg.to_string();
    if !path.starts_with('/') {
        let cwd = std::env::current_dir().unwrap_or_default();
        path = format!("{}/{path}", cwd.display());
    }
    match path.strip_prefix(&format!("{}/", wt_root.display())) {
        Some(rest) => rest.to_string(),
        None => path,
    }
}

/// The scope table this run reads, with its rows in the order they decide.
pub struct Scopes {
    pub path: Option<PathBuf>,
    rows: Vec<(String, String)>,
}

impl Scopes {
    /// _ko_scope_table(): the resolution order R93 fixes — the repository speaks first, then the
    /// machine, then the checkout of D-STACK itself, so a fresh clone lints before install.sh ran.
    pub fn load(main_root: &Path, home: &Path) -> Scopes {
        let machine = PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let candidates = [
            main_root.join(".dstack/project/ko-scope.tsv"),
            machine.join(".claude/lint/ko-scope.tsv"),
            home.join("lint/ko-scope.tsv"),
        ];
        let path = candidates.into_iter().find(|candidate| candidate.is_file());
        let rows = match &path {
            Some(path) => read_rows(path),
            None => Vec::new(),
        };
        Scopes { path, rows }
    }

    /// _ko_scope_of(): the first matching row wins, and "**/" is also tried as "" so that
    /// docs/**/*.md covers docs/x.md. An unmatched path is the empty scope: unclassified.
    pub fn of(&self, rel: &str) -> &str {
        for (pattern, scope) in &self.rows {
            if glob(pattern, rel) {
                return scope;
            }
            let alternative = pattern.replace("**/", "");
            if alternative != *pattern && glob(&alternative, rel) {
                return scope;
            }
        }
        ""
    }
}

fn read_rows(path: &Path) -> Vec<(String, String)> {
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut rows = Vec::new();
    for line in text.lines() {
        let mut columns = line.splitn(2, '\t');
        let pattern = columns.next().unwrap_or_default();
        if pattern.is_empty() || pattern.starts_with('#') || pattern == "pattern" {
            continue;
        }
        rows.push((
            pattern.to_string(),
            columns.next().unwrap_or_default().to_string(),
        ));
    }
    rows
}

/// The pattern of a bash `case` arm: * covers any string including /, ? one character, [...] a
/// class with !/^ negation and ranges, and a backslash escapes the next character. A case pattern
/// knows no FNM_PATHNAME, which is why *.ko.json also matches a path that has directories.
pub fn glob(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let (mut p, mut t) = (0, 0);
    let (mut star, mut retry) = (None, 0);
    while t < text.len() {
        if pattern.get(p) == Some(&'*') {
            star = Some(p);
            retry = t;
            p += 1;
            continue;
        }
        let matched = match p < pattern.len() {
            true => one(&pattern, p, text[t]),
            false => None,
        };
        match matched {
            Some(next) => {
                p = next;
                t += 1;
            }
            // The last * takes one more character and the rest of the pattern is tried again.
            None => match star {
                Some(at) => {
                    p = at + 1;
                    retry += 1;
                    t = retry;
                }
                None => return false,
            },
        }
    }
    while pattern.get(p) == Some(&'*') {
        p += 1;
    }
    p == pattern.len()
}

/// One pattern element against one character: the position behind it when it matches.
fn one(pattern: &[char], p: usize, ch: char) -> Option<usize> {
    match pattern[p] {
        '?' => Some(p + 1),
        '\\' if p + 1 < pattern.len() => (pattern[p + 1] == ch).then_some(p + 2),
        '[' => match class(pattern, p, ch) {
            Some((next, true)) => Some(next),
            Some((_, false)) => None,
            None => (ch == '[').then_some(p + 1),
        },
        literal => (literal == ch).then_some(p + 1),
    }
}

/// A [...] class: the position behind it and whether the character is in it. None when the class
/// is never closed, which bash reads as a literal [.
fn class(pattern: &[char], start: usize, ch: char) -> Option<(usize, bool)> {
    let mut i = start + 1;
    let negated = matches!(pattern.get(i), Some('!') | Some('^'));
    if negated {
        i += 1;
    }
    // A ] in the first position is a member of the class, not its end.
    let first = i;
    let mut found = false;
    while i < pattern.len() && !(pattern[i] == ']' && i > first) {
        if i + 2 < pattern.len() && pattern[i + 1] == '-' && pattern[i + 2] != ']' {
            found = found || (pattern[i] <= ch && ch <= pattern[i + 2]);
            i += 3;
        } else {
            found = found || pattern[i] == ch;
            i += 1;
        }
    }
    if i >= pattern.len() {
        return None;
    }
    Some((i + 1, found != negated))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r06__only_three_scopes_block() {
        assert!(blocks("ko-haeyo") && blocks("ko-plain") && blocks("commit-msg"));
        assert!(!blocks("en") && !blocks("exempt") && !blocks("ko-data") && !blocks(""));
    }

    #[test]
    fn r06__a_relative_path_is_cut_at_the_worktree_root() {
        let cwd = std::env::current_dir().expect("a working directory");
        let root = cwd.parent().expect("a parent").to_path_buf();
        let here = cwd
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .to_string();
        assert_eq!(rel(&root, "x.md"), format!("{here}/x.md"));
        assert_eq!(rel(&cwd, "x.md"), "x.md");
        // Nothing is normalised: the hook asks about the path it typed.
        assert_eq!(rel(&cwd, "./x.md"), "./x.md");
        assert_eq!(rel(&cwd, "/elsewhere/x.md"), "/elsewhere/x.md");
    }
}
