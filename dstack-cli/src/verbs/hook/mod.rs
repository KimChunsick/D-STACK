// verbs/hook/mod.rs
// dstack hook inject|stop|agent-model|pre-write: the four Claude Code hook events in process.
//
// Claude Code treats only exit 2 as a block. A crashed hook, a timed-out hook and a hook that
// printed malformed JSON all become a notification and the turn continues, so every path that
// cannot compute a verdict ends in exit 2 on purpose (R101). D-01 leaves the registered script
// with one job — finding this binary — and everything below is what it used to do with jq.

use std::io::Read;
use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::utc_now;
use crate::core::roots::Roots;
use crate::core::verb::Verb;
use crate::selftest::Selftest;

use json::Payload;

mod command;
mod events;
mod json;
mod prewrite;
mod selftests;
mod wellformed;

/// The events the four registrations name; anything else blocks, because a hook nobody registered
/// is a configuration the wrapper cannot compute a verdict for.
const EVENTS: [&str; 4] = ["inject", "stop", "agent-model", "pre-write"];

struct HookEvent;

impl Verb for HookEvent {
    fn name(&self) -> &'static str {
        "hook"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        hook(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(HookEvent)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    selftests::all()
}

/// One hook run: the event, the payload it was handed, and the roots of the repository the payload
/// pointed the hook at.
struct Hook {
    event: String,
    payload: Payload,
    roots: Option<Roots>,
}

fn hook(ctx: &mut Context, args: &[String]) -> Result<()> {
    let event = args.first().cloned().unwrap_or_default();
    // stdin is read exactly once, before anything can fail: the caller writes the whole payload and
    // a hook that exits without draining it can leave the client blocked on the pipe.
    let mut raw = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut raw);
    if !EVENTS.contains(&event.as_str()) {
        let named = match event.is_empty() {
            true => "<none>",
            false => &event,
        };
        ctx.out.err_line(&format!(
            "dstack-hook {named}: cannot decide — unknown event — fix: register one of inject|stop|agent-model|pre-write; escape: {} run pause",
            ctx.self_exe.display()
        ));
        return Err(Error::Exit(2));
    }
    let mut hook = Hook {
        event,
        payload: Payload::empty(),
        roots: None,
    };
    // A payload jq reads and serde_json refuses must not read as an empty one: every event would
    // take its "nothing to judge" branch and a model rewrite or a Korean check would be skipped in
    // silence (round 063). What the hook does with it is below, after the roots are there to log
    // with; the cwd of such a payload is not read either, so the hook stays where it was started.
    let readable = match Payload::parse(&String::from_utf8_lossy(&raw)) {
        Some(payload) => {
            hook.payload = payload;
            true
        }
        None => false,
    };
    let cwd = hook.payload.field("cwd");
    if !cwd.is_empty() && Path::new(&cwd).is_dir() && std::env::set_current_dir(&cwd).is_err() {
        return hook.cannot_decide(
            ctx,
            &format!("cannot enter cwd {cwd}"),
            "check the directory still exists",
        );
    }
    // resolve_roots(), best effort: outside a git repository the shell's `git rev-parse` answered
    // nothing and the hook carried on without a store and without a log.
    hook.roots = ctx.roots().ok();
    if !readable {
        return hook.unreadable(ctx);
    }
    match hook.event.as_str() {
        "inject" => events::inject(ctx, &hook),
        "agent-model" => events::agent_model(ctx, &hook),
        "stop" => events::stop(ctx, &hook),
        _ => prewrite::pre_write(ctx, &hook),
    }
}

impl Hook {
    /// The store lives beside the main worktree so every linked worktree shares it (design §2); a
    /// repository without one has no pipeline for the hook to have an opinion about.
    fn has_store(&self) -> bool {
        self.roots
            .as_ref()
            .is_some_and(|roots| roots.store.join("version").is_file())
    }

    /// <worktree>/.dstack/local/hooks/<event>.last — what `dstack doctor` reads back to show the
    /// last result of every registered hook (R101). The log decides nothing, so it never fails the
    /// hook: a read-only or absent store just means no line.
    fn log(&self, code: i32, note: &str) {
        let Some(roots) = &self.roots else {
            return;
        };
        if !roots.local.is_dir() {
            return;
        }
        let dir = roots.local.join("hooks");
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let line = format!("{}\t{code}\t{}\t{note}\n", self.event, utc_now());
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(format!("{}.last", self.event)))
            .map(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
    }

    /// The payload is JSON the reference reads and this build does not. inject never blocks a
    /// prompt (D-01), so it carries a note the way it does for a binary it cannot find; the three
    /// events that decide something block rather than judge fields they could not read.
    fn unreadable(&self, ctx: &mut Context) -> Result<()> {
        if self.event == "inject" {
            ctx.out.say(
                "dstack: status unavailable — the payload of this turn is JSON this build cannot read — fix: report the tool call that sent it",
            );
            self.log(0, "payload unreadable");
            return Ok(());
        }
        self.cannot_decide(
            ctx,
            "the payload is JSON this build cannot read (a number outside f64, nesting past the limit, or a lone surrogate)",
            "report the tool call that sent it; blocking is on purpose, so a payload the hook could not read never passes",
        )
    }

    /// The one line a hook without a verdict prints, and the exit code Claude Code reads as a
    /// block. The escape hatch is named in it, because a hook that repeats has to be stoppable.
    fn cannot_decide(&self, ctx: &mut Context, why: &str, fix: &str) -> Result<()> {
        ctx.out.err_line(&format!(
            "dstack-hook {}: cannot decide — {why} — fix: {fix}; escape: {} run pause",
            self.event,
            ctx.self_exe.display()
        ));
        self.log(2, &format!("cannot decide: {why}"));
        Err(Error::Exit(2))
    }
}

/// `${tool:-<none>}`: what the log calls a field the payload did not carry.
fn named(value: &str) -> &str {
    match value.is_empty() {
        true => "<none>",
        false => value,
    }
}

/// `printf '%s' "$out" | head -1` and `| tail -1` over a capture whose trailing newlines are
/// already gone, and `| head -N` for the reason of a deny.
fn first_line(output: &str) -> &str {
    output.lines().next().unwrap_or("")
}

fn last_line(output: &str) -> &str {
    output.lines().next_back().unwrap_or("")
}

fn head(output: &str, lines: usize) -> String {
    output.lines().take(lines).collect::<Vec<&str>>().join("\n")
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r07__the_pieces_of_a_capture_the_hook_quotes() {
        assert_eq!(first_line("one\ntwo\nthree"), "one");
        assert_eq!(last_line("one\ntwo\nthree"), "three");
        assert_eq!(head("one\ntwo\nthree", 2), "one\ntwo");
        assert_eq!(head("one", 20), "one");
        assert_eq!(first_line(""), "");
        assert_eq!(last_line(""), "");
        assert_eq!(named(""), "<none>");
        assert_eq!(named("Agent"), "Agent");
    }
}
