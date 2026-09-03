// verbs/hook/prewrite.rs
// pre-write (PreToolUse, tools Write|Edit|Bash, R93): what a pending write shows of its content,
// judged by lint-ko before it lands.

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::verbs::lint::run::lint_ko_stdin;

use super::command;
use super::json::Json;
use super::{first_line, head, last_line, named, Hook};

/// What the tool showed of the content that is about to land, and what it is scoped by: a path in
/// the worktree, or the commit-message scope, which belongs to no path at all.
struct Pending {
    path: String,
    content: String,
    fragment: bool,
    commit: bool,
}

pub(super) fn pre_write(ctx: &mut Context, hook: &Hook) -> Result<()> {
    let tool = hook.payload.field("tool_name");
    let pending = match tool.as_str() {
        "Write" => Pending {
            path: hook.payload.field("tool_input.file_path"),
            content: captured(hook.payload.field("tool_input.content")),
            fragment: false,
            commit: false,
        },
        // An Edit shows one fragment of a file; sentence-level rules are deferred to the Stop
        // gate, where the whole file is readable (R93).
        "Edit" => Pending {
            path: hook.payload.field("tool_input.file_path"),
            content: captured(hook.payload.field("tool_input.new_string")),
            fragment: true,
            commit: false,
        },
        "Bash" => match bash(hook) {
            Some(pending) => pending,
            None => return Ok(()),
        },
        _ => {
            hook.log(0, &format!("tool {}: not linted", named(&tool)));
            return Ok(());
        }
    };
    if !pending.commit && pending.path.is_empty() {
        hook.log(0, "no path to scope");
        return Ok(());
    }
    judge(ctx, hook, &pending)
}

/// `$( )`: a command substitution drops the trailing newlines of what it captured, whichever of
/// the four sources the body came from.
fn captured(text: String) -> String {
    text.trim_end_matches('\n').to_string()
}

/// The two things a Bash call can show: the message of a `git commit`, and the heredoc body a
/// redirect is about to write. Everything else is nothing to judge, and says so in the log.
fn bash(hook: &Hook) -> Option<Pending> {
    let command = hook.payload.field("tool_input.command");
    if command.is_empty() {
        hook.log(0, "empty command");
        return None;
    }
    if command::is_git_commit(&command) {
        let content = captured(command::commit_text(&command));
        if content.is_empty() {
            hook.log(0, "git commit with no visible message");
            return None;
        }
        return Some(Pending {
            path: String::new(),
            content,
            fragment: false,
            commit: true,
        });
    }
    let path = command::redirect_path(&command);
    if path.is_empty() {
        hook.log(0, "no file creation detected");
        return None;
    }
    let content = captured(command::heredoc_bodies(&command));
    if content.is_empty() {
        hook.log(0, &format!("redirect to {path} with no visible content"));
        return None;
    }
    Some(Pending {
        path,
        content,
        fragment: false,
        commit: false,
    })
}

/// lint-ko needs no store: a commit message in a repository that never ran `dstack init` is still
/// checkable, and an unscoped path simply matches no row and blocks nothing (R93).
fn judge(ctx: &mut Context, hook: &Hook, pending: &Pending) -> Result<()> {
    let mut args = vec!["--stdin".to_string()];
    match pending.commit {
        true => args.extend(["--scope".to_string(), "commit-msg".to_string()]),
        false => {
            args.extend(["--path".to_string(), pending.path.clone()]);
            if pending.fragment {
                args.push("--fragment".to_string());
            }
        }
    }
    let called = call_lint(ctx, &args, &format!("{}\n", pending.content));
    let merged = format!("{}{}", called.stdout, called.stderr);
    let output = merged.trim_end_matches('\n');
    let what = match pending.path.is_empty() {
        true => "<commit-msg>",
        false => &pending.path,
    };
    match called.code {
        0 => {
            hook.log(0, &format!("allow {what}"));
            Ok(())
        }
        1 => {
            ctx.out.say(&deny(&head(output, 20)));
            hook.log(0, &format!("deny {what}: {}", first_line(output)));
            Ok(())
        }
        code => hook.cannot_decide(
            ctx,
            &format!("dstack lint-ko exited {code}: {}", last_line(output)),
            "fix the rule or scope table the message names",
        ),
    }
}

fn deny(reason: &str) -> String {
    Json::Object(vec![(
        "hookSpecificOutput".to_string(),
        Json::Object(vec![
            (
                "hookEventName".to_string(),
                Json::Text("PreToolUse".to_string()),
            ),
            (
                "permissionDecision".to_string(),
                Json::Text("deny".to_string()),
            ),
            (
                "permissionDecisionReason".to_string(),
                Json::Text(reason.to_string()),
            ),
        ]),
    )])
    .compact()
}

/// What one lint-ko call left behind: the exit code and the two streams the shell captured from
/// the subprocess it spawned here.
struct Called {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Context::call for the one verb the hook hands a body to: lint-ko reads its stdin, and this
/// process spent stdin on the payload before anything else could fail.
fn call_lint(ctx: &mut Context, args: &[String], body: &str) -> Called {
    ctx.out.begin_capture();
    let result = lint_ko_stdin(ctx, args, Some(body));
    let (stdout, mut stderr) = ctx.out.end_capture();
    match result {
        Ok(()) => Called {
            code: 0,
            stdout,
            stderr,
        },
        Err(error) => {
            // Error::Exit prints nothing, so the caller reads a bare code, as it read the exit
            // status of the subprocess the shell spawned here.
            if !matches!(error, Error::Exit(_)) {
                stderr.push_str(&format!("{error}\n"));
            }
            Called {
                code: error.code(),
                stdout,
                stderr,
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r07__the_deny_payload_is_the_one_claude_code_reads() {
        assert_eq!(
            deny("README.md:1: K01 (S1) matched '정본'"),
            "{\"hookSpecificOutput\":{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"deny\",\
             \"permissionDecisionReason\":\"README.md:1: K01 (S1) matched '정본'\"}}"
        );
    }
}
