// verbs/request/prd_check.rs
// The PRD half of check request (R03): the prose and the rows a request must fill before
// approval, and the lines the R43 cap counts.

use crate::core::context::Context;
use crate::core::fsx::sha256_file;
use crate::core::target::{Target, TargetKind};
use crate::store::request::{approval_matches, RequestDoc};
use crate::store::request_sections::{body, is_blank, label, Place, REQUIREMENTS, SECTION_KEYS};
use crate::store::rows;

use super::{background as block, is_approved, request_file, target_flags};

/// The part-1 sections a Goal request fills before approval (D-01). 요구사항 is filled by its
/// rows instead, so its standing guidance is an instruction, never a gap; part 2 answers to
/// design_review.
const GOAL: [&str; 5] = [
    "background",
    "goals",
    "non-goals",
    "scenarios",
    "assumptions",
];

/// The heading a request written before the Korean layout holds its rows under.
const LEGACY: &str = "Requirements";

/// The headings only the PRD layout has besides the ten prose sections of `SECTION_KEYS`, as
/// (level, text): the two part markers and the brief.
const LAYOUT: [(usize, &str); 3] = [(1, "1부 요청"), (1, "2부 설계"), (2, "한눈에 보기")];

/// How many required places fail: each of the five part-1 sections of a Goal request, or the
/// summary paragraph of a quick one, that is missing, empty or still holds its template
/// guidance, and the requirements section when it holds no R row. An approved request whose text
/// still matches its stamp is not judged again: it passed these checks or predates them. Once
/// changed (an edit, a merged pending row) it is judged again before a re-stamp, except a Goal
/// request with none of the PRD layout headings (D-22): `request section` refuses an approved
/// request, so a legacy one could never gain part 1, and no earlier approval starts failing. A
/// quick request never has those headings, but its one required place, the summary, an old quick
/// request has too, so it is always judged (D-23). While approved, a failing place is hinted as
/// an edit and a new approval.
pub fn sections(ctx: &mut Context, target: &Target, doc: &RequestDoc) -> usize {
    let approved = is_approved(target);
    if approved {
        // An unreadable file or stamp proves nothing unchanged, so the request is judged.
        let unchanged = sha256_file(&request_file(target))
            .is_ok_and(|hash| approval_matches(&target.dir, &hash).unwrap_or(false));
        if unchanged {
            ctx.out.say(
                "  sections: not judged (approved and unchanged since; the approval hash guards the text)",
            );
            return 0;
        }
        if target.kind == TargetKind::Run && !prd_layout(doc.text()) {
            ctx.out.say("  sections: not judged (approved without the PRD layout headings; request section refuses an approved request)");
            return 0;
        }
    }
    let keys: &[&str] = match target.kind {
        TargetKind::Run => &GOAL,
        TargetKind::Quick => &["summary"],
    };
    let (mut failed, mut refusals) = (0, Vec::new());
    for key in keys {
        let place = Place::of_key(key).expect("a prose key");
        let problem = match body(doc.text(), &place) {
            Ok(prose) if is_blank(prose) => "empty or template guidance only",
            Ok(prose) if keeps_guidance(prose, key) => "template guidance still present",
            Ok(_) => continue,
            // An outline the reader refuses fails every place with the same line: one condition.
            Err(error) if refusals.contains(&error.message().to_string()) => continue,
            Err(error) => {
                say!(ctx, "  section {key}: {}", error.message());
                refusals.push(error.message().to_string());
                failed += 1;
                continue;
            }
        };
        let hint = match approved {
            true => "edit it in request.md, then dstack request approve again".to_string(),
            false => format!("dstack request section {key} --from <file>"),
        };
        say!(
            ctx,
            "  section {}: {problem} ({hint})",
            label(doc.text(), &place)
        );
        failed += 1;
    }
    // Only rows inside the requirements section count; a request with neither heading reads its
    // rows anywhere, as every row reader does. Either way a row inside a comment is unseen (D-24).
    match requirements(doc.text()) {
        Err(message) if refusals.contains(&message) => {}
        Err(message) => {
            say!(ctx, "  section requirements: {message}");
            failed += 1;
        }
        Ok(section) => {
            let (heading, found) = match section {
                Some((heading, rows)) => (heading, holds_row(rows)),
                None => (REQUIREMENTS, holds_row(doc.text())),
            };
            if !found {
                say!(
                    ctx,
                    "  section ## {heading}: no R row (dstack req add \"<requirement>\" --accept \"<criterion>\")"
                );
                failed += 1;
            }
        }
    }
    say!(
        ctx,
        "  sections: required {}, unfilled {failed}",
        keys.len() + 1
    );
    failed
}

/// R09: a row merged into an approved Goal request carries the background it adds, and a block
/// explains a merged row. Pending rows without a pending background block fail, as does a block
/// with no pending row, and each pending label with no prose under it, even beside a block that
/// counts; `request approve` clears both kinds of marker together. Judged in both
/// modes, apart from `sections`, which skips an unchanged approval. A quick request (which
/// `request background` refuses) and one with none of the PRD layout headings, which has no
/// background section to append to, are exempt, so their merged rows still approve (D-28).
pub fn background(ctx: &mut Context, target: &Target, doc: &RequestDoc) -> usize {
    if target.kind != TargetKind::Run || !prd_layout(doc.text()) {
        return 0;
    }
    let rows = doc.rows().iter().filter(|row| row.is_pending()).count();
    let blocks = block::pending_blocks(doc.text());
    let bare = block::bare_labels(doc.text());
    for (lineno, line) in &bare {
        say!(ctx, "  background: line {lineno} is a pending background label with no prose before the next heading (write prose under it, or remove it): {line}");
    }
    let failed = match (rows, blocks) {
        (0, 0) => 0,
        (_, 0) => {
            say!(ctx, "  background: {rows} pending row(s) and no pending background block (dstack request background {} --from <file>)", target_flags(target).join(" "));
            1
        }
        (0, _) => {
            say!(ctx, "  background: {blocks} pending background block(s) and no pending row (a block explains a merged row: dstack req add, or remove the block)");
            1
        }
        _ => {
            say!(
                ctx,
                "  background: pending rows {rows}, pending background blocks {blocks}"
            );
            0
        }
    };
    failed + bare.len()
}

/// Whether any column-0 ATX heading of the text is one of the PRD layout's, at its level. Unlike
/// the section reader this skips no fence or comment, so a doubt only ever means judging.
fn prd_layout(text: &str) -> bool {
    let sections = SECTION_KEYS.iter().map(|&(_, heading)| (2, heading));
    let layout: Vec<(usize, &str)> = LAYOUT.into_iter().chain(sections).collect();
    text.lines().any(|line| {
        let level = line.bytes().take_while(|b| *b == b'#').count();
        let rest = &line[level..];
        (rest.is_empty() || rest.starts_with([' ', '\t']))
            && layout.contains(&(level, rest.trim_matches([' ', '\t'])))
    })
}

/// Whether a body still holds its template guidance: an HTML comment carrying the `(키: <key>)`
/// marker every template guidance comment ends with. The marker names no template version, and
/// a legacy template has none; `request section` replaces the whole body, so it drops the comment.
fn keeps_guidance(body: &str, key: &str) -> bool {
    let marker = format!("(키: {key})");
    let mut rest = body;
    while let Some((_, after)) = rest.split_once("<!--") {
        // An unclosed comment runs to the end of the body, as `is_blank` reads it.
        let (comment, next) = after.split_once("-->").unwrap_or((after, ""));
        if comment.contains(&marker) {
            return true;
        }
        rest = next;
    }
    false
}

/// The requirements section (D-21): the heading and body of exactly one of `## 요구사항` and the
/// legacy `## Requirements`, None when neither heading exists, or the line naming why the outline
/// gives no single section (both headings, a repeated one, an outline the reader refuses).
/// `locate` says a heading is absent only through its message, so that message is matched here.
fn requirements(text: &str) -> Result<Option<(&'static str, &str)>, String> {
    let mut found = None;
    for heading in [REQUIREMENTS, LEGACY] {
        match body(text, &Place::Section(heading)) {
            Ok(_) if found.is_some() => {
                return Err(format!(
                    "'## {REQUIREMENTS}' and '## {LEGACY}' both appear; keep one"
                ))
            }
            Ok(rows) => found = Some((heading, rows)),
            Err(error) if error.message() == format!("no '## {heading}' heading") => {}
            Err(error) => return Err(error.message().to_string()),
        }
    }
    Ok(found)
}

/// Whether a text holds an R row a reader sees, read line by line as `RequestDoc::rows` reads the
/// file, less a line that starts inside an HTML comment (D-24).
fn holds_row(text: &str) -> bool {
    let mut open = false;
    text.split('\n').enumerate().any(|(index, line)| {
        let hidden = open;
        open = comment_open(line, open);
        !hidden && rows::parse_line(index + 1, line).is_some()
    })
}

/// Whether an HTML comment is still open at the end of `line`, given whether one was at its
/// start. A comment runs from `<!--` to the next `-->`, as `is_blank` reads it, so an unclosed
/// one hides the rest of the text.
fn comment_open(line: &str, mut open: bool) -> bool {
    let mut rest = line;
    loop {
        let mark = if open { "-->" } else { "<!--" };
        let Some(at) = rest.find(mark) else {
            return open;
        };
        (rest, open) = (&rest[at + mark.len()..], !open);
    }
}

/// The lines the R43 cap counts: the whole requirements section, comments and blank lines
/// included, so prose in other sections costs nothing. A request with neither heading counts the
/// span from its first R row to its last; one whose outline gives no single section (already a
/// failure before approval) counts the whole file, as the cap did before the sections existed.
pub fn requirement_lines(doc: &RequestDoc) -> usize {
    match requirements(doc.text()) {
        Ok(Some((_, rows))) => rows.matches('\n').count(),
        Ok(None) => {
            let rows = doc.rows();
            match (rows.first(), rows.last()) {
                (Some(first), Some(last)) => last.lineno - first.lineno + 1,
                _ => 0,
            }
        }
        Err(_) => doc.line_count(),
    }
}
