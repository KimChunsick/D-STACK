// store/request_part3_qa.rs
// The QA section of part 3 (D-40): every Goal QA scenario of the run-local ledger with its usage
// scenario, status and the first line of its text, rendered under the plan on every regeneration.

use crate::store::qa::{QaEntry, NO_SCENARIO};
use crate::store::request_part3::{code, inert, item};
use crate::store::visible::visible;

/// The section heading; no Milestone heading repeats it, since each one starts with its M id.
const HEADING: &str = "QA 시나리오";

/// What a scenario whose text is gone or cannot be read shows in place of its first line.
const UNREADABLE: &str = "(내용 파일을 읽을 수 없어요)";

/// The QA section part 3 ends with: each ledger row with its text (None when the text cannot be
/// read) in ledger order, or the placeholder when there is none yet. Every value follows a label
/// or an id and is made inert, so none opens a comment.
pub fn render(entries: &[QaEntry]) -> String {
    let mut out = format!("\n## {HEADING}\n\n");
    if entries.is_empty() {
        item(&mut out, "", "QA", "");
    }
    for (row, text) in entries {
        let scenario = match row.scenario.as_str() {
            NO_SCENARIO => "사용 시나리오 없음".to_string(),
            id => format!("사용 시나리오 {}", inert(id)),
        };
        out.push_str(&format!("- {}: {scenario}\n", inert(&row.qa)));
        item(&mut out, "  ", "상태", &code(&row.status));
        let first = text.as_deref().map_or(UNREADABLE.to_string(), first_line);
        item(&mut out, "  ", "내용 첫 줄", &first);
    }
    out
}

/// The first line of a text that shows: comments hidden, blank lines skipped, and any control
/// character a space, so the line keeps the request's outline readable.
fn first_line(text: &str) -> String {
    let shown = visible(text);
    let line = shown.lines().map(str::trim).find(|line| !line.is_empty());
    let line = line.unwrap_or_default();
    line.chars().map(|c| if c.is_control() { ' ' } else { c }).collect()
}
