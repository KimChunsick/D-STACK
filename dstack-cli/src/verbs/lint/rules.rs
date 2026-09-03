// verbs/lint/rules.rs
// The R91 rule table and the leftmost-longest scanner that stands in for grep -nEo.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::core::error::{Error, Result};

/// One row of ko-rules.tsv. The source column is read past: nothing prints it.
pub struct Rule {
    pub id: String,
    pub kind: String,
    pub pattern: String,
    pub severity: String,
    pub replacement: String,
    pub example: String,
    pub level: String,
    pub matcher: Option<Matcher>,
}

/// The rows of a table with nothing compiled yet. The rule checker reports a pattern that does
/// not compile, so it has to reach the row even when the pattern is broken.
pub fn rows(path: &Path) -> Result<Vec<Rule>> {
    let missing = || Error::cannot_decide(format!("rule table missing: {} (R91)", path.display()));
    if !path.is_file() {
        return Err(missing());
    }
    let raw = fs::read(path).map_err(|_| missing())?;
    let text = String::from_utf8_lossy(&raw);
    let mut rows = Vec::new();
    for line in text.lines() {
        let mut columns = line.splitn(8, '\t');
        let mut column = || columns.next().unwrap_or_default().to_string();
        let id = column();
        if id.is_empty() || id.starts_with('#') || id == "id" {
            continue;
        }
        let (kind, pattern, severity) = (column(), column(), column());
        let (replacement, example, _source) = (column(), column(), column());
        let level = match column() {
            level if level.is_empty() => "sentence".to_string(),
            level => level,
        };
        rows.push(Rule {
            id,
            kind,
            pattern,
            severity,
            replacement,
            example,
            level,
            matcher: None,
        });
    }
    if rows.is_empty() {
        return Err(Error::cannot_decide(format!(
            "rule table has no rows: {}",
            path.display()
        )));
    }
    Ok(rows)
}

pub struct Table {
    pub path: PathBuf,
    pub rules: Vec<Rule>,
    pub regex_n: usize,
    pub judgment_n: usize,
}

impl Table {
    /// _ko_load_rules(): every row is read once, and every regex row is compiled here so a rule
    /// this engine cannot run is reported by its id before any file is scanned (R06).
    pub fn load(path: &Path) -> Result<Table> {
        let mut table = Table {
            path: path.to_path_buf(),
            rules: rows(path)?,
            regex_n: 0,
            judgment_n: 0,
        };
        for rule in &mut table.rules {
            match rule.kind.as_str() {
                "regex" => {
                    table.regex_n += 1;
                    rule.matcher = Some(Matcher::compile(&rule.id, &rule.pattern)?);
                }
                _ => table.judgment_n += 1,
            }
        }
        Ok(table)
    }
}

/// One compiled pattern.
///
/// POSIX ERE, which grep -E runs, is leftmost-longest: of the matches that start at the earliest
/// position the longest one wins. The regex crate is leftmost-first — it takes the first
/// alternative that can match — and `grep -o` prints the matched text, so the difference shows.
/// The two engines agree on where a match starts, so the start comes from the plain pattern and
/// only the end is searched for: `(?:p)$` is tried against the line cut at every candidate end,
/// from the last one down to the end leftmost-first found, and the first that answers is the
/// POSIX end. (No locale handling belongs here: grep needed a UTF-8 locale for the [가-힣]
/// ranges, and the regex crate is Unicode by default.)
pub struct Matcher {
    plain: Regex,
    ends: Regex,
    span: Option<usize>,
}

impl Matcher {
    pub fn compile(id: &str, pattern: &str) -> Result<Matcher> {
        let compile = |source: &str| {
            Regex::new(source).map_err(|error| {
                Error::cannot_decide(format!("rule {id}: pattern does not compile: {error}"))
            })
        };
        Ok(Matcher {
            plain: compile(pattern)?,
            ends: compile(&format!("(?:{pattern})$"))?,
            span: span_bound(pattern),
        })
    }

    /// Every match `grep -o` would print for one line, in order and never overlapping. An empty
    /// match prints nothing and the scan moves on by one character, which is what grep does.
    pub fn find<'a>(&self, line: &'a str) -> Vec<&'a str> {
        let mut found = Vec::new();
        let mut at = 0;
        while at <= line.len() {
            let first = match self.plain.find_at(line, at) {
                Some(first) => first,
                None => break,
            };
            let (start, end) = (
                first.start(),
                self.longest(line, first.start(), first.end()),
            );
            if end > start {
                found.push(&line[start..end]);
                at = end;
                continue;
            }
            at = match line[start..].chars().next() {
                Some(ch) => start + ch.len_utf8(),
                None => break,
            };
        }
        found
    }

    /// The end POSIX would pick for the match that starts at `start`, given the end leftmost-first
    /// picked. The line keeps everything before the match, so a `^` in the pattern still sees the
    /// start of the line; a `$` in the pattern is read at the candidate end, which can only make
    /// a candidate shorter than the true match and therefore never wins the descending search.
    fn longest(&self, line: &str, start: usize, first: usize) -> usize {
        let ceiling = match self.span {
            Some(span) => (start + span).min(line.len()).max(first),
            None => line.len(),
        };
        let mut end = ceiling;
        while end > first {
            if line.is_char_boundary(end) {
                if let Some(found) = self.ends.find_at(&line[..end], start) {
                    if found.start() == start {
                        return end;
                    }
                }
            }
            end -= 1;
        }
        first
    }
}

/// An upper bound on the byte length of a match, so the search for the end does not walk a whole
/// line for a pattern that cannot reach that far. Every character of the pattern consumes at most
/// one character of the line and a character is at most four bytes; a pattern that repeats
/// without an upper bound has no such bound at all.
fn span_bound(pattern: &str) -> Option<usize> {
    let mut in_class = false;
    let mut escaped = false;
    for ch in pattern.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '[' if !in_class => in_class = true,
            ']' if in_class => in_class = false,
            '*' | '+' | '{' if !in_class => return None,
            _ => {}
        }
    }
    Some(pattern.chars().count() * 4)
}

/// The `grep -nEo` run of _ko_scan over one text: (line number, matched text) in grep's order.
pub fn hits<'a>(matcher: &Matcher, text: &'a str) -> Vec<(usize, &'a str)> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    // The split behind a closing newline is not a line of the file.
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut found = Vec::new();
    for (number, line) in lines.iter().enumerate() {
        for matched in matcher.find(line) {
            found.push((number + 1, matched));
        }
    }
    found
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r06__the_longest_match_of_a_position_wins() {
        let matcher = Matcher::compile("K", "에의|에서의|으로부터의").expect("compiles");
        assert_eq!(matcher.find("에서의 에의"), vec!["에서의", "에의"]);
        // A nested alternation the branch-splitting emulation used to get wrong.
        let matcher = Matcher::compile("K", "(a|[a]b?)").expect("compiles");
        assert_eq!(matcher.find("ab"), vec!["ab"]);
    }

    #[test]
    fn r06__an_empty_match_prints_nothing_and_moves_on() {
        let matcher = Matcher::compile("K", "x*").expect("compiles");
        assert_eq!(matcher.find("yyy"), Vec::<&str>::new());
        assert_eq!(matcher.find("yxxy"), vec!["xx"]);
    }

    #[test]
    fn r06__the_line_anchor_belongs_to_the_line() {
        let matcher = Matcher::compile("K", "(^|[^가-힣])즉,").expect("compiles");
        assert_eq!(
            hits(&matcher, "즉, 그래요\n그리고즉, 끝\n 즉, 끝\n"),
            vec![(1, "즉,"), (3, " 즉,")]
        );
    }

    #[test]
    fn r06__a_pattern_that_cannot_repeat_forever_bounds_the_search() {
        assert_eq!(span_bound("정본"), Some(8));
        assert_eq!(span_bound("판별 ?유니언"), Some(28));
        assert_eq!(span_bound("[*+]x"), Some(20));
        assert_eq!(span_bound("a.*b"), None);
        assert_eq!(span_bound("[가-힣 ]+"), None);
    }
}
