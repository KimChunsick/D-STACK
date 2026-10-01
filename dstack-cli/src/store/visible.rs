// store/visible.rs
// What a renderer shows of request text: HTML comments hidden as the section outline reads them.
// The line grammar (headings, fences) the outline and this reader share lives here too.

use std::ops::Range;

/// CommonMark's whitespace in a line; any other Unicode space (U+00A0) is text.
pub const SPACE: [char; 2] = [' ', '\t'];

/// One line of a text, split at '\n' with any carriage return kept, as the outline reads it.
pub struct Line<'a> {
    pub raw: &'a str,
    /// Whether the line starts inside an HTML comment, so nothing at its start shows and a row
    /// written on it is unseen. The empty piece after a final '\n' tells whether one is still open.
    pub hidden: bool,
    /// Whether it is a column-0 ATX heading outside every comment and fence.
    pub heading: bool,
    /// What a renderer shows of it: the line less every comment span on it.
    pub shown: String,
}

/// Every line of a text with how it reads.
pub fn lines(text: &str) -> Vec<Line<'_>> {
    let (spans, lines) = read(text);
    let (mut start, mut first) = (0, 0);
    let mut out = Vec::new();
    for (raw, (hidden, heading)) in text.split('\n').zip(lines) {
        // The spans are in order and apart: one that ends before this line ends before every
        // later line too, so the cursor only moves forward and the whole text costs one pass.
        while spans.get(first).is_some_and(|span| span.end <= start) {
            first += 1;
        }
        let shown = without(text, &spans[first..], start..start + raw.len());
        out.push(Line {
            raw,
            hidden,
            heading,
            shown,
        });
        start += raw.len() + 1;
    }
    out
}

/// The HTML comments of a text, `<!--` through `-->` (or the end of the text), in order.
pub fn comments(text: &str) -> Vec<&str> {
    read(text).0.into_iter().map(|span| &text[span]).collect()
}

/// What a renderer shows of a text: the text less its comments, the line breaks inside them too.
pub fn visible(text: &str) -> String {
    without(text, &read(text).0, 0..text.len())
}

/// Whether each line of a text starts inside an HTML comment, as `lines` reads it.
pub fn hidden(text: &str) -> Vec<bool> {
    read(text).1.into_iter().map(|(hidden, _)| hidden).collect()
}

/// The bytes of `range` that no span covers. The spans are in order and apart, so the ones that
/// cover part of the range are a run, and the scan stops at the first span past it.
fn without(text: &str, spans: &[Range<usize>], range: Range<usize>) -> String {
    let mut kept = String::new();
    let mut at = range.start;
    let touching = spans.iter().skip_while(|s| s.end <= range.start);
    for span in touching.take_while(|s| s.start < range.end) {
        kept.push_str(&text[at..span.start.max(at)]);
        at = span.end.min(range.end);
    }
    kept.push_str(&text[at.min(range.end)..range.end]);
    kept
}

/// The comment spans of a text, and per line whether it starts inside one and is a heading.
/// Column-0 fence content is literal (a `<!--` in it is text), a fence or heading line outside a
/// comment opens none, an unclosed comment hides the rest of the text.
fn read(text: &str) -> (Vec<Range<usize>>, Vec<(bool, bool)>) {
    let (mut spans, mut lines) = (Vec::<Range<usize>>::new(), Vec::new());
    let mut fence: Option<(u8, usize)> = None;
    let (mut open, mut start) = (false, 0);
    for raw in text.split('\n') {
        let bare = raw.trim_end_matches('\r');
        let (hidden, mut head) = (open, false);
        if let Some(mark) = fence {
            fence = fence.filter(|_| !closes(bare, mark));
        } else if let Some(mark) = fence_mark(bare).filter(|_| !open) {
            fence = Some(mark);
        } else if !open && heading(bare).is_some() {
            head = true;
        } else {
            let (found, still) = scan(bare, open);
            for (index, span) in found.into_iter().enumerate() {
                let span = start + span.start..start + span.end;
                match spans.last_mut() {
                    Some(last) if open && index == 0 => last.end = span.end,
                    _ => spans.push(span),
                }
            }
            open = still;
        }
        lines.push((hidden, head));
        start += raw.len() + 1;
    }
    (spans, lines)
}

/// The HTML comments of one line read from `open` (whether one is open at its start), as byte
/// ranges of the line, and whether one is still open at its end. A `<!--` runs to the next `-->`
/// looked for from its own `--`, so `<!-->` and `<!--->` close themselves.
pub fn scan(line: &str, open: bool) -> (Vec<Range<usize>>, bool) {
    let mut spans = Vec::new();
    let (mut begin, mut at) = (open.then_some(0), 0);
    loop {
        match begin {
            Some(from) => {
                let Some(close) = line[at..].find("-->") else {
                    spans.push(from..line.len());
                    return (spans, true);
                };
                at += close + 3;
                spans.push(from..at);
                begin = None;
            }
            None => {
                let Some(found) = line[at..].find("<!--") else {
                    return (spans, false);
                };
                begin = Some(at + found);
                at += found + 2;
            }
        }
    }
}

/// An ATX heading: one to six `#`, then a space, a tab or the end of the line.
pub fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|b| *b == b'#').count();
    let after = &line[level..];
    let atx = (1..=6).contains(&level) && (after.is_empty() || after.starts_with(SPACE));
    atx.then(|| (level, after.trim_matches(SPACE)))
}

/// A code fence marker: three or more backticks or tildes, and no backtick after a backtick run.
pub fn fence_mark(line: &str) -> Option<(u8, usize)> {
    let mark = line.bytes().next().filter(|b| matches!(b, b'`' | b'~'))?;
    let run = line.bytes().take_while(|b| *b == mark).count();
    (run >= 3 && !(mark == b'`' && line[run..].contains('`'))).then_some((mark, run))
}

/// A closing fence: up to three spaces, the opening's mark at least as long, then spaces or tabs.
pub fn closes(line: &str, (mark, run): (u8, usize)) -> bool {
    let rest = line.trim_start_matches(' ');
    let length = rest.bytes().take_while(|b| *b == mark).count();
    line.len() - rest.len() <= 3 && length >= run && rest[length..].trim_matches(SPACE).is_empty()
}
