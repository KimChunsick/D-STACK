// store/request_part3.rs
// Part 3 of a Goal request (D-38, D-39): the marker line under `# 3부 계획과 검증`, the bytes the
// approval hash covers, the requirement lines above it, and the plan the CLI regenerates below the
// marker from plan.json and the QA ledger.

use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text, sha256_bytes};
use crate::store::plan::{self, Milestone, Plan, PlanDoc};
use crate::store::plan_graph::plan_covers;
use crate::store::qa::{self, QaEntry};
use crate::store::request_part3_qa;
use crate::store::rows::{self, Row};
use crate::store::visible::{heading, lines, Line};

/// The line that ends what the approval hash covers. Only the CLI writes it (the templates put it
/// right under the visible heading), so a request without it is a legacy one, hashed whole.
pub const MARKER: &str = "<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->";

/// The guidance the templates put under the marker; every render starts with it, so a request
/// before its first plan and one after it read alike.
const GUIDANCE: &str = "<!-- 이 부분은 직접 쓰지 않아요. 계획 대장(plan.json)이 바뀔 때마다 CLI가 Milestone, Plan, Task 분해를 채워요. -->";

/// What a field without a value shows, so an empty one reads as empty rather than as missing.
const EMPTY: &str = "(비어 있어요)";

/// The index of the first marker line. The marker counts only on a line that is the marker once
/// trimmed and shows nothing: inside a fence the comment is text a renderer shows, and a line that
/// starts inside a comment opened above is hidden (D-39).
fn marker_at(lines: &[Line]) -> Option<usize> {
    lines.iter().position(|line| {
        !line.hidden && line.raw.trim() == MARKER && line.shown.trim().is_empty()
    })
}

/// The byte offset just past the first marker line, its line break included.
fn marker_end(text: &str) -> Option<usize> {
    let lines = lines(text);
    let at = marker_at(&lines)?;
    let end: usize = lines[..=at].iter().map(|line| line.raw.len() + 1).sum();
    Some(end.min(text.len()))
}

/// The lines a reader sees, numbered from 1 with any carriage return kept: a line that starts
/// inside an HTML comment is not there (D-33), and neither is the marker line or anything below
/// it, which the CLI generates and the approval does not cover (R15). So no row reader, row edit,
/// row insertion or marker clearing reaches a row there. The empty piece after a final newline is
/// a line here, which no row matches.
pub fn seen_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let lines = lines(text);
    let end = marker_at(&lines).unwrap_or(lines.len());
    lines
        .into_iter()
        .take(end)
        .enumerate()
        .filter(|(_, line)| !line.hidden)
        .map(|(index, line)| (index + 1, line.raw))
}

/// The R rows a reader sees in a text, as `RequestDoc::rows` reads them.
pub fn seen_rows(text: &str) -> Vec<Row> {
    seen_lines(text)
        .filter_map(|(lineno, line)| rows::parse_line(lineno, line))
        .collect()
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

/// Rewrite part 3 of the request in the run directory `dir` from `doc` and the run's QA ledger,
/// written only when it changes. The bytes through the marker never change, so the approval stays
/// valid. A run without request.md or without the marker (a legacy request) is left alone, and so
/// is a marker on the last line without a line break, since appending would change the line. A QA
/// text that cannot be read shows a placeholder, so one bad file never stops a plan write.
pub fn regenerate(dir: &Path, run: &str, doc: &PlanDoc) -> Result<()> {
    match rendered(dir, run, doc, &qa::entries(dir)?)? {
        Some(text) => publish(dir, &text),
        None => Ok(()),
    }
}

/// The request text with part 3 rendered from `doc` and the QA ledger `qa` (each row with its
/// text), None when regenerate leaves the request alone or part 3 already reads so. It writes
/// nothing, so a writer renders before its first write and refuses with the run as it was.
pub fn rendered(dir: &Path, run: &str, doc: &PlanDoc, qa: &[QaEntry]) -> Result<Option<String>> {
    let Some(text) = read_text(&dir.join("request.md"))? else {
        return Ok(None);
    };
    let Some(head) = marker_end(&text).map(|end| &text[..end]) else {
        return Ok(None);
    };
    if !head.ends_with('\n') {
        return Ok(None);
    }
    let qa = request_part3_qa::render(qa);
    let fresh = format!("{head}{}{qa}", render(&title(&text), run, doc));
    Ok((fresh != text).then_some(fresh))
}

/// Part 3 for a writer that changes no plan (the QA ledger), rendered with the ledger it is about
/// to write: from plan.json, or from no plan yet when the run has none, so a QA scenario recorded
/// before planning shows too.
pub fn preview(dir: &Path, run: &str, qa: &[QaEntry]) -> Result<Option<String>> {
    let doc = match plan::exists(dir) {
        true => plan::load(dir)?,
        false => PlanDoc {
            v: 2,
            milestones: Vec::new(),
            plans: Vec::new(),
        },
    };
    rendered(dir, run, &doc, qa)
}

/// Write the request text `rendered` built.
pub fn publish(dir: &Path, text: &str) -> Result<()> {
    let file = dir.join("request.md");
    atomic_write(&file, text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))
}

/// What check request holds a marker-bearing request to (D-38): a row below the marker is no row
/// and part 3 is rewritten anyway, so one written there fails by its line instead of vanishing;
/// and part 3 shows every Milestone's goal and every Plan's purpose, so an empty one is a failure
/// naming the command that fills it. A legacy request has none, a run without plan.json no plan.
pub fn part3_problems(dir: &Path, text: &str) -> Result<Vec<String>> {
    let lines = lines(text);
    let Some(at) = marker_at(&lines) else {
        return Ok(Vec::new());
    };
    let below = lines.iter().enumerate().skip(at + 1).filter(|(index, line)| {
        !line.hidden && rows::parse_line(index + 1, line.raw).is_some()
    });
    let mut found: Vec<String> = below
        .map(|(index, line)| {
            let line = line.raw.trim_end_matches('\r');
            let lineno = index + 1;
            format!("line {lineno}: R rows belong in ## 요구사항 above part 3; part 3 is generated: {line}")
        })
        .collect();
    if !plan::exists(dir) {
        return Ok(found);
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
    found.extend(goals.chain(purposes));
    Ok(found)
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
/// Plans under `###` headings and their Tasks as nested lists. Every entered text follows a label
/// or an id and is made inert, so none opens a comment; no row reader reads below the marker, so
/// an R id in it counts nowhere. Every heading carries its unit's id, so none repeats.
pub fn render(title: &str, run: &str, doc: &PlanDoc) -> String {
    let mut out = format!("{GUIDANCE}\n\nGoal `{run}`: {}\n", or_empty(&inert(title)));
    let mut milestones: Vec<&Milestone> = doc.milestones.iter().collect();
    milestones.sort_by_key(|milestone| milestone.order);
    if milestones.is_empty() {
        out.push('\n');
        item(&mut out, "", "Milestone", "");
    }
    for milestone in milestones {
        out.push_str(&format!("\n## {} {}\n\n", milestone.id, inert(&milestone.slug)));
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
    out.push_str(&format!("\n### {} {}\n\n", plan.id, inert(&plan.slug)));
    item(out, "", "목적", &plan.purpose);
    item(out, "", "E2E 초점", &plan.e2e_focus);
    item(out, "", "다루는 R 행", &plan_covers(doc, &plan.id).join(", "));
    item(out, "", "선언 파일", &files(&plan.files));
    item(out, "", "선행 Plan", &plan.deps.join(", "));
    item(out, "", "상태", &code(&plan.status));
    if plan.tasks.is_empty() {
        item(out, "", "Task", "");
    }
    for task in &plan.tasks {
        out.push_str(&format!("- Task {} {}\n", task.id, inert(&task.slug)));
        let status = match task.commit.as_str() {
            "" => "아직 커밋하지 않았어요".to_string(),
            commit => format!("커밋했어요 ({})", code(commit)),
        };
        item(out, "  ", "목적", &task.purpose);
        item(out, "  ", "다루는 R 행", &task.covers.join(", "));
        item(out, "  ", "선언 파일", &files(&task.files));
        item(out, "  ", "선행 Task", &task.deps.join(", "));
        item(out, "  ", "상태", &status);
    }
}

/// One `- label: value` list item, the value made inert and the placeholder standing in for an
/// empty one.
pub(crate) fn item(out: &mut String, indent: &str, label: &str, value: &str) {
    out.push_str(&format!("{indent}- {label}: {}\n", or_empty(&inert(value))));
}

/// Entered text as part 3 shows it: a `<` would open an HTML comment or tag that hides the rest
/// of the part from a reader, so it is the entity, which still reads as `<`.
pub(crate) fn inert(value: &str) -> String {
    value.replace('<', "&lt;")
}

/// Declared files as inline code, comma separated.
fn files(files: &[String]) -> String {
    files.iter().map(|file| code(file)).collect::<Vec<_>>().join(", ")
}

fn or_empty(value: &str) -> &str {
    match value.trim().is_empty() {
        true => EMPTY,
        false => value,
    }
}

/// An identifier as inline code, or nothing when it is empty, so the placeholder shows instead.
pub(crate) fn code(value: &str) -> String {
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

    #[test]
    fn R15_part3_rows_stop_at_the_marker_line() {
        let row = |id: &str| format!("- [ ] **{id}** 요구사항이에요. — accept: 확인해요.");
        let (r01, r02) = (row("R01"), row("R02"));
        let text = format!("{r01}\n<!--\n{r02}\n-->\n{MARKER}\n{r02}\n");
        let seen: Vec<usize> = seen_lines(&text).map(|(lineno, _)| lineno).collect();
        assert_eq!(seen, [1, 2]);
        let fenced = format!("{r01}\n```\n{MARKER}\n```\n{r02}\n");
        let ids: Vec<String> = seen_rows(&fenced).into_iter().map(|row| row.id).collect();
        assert_eq!(ids, ["R01", "R02"]);
    }
}
