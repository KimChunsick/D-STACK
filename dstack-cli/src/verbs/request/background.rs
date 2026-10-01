// verbs/request/background.rs
// dstack request background: the background a merged row adds to an approved Goal request, as a
// pending block after every earlier byte, and the marker `request approve` clears (R09).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{read_text, utc_now};
use crate::core::target::{resolve_target, TargetKind};
use crate::store::request_sections::{check_prose, is_blank, locate, Place};
use crate::store::visible;

use super::{counts, is_approved, load, require_file, rowfile, take, target_flags};

/// The section a block is appended to.
const HEADING: &str = "배경과 문제";
const SECTION: Place = Place::Section(HEADING);

/// The label a block opens with, and the marker it carries until `request approve` re-stamps.
/// The marker is a row's status segment, so a reader meets one spelling of "awaiting approval".
const LABEL: &str = "### 추가 배경";
const MARKER: &str = " — status: pending-approval";

/// CommonMark's whitespace across lines, as the section writer trims new prose.
const BLANK: [char; 4] = [' ', '\t', '\n', '\r'];

pub fn background(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let mut from = String::new();
    let mut i = 0;
    while i < rest.len() {
        if let Some((value, eaten)) = take(&rest, i, "from")? {
            from = value;
            i += eaten;
        } else if rest[i].starts_with('-') {
            fail!("unknown option: {}", rest[i]);
        } else {
            fail!("unexpected argument: {}", rest[i]);
        }
    }
    if from.is_empty() {
        fail!("usage: dstack request background --run <id> --from <file>");
    }
    if target.kind == TargetKind::Quick {
        fail!("request background writes a Goal request; a quick task has no ## 배경과 문제 (its merged row approves without one)");
    }
    let file = require_file(&target)?;
    // Before approval the whole section is still open to its own writer, so no block is needed.
    if !is_approved(&target) {
        fail!(
            "{} is not approved; write the background with dstack request section background --from <file>",
            file.display()
        );
    }
    let content = match read_text(Path::new(&from))? {
        Some(content) => content,
        None => fail!("--from file not found: {from}"),
    };
    if is_blank(&content) {
        fail!("--from file {from} is empty (or comments only); a background block needs prose");
    }
    check_prose(&content, &SECTION)?;
    // A line the block writes itself would be read back as a second block, or cleared as one.
    if let Some((index, line)) = content.lines().enumerate().find(|(_, l)| is_pending(l)) {
        fail!(
            "line {} of the content is a pending background label (request background writes it): {line}",
            index + 1
        );
    }
    // The label is the block's one heading: another would leave the label without prose or
    // split the block, so the content holds none outside comments and fences (an example).
    if let Some((index, (line, _))) = read(&content).enumerate().find(|(_, (_, l))| l.is_none()) {
        fail!(
            "line {} of the content is a heading (a background block is prose under the label request background writes): {}",
            index + 1,
            line.trim_end_matches('\r')
        );
    }
    let doc = load(&target)?;
    let date = utc_now()[..10].to_string();
    let text = match append(doc.text(), &content, &date) {
        Ok(text) => text,
        Err(error) if error.message() == format!("no '## {HEADING}' heading") => fail!(
            "{}: {} (a legacy-layout request; its merged row approves without a background block)",
            file.display(),
            error.message()
        ),
        Err(error) => fail!("{}: {}", file.display(), error.message()),
    };
    rowfile::write(&file, &text)?;
    let doc = load(&target)?;
    let (rows, _live, pending) = counts(&doc);
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  appended to ## {HEADING}: {LABEL} ({date}){MARKER}");
    say!(
        ctx,
        "  rows {rows}, pending {pending}, pending background blocks {}",
        pending_blocks(doc.text())
    );
    say!(
        ctx,
        "  dstack request approve {} clears the markers and re-stamps the hash",
        target_flags(&target).join(" ")
    );
    Ok(())
}

/// The text with a pending block after the last byte of the background body: every earlier byte
/// is kept, and blank lines are added around the block only where none stand.
fn append(text: &str, content: &str, date: &str) -> Result<String> {
    let end = locate(text, &SECTION)?.end;
    let content = content.trim_end_matches(BLANK);
    let lead = &content[..content.len() - content.trim_start_matches(BLANK).len()];
    let content = &content[lead.rfind('\n').map_or(0, |at| at + 1)..];
    let before = &text[..end];
    let mut out = before.to_string();
    if !before.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    out.push_str(&format!("{LABEL} ({date}){MARKER}\n\n{content}\n"));
    if end < text.len() {
        out.push('\n');
    }
    out.push_str(&text[end..]);
    Ok(out)
}

/// Whether a line is a pending label, read as the heading reader reads it: without its CRs.
fn is_pending(line: &str) -> bool {
    let line = line.trim_end_matches('\r');
    line.starts_with(LABEL) && line.ends_with(MARKER)
}

/// The pending block labels of the background body as (line number, line, whether the block
/// shows text outside comments before the next heading or the end of the section); none when
/// the outline gives no single section. A label is a real heading (column 0, outside every
/// comment and fence), so an example in a comment or fence is neither counted nor cleared.
fn labels(text: &str) -> Vec<(usize, &str, bool)> {
    let Ok(range) = locate(text, &SECTION) else {
        return Vec::new();
    };
    let first = text[..range.start].matches('\n').count() + 1;
    let mut found: Vec<(usize, &str, bool)> = Vec::new();
    // Whether the block being read opened with a label.
    let mut open = false;
    for (index, (line, read)) in read(&text[range]).enumerate() {
        match (read, found.last_mut()) {
            (None, _) => {
                open = is_pending(line);
                if open {
                    found.push((first + index, line, false));
                }
            }
            (Some(shown), Some((_, _, seen))) if open => *seen |= shown,
            _ => {}
        }
    }
    found
}

/// Each raw line of a text (as `clear` strips the marker from it) with how the block reader
/// sees it, as store::visible reads it: None for a column-0 heading outside every comment and
/// fence, else whether it shows text outside comments.
fn read(text: &str) -> impl Iterator<Item = (&str, Option<bool>)> {
    visible::lines(text).into_iter().map(|line| {
        let shown = !line.shown.trim_matches(BLANK).is_empty();
        (line.raw, (!line.heading).then_some(shown))
    })
}

/// The label line numbers of the blocks that count: a label with prose under it.
fn blocks(text: &str) -> Vec<usize> {
    let labels = labels(text).into_iter().filter(|(_, _, prose)| *prose);
    labels.map(|(lineno, _, _)| lineno).collect()
}

/// How many blocks still wait for `request approve`.
pub fn pending_blocks(text: &str) -> usize {
    blocks(text).len()
}

/// The pending labels with no prose under them, as (line number, line): `check request` fails
/// each one, so `request approve` never leaves one behind with its marker.
pub fn bare_labels(text: &str) -> Vec<(usize, &str)> {
    let labels = labels(text).into_iter().filter(|(_, _, prose)| !prose);
    labels.map(|(lineno, line, _)| (lineno, line)).collect()
}

/// The text with every block's marker removed, and how many there were: what `request approve`
/// writes before it re-stamps, so the block stays as plain prose. A label keeps its own line
/// ending, so a CRLF label stays CRLF.
pub fn clear(text: &str) -> (String, usize) {
    let lines = blocks(text);
    let mut out = text.to_string();
    for &lineno in &lines {
        let line = rowfile::lines(&out)[lineno - 1].to_string();
        let body = line.trim_end_matches('\r');
        let ending = &line[body.len()..];
        let plain = body.strip_suffix(MARKER).unwrap_or(body);
        out = rowfile::set_line(&out, lineno, &format!("{plain}{ending}"));
    }
    (out, lines.len())
}
