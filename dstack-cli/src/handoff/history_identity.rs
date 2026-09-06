// Validate only visible cwd metadata; cache a bounded set of canonical Git roots.
use crate::core::error::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MAX_CWDS: usize = 64;
const MAX_PATH: usize = 4096;
const MAX_WARNING: usize = 8192;

pub(super) struct Identity<'a> {
    selected: &'a Path,
    selected_seen: bool,
    common: Option<PathBuf>,
    spellings: BTreeSet<String>,
    roots: BTreeMap<PathBuf, usize>,
}

impl<'a> Identity<'a> {
    pub(super) fn new(selected: &'a Path) -> Self {
        Self {
            selected,
            selected_seen: false,
            common: None,
            spellings: BTreeSet::new(),
            roots: BTreeMap::new(),
        }
    }

    pub(super) fn check(&mut self, text: &str, line: usize) -> Result<()> {
        let bad = || {
            Error::failed(format!(
                "history line {line}: history worktree mismatch or unavailable cwd"
            ))
        };
        if self.spellings.contains(text) {
            return Ok(());
        }
        if self.spellings.len() == MAX_CWDS || text.len() > MAX_PATH {
            return Err(Error::failed(format!(
                "history line {line}: history cwd checks exceed their bound"
            )));
        }
        let path = Path::new(text);
        if !path.is_absolute() {
            return Err(bad());
        }
        let cwd = path.canonicalize().map_err(|_| bad())?;
        if !cwd.is_dir() {
            return Err(bad());
        }
        if cwd == self.selected {
            self.selected_seen = true;
        } else if !self.roots.contains_key(&cwd) {
            if self.common.is_none() {
                self.common = Some(git_common(self.selected).ok_or_else(bad)?);
            }
            if git_common(&cwd).as_ref() != self.common.as_ref() {
                return Err(bad());
            }
        }
        self.roots.entry(cwd).or_insert(line);
        self.spellings.insert(text.to_owned());
        Ok(())
    }

    pub(super) fn finish(self) -> Result<Option<String>> {
        if !self.selected_seen {
            return Err(Error::failed("history never visits the selected worktree"));
        }
        if self.roots.len() <= 1 {
            return Ok(None);
        }
        let mut warning = String::from(
            "history cwd moved within the same Git repository; first original references:",
        );
        let mut omitted = 0;
        for (path, line) in self.roots {
            let entry = format!(" {} (history:{line});", path.display());
            if warning.len() + entry.len() + 80 <= MAX_WARNING {
                warning.push_str(&entry);
            } else {
                omitted += 1;
            }
        }
        if omitted > 0 {
            warning.push_str(&format!(" omitted {omitted} additional cwd paths"));
        }
        Ok(Some(warning))
    }
}

fn git_common(tree: &Path) -> Option<PathBuf> {
    let mut command = Command::new("git");
    command
        .args([
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
        ])
        .arg("-C")
        .arg(tree)
        .args(["rev-parse", "--show-toplevel", "--git-common-dir"])
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for key in [
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        command.env_remove(key);
    }
    let mut child = command.spawn().ok()?;
    let mut bytes = Vec::new();
    let read = child.stdout.take()?.take((MAX_PATH * 2 + 3) as u64).read_to_end(&mut bytes);
    if read.is_err() || bytes.len() > MAX_PATH * 2 + 2 {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    if !child.wait().ok()?.success() {
        return None;
    }
    let output = String::from_utf8(bytes).ok()?;
    let mut lines = output.strip_suffix('\n')?.split('\n');
    let top = Path::new(lines.next()?);
    let common = lines.next()?;
    if lines.next().is_some() || top.canonicalize().ok()?.as_path() != tree {
        return None;
    }
    tree.join(common).canonicalize().ok()
}
