// verbs/report/qa.rs
// The QA table of dstack report (R08, D-41): every QA scenario of a Goal run and every usage
// scenario none covers, with the reason a failed, skipped or blocked result was recorded with.
//
// The states come from verify::qa_states, the evaluation verify prints, so the report refuses a
// Goal close exactly when verify does: its exit is 1 whenever that check refuses.

use crate::core::context::Context;
use crate::verbs::verify::qa_states::QaCheck;

/// Print the table and its `qa:` line; true when the report has to exit 1 for it.
pub(super) fn table(ctx: &mut Context, check: &QaCheck) -> bool {
    ctx.out.say("");
    ctx.out.say("| QA | scenario | status | reason | artifact |");
    ctx.out.say("|---|---|---|---|---|");
    for state in &check.states {
        let cells = [
            state.qa.clone().unwrap_or_else(|| "-".to_string()),
            state.scenario.clone(),
            state.status.clone(),
            state.reason(),
            state.artifact.clone(),
        ];
        let cells: Vec<String> = cells.iter().map(|cell| cell.replace('|', "\\|")).collect();
        ctx.out.say(&format!("| {} |", cells.join(" | ")));
    }
    ctx.out.say(&check.summary());
    check.refused()
}
