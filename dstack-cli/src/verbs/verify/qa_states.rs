// verbs/verify/qa_states.rs
// The Goal-close QA check (R08, D-41, D-42, D-46, D-47) dstack verify and dstack report share:
// which targets it holds, whether the Goal is closing, and per QA scenario and uncovered usage
// scenario whether it passes and why not.
//
// Both commands print what `of` returned and nothing else, so they cannot disagree on a refusal;
// only verify --at-close, the close itself, prints what `at_close` returned instead.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::sha256_file;
use crate::core::target::TargetKind;
use crate::store::plan;
use crate::store::qa::{self, QaRow, QA_RESULTS};
use crate::store::request_sections::SECTION_KEYS;
use crate::store::visible::{heading, lines};

/// The results that close a Goal; failed and open never do.
const PASSING: [&str; 3] = ["met", "skipped", "blocked"];

/// The Plan statuses of work still to come; while one is left the Goal is not closing (D-46).
const LEFT: [&str; 3] = ["pending", "ready", "in-progress"];

/// One line of the check: a QA scenario of the ledger, or a usage scenario no QA scenario covers.
pub struct QaState {
    /// The QA id, None for an uncovered usage scenario.
    pub qa: Option<String>,
    /// The usage scenario, `none` for a check tied to none.
    pub scenario: String,
    /// The recorded status, `uncovered` for a usage scenario without a QA scenario.
    pub status: String,
    /// The recorded reason of a failed, skipped or blocked result; None when there is none.
    pub note: Option<String>,
    /// Before Goal close, what an open result or an uncovered usage scenario still needs; it
    /// informs and never refuses.
    pub info: Option<String>,
    /// The artifact cell as recorded, `-` when there is none.
    pub artifact: String,
    /// Why this line refuses the Goal close; empty when it passes.
    pub refusals: Vec<String>,
}

impl QaState {
    /// How verify and report name it: `QA1 (S1)`, or `S2` alone when uncovered.
    pub fn label(&self) -> String {
        match &self.qa {
            Some(qa) => format!("{qa} ({})", self.scenario),
            None => self.scenario.clone(),
        }
    }

    /// The recorded reason first, then what it still needs, then every refusal; `-` when there is
    /// none of them.
    pub fn reason(&self) -> String {
        let notes = self.note.iter().chain(&self.info);
        let parts: Vec<&str> = notes.chain(&self.refusals).map(String::as_str).collect();
        match parts.is_empty() {
            true => "-".to_string(),
            false => parts.join("; "),
        }
    }

    /// The verify line: the status that passes with its reason, or FAIL with every reason.
    pub fn line(&self) -> String {
        match (self.refusals.is_empty(), self.reason()) {
            (true, reason) if reason == "-" => format!("{} {}", self.label(), self.status),
            (true, reason) => format!("{} {}: {reason}", self.label(), self.status),
            (false, reason) => format!("{} FAIL ({}): {reason}", self.label(), self.status),
        }
    }
}

/// The check of one Goal run: its usage scenarios and one state per line.
pub struct QaCheck {
    pub scenarios: usize,
    /// Whether the Goal is closing: no Plan is pending, ready or in progress (D-46), or the close
    /// itself asks (`at_close`, D-47).
    pub closing: bool,
    pub states: Vec<QaState>,
}

impl QaCheck {
    /// Whether the Goal close is refused: one refusing line is enough.
    pub fn refused(&self) -> bool {
        self.states.iter().any(|state| !state.refusals.is_empty())
    }

    /// The `qa:` line both commands end the QA block with. Open and uncovered count every such
    /// line, refused or not; the other results only those that pass.
    pub fn summary(&self) -> String {
        let ledger = self.states.iter().filter(|state| state.qa.is_some());
        let count = |status: &str| ledger.clone().filter(|state| state.status == status && state.refusals.is_empty()).count();
        let all = |status: &str| self.states.iter().filter(|state| state.status == status).count();
        let refused = self.states.iter().filter(|state| !state.refusals.is_empty()).count();
        format!(
            "qa: usage scenarios {}, QA scenarios {}, met {}, skipped {}, blocked {}, open {}, uncovered {}, refused {refused}, {} → {}",
            self.scenarios,
            ledger.clone().count(),
            count("met"),
            count("skipped"),
            count("blocked"),
            all("open"),
            all("uncovered"),
            match self.closing {
                true => "at Goal close",
                false => "before Goal close",
            },
            match self.refused() {
                true => "refused",
                false => "ok",
            }
        )
    }
}

/// The usage scenarios a target's Goal close is held to (D-41). A quick task and a request with
/// no visible `## 사용 시나리오` heading — every request written before the usage scenario
/// section — hold none, so they are exempt. A request whose section names no `### S<n>` holds
/// none either; one whose section cannot be read refuses, naming the file, as qa add does.
pub fn usage_ids(kind: TargetKind, request: &Path, text: &str) -> Result<Vec<String>> {
    let section = SECTION_KEYS.iter().find(|(key, _)| *key == "scenarios").map(|(_, name)| *name);
    let held = lines(text).iter().any(|line| match heading(line.raw.trim_end_matches('\r')) {
        Some((2, name)) => line.heading && Some(name) == section,
        _ => false,
    });
    if kind == TargetKind::Quick || !held {
        return Ok(Vec::new());
    }
    qa::scenario_ids(text).map_err(|e| Error::failed(format!("{}: {}", request.display(), e.message())))
}

/// The check of the target in `dir`, None when it is exempt; whether the Goal is closing is read
/// from its Plans (D-46).
pub fn of(dir: &Path, main_root: &Path, kind: TargetKind, text: &str) -> Result<Option<QaCheck>> {
    check(dir, main_root, kind, text, false)
}

/// The same check held to the Goal close whatever Plans are left, as closing the Goal itself needs
/// it (verify --at-close, D-47): an open result and an uncovered usage scenario refuse.
pub fn at_close(dir: &Path, main_root: &Path, kind: TargetKind, text: &str) -> Result<Option<QaCheck>> {
    check(dir, main_root, kind, text, true)
}

/// The run's QA ledger, every file it names and, unless the close is `forced`, plan.json are read
/// here, once; `judge` decides from what was read.
fn check(dir: &Path, main_root: &Path, kind: TargetKind, text: &str, forced: bool) -> Result<Option<QaCheck>> {
    let ids = usage_ids(kind, &dir.join("request.md"), text)?;
    if ids.is_empty() {
        return Ok(None);
    }
    let closing = forced || closing(dir)?;
    let rows = qa::rows(dir)?;
    let files: Vec<Vec<String>> = rows.iter().map(|row| file_refusals(dir, main_root, row)).collect();
    Ok(Some(judge(&ids, &rows, files, closing)))
}

/// D-46: the Goal is closing once no Plan is left — none pending, ready or in progress, or no
/// plan.json at all. A plan.json that cannot be read refuses, since guessing either way would
/// pass an open result at Goal close or refuse a milestone close for it.
fn closing(dir: &Path) -> Result<bool> {
    if !plan::exists(dir) {
        return Ok(true);
    }
    let doc = plan::load(dir)?;
    Ok(!doc.plans.iter().any(|plan| LEFT.contains(&plan.status.as_str())))
}

/// What an open result or an uncovered usage scenario needs: a refusal at Goal close, information
/// before it.
fn pending(closing: bool, need: &str, fix: String) -> (Option<String>, Vec<String>) {
    match closing {
        true => (None, vec![format!("{need} — {fix}")]),
        false => (Some(format!("{need} yet — checked at Goal close; {fix}")), Vec::new()),
    }
}

/// The per-line states from the usage scenario ids, the ledger rows, each row's file refusals and
/// whether the Goal is closing.
fn judge(ids: &[String], rows: &[QaRow], files: Vec<Vec<String>>, closing: bool) -> QaCheck {
    let mut states = Vec::new();
    for (row, files) in rows.iter().zip(files) {
        let mut refusals = Vec::new();
        let mut info = None;
        match row.status.as_str() {
            "open" => {
                let fix = format!("record it: dstack evidence add --qa {} --artifact <file> --produced-by <command>", row.qa);
                (info, refusals) = pending(closing, "no recorded result", fix);
            }
            "failed" => refusals.push("a failed result does not close the Goal".to_string()),
            status if PASSING.contains(&status) => {}
            status => refusals.push(format!("status {status} is no QA result (open, {})", QA_RESULTS.join(", "))),
        }
        refusals.extend(files);
        let noted = row.status != "met" && row.status != "open" && !matches!(row.note.as_str(), "" | "-");
        states.push(QaState {
            qa: Some(row.qa.clone()),
            scenario: row.scenario.clone(),
            status: row.status.clone(),
            note: noted.then(|| row.note.clone()),
            info,
            artifact: row.artifact.clone(),
            refusals,
        });
    }
    for id in ids.iter().filter(|id| !rows.iter().any(|row| row.scenario == **id)) {
        let fix = format!("add one: dstack qa add --scenario {id} --from <file>");
        let (info, refusals) = pending(closing, "no QA scenario", fix);
        states.push(QaState {
            qa: None,
            scenario: id.clone(),
            status: "uncovered".to_string(),
            note: None,
            info,
            artifact: "-".to_string(),
            refusals,
        });
    }
    QaCheck { scenarios: ids.len(), closing, states }
}

/// D-42: the text of every QA scenario, and the artifact of every recorded result, still hash to
/// what was recorded. Each refusal names the file.
fn file_refusals(dir: &Path, main_root: &Path, row: &QaRow) -> Vec<String> {
    let mut out = Vec::new();
    out.extend(unchanged("text", &dir.join(&row.body), &row.body_sha256));
    if row.status != "open" {
        match row.artifact.as_str() {
            "" | "-" => out.push("the recorded result names no artifact".to_string()),
            artifact => {
                let path = match artifact.starts_with('/') {
                    true => PathBuf::from(artifact),
                    false => main_root.join(artifact),
                };
                out.extend(unchanged("artifact", &path, &row.artifact_sha256));
            }
        }
    }
    out
}

/// None when `path` is a file hashing to `sha256`; otherwise why not, naming the path.
fn unchanged(what: &str, path: &Path, sha256: &str) -> Option<String> {
    if !path.is_file() {
        return Some(format!("{what} {} is gone", path.display()));
    }
    match sha256_file(path).ok().as_deref() == Some(sha256) {
        true => None,
        false => Some(format!("{what} {} changed after it was recorded (sha256 mismatch)", path.display())),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn R08_qa_close_only_a_goal_request_with_usage_scenarios_is_held() {
        let request = Path::new("request.md");
        let legacy = "---\nwork_type: cli\n---\n# 예전 요청\n\n요약이에요.\n\n## 요구사항\n\n- [ ] **R01** 하나예요. — accept: 확인해요.\n";
        assert!(qa::scenario_ids(legacy).is_err(), "the store reader refuses a request without the section");
        assert_eq!(usage_ids(TargetKind::Run, request, legacy).expect("exempt"), Vec::<String>::new());
        let hidden = "# 요청\n\n<!--\n## 사용 시나리오\n\n### S1 숨어요\n-->\n";
        assert_eq!(usage_ids(TargetKind::Run, request, hidden).expect("exempt"), Vec::<String>::new());
        let held = "# 요청\n\n요약이에요.\n\n## 사용 시나리오\n\n### S1 써요\n\n### S2 봐요\n\n## 요구사항\n";
        assert_eq!(usage_ids(TargetKind::Run, request, held).expect("held"), ["S1", "S2"]);
        assert_eq!(usage_ids(TargetKind::Quick, request, held).expect("exempt"), Vec::<String>::new());
        let empty = "# 요청\n\n요약이에요.\n\n## 사용 시나리오\n\n<!-- 시나리오를 적어요. -->\n\n## 요구사항\n";
        assert_eq!(usage_ids(TargetKind::Run, request, empty).expect("none named"), Vec::<String>::new());
        let twice = "# 요청\n\n요약이에요.\n\n## 사용 시나리오\n\n### S1 써요\n\n## 사용 시나리오\n";
        let refused = usage_ids(TargetKind::Run, request, twice).expect_err("unreadable");
        assert!(refused.message().starts_with("request.md: "), "{}", refused.message());
    }
}
