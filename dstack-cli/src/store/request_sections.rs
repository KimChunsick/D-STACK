// store/request_sections.rs
// The prose sections of request.md: locate a body, check new prose, replace one body in place.

use std::ops::Range;

use crate::core::error::{Error, Result};
use crate::store::rows;
use crate::store::visible::{self, closes, fence_mark, heading, SPACE};

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

/// CommonMark's whitespace across lines; any other Unicode space (U+00A0) is text.
const BLANK: [char; 4] = [' ', '\t', '\n', '\r'];

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

/// The byte range of one body. A missing or repeated heading, a missing title and a document the
/// outline cannot read for certain refuse, naming it, so no caller writes to a guessed place.
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

/// Blank is what a template leaves behind: whitespace and HTML comments only, read as the
/// outline reads them. An unclosed comment runs to the end of the body, as a renderer shows it.
pub fn is_blank(body: &str) -> bool {
    visible::visible(body).trim_matches(BLANK).is_empty()
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

/// The text with one body replaced by `content` less its leading blank lines and trailing
/// whitespace, set off by blank lines. A body holding an R row refuses: it would drop the row.
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
    let content = content.trim_end_matches(BLANK);
    let lead = &content[..content.len() - content.trim_start_matches(BLANK).len()];
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

/// The column-0 ATX headings from byte `from` on, as `read_line` reads them. Refused, naming the
/// line `of` the text: a carriage return without a line feed anywhere, what `read_line` refuses,
/// a fence or comment that never closes and a non-text line in a comment opened past column 0.
fn outline<'a>(text: &'a str, from: usize, of: &str) -> Result<Vec<Heading<'a>>> {
    let refuse = |line: usize, why: String| Err(Error::failed(format!("line {line}{of} {why}")));
    let lf = text.replace("\r\n", "\n");
    if let Some(at) = lf.find('\r') {
        let why = format!("holds a carriage return without a line feed{AMBIGUOUS}");
        return refuse(lf[..at].matches('\n').count() + 1, why);
    }
    let mut headings = Vec::new();
    // The open fence: mark, length, line. The open comment: line, opened at column 0 (a block).
    let mut fence: Option<((u8, usize), usize)> = None;
    let mut comment: Option<(usize, bool)> = None;
    // Paragraph text above, which an underline makes a setext heading; not a column-0 comment.
    let mut text_above = false;
    let mut start = from;
    let skipped = text[..from].matches('\n').count();
    for (index, line) in text[from..].split_inclusive('\n').enumerate() {
        let lineno = skipped + index + 1;
        let end = start + line.len();
        let bare = line.trim_end_matches(['\n', '\r']);
        let above = std::mem::take(&mut text_above);
        let paragraph = !bare.trim_matches(SPACE).is_empty() && !bare.starts_with("<!--");
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
    let rest = line.trim_start_matches(SPACE);
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
    let run = rest.trim_end_matches(SPACE);
    let one_mark = run.trim_matches('-').is_empty() || run.trim_matches('=').is_empty();
    line.len() - rest.len() <= 3 && !run.is_empty() && one_mark
}

/// The HTML comment still open at the end of line `n`, given the one open at its start: the line
/// it opened on and whether it opened at column 0. `<!-->` and `<!--->` close themselves.
fn comment_end(line: &str, n: usize, open: Option<(usize, bool)>) -> Option<(usize, bool)> {
    let (spans, still) = visible::scan(line, open.is_some());
    let last = spans.last().filter(|_| still)?;
    Some(open.filter(|_| spans.len() == 1).unwrap_or((n, last.start == 0)))
}
