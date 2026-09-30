// store/request_sections.rs
// The prose sections of request.md: locate a body, check new prose, replace one body in place.

use std::ops::Range;

use crate::core::error::{Error, Result};
use crate::store::rows;

/// The `##` heading that holds the R rows; only `req add` and the row verbs write under it.
pub const REQUIREMENTS: &str = "요구사항";

/// The prose keys of parts 1 and 2 and the `##` heading each one names.
pub const SECTION_KEYS: [(&str, &str); 10] = [
    ("background", "배경과 문제"),
    ("goals", "목표"),
    ("non-goals", "비목표"),
    ("scenarios", "사용 시나리오"),
    ("assumptions", "열린 가정"),
    ("current", "지금 구조"),
    ("proposed", "바꿀 구조"),
    ("alternatives", "검토한 대안과 버린 이유"),
    ("mapping", "R 행과 모듈의 대응"),
    ("risks", "위험"),
];

/// Where a body sits: the paragraph between the `# <title>` line and the first heading, or the
/// lines under one `## <heading>` up to the next `#` or `##` heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Summary,
    Section(&'static str),
}

impl Place {
    /// The place a writer key names; None for a key that names no prose section.
    pub fn of_key(key: &str) -> Option<Place> {
        if key == "summary" {
            return Some(Place::Summary);
        }
        SECTION_KEYS
            .iter()
            .find(|(name, _)| *name == key)
            .map(|&(_, heading)| Place::Section(heading))
    }

    /// The deepest heading level that ends this body; one in new content would cut it short.
    fn boundary(&self) -> usize {
        match self {
            Place::Summary => 6,
            Place::Section(_) => 2,
        }
    }
}

/// One heading `outline` found; `lineno` counts every line of the text, frontmatter included.
struct Heading<'a> {
    level: usize,
    text: &'a str,
    lineno: usize,
    start: usize,
    end: usize,
}

/// The byte range of one body. A missing heading, a heading that appears more than once, a
/// document without a title and a document the outline cannot read for certain are refusals
/// naming the heading or line, so no caller writes to a guessed place.
pub fn locate(text: &str, place: &Place) -> Result<Range<usize>> {
    let headings = outline(text, frontmatter_end(text), "")?;
    let at = match place {
        Place::Summary => match headings.first() {
            Some(first) if first.level == 1 => 0,
            _ => {
                return Err(Error::failed(
                    "no '# <title>' heading above the first section",
                ))
            }
        },
        Place::Section(name) => {
            let found: Vec<usize> = headings
                .iter()
                .enumerate()
                .filter(|(_, heading)| heading.level == 2 && heading.text == *name)
                .map(|(index, _)| index)
                .collect();
            match found.len() {
                0 => return Err(Error::failed(format!("no '## {name}' heading"))),
                1 => found[0],
                n => return Err(Error::failed(format!("'## {name}' appears {n} times"))),
            }
        }
    };
    let end = headings[at + 1..]
        .iter()
        .find(|heading| heading.level <= place.boundary())
        .map_or(text.len(), |heading| heading.start);
    Ok(headings[at].end..end)
}

/// The body of one place as the file holds it.
pub fn body<'a>(text: &'a str, place: &Place) -> Result<&'a str> {
    Ok(&text[locate(text, place)?])
}

/// How a verb names the place it wrote.
pub fn label(text: &str, place: &Place) -> String {
    match place {
        Place::Summary => {
            let headings = outline(text, frontmatter_end(text), "").unwrap_or_default();
            let title = headings.first().map_or("", |heading| heading.text);
            format!("the paragraph under # {title}")
        }
        Place::Section(name) => format!("## {name}"),
    }
}

/// Blank is what a template leaves behind: whitespace and HTML comments only. An unclosed
/// comment runs to the end of the body, as a renderer shows it.
pub fn is_blank(body: &str) -> bool {
    let mut rest = body;
    while let Some(at) = rest.find("<!--") {
        if !rest[..at].trim().is_empty() {
            return false;
        }
        rest = match rest[at + 4..].find("-->") {
            Some(close) => &rest[at + 4 + close + 3..],
            None => "",
        };
    }
    rest.trim().is_empty()
}

/// What new prose may hold: no row-shaped line (read back as an R row), no heading ending this
/// body, nothing the outline refuses, read as if under the blank line `replace` puts above it.
pub fn check_prose(content: &str, place: &Place) -> Result<()> {
    for (index, line) in content.lines().enumerate() {
        if row_shaped(line) {
            return Err(Error::failed(format!(
                "line {} of the content looks like an R row (rows are added with dstack req add): {line}",
                index + 1
            )));
        }
    }
    let headings = outline(content, 0, " of the content")?;
    if let Some(heading) = headings.iter().find(|h| h.level <= place.boundary()) {
        let allowed = match place {
            Place::Summary => "the summary holds no headings",
            Place::Section(_) => "a section holds ### subsections only",
        };
        return Err(Error::failed(format!(
            "line {} of the content is a heading ({allowed}): {}",
            heading.lineno,
            content.lines().nth(heading.lineno - 1).unwrap_or("")
        )));
    }
    Ok(())
}

/// The text with one body replaced by `content`: a blank line after the heading, the content
/// without its leading blank lines and trailing whitespace, and a blank line before the next
/// heading. A body holding an R row is refused, because replacing it would drop the row.
pub fn replace(text: &str, place: &Place, content: &str) -> Result<String> {
    let range = locate(text, place)?;
    for line in text[range.clone()].lines() {
        if let Some(row) = rows::parse_line(0, line) {
            return Err(Error::failed(format!(
                "{} holds R row {}; rows move only through the req verbs",
                label(text, place),
                row.id
            )));
        }
    }
    let content = content.trim_end();
    let lead = &content[..content.len() - content.trim_start().len()];
    let content = &content[lead.rfind('\n').map_or(0, |at| at + 1)..];
    let mut out = String::with_capacity(text.len() + content.len() + 2);
    out.push_str(&text[..range.start]);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    if !content.is_empty() {
        out.push_str(content);
        out.push('\n');
        if range.end < text.len() {
            out.push('\n');
        }
    }
    out.push_str(&text[range.end..]);
    Ok(out)
}

/// A bold R id anywhere is how the row readers find a row (`rows()`, the `] **R<NN>** ` search
/// of `row_lineno` and the grammar of `check request`), and `- [` opens a row's box.
fn row_shaped(line: &str) -> bool {
    line.starts_with("- [")
        || line.match_indices("**R").any(|(at, _)| {
            let rest = &line[at + 3..];
            let digits = rest.chars().take_while(char::is_ascii_digit).count();
            digits > 0 && rest[digits..].starts_with("**")
        })
}

/// Where the first `---` block ends: the block every frontmatter reader stops at.
fn frontmatter_end(text: &str) -> usize {
    let mut lines = text.split_inclusive('\n');
    if lines.next().map(|line| line.trim_end_matches('\n')) != Some("---") {
        return 0;
    }
    let mut end = "---\n".len();
    for line in lines {
        end += line.len();
        if line.trim_end_matches('\n') == "---" {
            return end;
        }
    }
    text.len()
}

/// Why a line refuses: a renderer may read it otherwise, or the grammar has no place for it.
const AMBIGUOUS: &str = "; a renderer may read it otherwise, so the sections are ambiguous";
const UNSUPPORTED: &str = ", which request section does not support";

/// The column-0 ATX headings from byte `from` on, in the grammar `read_line` reads. Refused,
/// naming the line `of` the text: what `read_line` refuses, a fence or comment that never closes
/// and a non-text line in a comment opened past column 0, which a renderer need not hide.
fn outline<'a>(text: &'a str, from: usize, of: &str) -> Result<Vec<Heading<'a>>> {
    let refuse = |line: usize, why: String| Err(Error::failed(format!("line {line}{of} {why}")));
    let mut headings = Vec::new();
    // The open fence: mark, length, line. The open comment: line, opened at column 0 (a block).
    let mut fence: Option<((u8, usize), usize)> = None;
    let mut comment: Option<(usize, bool)> = None;
    // Whether the last line was paragraph text, which an underline makes a setext heading. A
    // column-0 comment opens an HTML block instead, even one that closes on its own line.
    let mut text_above = false;
    let mut start = from;
    let skipped = text[..from].matches('\n').count();
    for (index, line) in text[from..].split_inclusive('\n').enumerate() {
        let lineno = skipped + index + 1;
        let end = start + line.len();
        let bare = line.trim_end_matches(['\n', '\r']);
        let above = std::mem::take(&mut text_above);
        let paragraph = !bare.trim_matches([' ', '\t']).is_empty() && !bare.starts_with("<!--");
        if let Some((open, _)) = fence {
            fence = fence.filter(|_| !closes(bare, open));
        } else if let Some((opened, block)) = comment {
            if !block && !matches!(read_line(bare, above), Line::Text) {
                let why = format!("is a heading, fence or HTML line in the comment line {opened}");
                return refuse(lineno, why + " opens past column 0" + AMBIGUOUS);
            }
            comment = comment_end(bare, lineno, comment);
            text_above = !block && paragraph;
        } else {
            match read_line(bare, above) {
                Line::Text => (comment, text_above) = (comment_end(bare, lineno, None), paragraph),
                Line::Fence(open) => fence = Some((open, lineno)),
                Line::Heading(level, text) => headings.push(Heading {
                    level,
                    text,
                    lineno,
                    start,
                    end,
                }),
                Line::Refused(what, why) => return refuse(lineno, format!("{what}{why}")),
            }
        }
        start = end;
    }
    let hides = "never closes; it hides every heading after it";
    match (fence, comment) {
        (Some((_, line)), _) => refuse(line, format!("opens a code fence that {hides}")),
        (_, Some((line, _))) => refuse(line, format!("opens an HTML comment that {hides}")),
        _ => Ok(headings),
    }
}

/// How one line outside every fence and comment reads.
enum Line<'a> {
    Text,
    Heading(usize, &'a str),
    Fence((u8, usize)),
    Refused(&'static str, &'static str),
}

/// Column-0 headings and ``` or ~~~ fences; the rest is text (paragraphs, lists, tables, quotes,
/// comments), but refused: a tab before a heading, fence or `<`; a `<` within three spaces that
/// opens no comment (raw HTML); a list-indented fence; a setext underline under text `above`.
fn read_line(line: &str, above: bool) -> Line<'_> {
    let rest = line.trim_start_matches([' ', '\t']);
    let indent = line.len() - rest.len();
    let (fence, head, tag) = (fence_mark(rest), heading(rest), rest.starts_with('<'));
    if line[..indent].contains('\t') && (fence.is_some() || head.is_some() || tag) {
        return Line::Refused("indents a heading, fence or `<` with a tab", UNSUPPORTED);
    }
    match (indent, fence, head) {
        (4.., _, _) => Line::Text,
        _ if tag && !rest.starts_with("<!--") => Line::Refused("starts raw HTML", UNSUPPORTED),
        (0, Some(open), _) => Line::Fence(open),
        (_, Some(_), _) => Line::Refused("indents a fence marker as in a list item", AMBIGUOUS),
        (0, _, Some((level, name))) => Line::Heading(level, name),
        _ if above && underline(line) => Line::Refused("underlines a setext heading", UNSUPPORTED),
        _ => Line::Text,
    }
}

/// A setext underline: up to three spaces, a run of `-` or of `=`, then spaces or tabs only.
fn underline(line: &str) -> bool {
    let rest = line.trim_start_matches(' ');
    let run = rest.trim_end_matches([' ', '\t']);
    let one_mark = run.trim_matches('-').is_empty() || run.trim_matches('=').is_empty();
    line.len() - rest.len() <= 3 && !run.is_empty() && one_mark
}

/// The HTML comment still open at the end of line `n`, given the one open at its start: the line
/// it opened on and whether it opened at column 0. `<!-->` and `<!--->` close themselves.
fn comment_end(line: &str, n: usize, mut open: Option<(usize, bool)>) -> Option<(usize, bool)> {
    let mut at = 0;
    loop {
        if open.is_some() {
            let Some(close) = line[at..].find("-->") else {
                return open;
            };
            (at, open) = (at + close + 3, None);
        } else {
            let begin = at + line[at..].find("<!--")?;
            (at, open) = (begin + 2, Some((n, begin == 0)));
        }
    }
}

/// An ATX heading: one to six `#`, then a space, a tab or the end of the line.
fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|b| *b == b'#').count();
    let after = &line[level..];
    let atx = (1..=6).contains(&level) && (after.is_empty() || after.starts_with([' ', '\t']));
    atx.then(|| (level, after.trim()))
}

/// A code fence marker: three or more backticks or tildes, and no backtick after a backtick run.
fn fence_mark(line: &str) -> Option<(u8, usize)> {
    let mark = line.bytes().next().filter(|b| matches!(b, b'`' | b'~'))?;
    let run = line.bytes().take_while(|b| *b == mark).count();
    (run >= 3 && !(mark == b'`' && line[run..].contains('`'))).then_some((mark, run))
}

/// A closing fence: up to three spaces, the opening's mark at least as long, nothing after it.
fn closes(line: &str, (mark, run): (u8, usize)) -> bool {
    let rest = line.trim_start_matches(' ');
    let length = rest.bytes().take_while(|b| *b == mark).count();
    line.len() - rest.len() <= 3 && length >= run && rest[length..].trim().is_empty()
}
