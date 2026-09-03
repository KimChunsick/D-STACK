// verbs/issue/verdict.rs
// The verdict one fixture earned: how its filings ended, read against what they left (R06).

use std::path::Path;

use crate::core::error::{Error, Result};
use crate::selftest::Verdict;

use super::asked::Asked;
use super::filed::{look, Look};
use super::run::Run;

/// The whole verdict contract: a rejection is every filing exiting 1 with nothing written (D-08),
/// a pass is every filing exiting 0 with the file the fixture asked for behind them (D-06), and
/// every other pairing of endings with a folder is a cannot-decide naming both.
///
/// The pairing that makes this checker worth having is exit 0 with nothing written: a verb that
/// accepted what a bad fixture fed it and wrote no file would, read as a rejection, agree with
/// every bad-* fixture in the directory and keep the run at `failed 0`. It is not a rejection, and
/// it is not a pass either — it is the runner saying what it saw and refusing to judge it.
pub(super) fn verdict(runs: &[Run], dir: &Path, planted: bool, asked: &Asked) -> Result<Verdict> {
    let found = look(dir, asked)?;
    match (code(runs), &found, planted) {
        (Some(0), Look::Asked, false) => Ok(Verdict::Pass),
        (Some(1), Look::Nothing, false) => Ok(Verdict::Reject),
        (Some(1), Look::Planted, true) => Ok(Verdict::Reject),
        _ => Err(Error::cannot_decide(format!(
            "selftest: dstack issue new {} and left {} in {}{}",
            ending(runs),
            found.phrase(),
            dir.display(),
            tail(runs)
        ))),
    }
}

/// The one exit code every filing of the fixture returned, or None when they did not all return
/// the same one.
///
/// A fixture's filings are the same command run again: a good-* fixture is accepted every time,
/// each filing adding its sighting, and a bad-* one is refused every time. Reading the last filing
/// alone would take a set that disagrees — the first refused after writing, the rest accepted —
/// for a clean repeat, because the folder it leaves can be the very folder a clean repeat leaves.
/// So a set that does not agree decides nothing, whichever way its last filing went.
fn code(runs: &[Run]) -> Option<i32> {
    let first = runs.first()?.code?;
    match runs.iter().all(|run| run.code == Some(first)) {
        true => Some(first),
        false => None,
    }
}

/// How the filings ended, in the words the row carries: the one ending they share, or each of
/// them in turn when they do not share one.
fn ending(runs: &[Run]) -> String {
    let first = match runs.first() {
        Some(first) => first,
        None => return "was never run".to_string(),
    };
    if runs.iter().all(|run| run.ended == first.ended) {
        return first.ended.clone();
    }
    let each: Vec<String> = runs
        .iter()
        .enumerate()
        .map(|(at, run)| format!("filing {} {}", at + 1, run.ended))
        .collect();
    format!("did not end the same way each time ({})", each.join(", "))
}

/// The line the row carries, which is all a reader has to act on.
///
/// When the filings agree it is the last line any of them printed. When they disagree it is the
/// line of the filing that did not do what the others did, named by its place in the repeat: the
/// refusal is what has to be acted on, and it is never the last line — the filings that went on
/// working print over it. Carrying the odd one is chosen over listing every line because the row
/// is one line of a table, and the endings of all the filings are already in the sentence.
fn tail(runs: &[Run]) -> String {
    let spoke = |run: &Run| !run.said.is_empty();
    if runs.iter().all(|run| run.ended == runs[0].ended) {
        return match runs.iter().rev().find(|run| spoke(run)) {
            Some(run) => format!(": {}", run.said),
            None => String::new(),
        };
    }
    // The filing whose ending the fewest of them shared, and the first of those when several
    // share the fewest; a filing that printed nothing hands the row the last line there is.
    let shared = |run: &Run| runs.iter().filter(|other| other.ended == run.ended).count();
    let odd = runs
        .iter()
        .enumerate()
        .filter(|(_, run)| spoke(run))
        .min_by_key(|(at, run)| (shared(run), *at));
    match odd {
        Some((at, run)) => format!(": filing {} said {}", at + 1, run.said),
        None => String::new(),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::selftest::sandbox::Sandbox;
    use crate::verbs::issue::asked::Asked;
    use crate::verbs::issue::file::{render, Filing, Sighting};
    use crate::verbs::issue::filed::{issues, plant};
    use crate::verbs::issue::slug::slug;

    const TITLE: &str = "plan start refuses a file worktree";

    fn asked(runs: u32) -> Asked {
        Asked {
            filing: Filing {
                title: TITLE.to_string(),
                symptom: "it exits 1".to_string(),
                repro: "dstack plan start P4".to_string(),
                source: "lifecycle.rs".to_string(),
                proposal: String::new(),
            },
            runs,
        }
    }

    /// Filings that all ended with the code given, having said what a refusal says.
    fn ran(codes: &[i32]) -> Vec<Run> {
        codes
            .iter()
            .map(|code| Run {
                code: Some(*code),
                ended: format!("exited {code}"),
                said: "dstack: --symptom is required".to_string(),
            })
            .collect()
    }

    /// The file `issue new` writes for the filing, with one sighting per run.
    fn file(dir: &Path, runs: u32) {
        let mut text = render(
            &asked(runs).filing,
            &Sighting {
                stamp: "2026-09-03T05:10:22Z".to_string(),
                run: "r".to_string(),
                plan: "P2".to_string(),
            },
        );
        for _ in 1..runs {
            text = crate::verbs::issue::file::append(
                &text,
                &Sighting {
                    stamp: "2026-09-03T06:02:41Z".to_string(),
                    run: "r".to_string(),
                    plan: "P2".to_string(),
                },
            )
            .expect("a sighting")
            .0;
        }
        std::fs::create_dir_all(dir).expect("the folder");
        std::fs::write(dir.join(format!("{}.md", slug(TITLE))), text).expect("the filed issue");
    }

    /// The defect this checker exists to catch: a verb that accepts what a bad fixture feeds it,
    /// writes nothing and exits 0. Read as a rejection it would agree with every bad-* fixture in
    /// the directory, and the run would still close at `failed 0`.
    #[test]
    fn r06__accepting_a_bad_filing_without_writing_is_not_a_rejection() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let dir = issues(&sandbox);
        assert_eq!(
            verdict(&ran(&[1]), &dir, false, &asked(1)).expect("reject"),
            Verdict::Reject
        );
        let cannot = verdict(&ran(&[0]), &dir, false, &asked(1)).expect_err("cannot decide");
        assert_eq!(cannot.code(), 2);
        assert_eq!(
            cannot.message(),
            format!(
                "selftest: dstack issue new exited 0 and left nothing in {}\
                 : dstack: --symptom is required",
                dir.display()
            )
        );
        // A child that printed nothing at all leaves the message with the exit code alone.
        let silent = vec![Run {
            code: Some(2),
            ended: "exited 2".to_string(),
            said: String::new(),
        }];
        assert_eq!(
            verdict(&silent, &dir, false, &asked(1))
                .expect_err("cannot decide")
                .message(),
            format!(
                "selftest: dstack issue new exited 2 and left nothing in {}",
                dir.display()
            )
        );
    }

    /// A pass is the file the fixture asked for, and nothing less: the sightings count alone
    /// would let a filing mishandle every field it was given and still land on a passing row.
    #[test]
    fn r06__a_pass_is_the_file_the_fixture_asked_for() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let dir = issues(&sandbox);
        file(&dir, 3);
        assert_eq!(
            verdict(&ran(&[0, 0, 0]), &dir, false, &asked(3)).expect("pass"),
            Verdict::Pass
        );
        // The same folder judged against two filings: the count is a mismatch of its own.
        assert_eq!(
            verdict(&ran(&[0, 0]), &dir, false, &asked(2))
                .expect_err("cannot decide")
                .message(),
            format!(
                "selftest: dstack issue new exited 0 and left a file whose frontmatter sightings \
                 is \"3\", not \"2\" in {}: dstack: --symptom is required",
                dir.display()
            )
        );
        // A file carrying the right count and nothing else the fixture asked for is not the file
        // it asked for.
        std::fs::write(
            dir.join(format!("{}.md", slug(TITLE))),
            "---\ntitle: t\nfirst_seen: s\nsightings: 3\n---\n",
        )
        .expect("a stub");
        let cannot = verdict(&ran(&[0, 0, 0]), &dir, false, &asked(3)).expect_err("cannot decide");
        assert!(
            cannot
                .message()
                .contains("whose frontmatter title is \"t\""),
            "{}",
            cannot.message()
        );
    }

    /// Every filing is judged, not only the last one: a repeat whose first filing was refused
    /// after writing leaves the folder a clean repeat leaves.
    #[test]
    fn r06__filings_that_did_not_all_end_the_same_way_are_no_verdict() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let dir = issues(&sandbox);
        file(&dir, 3);
        assert_eq!(
            verdict(&ran(&[1, 0, 0]), &dir, false, &asked(3))
                .expect_err("cannot decide")
                .message(),
            format!(
                "selftest: dstack issue new did not end the same way each time (filing 1 exited 1, \
                 filing 2 exited 0, filing 3 exited 0) and left the file the fixture asked for \
                 in {}: filing 1 said dstack: --symptom is required",
                dir.display()
            )
        );
        // A refusal that left the planted file alone is a rejection; one filing of it accepted
        // is not.
        let alone = Sandbox::scratch().expect("scratch repository");
        let planted = issues(&alone);
        plant(&planted, &slug(TITLE)).expect("plant");
        assert_eq!(
            verdict(&ran(&[1, 1]), &planted, true, &asked(2)).expect("reject"),
            Verdict::Reject
        );
        assert_eq!(
            verdict(&ran(&[1, 0]), &planted, true, &asked(2))
                .expect_err("cannot decide")
                .code(),
            2
        );
        // Nothing ran at all: there is no ending to read.
        assert!(verdict(&[], &planted, true, &asked(0))
            .expect_err("cannot decide")
            .message()
            .contains("was never run"));
    }

    /// A filing that returned no exit code at all — a signal ended it — decides nothing, and
    /// the row says how it ended. Run::from_status is what reads that off a status (run.rs).
    #[test]
    fn r06__a_filing_that_returned_no_code_decides_nothing() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let dir = issues(&sandbox);
        let killed = vec![Run {
            code: None,
            ended: "was terminated by signal 9".to_string(),
            said: String::new(),
        }];
        assert_eq!(
            verdict(&killed, &dir, false, &asked(1))
                .expect_err("cannot decide")
                .message(),
            format!(
                "selftest: dstack issue new was terminated by signal 9 and left nothing in {}",
                dir.display()
            )
        );
    }

    /// The line worth showing on a row whose filings disagree is the one from the filing that
    /// did not do what the others did: the refusal, not the success that followed it.
    #[test]
    fn r06__a_disagreeing_row_carries_the_odd_filings_line() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let dir = issues(&sandbox);
        file(&dir, 3);
        let said = |code: i32, said: &str| Run {
            code: Some(code),
            ended: format!("exited {code}"),
            said: said.to_string(),
        };
        let runs = vec![
            said(1, "dstack: the first filing was refused"),
            said(0, "  sighting 2  2026-09-03T06:02:41Z  run r  plan P2"),
            said(0, "  sighting 3  2026-09-03T06:02:42Z  run r  plan P2"),
        ];
        let cannot = verdict(&runs, &dir, false, &asked(3)).expect_err("cannot decide");
        assert!(
            cannot
                .message()
                .ends_with(": filing 1 said dstack: the first filing was refused"),
            "{}",
            cannot.message()
        );
    }
}
