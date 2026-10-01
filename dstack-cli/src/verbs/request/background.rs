// verbs/request/background.rs
// dstack request background: the background a merged row adds to an approved Goal request, as a
// pending block after every earlier byte, and the marker `request approve` clears (R09).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{read_text, utc_now};
use crate::core::target::{resolve_target, TargetKind};
use crate::store::request_sections::{check_prose, is_blank, locate, Place};

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

fn is_pending(line: &str) -> bool {
    line.starts_with(LABEL) && line.ends_with(MARKER)
}

/// The line numbers of the pending block labels in the background body; none when the outline
/// gives no single section.
fn pending_lines(text: &str) -> Vec<usize> {
    let Ok(range) = locate(text, &SECTION) else {
        return Vec::new();
    };
    let first = text[..range.start].matches('\n').count() + 1;
    text[range]
        .split('\n')
        .enumerate()
        .filter(|(_, line)| is_pending(line))
        .map(|(index, _)| first + index)
        .collect()
}

/// How many blocks still wait for `request approve`.
pub fn pending_blocks(text: &str) -> usize {
    pending_lines(text).len()
}

/// The text with every block's marker removed, and how many there were: what `request approve`
/// writes before it re-stamps, so the block stays as plain prose.
pub fn clear(text: &str) -> (String, usize) {
    let lines = pending_lines(text);
    let mut out = text.to_string();
    for &lineno in &lines {
        let line = rowfile::lines(&out)[lineno - 1].to_string();
        let plain = line.strip_suffix(MARKER).unwrap_or(&line);
        out = rowfile::set_line(&out, lineno, plain);
    }
    (out, lines.len())
}
