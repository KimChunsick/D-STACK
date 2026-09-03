// verbs/hook/selftests.rs
// The two checkers of R101, over claude/lint/fixtures/{agent-model-hook,hook-fail-closed}. Both
// drive the registered wrapper itself, because what they judge is the whole path from the script
// Claude Code calls to the verdict — including the case where the binary behind it is missing.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

use super::json::Payload;

pub(super) fn all() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(AgentModelHook), Box::new(HookFailClosed)]
}

/// The one hook script registered anywhere (R101), in the repository this binary belongs to.
fn wrapper(ctx: &Context) -> PathBuf {
    ctx.home.home.join("hooks/dstack-hook.sh")
}

/// The wrapper with the fixture on its stdin: its exit code and what it printed on stdout. A
/// wrapper that cannot be started at all is the runner failing, not a fixture being rejected.
fn drive(command: &mut Command, fixture: &Path) -> Result<(i32, String)> {
    let payload = std::fs::File::open(fixture).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot read {}: {e}", fixture.display()))
    })?;
    let done = command
        .stdin(Stdio::from(payload))
        .stderr(Stdio::null())
        .output()
        .map_err(|e| Error::cannot_decide(format!("selftest: cannot run the hook wrapper: {e}")))?;
    Ok((
        done.status.code().unwrap_or(2),
        String::from_utf8_lossy(&done.stdout).into_owned(),
    ))
}

/// claude/lint/fixtures/agent-model-hook/*.json — PreToolUse payloads for the Agent tool.
/// "reject" means the hook rewrote the model to opus through updatedInput (R22); "pass" means it
/// said nothing at all, which is what a sonnet/opus call must produce.
struct AgentModelHook;

impl Selftest for AgentModelHook {
    fn checker(&self) -> &'static str {
        "agent-model-hook"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        // DSTACK_BIN is pinned so a machine without ~/.claude/bin/dstack takes the model path and
        // not the fail-closed path — otherwise every fixture would "reject" for the wrong reason.
        // The scratch repository is the cwd, so the hook log of this run lands there instead of in
        // the store of whatever repository doctor was started in.
        let sandbox = Sandbox::scratch()?;
        let mut command = Command::new(wrapper(ctx));
        command
            .arg("agent-model")
            .current_dir(&sandbox.dir)
            .env("DSTACK_BIN", &ctx.self_exe)
            .env("DSTACK_HOOK_LOG", sandbox.dir.join("agent-model.log"));
        let (code, stdout) = drive(&mut command, fixture)?;
        if code != 0 {
            return Ok(Verdict::Reject);
        }
        if stdout.is_empty() {
            return Ok(Verdict::Pass);
        }
        // The checker reads the hook's own output, not a payload from outside: a line this build
        // cannot read is no model at all.
        let model = Payload::parse(&stdout)
            .map(|out| out.field("hookSpecificOutput.updatedInput.model"))
            .unwrap_or_default();
        Ok(match model == "opus" {
            true => Verdict::Reject,
            false => Verdict::Pass,
        })
    }
}

/// The DSTACK_BIN a bad-* fixture wants the locator to look at, if it names one.
fn candidate(fixture: &Path) -> Result<Option<String>> {
    let text = std::fs::read_to_string(fixture).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot read {}: {e}", fixture.display()))
    })?;
    let named = Payload::parse(&text)
        .map(|fixture| fixture.field("selftest_dstack_bin"))
        .unwrap_or_default();
    Ok(match named.is_empty() {
        true => None,
        false => Some(named),
    })
}

/// claude/lint/fixtures/hook-fail-closed/*.json — R101's evidence, automated. bad-*: nothing the
/// wrapper could run, so it must exit 2 (block) instead of exiting 0 (silent pass); a fixture that
/// carries `selftest_dstack_bin` names the candidate the locator has to refuse — a directory is
/// executable and starting one ends in 126, which reads as "carry on". good-*: a reachable binary
/// and a sandbox with no open run exit 0.
struct HookFailClosed;

impl Selftest for HookFailClosed {
    fn checker(&self) -> &'static str {
        "hook-fail-closed"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let named_bad = fixture
            .file_name()
            .map(|name| name.to_string_lossy().starts_with("bad-"))
            .unwrap_or(false);
        if named_bad {
            let sandbox = Sandbox::scratch()?;
            let mut command = Command::new(wrapper(ctx));
            command
                .arg("stop")
                .current_dir(&sandbox.dir)
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", &sandbox.dir);
            match candidate(fixture)? {
                Some(bin) => command.env("DSTACK_BIN", bin),
                None => command.env_remove("DSTACK_BIN"),
            };
            let (code, _) = drive(&mut command, fixture)?;
            return Ok(match code {
                2 => Verdict::Reject,
                _ => Verdict::Pass,
            });
        }
        let sandbox = Sandbox::new(ctx)?;
        // The run the sandbox opened is paused: `run pause` is the escape hatch from a repeating
        // Stop block (R101), so after it the gate has nothing to hold the turn on.
        sandbox.dsx(ctx, &["run", "pause"])?;
        let mut command = Command::new(wrapper(ctx));
        command
            .arg("stop")
            .current_dir(&sandbox.dir)
            .env("DSTACK_BIN", &ctx.self_exe)
            .env("DSTACK_DEPS", sandbox.dir.join(".deps.tsv"))
            .env("DSTACK_HOOK_LOG", sandbox.dir.join("agent-model.log"));
        let (code, _) = drive(&mut command, fixture)?;
        Ok(match code {
            0 => Verdict::Pass,
            _ => Verdict::Reject,
        })
    }
}
