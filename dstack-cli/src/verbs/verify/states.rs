// verbs/verify/states.rs
// The verdict engine `dstack verify` and `dstack report` share: the policy ceiling (R75), the
// per-field evidence rule (R74), the sha256 recheck and the branch containment (R38).
//
// A state is computed here and nowhere else, so verify and report can never recompute a status
// two ways (R79); both commands only print what states::of returned.

use std::path::{Path, PathBuf};

use crate::core::error::Result;
use crate::core::fsx::{read_text, sha256_file};
use crate::core::meta::meta_get;
use crate::core::roots::git_out;
use crate::core::target::TargetKind;
use crate::core::tools::policy_get;
use crate::store::cases::{self, CaseRow};
use crate::store::request::RequestDoc;
use crate::store::review_index::closed_rows;
use crate::store::tsv::undash;

/// The evidence kinds an e2e field of cli or capture accepts (VERIFY_KINDS_E2E).
pub const KINDS_E2E: [&str; 3] = ["cli", "capture", "transcript"];

/// The statuses that count as recorded evidence; open, skipped, unreported and retired do not.
const RECORDED: [&str; 3] = ["met", "abstain", "blocked"];


/// One line of _verify_states: the R, its state and the reason tokens, which report maps to check
/// names so both commands name the same failure the same way.
pub struct RState {
    pub r: String,
    pub state: &'static str,
    pub reasons: String,
}

/// The latest sealed verdict for one R (R69) with the round that holds it.
///
/// Rounds are `codex-review-NNN.md` and NNN is zero-padded, so the glob's own order is round
/// order; the last file that mentions the R wins, because a later round supersedes an older
/// "absent". None when no round mentions it.
pub fn review_verdict_for(dir: &Path, r: &str) -> Result<Option<(String, String)>> {
    let mut found = None;
    for file in rounds(dir) {
        // A sealed round the glob just listed and that cannot be read is a cannot-decide (D-12):
        // reading it as no verdict would turn a broken round into "review: none".
        let text = match read_text(&file)? {
            Some(text) => text,
            None => continue,
        };
        let mut last = None;
        for line in text.lines() {
            let cells: Vec<&str> = line.split('|').collect();
            if cells.len() < 3 {
                continue;
            }
            let trim = |cell: &str| cell.trim_matches([' ', '\t']).to_string();
            let verdict = trim(cells[2]);
            if trim(cells[1]) == r && matches!(verdict.as_str(), "covered" | "partial" | "absent") {
                last = Some(verdict);
            }
        }
        if let Some(verdict) = last {
            found = Some((verdict, round_of(&file)));
        }
    }
    Ok(found)
}

/// `dir/review/codex-review-*.md`, in the glob's own order.
fn rounds(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir.join("review"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            name.starts_with("codex-review-") && name.ends_with(".md")
        })
        .collect();
    files.sort();
    files
}

fn round_of(file: &Path) -> String {
    let name = file.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let round = name.strip_prefix("codex-review-").unwrap_or(&name);
    round.strip_suffix(".md").unwrap_or(round).to_string()
}

/// `dstack review close` records "after round N: why" per R in review/closed.tsv; this is the
/// round and the reason of the latest close naming this R. A close outranks every sealed round up
/// to and including N: the reviewer's last word on that R was never re-verified, so the R reads
/// ABSTAIN (owner-accepted, R79), never MET. A round sealed AFTER the close wins again.
pub fn review_closed_for(dir: &Path, r: &str) -> Result<Option<(String, String)>> {
    let mut last = None;
    for row in closed_rows(dir)? {
        if row.ids.split(',').any(|id| id == r) {
            last = Some((row.round, row.why));
        }
    }
    Ok(last)
}

fn e2e_rank(word: &str) -> i32 {
    match word {
        "none" => 0,
        "cli" => 1,
        "capture" => 2,
        _ => -1,
    }
}

/// One line per violation, empty when clean. The repository policy is the ceiling, a request may
/// only narrow it, and the policy's own `why` line is printed with the refusal so the reader
/// learns the reason, not just the rule.
pub fn policy_violations(store: &Path, doc: &RequestDoc) -> Vec<String> {
    let policy = |key: &str| policy_get(store, key).unwrap_or_default();
    let field = |key: &str| doc.field(key).unwrap_or_default();
    let (cap, vdiff, surfaces) = (policy("e2e_evidence"), policy("visual_diff"), policy("surfaces"));
    let (work_type, e2e, visual) = (field("work_type"), field("e2e"), field("visual"));
    let mut bad: Vec<String> = Vec::new();
    if !cap.is_empty() && e2e_rank(&e2e) > e2e_rank(&cap) {
        bad.push(format!("request e2e: {e2e} exceeds policy e2e_evidence: {cap}"));
    }
    if vdiff == "forbidden" && !visual.is_empty() && visual != "none" {
        bad.push(format!("request visual: {visual} but policy visual_diff: forbidden"));
    }
    if !surfaces.is_empty()
        && !surfaces.replace(',', " ").split_whitespace().any(|surface| surface == work_type)
    {
        bad.push(format!("request work_type: {work_type} is not a surface this repository verifies (policy surfaces: {surfaces})"));
    }
    if !bad.is_empty() {
        let why = policy("why");
        let why = match why.is_empty() {
            true => "(no why line in the policy block)".to_string(),
            false => why,
        };
        bad.push(format!("policy why: {why}"));
    }
    bad
}

/// `$(policy_get <key> || echo -)`: a store without a PROJECT.md prints the dash the shell's `||`
/// branch prints, while a policy block without the key prints the empty value awk found.
/// The state of every live R of this request (_verify_states).
pub fn of(dir: &Path, main_root: &Path, kind: TargetKind, polfail: bool) -> Result<Vec<RState>> {
    let doc = RequestDoc::load(&dir.join("request.md"))?;
    let field = |key: &str| doc.field(key).unwrap_or_default();
    let (e2e, tests) = (field("e2e"), field("unit_tests"));
    let (visual, review) = (field("visual"), field("review"));
    let ledger = cases::rows(dir)?;
    let mut out: Vec<RState> = Vec::new();
    for r in doc.live_ids() {
        let rows: Vec<&CaseRow> = ledger.iter().filter(|row| row.r == r).collect();
        let mut reasons: Vec<String> = Vec::new();
        if polfail {
            reasons.push("policy-ceiling".to_string());
        }
        if (e2e == "cli" || e2e == "capture") && !has_kind(&rows, &KINDS_E2E) {
            reasons.push(format!("evidence:e2e={e2e}"));
        }
        if tests == "on" && !has_kind(&rows, &["test"]) {
            reasons.push("evidence:unit_tests".to_string());
        }
        if !visual.is_empty() && visual != "none" && !has_kind(&rows, &["visual"]) {
            reasons.push(format!("evidence:visual={visual}"));
        }
        // A recorded artifact that no longer hashes to what was recorded is the ledger being
        // edited by hand; R74 makes that a failure of the R it was recorded against.
        reasons.extend(sha_bad(&rows, main_root));
        // R68: a worker's silence about an R stays a failure until the main session records real
        // evidence into that row (evidence add fills an unreported row as it fills an open one).
        let silent: Vec<&str> = rows
            .iter()
            .filter(|row| row.status == "unreported")
            .map(|row| row.case_id.as_str())
            .collect();
        if !silent.is_empty() {
            reasons.push(format!("unreported:{}", silent.join(",")));
        }

        let (mut verdict, mut round, mut closed) = (String::new(), String::new(), String::new());
        // R69/R79: the per-R review verdict is part of MET whatever `review` says; the field only
        // tunes rounds and axes. The quick track is the one place review: off skips it (R99).
        if review == "on" || kind != TargetKind::Quick {
            match review_verdict_for(dir, &r)? {
                Some((found, at)) => {
                    if found == "partial" || found == "absent" {
                        reasons.push(format!("review:{found}:{at}"));
                    }
                    verdict = found;
                    round = at;
                }
                None => verdict = "none".to_string(),
            }
            // A closed review covers this R when no round newer than the close sealed a verdict
            // for it. It never erases a partial/absent verdict, only an unverified covered one or
            // the absence of a round.
            if let Some((at, why)) = review_closed_for(dir, &r)? {
                if (verdict == "none" || verdict == "covered") && round_le(&round, &at) {
                    closed = format!("review closed after round {at}: {why}");
                }
            }
        }

        let first = |status: &str| rows.iter().find(|row| row.status == status);
        let (state, reasons) = if !reasons.is_empty() {
            ("FAIL", reasons.join(";"))
        } else if first("met").is_some() && verdict != "none" && closed.is_empty() {
            // A met row decides the R; an earlier abstain or blocked attempt on another case is
            // history.
            ("PASS", String::new())
        } else if let Some(row) = first("blocked") {
            ("BLOCKED", format!("blocked:{}", undash(&row.note)))
        } else if let Some(row) = first("abstain") {
            ("ABSTAIN", format!("abstain:{}", undash(&row.note)))
        } else if !closed.is_empty() {
            ("ABSTAIN", format!("abstain:{closed}"))
        } else if verdict == "none" {
            // review: on with nothing sealed is not a failure and not a pass: nobody judged it.
            ("ABSTAIN", "abstain:no sealed review round".to_string())
        } else {
            ("PASS", String::new())
        };
        out.push(RState { r, state, reasons });
    }
    Ok(out)
}

fn has_kind(rows: &[&CaseRow], kinds: &[&str]) -> bool {
    rows.iter()
        .any(|row| kinds.contains(&row.kind.as_str()) && RECORDED.contains(&row.status.as_str()))
}

/// `[ "${round:-000}" -le "${cround:-000}" ] 2>/dev/null`: a round that is not a number makes the
/// test itself fail, which is a false, never a close that covers everything.
fn round_le(round: &str, closed_at: &str) -> bool {
    let number = |text: &str| match text.is_empty() {
        true => Some(0),
        false => text.parse::<i64>().ok(),
    };
    matches!((number(round), number(closed_at)), (Some(a), Some(b)) if a <= b)
}

/// Case ids whose artifact no longer matches its recorded sha256, or is gone.
fn sha_bad(rows: &[&CaseRow], main_root: &Path) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for row in rows {
        if matches!(row.status.as_str(), "open" | "retired" | "unreported") {
            continue;
        }
        if row.artifact.is_empty() || row.artifact == "-" {
            continue;
        }
        let path = match row.artifact.starts_with('/') {
            true => PathBuf::from(&row.artifact),
            false => main_root.join(&row.artifact),
        };
        if !path.is_file() {
            out.push(format!("artifact-missing:{}", row.case_id));
            continue;
        }
        if sha256_file(&path).ok().as_deref() != Some(row.sha256.as_str()) {
            out.push(format!("sha256:{}", row.case_id));
        }
    }
    out
}

/// The containment line and whether the Goal branch holds its base (R38).
pub fn branch_line(dir: &Path, wt_root: &Path) -> Result<(String, bool)> {
    let meta = |key: &str| -> Result<String> { Ok(meta_get(dir, key)?.unwrap_or_default()) };
    let (branch, base) = (meta("branch")?, meta("base_branch")?);
    if branch.is_empty() || base.is_empty() {
        return Ok(("branch containment: no branch recorded in meta.tsv — skipped".to_string(), true));
    }
    if branch == base {
        return Ok((format!("branch containment: branch = base ({branch}), nothing to contain"), true));
    }
    let recorded = PathBuf::from(meta("worktree")?);
    let wt = match recorded.is_dir() {
        true => recorded,
        false => wt_root.to_path_buf(),
    };
    let resolves = |name: &str| git_out(Some(&wt), &["rev-parse", "--verify", "-q", name]).is_some();
    if !resolves(&base) {
        return Ok((format!("branch containment: base branch '{base}' is not resolvable in {} — skipped (cannot prove containment against a branch that is gone)", wt.display()), true));
    }
    if !resolves(&branch) {
        return Ok((format!("branch containment: branch '{branch}' is not resolvable in {} — skipped", wt.display()), true));
    }
    if git_out(Some(&wt), &["merge-base", "--is-ancestor", &base, &branch]).is_some() {
        return Ok((format!("branch containment: {branch} contains {base} — ok"), true));
    }
    Ok((format!("branch containment: {branch} does not contain {base} — rebase first: git -C {} rebase {base}", wt.display()), false))
}

/// "$TARGET_KIND": the word every line of verify and report prints next to the target id.
pub fn kind_word(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Run => "run",
        TargetKind::Quick => "quick",
    }
}

/// verify: the fixture is a request.md; `<!-- selftest-tamper: yes -->` makes the driver edit a
/// recorded artifact after recording it, which is exactly the hand-edit the sha256 recheck owes

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_a_close_covers_the_rounds_up_to_its_own_number() {
        assert!(round_le("", "000"));
        assert!(round_le("001", "001"));
        assert!(!round_le("002", "001"));
        assert!(!round_le("draft", "001"));
    }
}
