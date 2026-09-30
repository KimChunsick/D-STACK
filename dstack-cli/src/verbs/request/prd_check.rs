// verbs/request/prd_check.rs
// The PRD half of check request (R03): the prose and the rows a request must fill before
// approval, and the lines the R43 cap counts.

use crate::core::context::Context;
use crate::core::target::{Target, TargetKind};
use crate::store::request::RequestDoc;
use crate::store::request_sections::{body, is_blank, label, Place, REQUIREMENTS};

use super::is_approved;

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

/// How many required places fail: each of the five part-1 sections of a Goal request, or the
/// summary paragraph of a quick one, that is missing, empty or still holds its template
/// guidance, and 요구사항 when the request has no R row. An approved request is not judged
/// again — it was stamped before these checks existed or passed them when it was, and its hash
/// guards the text since — so no earlier approval starts failing.
pub fn sections(ctx: &mut Context, target: &Target, doc: &RequestDoc) -> usize {
    if is_approved(target) {
        ctx.out
            .say("  sections: not judged (approved; the approval hash guards the text)");
        return 0;
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
    // Rows are read anywhere in the file, as every row reader reads them.
    if doc.rows().is_empty() {
        say!(
            ctx,
            "  section ## {REQUIREMENTS}: no R row (dstack req add \"<requirement>\" --accept \"<criterion>\")"
        );
        failed += 1;
    }
    say!(
        ctx,
        "  sections: required {}, unfilled {failed}",
        keys.len() + 1
    );
    failed
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

/// The lines the R43 cap counts: the `## 요구사항` section only, so prose above and below it
/// costs nothing. A request without that section (a legacy `## Requirements`, an outline the
/// reader refuses) counts the span from its first R row to its last, never its prose.
pub fn requirement_lines(doc: &RequestDoc) -> usize {
    if let Ok(rows) = body(doc.text(), &Place::Section(REQUIREMENTS)) {
        return rows.matches('\n').count();
    }
    let rows = doc.rows();
    match (rows.first(), rows.last()) {
        (Some(first), Some(last)) => last.lineno - first.lineno + 1,
        _ => 0,
    }
}
