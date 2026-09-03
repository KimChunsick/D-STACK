// core/roots.rs
// Where the repository and the store are: resolve_home, resolve_roots and require_store.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::error::{Error, Result};
use crate::core::fsx::read_text;

/// resolve_home(): home is the repository's claude/ directory, repo its parent.
pub struct Home {
    pub home: PathBuf,
    pub repo: PathBuf,
}

impl Home {
    /// DSTACK_HOME wins; otherwise the real path of the binary is walked up until a directory
    /// holds both deps.tsv and claude/ (this also covers the cargo test binaries and the
    /// ~/.claude/bin/dstack symlink, which canonicalize resolves).
    pub fn resolve() -> Result<Home> {
        if let Ok(home) = std::env::var("DSTACK_HOME") {
            if !home.is_empty() {
                let home = std::fs::canonicalize(&home).map_err(|_| {
                    Error::cannot_decide(format!("DSTACK_HOME does not exist: {home}"))
                })?;
                let repo = home.parent().unwrap_or(&home).to_path_buf();
                return Ok(Home { home, repo });
            }
        }
        let exe = std::env::current_exe()
            .and_then(std::fs::canonicalize)
            .map_err(|e| Error::cannot_decide(format!("cannot locate this binary: {e}")))?;
        for dir in exe.ancestors().skip(1) {
            if dir.join("deps.tsv").is_file() && dir.join("claude").is_dir() {
                return Ok(Home {
                    home: dir.join("claude"),
                    repo: dir.to_path_buf(),
                });
            }
        }
        Err(Error::cannot_decide(format!(
            "cannot locate the D-STACK repository from {} (set DSTACK_HOME)",
            exe.display()
        )))
    }
}

/// resolve_roots(): MAIN_ROOT is the main worktree, so every worktree shares one store (R32).
#[derive(Clone)]
pub struct Roots {
    pub main_root: PathBuf,
    pub wt_root: PathBuf,
    pub store: PathBuf,
    pub runs: PathBuf,
    pub local: PathBuf,
    pub quick: PathBuf,
}

impl Roots {
    pub fn resolve() -> Result<Roots> {
        let (main_root, wt_root) = match std::env::var("DSTACK_ROOT") {
            Ok(root) if !root.is_empty() => {
                let main = std::fs::canonicalize(&root).map_err(|_| {
                    Error::cannot_decide(format!("DSTACK_ROOT does not exist: {root}"))
                })?;
                (main.clone(), main)
            }
            _ => {
                // The shell asked git twice; one rev-parse answers both in the order asked, and
                // that spawn is most of what a hook-path command costs (R10).
                let answer = git_out(None, &["rev-parse", "--show-toplevel", "--git-common-dir"])
                    .ok_or_else(|| {
                        Error::cannot_decide("not a git repository (run inside the target repo)")
                    })?;
                let mut lines = answer.lines();
                let top = lines.next().unwrap_or_default().to_string();
                let common = PathBuf::from(lines.next().unwrap_or_default());
                let common = if common.is_absolute() {
                    common
                } else {
                    std::env::current_dir()
                        .map_err(|_| Error::cannot_decide("not a git repository"))?
                        .join(common)
                };
                let main = std::fs::canonicalize(common.join(".."))
                    .map_err(|_| Error::cannot_decide("not a git repository"))?;
                (main, PathBuf::from(top))
            }
        };
        let store = main_root.join(".dstack");
        Ok(Roots {
            runs: store.join("runs"),
            local: wt_root.join(".dstack/local"),
            quick: wt_root.join(".dstack/quick"),
            store,
            main_root,
            wt_root,
        })
    }

    pub fn require_store(&self) -> Result<()> {
        if self.store.is_dir() && self.store.join("version").is_file() {
            return Ok(());
        }
        Err(Error::cannot_decide(format!(
            "no .dstack store at {} — run: dstack init",
            self.main_root.display()
        )))
    }

    /// current_run_id(): the run id CURRENT names in this worktree, if any. A CURRENT that is
    /// there and cannot be read is a cannot-decide (D-12): read as "no current run", it would
    /// let a verb answer about a worktree whose open run it simply could not see.
    pub fn current_run_id(&self) -> Result<Option<String>> {
        let id = match read_text(&self.local.join("CURRENT"))? {
            Some(text) => text.trim_end_matches('\n').to_string(),
            None => return Ok(None),
        };
        Ok(match id.is_empty() {
            true => None,
            false => Some(id),
        })
    }
}

/// git is the only executable the port spawns (R01). None when git fails or is not there.
pub fn git_out(cwd: Option<&Path>, args: &[&str]) -> Option<String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    let out = command.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    Some(text.trim_end_matches('\n').to_string())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r01__home_is_the_repository_of_this_test_binary() {
        let home = Home::resolve().expect("the test binary lives in the repository");
        assert!(home.repo.join("deps.tsv").is_file());
        assert_eq!(home.home, home.repo.join("claude"));
    }

    #[test]
    fn r01__git_is_the_only_process() {
        assert!(git_out(None, &["rev-parse", "--is-inside-work-tree"]).is_some());
        assert!(git_out(None, &["rev-parse", "--verify", "no-such-ref-here"]).is_none());
    }
}
