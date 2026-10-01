// store/request_part3.rs
// Part 3 of a Goal request (D-38, D-39): the marker line under `# 3부 계획과 검증`, the bytes the
// approval hash covers, and the plan the CLI regenerates below the marker from plan.json.

use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text, sha256_bytes};
use crate::store::plan::{self, Milestone, Plan, PlanDoc};
use crate::store::plan_graph::plan_covers;
use crate::store::visible::{heading, lines};

/// The line that ends what the approval hash covers. Only the CLI writes it (the templates put it
/// right under the visible heading), so a request without it is a legacy one, hashed whole.
pub const MARKER: &str = "<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->";

/// The guidance the templates put under the marker; every render starts with it, so a request
/// before its first plan and one after it read alike.
const GUIDANCE: &str = "<!-- 이 부분은 직접 쓰지 않아요. 계획 대장(plan.json)이 바뀔 때마다 CLI가 Milestone, Plan, Task 분해를 채워요. -->";

/// What a field without a value shows, so an empty one reads as empty rather than as missing.
const EMPTY: &str = "(비어 있어요)";

/// The byte offset just past the first marker line, its line break included. The marker counts
/// only on a line that is the marker once trimmed and shows nothing: inside a fence the comment
/// is text a renderer shows, and a line that starts inside a comment opened above is hidden (D-39).
fn marker_end(text: &str) -> Option<usize> {
    let mut end = 0;
    for line in lines(text) {
        end += line.raw.len() + 1;
        if !line.hidden && line.raw.trim() == MARKER && line.shown.trim().is_empty() {
            return Some(end.min(text.len()));
        }
    }
    None
}

/// What the approval covers: every byte through the first marker line, or the whole text when
/// there is none. The marker line itself is covered, so deleting it together with part 3 changes
/// the bytes instead of turning the request into a legacy one, and so does an edit that reaches
/// it: a marker with anything else on its line is no marker.
pub fn approval_bytes(text: &str) -> &str {
    marker_end(text).map_or(text, |end| &text[..end])
}

/// The sha256 a stamp records for this text, in the hex form `request.approved` holds.
pub fn approval_hash(text: &str) -> String {
    sha256_bytes(approval_bytes(text).as_bytes())
}

/// The approval hash of the request file at `path`, as approve writes it and every check reads it.
pub fn sha256_approved(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map(|text| approval_hash(&text))
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", path.display())))
}

/// Rewrite part 3 of the request in the run directory `dir` from `doc`, written only when it
/// changes. The bytes through the marker never change, so the approval stays valid. A run without
/// request.md or without the marker (a legacy request) is left alone, and so is a marker on the
/// last line without a line break, since appending would have to change the line it ends.
pub fn regenerate(dir: &Path, run: &str, doc: &PlanDoc) -> Result<()> {
    let file = dir.join("request.md");
    let Some(text) = read_text(&file)? else {
        return Ok(());
    };
    let Some(head) = marker_end(&text).map(|end| &text[..end]) else {
        return Ok(());
    };
    if !head.ends_with('\n') {
        return Ok(());
    }
    let fresh = format!("{head}{}", render(&title(&text), run, doc));
    if fresh == text {
        return Ok(());
    }
    atomic_write(&file, fresh.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))
}

/// What check request holds a marker-bearing request to (D-38): part 3 shows every Milestone's
/// goal and every Plan's purpose, so an empty one is a failure naming the command that fills it.
/// A legacy request, or a run without plan.json, has none.
pub fn plan_problems(dir: &Path, text: &str) -> Result<Vec<String>> {
    if marker_end(text).is_none() || !plan::exists(dir) {
        return Ok(Vec::new());
    }
    let doc = plan::load(dir)?;
    let blank = |value: &str| value.trim().is_empty();
    let goals = doc.milestones.iter().filter(|m| blank(&m.goal)).map(|m| {
        let m = &m.id;
        format!("part 3: milestone {m} has no goal (dstack milestone edit {m} --goal <text>)")
    });
    let purposes = doc.plans.iter().filter(|p| blank(&p.purpose)).map(|p| {
        let p = &p.id;
        format!("part 3: plan {p} has no purpose (dstack plan edit {p} --purpose <text>)")
    });
    Ok(goals.chain(purposes).collect())
}

/// The request's title: its first level-1 heading, which the templates put under the frontmatter.
fn title(text: &str) -> String {
    lines(text)
        .into_iter()
        .filter(|line| line.heading)
        .find_map(|line| match heading(line.raw.trim_end_matches('\r')) {
            Some((1, title)) => Some(title.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Part 3 below the marker: the Goal, then each Milestone in order under a `##` heading with its
/// Plans under `###` headings and their Tasks as nested lists. R ids are inline text, never a row
/// line, so no row reader counts them; every heading carries its unit's id, so none repeats.
pub fn render(title: &str, run: &str, doc: &PlanDoc) -> String {
    let mut out = format!("{GUIDANCE}\n\nGoal `{run}`: {}\n", or_empty(title));
    let mut milestones: Vec<&Milestone> = doc.milestones.iter().collect();
    milestones.sort_by_key(|milestone| milestone.order);
    if milestones.is_empty() {
        out.push('\n');
        item(&mut out, "", "Milestone", "");
    }
    for milestone in milestones {
        out.push_str(&format!("\n## {} {}\n\n", milestone.id, milestone.slug));
        item(&mut out, "", "목표", &milestone.goal);
        item(&mut out, "", "확인한 Plan", &milestone.confirmed.join(", "));
        let plans: Vec<&Plan> = doc.plans.iter().filter(|p| p.milestone == milestone.id).collect();
        if plans.is_empty() {
            item(&mut out, "", "Plan", "");
        }
        for plan in plans {
            render_plan(&mut out, doc, plan);
        }
    }
    out
}

fn render_plan(out: &mut String, doc: &PlanDoc, plan: &Plan) {
    out.push_str(&format!("\n### {} {}\n\n", plan.id, plan.slug));
    let files: Vec<String> = plan.files.iter().map(|file| code(file)).collect();
    item(out, "", "목적", &plan.purpose);
    item(out, "", "E2E 초점", &plan.e2e_focus);
    item(out, "", "다루는 R 행", &plan_covers(doc, &plan.id).join(", "));
    item(out, "", "선언 파일", &files.join(", "));
    item(out, "", "선행 Plan", &plan.deps.join(", "));
    item(out, "", "상태", &code(&plan.status));
    if plan.tasks.is_empty() {
        item(out, "", "Task", "");
    }
    for task in &plan.tasks {
        out.push_str(&format!("- Task {} {}\n", task.id, task.slug));
        let status = match task.commit.as_str() {
            "" => "아직 커밋하지 않았어요".to_string(),
            commit => format!("커밋했어요 ({})", code(commit)),
        };
        item(out, "  ", "목적", &task.purpose);
        item(out, "  ", "다루는 R 행", &task.covers.join(", "));
        item(out, "  ", "상태", &status);
    }
}

/// One `- label: value` list item, the placeholder standing in for an empty value.
fn item(out: &mut String, indent: &str, label: &str, value: &str) {
    out.push_str(&format!("{indent}- {label}: {}\n", or_empty(value)));
}

fn or_empty(value: &str) -> &str {
    match value.trim().is_empty() {
        true => EMPTY,
        false => value,
    }
}

/// An identifier as inline code, or nothing when it is empty, so the placeholder shows instead.
fn code(value: &str) -> String {
    match value.is_empty() {
        true => String::new(),
        false => format!("`{value}`"),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn R15_part3_hash_bytes_end_with_the_marker_line() {
        let text = format!("위\n# 3부 계획과 검증\n{MARKER}\n아래\n");
        assert_eq!(approval_bytes(&text), format!("위\n# 3부 계획과 검증\n{MARKER}\n"));
        let crlf = format!("위\r\n{MARKER}\r\n아래\r\n");
        assert_eq!(approval_bytes(&crlf), format!("위\r\n{MARKER}\r\n"));
        let last = format!("위\n  {MARKER}");
        assert_eq!(approval_bytes(&last), last);
        assert_eq!(approval_bytes(&format!("{MARKER}\n아래\n")), format!("{MARKER}\n"));
        let two = format!("위\n{MARKER}\n가운데\n{MARKER}\n아래\n");
        assert_eq!(approval_bytes(&two), format!("위\n{MARKER}\n"));
    }

    #[test]
    fn R15_part3_hash_bytes_are_the_whole_text_without_a_whole_marker_line() {
        for text in [
            "위\n# 3부 계획과 검증\n아래\n".to_string(),
            format!("위\n{MARKER}x\n아래\n"),
            format!("위\n앞 {MARKER}\n아래\n"),
            String::new(),
        ] {
            assert_eq!(approval_bytes(&text), text);
        }
        assert_eq!(approval_hash("위\n"), sha256_bytes("위\n".as_bytes()));
    }

    #[test]
    fn R15_part3_hash_marker_counts_only_outside_fences_and_comments() {
        for text in [
            format!("위\n```\n{MARKER}\n```\n아래\n"),
            format!("위\n<!-- 열린 주석\n{MARKER}\n-->\n아래\n"),
        ] {
            assert_eq!(approval_bytes(&text), text);
        }
        let after = format!("~~~\n{MARKER}\n~~~\n{MARKER}\n아래\n");
        assert_eq!(approval_bytes(&after), format!("~~~\n{MARKER}\n~~~\n{MARKER}\n"));
    }
}
