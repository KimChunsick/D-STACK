// verbs/issue/mod.rs
// dstack issue: the friction a worker hit with dstack itself, filed as one file per issue.
//
// The folder is $HOME/Documents/dstack-issues and nothing configures it (D-05): what a worker
// files is the maintainer's inbox, not run state, so it lives outside the store and outlives every
// run. That is also why `issue` is on NO_ROOTS — a worker files what it hit wherever it is, and a
// repository without a store is not a reason to lose the report. The run and the plan are read
// from the store when there is one, under the one rule origin() states, and are "-" otherwise.

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::meta::meta_get;
use crate::core::paths::base_name;
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::plan::{self, PlanDoc};

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(crate::core::error::Error::failed(format!($($m)*))) }; }

/// One roster entry; the struct carries nothing but its name. Declared before the modules that
/// use it, because a macro_rules! is in scope only for what follows it.
macro_rules! issue_verb {
    ($handler:ident, $entry:literal, $body:path) => {
        pub(super) struct $handler;
        impl crate::core::verb::Verb for $handler {
            fn name(&self) -> &'static str {
                $entry
            }
            fn run(
                &self,
                ctx: &mut crate::core::context::Context,
                args: &[String],
            ) -> crate::core::error::Result<()> {
                $body(ctx, args)
            }
        }
    };
}

mod asked;
pub mod file;
mod filed;
pub mod list;
pub mod new;
mod run;
mod selftests;
pub mod slug;
mod verdict;

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(new::IssueNew), Box::new(list::IssueList)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    selftests::all()
}

/// The folder D-05 fixes: $HOME/Documents/dstack-issues, with no setting and no override. A test
/// or a sandbox points HOME at a scratch directory, which is the only way this path moves.
fn folder() -> Result<PathBuf> {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() => Ok(PathBuf::from(home).join("Documents/dstack-issues")),
        _ => Err(Error::cannot_decide(
            "HOME is not set, so there is no ~/Documents/dstack-issues to file into",
        )),
    }
}

/// Where a filing came from, as the pair the sighting line carries. Nothing here is worth losing a
/// report over: a directory that is not a repository, a store that is not there and a CURRENT that
/// cannot be read all answer "-", the same way a filing outside any Goal does.
///
/// One rule decides the plan, wherever the filing came from: EXACTLY ONE plan, IN PROGRESS, in an
/// OPEN run, recording THIS worktree. Every part of it earns its place, because every relaxation
/// of it has already put a wrong id on a sighting:
///
/// - exactly one, never the first match — `plan start` takes a path that already exists, so two
///   plans can record one directory, in one run (R30 allows several at once) or across runs. There
///   is nothing to choose between them, and choosing anyway is how an arbitrary id gets stamped.
/// - in progress — a plan record outlives the work. `plan done` leaves the worktree in the record
///   and the next plan may start in that same directory.
/// - in an open run — `run close` writes the run's status and never touches its plans, so a run
///   abandoned last month keeps a plan that still says in-progress and still names the directory.
///   Without this the abandoned plan takes the sighting, and once a live plan uses the checkout
///   too, the stale match makes the pair ambiguous and loses both fields.
///
/// The run is the one part that has a second source. CURRENT is the checkout's own statement of
/// which Goal it is working on, so when it is there the run is known whatever the plans say. With
/// no CURRENT the plan record is the only thread back to a run, and a run kept after its plan was
/// rejected as too stale to trust would be that same record with the field that showed why it
/// could not be trusted removed: "run <a Goal from March>, plan -" reads as if this friction came
/// out of that Goal, where "run -" says what is true.
fn origin(ctx: &mut Context) -> (String, String) {
    let roots = match ctx.roots() {
        Ok(roots) => roots,
        Err(_) => return unknown(),
    };
    let wt = std::fs::canonicalize(&roots.wt_root).unwrap_or_else(|_| roots.wt_root.clone());
    if let Ok(Some(run)) = roots.current_run_id() {
        let dir = roots.runs.join(&run);
        // The run is CURRENT's, but the plan still has to earn its id under the same rule — and a
        // CURRENT left pointing at a run that has been closed proves nothing about a plan.
        let plan = match is_open(&dir) {
            true => one_of(running_here(&dir, &wt)),
            false => "-".to_string(),
        };
        return (run, plan);
    }
    by_worktree(&roots.runs, &wt)
}

fn unknown() -> (String, String) {
    ("-".to_string(), "-".to_string())
}

/// The one candidate, or "-" when there is none and when there is more than one.
fn one_of(mut candidates: Vec<String>) -> String {
    match candidates.len() {
        1 => candidates.remove(0),
        _ => "-".to_string(),
    }
}

/// meta.tsv says the run is open. A status that cannot be read is not an open run: the whole point
/// of the check is that a run which stopped is not where this filing came from, and a run that
/// cannot say is a run that cannot say so.
fn is_open(run_dir: &Path) -> bool {
    matches!(meta_get(run_dir, "status"), Ok(Some(status)) if status == "open")
}

/// The ids of every plan of this run that is in progress in this worktree — usually none or one.
fn running_here(run_dir: &Path, wt: &Path) -> Vec<String> {
    let doc = match plan::exists(run_dir) {
        true => plan::load(run_dir),
        false => return Vec::new(),
    };
    match doc {
        Ok(doc) => plans_here(&doc, wt),
        Err(_) => Vec::new(),
    }
}

/// The same reading over a document already in hand, so the rule has one implementation.
fn plans_here(doc: &PlanDoc, wt: &Path) -> Vec<String> {
    doc.plans
        .iter()
        .filter(|p| p.status == "in-progress" && Path::new(&p.worktree) == wt)
        .map(|p| p.id.clone())
        .collect()
}

/// R36: a worker runs in a worktree dstack made for one plan, and that worktree carries no CURRENT
/// of its own — CURRENT belongs to the checkout that opened the run. Filing from there would name
/// no run at all, and that is where most issues are filed, so the run and the plan are read from
/// the plan record that names this worktree, under the rule origin() states in full.
///
/// The candidates of every open run go into one list before it is counted, so an ambiguity inside
/// a run and an ambiguity across runs are the same answer. A run that is not open contributes
/// nothing at all — not a candidate, and so not an ambiguity either.
fn by_worktree(runs: &Path, wt: &Path) -> (String, String) {
    let mut found: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(runs).into_iter().flatten().flatten() {
        let dir = entry.path();
        if !is_open(&dir) {
            continue;
        }
        found.extend(
            running_here(&dir, wt)
                .into_iter()
                .map(|id| (base_name(&dir), id)),
        );
    }
    match found.len() {
        1 => found.remove(0),
        _ => unknown(),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn planned(id: &str, status: &str, worktree: &str) -> plan::Plan {
        plan::Plan {
            id: id.to_string(),
            milestone: "M1".to_string(),
            slug: "slug".to_string(),
            files: Vec::new(),
            deps: Vec::new(),
            status: status.to_string(),
            worktree: worktree.to_string(),
            started_at: String::new(),
            done_at: String::new(),
            tasks: Vec::new(),
        }
    }

    /// A runs/ tree with one plan.json per named run, and its meta.tsv status (empty: no file).
    fn runs_dir(tag: &str, runs: &[(&str, &str, PlanDoc)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dstack-issue-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (id, status, doc) in runs {
            let run = dir.join(id);
            std::fs::create_dir_all(&run).expect("run directory");
            std::fs::write(plan::path(&run), doc.to_json()).expect("write plan.json");
            if !status.is_empty() {
                std::fs::write(run.join("meta.tsv"), format!("status\t{status}\n"))
                    .expect("write meta.tsv");
            }
        }
        dir
    }

    fn doc_of(plans: Vec<plan::Plan>) -> PlanDoc {
        PlanDoc {
            v: 2,
            milestones: Vec::new(),
            plans,
        }
    }

    #[test]
    fn r01__only_a_plan_running_in_this_worktree_takes_a_filing() {
        let wt = Path::new("/wt/shared");
        let live = || {
            (
                "20260301T000000Z_current",
                "open",
                doc_of(vec![
                    planned("P1", "done", "/wt/one"),
                    planned("P2", "in-progress", "/wt/shared"),
                ]),
            )
        };
        let current = || ("20260301T000000Z_current".to_string(), "P2".to_string());

        // The closed Goal's plan finished in this directory; the open one is running in it now.
        let dir = runs_dir(
            "closed",
            &[
                (
                    "20260101T000000Z_older",
                    "closed",
                    doc_of(vec![planned("P1", "done", "/wt/shared")]),
                ),
                live(),
            ],
        );
        assert_eq!(by_worktree(&dir, wt), current());
        std::fs::remove_dir_all(&dir).expect("clean up");

        // Nothing is running there any more: the record is stale, and so is the run it names.
        let dir = runs_dir(
            "stale",
            &[(
                "20260101T000000Z_older",
                "closed",
                doc_of(vec![planned("P1", "done", "/wt/shared")]),
            )],
        );
        assert_eq!(by_worktree(&dir, wt), unknown());
        std::fs::remove_dir_all(&dir).expect("clean up");

        // run close --abandon never touches the plan, so the abandoned Goal keeps one that still
        // says in-progress here. It is not a candidate, and not an ambiguity either.
        let abandoned = || {
            (
                "20260201T000000Z_dropped",
                "abandoned",
                doc_of(vec![planned("P1", "in-progress", "/wt/shared")]),
            )
        };
        let dir = runs_dir("abandoned", &[abandoned()]);
        assert_eq!(by_worktree(&dir, wt), unknown());
        std::fs::remove_dir_all(&dir).expect("clean up");

        let dir = runs_dir("abandoned-and-live", &[abandoned(), live()]);
        assert_eq!(by_worktree(&dir, wt), current());
        std::fs::remove_dir_all(&dir).expect("clean up");

        // Two OPEN runs claiming one directory contradict each other; there is nothing to choose.
        let dir = runs_dir(
            "both",
            &[
                (
                    "20260101T000000Z_older",
                    "open",
                    doc_of(vec![planned("P1", "in-progress", "/wt/shared")]),
                ),
                live(),
            ],
        );
        assert_eq!(by_worktree(&dir, wt), unknown());
        std::fs::remove_dir_all(&dir).expect("clean up");

        // A run whose meta.tsv is missing cannot say it is open, so it says nothing at all.
        let dir = runs_dir(
            "no-meta",
            &[(
                "20260101T000000Z_older",
                "",
                doc_of(vec![planned("P1", "in-progress", "/wt/shared")]),
            )],
        );
        assert_eq!(by_worktree(&dir, wt), unknown());
        std::fs::remove_dir_all(&dir).expect("clean up");

        // A runs/ directory that is not there at all is not a reason to refuse a filing.
        assert_eq!(by_worktree(Path::new("/nowhere/runs"), wt), unknown());
    }

    #[test]
    fn r01__the_plan_of_a_filing_is_the_one_running_in_this_worktree() {
        let doc = PlanDoc {
            v: 2,
            milestones: Vec::new(),
            plans: vec![
                planned("P1", "done", "/wt/one"),
                planned("P2", "in-progress", "/wt/two"),
                planned("P3", "in-progress", "/wt/three"),
            ],
        };
        let plan_at = |wt: &str| one_of(plans_here(&doc, Path::new(wt)));
        assert_eq!(plan_at("/wt/three"), "P3");
        assert_eq!(plan_at("/wt/two"), "P2");
        // Two plans are running and this checkout is neither: naming one of them would be a guess.
        assert_eq!(plan_at("/wt/elsewhere"), "-");
        // The worktree of a plan that is not running is not the plan a filing came from either.
        assert_eq!(plan_at("/wt/one"), "-");
        let idle = doc_of(vec![planned("P1", "ready", "")]);
        assert_eq!(one_of(plans_here(&idle, Path::new("/wt/one"))), "-");

        // Two plans of one run in one checkout: the run is known, which plan filed is not.
        let both = doc_of(vec![
            planned("P1", "in-progress", "/wt/shared"),
            planned("P2", "in-progress", "/wt/shared"),
        ]);
        assert_eq!(plans_here(&both, Path::new("/wt/shared")).len(), 2);
        assert_eq!(one_of(plans_here(&both, Path::new("/wt/shared"))), "-");
    }
}
