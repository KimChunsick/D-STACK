// verbs/hook/events.rs
// The three events that carry a state verdict: inject (R24), agent-model (R22) and stop (D-13).

use std::io::Write;
use std::path::PathBuf;

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::fsx::utc_now;
use crate::core::meta::meta_set;

use super::json::Json;
use super::{last_line, named, Hook};

/// The fix hint the wrapper prints when its own tools let it down (D-02 keeps the wording).
const JQ_FIX: &str = "install jq (brew install jq) and run D-STACK's install.sh so the CLI is installed at ~/.claude/bin/dstack";

/// D-12: inject is a status carrier, not a verdict. It must never block a prompt, so every path
/// out of it ends in exit 0 — including the one where the CLI cannot read this repository.
pub(super) fn inject(ctx: &mut Context, hook: &Hook) -> Result<()> {
    if !hook.has_store() {
        hook.log(0, "no store in this repository");
        return Ok(());
    }
    // `2>/dev/null`: whatever the status verb warned about is not part of the prompt.
    let called = ctx.call("status", &["--oneline".to_string()]);
    let line = called.stdout.trim_end_matches('\n').to_string();
    if called.code == 2 || line.is_empty() {
        let self_exe = ctx.self_exe.display().to_string();
        ctx.out.say(&format!(
            "dstack: status unavailable — the CLI could not read this repository state — fix: run {self_exe} run verify"
        ));
        hook.log(0, &format!("status exit {}", called.code));
        return Ok(());
    }
    ctx.out.say(&line);
    hook.log(0, &format!("injected {} bytes", line.len()));
    Ok(())
}

/// R22: a subagent that inherits the session model inherits the Fable this rule exists to keep out
/// of subagents; "inherit", "haiku" and a pinned full id are the same problem, and every one of
/// them is rewritten to opus through updatedInput.
pub(super) fn agent_model(ctx: &mut Context, hook: &Hook) -> Result<()> {
    let tool = hook.payload.field("tool_name");
    if tool != "Agent" {
        hook.log(0, &format!("tool {}: not Agent", named(&tool)));
        return Ok(());
    }
    let model = hook.payload.field("tool_input.model");
    if model == "sonnet" || model == "opus" {
        hook.log(0, &format!("model {model}: unchanged"));
        return Ok(());
    }
    let was = match model.is_empty() {
        true => "(none)",
        false => &model,
    };
    let Some(input) = hook.payload.tool_input() else {
        // jq refused the object addition, and the wrapper blocks on that rather than approving a
        // call it could not rewrite. The wording is the reference's (D-02); jq's own diagnostic
        // line above it belongs to a tool the port does not run (D-11).
        return hook.cannot_decide(ctx, "jq could not build the updatedInput payload", JQ_FIX);
    };
    ctx.out.say(&rewrite(was, input));
    note_rewrite(was);
    hook.log(0, &format!("model {was} → opus"));
    Ok(())
}

/// The payload jq built with `$ti + {model:"opus"}`: the key is replaced where it stands when the
/// call carried one and appended when it did not, and every other key keeps the caller's order.
fn rewrite(was: &str, input: &[(String, Json)]) -> String {
    let mut updated: Vec<(String, Json)> = Vec::new();
    let mut replaced = false;
    for (key, value) in input {
        let value = match key == "model" {
            true => {
                replaced = true;
                Json::Text("opus".to_string())
            }
            false => value.clone(),
        };
        updated.push((key.clone(), value));
    }
    if !replaced {
        updated.push(("model".to_string(), Json::Text("opus".to_string())));
    }
    Json::Object(vec![(
        "hookSpecificOutput".to_string(),
        Json::Object(vec![
            (
                "hookEventName".to_string(),
                Json::Text("PreToolUse".to_string()),
            ),
            (
                "permissionDecision".to_string(),
                Json::Text("allow".to_string()),
            ),
            (
                "permissionDecisionReason".to_string(),
                Json::Text(format!("dstack: model '{was}' → opus (R22)")),
            ),
            ("updatedInput".to_string(), Json::Object(updated)),
        ]),
    )])
    .compact()
}

/// Every rewrite is also written where the user can read it back later. The log decides nothing,
/// so a path that cannot be written is not the hook's problem.
fn note_rewrite(was: &str) {
    let path = match std::env::var("DSTACK_HOOK_LOG") {
        Ok(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = match std::env::var("HOME") {
                Ok(home) if !home.is_empty() => home,
                _ => "/tmp".to_string(),
            };
            PathBuf::from(home).join(".claude/dstack-hook.log")
        }
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format!("{}\tagent-model\tmodel {was} → opus (R22)\n", utc_now());
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map(|mut file| file.write_all(line.as_bytes()));
}

/// R33/R65/R99: the Stop verdict is the gate's, carried back as the block payload Claude Code
/// reads. A gate that cannot decide blocks the turn (R101), which is the exit 2 below.
pub(super) fn stop(ctx: &mut Context, hook: &Hook) -> Result<()> {
    if !hook.has_store() {
        hook.log(0, "no store in this repository");
        return Ok(());
    }
    // R01 wants the transcript findable from the run folder, and meta_set is the only writer of
    // meta.tsv, so the hook goes through it rather than appending a line itself.
    record_meta(hook);
    // D-13: the block is stated once per turn. A second stop in the same turn ends the turn, which
    // is what lets a turn waiting on a background `dstack exec` be re-entered when the run finishes
    // — a gate that could never let a turn end could never be woken up again.
    if hook.payload.field("stop_hook_active") == "true" {
        hook.log(0, "stop_hook_active: block already stated this turn");
        return Ok(());
    }
    let called = ctx.call("gate", &[]);
    // `2>&1`: what the gate says about a state it cannot read belongs in the reason too.
    let merged = format!("{}{}", called.stdout, called.stderr);
    let output = merged.trim_end_matches('\n');
    // Written again after the gate: every checker the gate runs refreshes ownership through
    // touch_owner, so the payload's session id has to be the last writer or the recorded owner is
    // whatever CLAUDE_CODE_SESSION_ID the checker carried.
    record_meta(hook);
    match called.code {
        0 => {
            hook.log(0, "gate clear");
            Ok(())
        }
        1 => {
            let reason: String = output.chars().take(4000).collect();
            ctx.out.say(
                &Json::Object(vec![
                    ("decision".to_string(), Json::Text("block".to_string())),
                    ("reason".to_string(), Json::Text(reason)),
                ])
                .compact(),
            );
            hook.log(0, &format!("gate blocked: {}", last_line(output)));
            Ok(())
        }
        code => hook.cannot_decide(
            ctx,
            &format!("dstack gate exited {code}: {}", last_line(output)),
            "fix the state named above, or pause the run",
        ),
    }
}

/// The transcript and the session of the turn, recorded on the run CURRENT names. A run that is
/// not there is not written to: the hook records what a run already has, it never mints one.
///
/// This is bookkeeping, not a verdict: nothing here decides anything, and inject and stop must
/// not block on it, so a CURRENT that cannot be read ends the recording the same way a failed
/// meta_set below does. The verdict paths read CURRENT through the readers D-12 governs.
fn record_meta(hook: &Hook) {
    let Some(roots) = &hook.roots else {
        return;
    };
    let Ok(Some(id)) = roots.current_run_id() else {
        return;
    };
    let dir = roots.runs.join(&id);
    if !dir.is_dir() {
        return;
    }
    let transcript = hook.payload.field("transcript_path");
    if !transcript.is_empty() {
        let _ = meta_set(&dir, "transcript_path", &transcript);
    }
    let session = hook.payload.field("session_id");
    if !session.is_empty() {
        let _ = meta_set(&dir, "owner_session", &session);
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::super::json::Payload;
    use super::*;

    fn input(text: &str) -> Payload {
        Payload::parse(text).expect("both parsers read this one")
    }

    #[test]
    fn r07__the_rewrite_replaces_the_model_where_it_stands() {
        let payload = input(r#"{"tool_input":{"description":"probe","model":"fable","n":2}}"#);
        assert_eq!(
            rewrite("fable", payload.tool_input().expect("an object")),
            "{\"hookSpecificOutput\":{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"allow\",\
             \"permissionDecisionReason\":\"dstack: model 'fable' → opus (R22)\",\
             \"updatedInput\":{\"description\":\"probe\",\"model\":\"opus\",\"n\":2}}}"
        );
    }

    #[test]
    fn r07__a_call_without_a_model_gets_one_appended() {
        let payload = input(r#"{"tool_input":{"description":"probe"}}"#);
        assert!(
            rewrite("(none)", payload.tool_input().expect("an object")).ends_with(
                "\"permissionDecisionReason\":\"dstack: model '(none)' → opus (R22)\",\
             \"updatedInput\":{\"description\":\"probe\",\"model\":\"opus\"}}}"
            )
        );
        // No tool_input at all is jq's `// {}`: the rewrite is the whole updatedInput.
        assert!(
            rewrite("(none)", input("{}").tool_input().expect("an object"))
                .ends_with("\"updatedInput\":{\"model\":\"opus\"}}}")
        );
    }
}
