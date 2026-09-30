// verbs/request/design_gate.rs
// The design gate of request approve (R13): part 2 of a Goal request against its design_review.

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::sha256_file;
use crate::core::target::{Target, TargetKind};
use crate::store::request::{approval_matches, RequestDoc};
use crate::store::request_sections::{body, is_blank, label, Place, SECTION_KEYS};
use crate::store::tables::decisions;

use super::{is_approved, request_file};

/// The decision text `request design-skip` writes and this gate looks for.
pub const SKIP_PREFIX: &str = "design skipped:";

/// The part-2 keys of `SECTION_KEYS`, in template order.
const PART_TWO: [&str; 5] = ["current", "proposed", "alternatives", "mapping", "risks"];

/// The headings only the PRD layout has besides the ten prose sections; the same list as
/// prd_check.rs `LAYOUT` (sharing the two is a follow-up).
const LAYOUT: [(usize, &str); 3] = [(1, "1부 요청"), (1, "2부 설계"), (2, "한눈에 보기")];

/// Whether the request may be stamped as far as design goes. required: all five part-2 sections
/// filled. auto: all five filled, or a design-skip reason recorded. skip: nothing asked. A quick
/// request has no part 2; an approved request unchanged since its stamp is not judged again, nor
/// one without any PRD layout heading (D-22), so legacy runs still re-approve after a merge.
/// Whenever a skip lets the approval through, the recorded reason is printed.
pub fn gate(ctx: &mut Context, target: &Target, doc: &RequestDoc) -> Result<()> {
    let value = doc.field("design_review").unwrap_or_default();
    say!(ctx, "== design review (design_review: {value})");
    if target.kind == TargetKind::Quick {
        ctx.out.say("  not judged (a quick request has no part 2)");
        return Ok(());
    }
    let approved = is_approved(target);
    if approved {
        let unchanged = sha256_file(&request_file(target))
            .is_ok_and(|hash| approval_matches(&target.dir, &hash).unwrap_or(false));
        if unchanged {
            ctx.out.say("  not judged (approved and unchanged since)");
            return Ok(());
        }
        if !prd_layout(doc.text()) {
            ctx.out.say("  not judged (approved without the PRD layout headings)");
            return Ok(());
        }
    }
    let skip = recorded_skip(&target.dir)?;
    if value == "skip" {
        ctx.out.say("  design review skipped by the frontmatter (design_review: skip)");
        if let Some((id, reason)) = &skip {
            say!(ctx, "  recorded reason ({id}): {reason}");
        }
        return Ok(());
    }
    let unfilled = unfilled(doc.text());
    if unfilled.is_empty() {
        say!(ctx, "  part 2: {} of {} sections filled", PART_TWO.len(), PART_TWO.len());
        return Ok(());
    }
    if value == "auto" {
        if let Some((id, reason)) = &skip {
            say!(ctx, "  design review skipped ({id}): {reason}");
            return Ok(());
        }
    }
    for (key, heading, problem) in &unfilled {
        let hint = match approved {
            true => "edit it in request.md, then dstack request approve again".to_string(),
            false => format!("dstack request section {key} --from <file>"),
        };
        say!(ctx, "  section {heading}: {problem} ({hint})");
    }
    let n = unfilled.len();
    if value == "auto" && !approved {
        fail!("design_review is auto and {n} part-2 section(s) are unfilled: fill them, or record why design is skipped: dstack request design-skip --why \"<reason>\" --run {}", target.id);
    }
    fail!("design_review is {value} and {n} part-2 section(s) are unfilled: fill them and approve again")
}

/// Each part-2 section that is missing, empty or still holds its template guidance, as
/// (key, label, problem).
fn unfilled(text: &str) -> Vec<(&'static str, String, String)> {
    let mut found = Vec::new();
    for key in PART_TWO {
        let place = Place::of_key(key).expect("a part-2 key");
        let problem = match body(text, &place) {
            Ok(prose) if is_blank(prose) => "empty or template guidance only".to_string(),
            Ok(prose) if keeps_guidance(prose, key) => "template guidance still present".to_string(),
            Ok(_) => continue,
            Err(error) => error.message().to_string(),
        };
        found.push((key, label(text, &place), problem));
    }
    found
}

/// The latest design-skip decision of a run that gives a reason, as (D id, reason). A row with
/// nothing but blanks after the prefix (`decision add "design skipped:"` writes one) is no reason.
pub fn recorded_skip(dir: &Path) -> Result<Option<(String, String)>> {
    Ok(decisions(&dec_file(dir))?
        .into_iter()
        .rev()
        .find_map(|row| {
            let reason = row.text.strip_prefix(SKIP_PREFIX)?.trim().to_string();
            (!reason.is_empty()).then_some((row.id, reason))
        }))
}

pub fn dec_file(dir: &Path) -> PathBuf {
    dir.join("decisions.md")
}

/// prd_check.rs `prd_layout`: whether any column-0 ATX heading is one of the PRD layout's.
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

/// prd_check.rs `keeps_guidance`: an HTML comment in the body carrying the `(키: <key>)` marker.
fn keeps_guidance(body: &str, key: &str) -> bool {
    let marker = format!("(키: {key})");
    let mut rest = body;
    while let Some((_, after)) = rest.split_once("<!--") {
        let (comment, next) = after.split_once("-->").unwrap_or((after, ""));
        if comment.contains(&marker) {
            return true;
        }
        rest = next;
    }
    false
}
