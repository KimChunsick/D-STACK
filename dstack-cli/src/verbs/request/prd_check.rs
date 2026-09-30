// verbs/request/prd_check.rs
// The PRD half of check request (R03): the prose a request must fill before approval, and the
// lines the R43 cap counts.

use crate::core::context::Context;
use crate::core::target::{Target, TargetKind};
use crate::store::request::RequestDoc;
use crate::store::request_sections::{body, is_blank, label, Place, REQUIREMENTS};

use super::is_approved;

/// The part-1 sections a Goal request fills before approval (D-01). 요구사항 is not among them:
/// its rows answer to the row rules, and part 2 answers to design_review.
const GOAL: [&str; 5] = [
    "background",
    "goals",
    "non-goals",
    "scenarios",
    "assumptions",
];

/// How many required places are missing or hold only template guidance: the five part-1
/// sections of a Goal request, the summary paragraph of a quick one. An approved request is not
/// judged again — it was stamped before these checks existed or passed them when it was, and its
/// hash guards the text since — so no earlier approval starts failing.
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
        match body(doc.text(), &place) {
            Ok(prose) if !is_blank(prose) => continue,
            Ok(_) => say!(
                ctx,
                "  section {}: empty or template guidance only (dstack request section {key} --from <file>)",
                label(doc.text(), &place)
            ),
            // An outline the reader refuses fails every place with the same line: one condition.
            Err(error) if refusals.contains(&error.message().to_string()) => continue,
            Err(error) => {
                say!(ctx, "  section {key}: {}", error.message());
                refusals.push(error.message().to_string());
            }
        }
        failed += 1;
    }
    say!(
        ctx,
        "  sections: required {}, empty or missing {failed}",
        keys.len()
    );
    failed
}

/// The lines the R43 cap counts: the `## 요구사항` section only, so prose above and below it
/// costs nothing. A request whose rows section cannot be found is counted whole, as before.
pub fn requirement_lines(doc: &RequestDoc) -> usize {
    match body(doc.text(), &Place::Section(REQUIREMENTS)) {
        Ok(rows) => rows.matches('\n').count(),
        Err(_) => doc.line_count(),
    }
}
