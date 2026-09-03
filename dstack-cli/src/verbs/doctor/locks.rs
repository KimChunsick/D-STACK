// verbs/doctor/locks.rs
// doctor section 5: what every worktree's CURRENT points at, and the locks nobody cleaned up (R31).

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::meta::{meta_get, owner_is_stale};
use crate::core::roots::git_out;

pub fn section(ctx: &mut Context) -> Result<bool> {
    say!(ctx, "locks and CURRENT (R31):");
    // The dispatcher does not resolve the roots for doctor (it must run on a machine with no
    // repository), so the sections that need one resolve it here and say "skipped" outside it.
    if git_out(None, &["rev-parse", "--git-common-dir"]).is_none() {
        say!(ctx, "  skipped: not in a git repository");
        return Ok(true);
    }
    let roots = ctx.roots()?;
    let listed = git_out(None, &["worktree", "list", "--porcelain"]).unwrap_or_default();
    let (mut n, mut bad, mut stale) = (0, 0, 0);
    for line in listed.lines() {
        let worktree = match line.strip_prefix("worktree ") {
            Some(worktree) => worktree,
            None => continue,
        };
        n += 1;
        let current = std::path::Path::new(worktree).join(".dstack/local/CURRENT");
        // `[ -s "$cur" ]`: a cleared CURRENT is an empty file, not a missing one.
        let named = std::fs::read_to_string(&current).unwrap_or_default();
        let id = named.trim_end_matches('\n').to_string();
        if named.is_empty() {
            say!(ctx, "  {worktree}: no current run");
        } else if !roots.runs.join(&id).is_dir() {
            bad += 1;
            say!(ctx, "  {worktree}: CURRENT → '{id}' but no such run — fix: (cd {worktree} && dstack run pause) or dstack run adopt <id>");
        } else {
            let dir = roots.runs.join(&id);
            let field = |key: &str| -> Result<String> { Ok(meta_get(&dir, key)?.unwrap_or_default()) };
            let status = field("status")?;
            if owner_is_stale(&dir)? {
                stale += 1;
                say!(ctx, "  {worktree}: run {id} [{status}] owner stale (>10 min) — auto-adopted by the next command, or: dstack run adopt {id}");
            } else {
                say!(
                    ctx,
                    "  {worktree}: run {id} [{status}] owner live ({}:{})",
                    field("owner_session")?,
                    field("owner_pid")?
                );
            }
        }
        let lock = std::path::Path::new(worktree).join(".dstack/local/lock");
        if lock.is_dir() {
            bad += 1;
            say!(
                ctx,
                "  {worktree}: stale lock — fix: rmdir {}",
                lock.display()
            );
        }
    }
    say!(
        ctx,
        "  worktrees: {n}, ownerless CURRENT or stale locks: {bad}, stale owners: {stale}"
    );
    Ok(bad == 0)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// This repository is a git checkout with worktrees, so the section reaches its count line.
    #[test]
    fn r31__every_worktree_of_this_repository_is_listed() {
        let (_, printed) = super::super::tests::printed(section);
        let lines: Vec<&str> = printed.lines().collect();
        assert_eq!(lines[0], "locks and CURRENT (R31):");
        let last = lines[lines.len() - 1];
        assert!(
            last.starts_with("  worktrees: ") && last.contains(", stale owners: "),
            "unexpected count line: {last}"
        );
        let counted: usize = last["  worktrees: ".len()..]
            .split(',')
            .next()
            .expect("the worktree count")
            .parse()
            .expect("a number");
        assert!(counted >= 1, "this checkout is a worktree of its own");
    }
}
