// verbs/gate.rs
// dstack gate: the Stop hook's verdict (R33, R65, R99, R101).
//
// The gate looks at exactly two things in THIS worktree: the run CURRENT points at, and the open
// quick items of this worktree — no "one GOAL.md" rule, no cross-worktree scan. Every sub-check
// runs through the same handler the shell spawned as a subprocess, so a checker that cannot
// decide arrives here as exit 2 and leaves here as exit 2: a gate that swallowed an unreadable
// state would be a silent pass, and R101 says a hook that cannot compute blocks.

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::read_text;
use crate::core::meta::meta_get;
use crate::core::roots::Roots;
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::request::RequestDoc;
use crate::verbs::quick::state;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

struct Gate;

impl Verb for Gate {
    fn name(&self) -> &'static str {
        "gate"
    }

    /// cmd_gate reads no argument at all: whatever the hook passes along is ignored.
    fn run(&self, ctx: &mut Context, _args: &[String]) -> Result<()> {
        gate(ctx)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(Gate)]
}

/// The checkers of the gate own their fixtures (check-coverage, lint-ko) and register there; the
/// gate itself is the wrapper the Stop hook calls, and the hook's own fixtures belong to P14.
pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![]
}

/// What the gate collected: the open items in the order they were found, and how many conditions
/// it looked at — the clear line reports the second even when the first is empty.
struct Found {
    items: Vec<String>,
    checked: usize,
}

fn gate(ctx: &mut Context) -> Result<()> {
    let roots = ctx.roots()?;
    // No store means no pipeline in this repository, and a repository without a pipeline is not
    // something the gate has an opinion about (the hook exits 0 before it gets here anyway).
    if !roots.store.join("version").is_file() {
        say!(
            ctx,
            "gate: clear (no .dstack store at {} — dstack init to start one)",
            roots.main_root.display()
        );
        return Ok(());
    }
    let id = roots.current_run_id()?.unwrap_or_default();
    // paused and closed runs are deliberately invisible here: `run pause` is the escape hatch
    // from a repeating Stop block (R101), so it has to actually stop the block.
    let have_run = !id.is_empty()
        && roots.runs.join(&id).is_dir()
        && meta_get(&roots.runs.join(&id), "status")?.as_deref() == Some("open");
    let quicks = state::open_slugs(&roots.quick)?;
    if !have_run && quicks.is_empty() {
        ctx.out
            .say("gate: clear (no current run, no open quick tasks)");
        return Ok(());
    }

    let mut found = Found {
        items: Vec::new(),
        checked: 0,
    };
    if have_run {
        run_conditions(ctx, &roots, &id, &mut found)?;
    }
    for slug in &quicks {
        quick_conditions(ctx, &roots, slug, &mut found)?;
    }
    lint_condition(ctx, &mut found)?;

    if found.items.is_empty() {
        let named = match id.is_empty() {
            true => "none",
            false => &id,
        };
        say!(
            ctx,
            "gate: clear (run {named}, quick open {}, checked {} condition(s), 0 open)",
            quicks.len(),
            found.checked
        );
        return Ok(());
    }
    for item in &found.items {
        ctx.out.say(&format!("- {item}"));
    }
    let escape = ctx.self_exe.display().to_string();
    say!(
        ctx,
        "gate: {} item(s) open; escape hatch: {escape} run pause",
        found.items.len()
    );
    Err(Error::Exit(1))
}

fn run_conditions(ctx: &mut Context, roots: &Roots, id: &str, found: &mut Found) -> Result<()> {
    let dir = roots.runs.join(id);
    let request = dir.join("request.md");
    found.checked += 1;
    if !request.is_file() {
        found.items.push(format!(
            "run {id}: no request.md yet: dstack request new --type <work_type>"
        ));
        return Ok(());
    }
    found.checked += 1;
    let pending = RequestDoc::load(&request)?
        .rows()
        .iter()
        .filter(|row| row.markers_string().contains("status=pending-approval"))
        .count();
    if pending > 0 {
        found.items.push(format!(
            "run {id}: {pending} request row(s) pending approval: dstack request approve"
        ));
    }
    found.checked += 1;
    let open = open_rows(&dir.join("questions.md"))?;
    if open > 0 {
        found.items.push(format!(
            "run {id}: {open} open question(s): dstack ask answer Q-NN | dstack ask assume Q-NN"
        ));
    }
    found.checked += 1;
    if !dir.join("request.approved").is_file() {
        found.items.push(format!(
            "run {id}: request not approved: dstack request approve"
        ));
    }
    coverage(ctx, &format!("run {id}"), &[], found)
}

/// R99: an open quick item is gated on the same conditions as a run — an approved request and
/// passing coverage — so the quick track cannot be used to route work around the gate.
fn quick_conditions(ctx: &mut Context, roots: &Roots, slug: &str, found: &mut Found) -> Result<()> {
    found.checked += 1;
    if !roots.quick.join(slug).join("request.approved").is_file() {
        found.items.push(format!(
            "quick {slug}: request not approved: dstack request approve --quick {slug}"
        ));
    }
    let args = ["--quick".to_string(), slug.to_string()];
    coverage(ctx, &format!("quick {slug}"), &args, found)
}

/// `dstack check coverage [--quick s]`. Exit 1 → its MISSING lines become items; exit 2 → the
/// gate cannot decide either, because "coverage unknown" must not read as "coverage fine".
fn coverage(ctx: &mut Context, what: &str, args: &[String], found: &mut Found) -> Result<()> {
    let called = ctx.call("check coverage", args);
    let output = format!("{}{}", called.stdout, called.stderr);
    found.checked += 1;
    if called.code == 2 {
        return Err(Error::cannot_decide(format!(
            "{what}: check coverage cannot decide: {}",
            tail(&output, 3)
        )));
    }
    if called.code == 0 {
        return Ok(());
    }
    let before = found.items.len();
    for line in output.lines().filter(|line| line.contains("MISSING")) {
        found.items.push(format!("{what}: {line}"));
    }
    // A failing checker that named no MISSING line still failed; report its reason rather than
    // dropping the failure on the floor.
    if found.items.len() == before {
        found.items.push(format!(
            "{what}: check coverage exited {}: {}",
            called.code,
            tail(&output, 1)
        ));
    }
    Ok(())
}

/// R93: whatever the PreToolUse hook could not see (a fragment, a shell redirect it did not
/// parse) is caught here, on the files this turn actually changed.
fn lint_condition(ctx: &mut Context, found: &mut Found) -> Result<()> {
    let called = ctx.call("lint-ko", &["--changed".to_string()]);
    let output = format!("{}{}", called.stdout, called.stderr);
    found.checked += 1;
    if called.code == 2 {
        return Err(Error::cannot_decide(format!(
            "lint-ko --changed cannot decide: {}",
            tail(&output, 2)
        )));
    }
    if called.code == 0 {
        return Ok(());
    }
    for line in output.lines().filter(|line| line.contains("(S1) matched ")) {
        found.items.push(format!("lint-ko: {line}"));
    }
    Ok(())
}

/// `printf '%s' "$out" | tail -N | tr '\n' ' '`: the last N lines on one line. The capture had
/// its trailing newlines dropped by `$( )`, so the joined text carries no trailing space either.
fn tail(output: &str, lines: usize) -> String {
    let all: Vec<&str> = output.trim_end_matches('\n').lines().collect();
    all[all.len().saturating_sub(lines)..].join(" ")
}

/// `grep -c '| open |'`: the number of lines carrying the marker, 0 when the file is not there.
/// A file that is there and cannot be read is a cannot-decide (D-12), not a count of 0.
fn open_rows(path: &Path) -> Result<usize> {
    Ok(read_text(path)?
        .unwrap_or_default()
        .lines()
        .filter(|line| line.contains("| open |"))
        .count())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__the_tail_of_a_capture_is_one_line() {
        assert_eq!(tail("one\ntwo\nthree\nfour\n", 3), "two three four");
        assert_eq!(tail("only\n", 3), "only");
        assert_eq!(tail("", 3), "");
        assert_eq!(tail("one\ntwo", 1), "two");
    }
}
