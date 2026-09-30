// verbs/request/prd_check.rs
// The PRD half of check request (R03): the prose and the rows a request must fill before
// approval, and the lines the R43 cap counts.

use crate::core::context::Context;
use crate::core::fsx::sha256_file;
use crate::core::target::{Target, TargetKind};
use crate::store::request::{approval_matches, RequestDoc};
use crate::store::request_sections::{body, is_blank, label, Place, REQUIREMENTS, SECTION_KEYS};
use crate::store::rows;

use super::{is_approved, request_file};

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
/// changed (an edit, a merged pending row) it is judged again before a re-stamp, except one with
/// none of the PRD layout headings (D-22): `request section` refuses an approved request, so a
/// legacy one could never gain part 1, and no earlier approval starts failing.
pub fn sections(ctx: &mut Context, target: &Target, doc: &RequestDoc) -> usize {
    if is_approved(target) {
        // An unreadable file or stamp proves nothing unchanged, so the request is judged.
        let unchanged = sha256_file(&request_file(target))
            .is_ok_and(|hash| approval_matches(&target.dir, &hash).unwrap_or(false));
        if unchanged {
            ctx.out.say(
                "  sections: not judged (approved and unchanged since; the approval hash guards the text)",
            );
            return 0;
        }
        if !prd_layout(doc.text()) {
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
        say!(
            ctx,
            "  section {}: {problem} (dstack request section {key} --from <file>)",
            label(doc.text(), &place)
        );
        failed += 1;
    }
    // Only rows inside the requirements section count; a request with neither heading reads its
    // rows anywhere, as every row reader does.
    match requirements(doc.text()) {
        Err(message) if refusals.contains(&message) => {}
        Err(message) => {
            say!(ctx, "  section requirements: {message}");
            failed += 1;
        }
        Ok(section) => {
            let (heading, found) = match section {
                Some((heading, rows)) => (heading, holds_row(rows)),
                None => (REQUIREMENTS, !doc.rows().is_empty()),
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

/// Whether one body holds an R row, read line by line as `RequestDoc::rows` reads the file.
fn holds_row(body: &str) -> bool {
    body.split('\n')
        .enumerate()
        .any(|(index, line)| rows::parse_line(index + 1, line).is_some())
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
