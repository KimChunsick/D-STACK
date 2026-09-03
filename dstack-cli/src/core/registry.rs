// core/registry.rs
// The ordered verb roster and the dispatch rules of claude/bin/dstack.

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::help;
use crate::core::verb::Verb;

/// The roster of the shell reference's help.sh in its order, with the hook entry of D-01 after gate.
/// dstack help renders it and the doctor sweep reads it, so a roster entry no handler answers
/// is a stated "not ported yet", never a silent gap.
#[rustfmt::skip]
pub const ROSTER: [(&str, &str); 62] = [
    ("init", "bootstrap the .dstack store in this repository (never expands cases)"),
    ("run new", "mint a run: .dstack/runs/<UTC>_<slug>, write CURRENT, check tools (--type, --worktree)"),
    ("run adopt", "take over a run after /clear or resume (stale owner auto-adopts, --force otherwise)"),
    ("run list", "list every run in the store with status, worktree, branch and counts"),
    ("run verify", "print pwd, git common-dir, store, worktree, branch, HEAD and CURRENT"),
    ("run close", "verify then stamp closed_at and clear CURRENT (--abandon <why> skips verify)"),
    ("run pause", "status paused and CURRENT cleared: the escape hatch from a repeating Stop block"),
    ("exec", "run a long command in the background capture dir: dstack exec <label> -- <cmd>"),
    ("request new", "create request.md from the work_type template (--type, --title)"),
    ("request open", "snapshot the agent draft and open the file in VSCode (code -g, never -w)"),
    ("request approve", "validate, clear pending rows, record the sha256, diff vs draft, sync cases"),
    ("request show", "print the request file and its approval state (a fresh read)"),
    ("req add", "mint the next R row (--id, --from-answer, --assumption --from Q-NN)"),
    ("req accept", "fill the pending accept criterion of a --from-answer row"),
    ("req split", "mark a row superseded by its children (--into R,R)"),
    ("req withdraw", "mark a row withdrawn with a reason (reports as WITHDRAWN)"),
    ("req defer", "mark a row deferred to a later Goal (reports as DEFERRED)"),
    ("req status", "counts and per-row state of the request"),
    ("ask add", "add an interview question to the Q ledger (--affects R.. or design)"),
    ("ask answer", "record the answer and its decision row (Q answered, D answered)"),
    ("ask assume", "adopt a default: Q assumed, D assumed, and one R row from the assumption"),
    ("ask list", "print the question ledger with counts"),
    ("decision add", "record a decision row (D-NN; --design for a design round)"),
    ("decision list", "print decisions with counts"),
    ("milestone add", "append a milestone (decimal id when --after is not the last)"),
    ("plan add", "add a plan with declared files and deps"),
    ("plan insert", "insert a decimal plan after another (refused while the subtree is in progress)"),
    ("plan remove", "remove a plan (refused when in progress or depended on)"),
    ("plan edit", "change files/deps/slug of a plan that has not started"),
    ("plan render", "print the plan table and regenerate ROADMAP.md and STATE.md"),
    ("plan start", "mark a plan in progress (--worktree records where it runs)"),
    ("plan done", "mark a plan done and refresh readiness"),
    ("task add", "add a task to a plan with --covers R.. --files --deps"),
    ("task done", "record the commit of a finished task"),
    ("next", "print ready plans, overlapping pairs with reasons, the cap, and the schedulable set"),
    ("cases sync", "expand approved R rows into cases.tsv (keeps recorded evidence)"),
    ("cases render", "print the ledger as a table"),
    ("evidence add", "the only writer of evidence rows (validates artifact, mtime, sharing, R mention)"),
    ("evidence retire", "retire a recorded row whose artifact was overwritten or proved the wrong thing (--why; the R needs a new row)"),
    ("check request", "validate frontmatter, rows, ledger counts and the approval hash"),
    ("check coverage", "every live R needs a covering task and an evidence row"),
    ("check decisions", "every D row needs a covering task or evidence"),
    ("check review-bundle", "REQUEST rows, plan covers and cited R ids must agree"),
    ("verify", "policy ceiling, per-field evidence, sha256 recheck, branch containment (--accept-abstain)"),
    ("report", "R table with computed status; --metrics prints the five R01 metrics"),
    ("review", "build a review bundle: --scope plan --plan P<id> | --scope milestone --milestone M<id>"),
    ("review seal", "seal a Codex round into review/codex-review-<NNN>.md and index its verdicts"),
    ("review close", "end a review on purpose (--scope plan|milestone|quick --id --why): its R ids read ABSTAIN until a newer round"),
    ("worker report", "diff the delegated covers set against the R lines of a worker report; unreported rows go to the ledger"),
    ("quick new", "open a quick task outside any run (--discuss --research --review --validate --full)"),
    ("quick list", "quick tasks by status"),
    ("quick status", "state of one quick task"),
    ("quick resume", "print what a quick task still needs"),
    ("quick close", "report and close a quick task"),
    ("issue new", "file the friction you hit with dstack itself into ~/Documents/dstack-issues (--symptom, --repro, --source, --proposal)"),
    ("issue list", "what has been filed: one row per issue with its sightings count and last seen"),
    ("status", "human status; --oneline is the ≤2KB line the inject hook adds to each prompt"),
    ("gate", "the Stop-hook verdict: pending rows, open questions, coverage, changed-file lint"),
    ("hook", "the four Claude Code hook events in-process: dstack hook inject|stop|agent-model|pre-write, payload on stdin, verdict JSON on stdout"),
    ("doctor", "deps, agents, codex flags, verb sweep, stale locks, hook results; --self runs fixtures"),
    ("lint-ko", "Korean rule check for files, stdin, or changed files"),
    ("help", "this list"),
];

/// The nouns claude/bin/dstack runs without a git repository, plus hook (D-01) and issue,
/// whose folder is outside the store and whose run and plan are read only when there is one (D-05).
const NO_ROOTS: [&str; 5] = ["help", "doctor", "lint-ko", "hook", "issue"];

pub struct Registry {
    handlers: Vec<Box<dyn Verb>>,
}

impl Registry {
    pub fn new(handlers: Vec<Box<dyn Verb>>) -> Registry {
        Registry { handlers }
    }

    /// The roster keys in order — the machine list help_verb_list prints for the doctor sweep.
    pub fn verb_list(&self) -> Vec<&'static str> {
        ROSTER.iter().map(|(name, _)| *name).collect()
    }

    /// The dispatcher of claude/bin/dstack: two-word entry first, then the one-word entry.
    pub fn dispatch(&self, ctx: &mut Context, argv: &[String]) -> Result<()> {
        let first = argv.first().map(String::as_str).unwrap_or("help");
        let noun = match first {
            "-h" | "--help" | "help" => "help",
            other => other,
        };
        let rest: &[String] = if argv.is_empty() { &[] } else { &argv[1..] };
        if noun == "help" {
            help::render(&mut ctx.out);
            return Ok(());
        }
        if !is_known_noun(noun) {
            return Err(Error::failed(format!(
                "unknown command: {noun} (dstack help)"
            )));
        }
        // The dispatcher resolves the roots before it looks up the handler, so an unknown verb
        // outside a repository still reports the missing repository first.
        if !NO_ROOTS.contains(&noun) {
            ctx.roots()?;
        }
        if let Some(verb) = rest.first() {
            let two_word = format!("{noun} {verb}");
            if has_entry(&two_word) {
                return self.invoke(ctx, &two_word, &rest[1..]);
            }
        }
        if has_entry(noun) {
            return self.invoke(ctx, noun, rest);
        }
        let verb = rest.first().map(String::as_str).unwrap_or("");
        Err(Error::failed(format!(
            "unknown verb for {noun}: {verb} (dstack help)"
        )))
    }

    /// True when the dispatcher can answer this roster entry — what the doctor's verb sweep
    /// counts. help is answered by the dispatcher itself, so it has no handler and is not a gap.
    pub fn has_handler(&self, name: &str) -> bool {
        name == "help" || self.handlers.iter().any(|handler| handler.name() == name)
    }

    /// Runs one roster entry by name; the in-process self-calls of Context::call land here too.
    pub fn invoke(&self, ctx: &mut Context, name: &str, args: &[String]) -> Result<()> {
        match self.handlers.iter().find(|h| h.name() == name) {
            Some(handler) => handler.run(ctx, args),
            None => Err(Error::cannot_decide(format!("not ported yet: {name}"))),
        }
    }
}

/// True when some roster entry starts with this noun (the shell asks whether the lib file exists).
fn is_known_noun(noun: &str) -> bool {
    let prefix = format!("{noun} ");
    ROSTER
        .iter()
        .any(|(name, _)| *name == noun || name.starts_with(&prefix))
}

fn has_entry(name: &str) -> bool {
    ROSTER.iter().any(|(entry, _)| *entry == name)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__roster_has_sixty_entries() {
        assert_eq!(ROSTER.len(), 62);
    }

    #[test]
    fn r13__hook_follows_gate() {
        let names: Vec<&str> = ROSTER.iter().map(|(name, _)| *name).collect();
        let gate = names
            .iter()
            .position(|name| *name == "gate")
            .expect("gate entry");
        assert_eq!(names[gate + 1], "hook");
    }

    #[test]
    fn r13__verb_list_is_the_roster_order() {
        let registry = Registry::new(Vec::new());
        let list = registry.verb_list();
        assert_eq!(list.len(), 62);
        assert_eq!(list[0], "init");
        assert_eq!(list[list.len() - 1], "help");
    }

    #[test]
    fn r13__known_nouns_and_entries() {
        assert!(is_known_noun("run"));
        assert!(is_known_noun("status"));
        assert!(!is_known_noun("bogus"));
        assert!(has_entry("run new"));
        assert!(!has_entry("run"));
    }
}
