// verbs/doctor/hooks.rs
// doctor section 6: which hooks settings.json registers, and what they last answered (R101, R18).

use std::path::PathBuf;

use super::hooks_check::{self, Judgement};
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::roots::git_out;

/// The registrations of this machine, judged by hooks_check: a dstack hook missing, registered
/// twice or under a stale event fails; a hook of another program is a note (R18).
pub fn section(ctx: &mut Context) -> Result<bool> {
    let settings =
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".claude/settings.json");
    say!(ctx, "hooks registered in {}:", settings.display());
    let judgement = match std::fs::read_to_string(&settings) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            unread(true, "  (no settings.json: no hook is registered on this machine)")
        }
        Err(e) => unread(false, &format!("  FAIL: cannot read settings.json: {e}")),
        Ok(text) => match hooks_check::expected(&ctx.home.home) {
            Ok(expected) => hooks_check::judge(&text, &expected),
            Err(e) => unread(false, &format!("  FAIL: {}", e.message())),
        },
    };
    for line in &judgement.lines {
        say!(ctx, "{line}");
    }
    say!(
        ctx,
        "  registered: {}, dstack: {}, other: {}",
        judgement.registered,
        judgement.dstack,
        judgement.other
    );
    say!(ctx, "hook last results (event | exit | at | note):");
    last_results(ctx)?;
    Ok(judgement.holds)
}

/// A settings.json the judgement never saw: one line, and whether the section holds without it.
fn unread(holds: bool, line: &str) -> Judgement {
    Judgement {
        lines: vec![line.to_string()],
        registered: 0,
        dstack: 0,
        other: 0,
        holds,
    }
}

/// The `.last` file each hook leaves behind in this worktree: its last row, as four columns.
fn last_results(ctx: &mut Context) -> Result<()> {
    if git_out(None, &["rev-parse", "--git-common-dir"]).is_none() {
        say!(ctx, "  skipped: not in a git repository");
        return Ok(());
    }
    let roots = ctx.roots()?;
    let files = super::glob(&roots.local.join("hooks"), ".last");
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        let last = text.lines().last().unwrap_or_default();
        let column: Vec<&str> = last.split('\t').collect();
        let field = |at: usize| column.get(at).copied().unwrap_or("");
        say!(
            ctx,
            "  {} | {} | {} | {}",
            field(0),
            field(1),
            field(2),
            field(3)
        );
    }
    if files.is_empty() {
        say!(ctx, "  no hook has run in this worktree yet");
    }
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r101__the_section_counts_the_registrations_of_this_machine() {
        let (_, printed) = super::super::tests::printed(section);
        let lines: Vec<&str> = printed.lines().collect();
        assert!(lines[0].starts_with("hooks registered in "));
        assert!(
            lines.iter().any(|line| line.starts_with("  registered: ")),
            "no count line:\n{printed}"
        );
        assert!(lines.contains(&"hook last results (event | exit | at | note):"));
    }
}
