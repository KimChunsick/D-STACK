// An opt-in review range proved by every completed Task, never a caller-chosen base.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::core::error::{Error, Result};
use crate::core::paths::valid_rel_path;
use crate::store::plan::Plan;
use crate::store::plan_ids::path_within;

use super::emit_diff::Counts;

pub struct CommittedRange {
    pub base: String,
    pub head: String,
    // Oldest first, independently of Task creation order; both recorded and resolved identity.
    tasks: Vec<(String, String, String)>,
}

impl CommittedRange {
    pub fn derive(wt: &Path, plan: &Plan) -> Result<Self> {
        if plan.tasks.is_empty() {
            fail!("committed review needs at least one completed Task");
        }
        let top = text(wt, &["rev-parse", "--show-toplevel"])?;
        if std::fs::canonicalize(wt).ok() != std::fs::canonicalize(&top).ok() {
            fail!(
                "committed review worktree must be the repository root: {}",
                wt.display()
            );
        }
        if text(wt, &["rev-parse", "--is-shallow-repository"])? != "false" {
            fail!("committed review refuses shallow history; the complete ancestry is required");
        }
        let declarations = declarations(plan)?;
        let mut records = BTreeMap::new();
        let mut ids = BTreeSet::new();
        for task in &plan.tasks {
            if task.commit.is_empty() || task.done_at.is_empty() {
                fail!(
                    "committed review needs a completion record and commit for Task {}",
                    task.id
                );
            }
            if !ids.insert(&task.id) {
                fail!("committed review has duplicate Task id: {}", task.id);
            }
            // task done historically accepted revision expressions. Mutable refs cannot prove
            // a legacy Task's immutable identity, but unambiguous abbreviated object IDs can.
            if !(4..=64).contains(&task.commit.len())
                || !task.commit.bytes().all(|b| b.is_ascii_hexdigit())
            {
                fail!(
                    "committed review needs an immutable commit ID for Task {}: {}",
                    task.id,
                    task.commit
                );
            }
            // Disambiguate object IDs directly: rev-parse <hex> can instead select a branch
            // whose name looks like an abbreviated hash.
            let candidates = text(
                wt,
                &[
                    "rev-parse",
                    &format!("--disambiguate={}", task.commit.to_ascii_lowercase()),
                ],
            )?;
            let objects: Vec<_> = candidates.lines().collect();
            if objects.len() != 1 {
                fail!(
                    "committed review commit ID is missing or ambiguous for Task {}: {}",
                    task.id,
                    task.commit
                );
            }
            let sha = objects[0].to_owned();
            if text(wt, &["cat-file", "-t", &sha])? != "commit" {
                fail!(
                    "committed review recorded object is not a commit for Task {}: {}",
                    task.id,
                    task.commit
                );
            }
            if records
                .insert(sha.clone(), (task.id.clone(), task.commit.clone()))
                .is_some()
            {
                fail!("committed review has duplicate recorded commit: {sha}");
            }
        }
        let head = text(wt, &["rev-parse", "--verify", "HEAD^{commit}"])?;
        if !records.contains_key(&head) {
            fail!("committed review HEAD is not a recorded Task end: {head}");
        }
        let mut cursor = head.clone();
        let mut tasks = Vec::new();
        while !records.is_empty() {
            let (id, recorded) = match records.remove(&cursor) {
                Some(task) => task,
                None => fail!(
                    "committed review has an unrecorded or omitted commit between Tasks: {cursor}"
                ),
            };
            // Read the object's own parent headers. Revision walkers can honor grafts; the
            // immutable object cannot, and --no-replace-objects also disables replacements.
            let object = git(wt, &["cat-file", "-p", &cursor])?;
            let object = String::from_utf8_lossy(&object);
            let parents: Vec<_> = object
                .lines()
                .take_while(|line| !line.is_empty())
                .filter_map(|line| line.strip_prefix("parent "))
                .collect();
            if parents.len() != 1 {
                fail!(
                    "committed review refuses root or merge Task commit: {cursor} ({} parents)",
                    parents.len()
                );
            }
            let parent = parents[0].to_owned();
            // Validate EACH delta, not just base..head: overwrite/revert cannot conceal an
            // undeclared intermediate path. Disable rename pairing so both endpoints count.
            for path in changed_paths(wt, &parent, &cursor)? {
                if !declarations
                    .iter()
                    .any(|declared| path_within(&path, declared))
                {
                    fail!("committed review Task {id} changes undeclared path: {path}");
                }
            }
            tasks.push((id, cursor, recorded));
            cursor = parent;
        }
        tasks.reverse();
        let range = Self {
            base: cursor,
            head,
            tasks,
        };
        range.require_current(wt)?;
        Ok(range)
    }

    fn require_current(&self, wt: &Path) -> Result<()> {
        if text(wt, &["rev-parse", "--verify", "HEAD^{commit}"])? != self.head {
            fail!("committed review HEAD changed while building the bundle");
        }
        // These index flags can suppress a real source edit from status; a sparse/assumed-clean
        // checkout does not establish the clean-worktree precondition of this mode.
        if git(wt, &["ls-files", "-v", "-z"])?
            .split(|b| *b == 0)
            .any(|entry| {
                entry
                    .first()
                    .is_some_and(|tag| tag.is_ascii_lowercase() || *tag == b'S')
            })
        {
            fail!("committed review refuses assume-unchanged or skip-worktree index entries");
        }
        if !git(
            wt,
            &[
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--ignore-submodules=none",
            ],
        )?
        .is_empty()
        {
            fail!(
                "committed review requires a clean worktree and index, including untracked files"
            );
        }
        Ok(())
    }

    pub fn emit(&self, out: &mut Vec<u8>, wt: &Path) -> Result<Counts> {
        out.extend_from_slice(
            format!(
                "mode: committed-plan\nbase: {}\nhead: {}\n",
                self.base, self.head
            )
            .as_bytes(),
        );
        for (id, sha, recorded) in &self.tasks {
            out.extend_from_slice(
                format!("task: {id} commit: {sha} recorded: {recorded}\n").as_bytes(),
            );
        }
        let paths = changed_paths(wt, &self.base, &self.head)?;
        for path in &paths {
            out.extend_from_slice(format!("--- file: {path}\n").as_bytes());
        }
        // All changed paths were proved above, so no pathspec can silently narrow the diff.
        // Binary patches and disabled external/textconv drivers preserve complete source bytes.
        out.extend_from_slice(&git(
            wt,
            &[
                "diff",
                "--binary",
                "--full-index",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                "--no-color",
                "--no-relative",
                "--unified=3",
                "--ignore-submodules=none",
                "--submodule=short",
                "--src-prefix=a/",
                "--dst-prefix=b/",
                &self.base,
                &self.head,
                "--",
            ],
        )?);
        if paths.is_empty() {
            out.extend_from_slice(b"(no changes against the base)\n");
        }
        self.require_current(wt)?;
        Ok(Counts { files: paths.len() })
    }
}

fn declarations(plan: &Plan) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for path in &plan.files {
        if !valid_rel_path(path) || path.chars().any(char::is_control) {
            fail!("committed review has an invalid declared path: {path:?}");
        }
        let normalized = path
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .collect::<Vec<_>>()
            .join("/");
        if normalized.is_empty() {
            fail!("committed review needs explicit repository-relative declarations");
        }
        files.push(normalized);
    }
    if files.is_empty() {
        fail!("committed review needs declared files");
    }
    Ok(files)
}

fn changed_paths(wt: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    let bytes = git(
        wt,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "-z",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            "--ignore-submodules=none",
            base,
            head,
            "--",
        ],
    )?;
    bytes
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|path| {
            let path = std::str::from_utf8(path).map_err(|_| {
                Error::failed("committed review cannot frame a non-UTF-8 changed path")
            })?;
            if path.chars().any(char::is_control) {
                fail!("committed review cannot frame a changed path with control characters");
            }
            Ok(path.to_owned())
        })
        .collect()
}

fn text(wt: &Path, args: &[&str]) -> Result<String> {
    String::from_utf8(git(wt, args)?)
        .map(|text| text.trim_end().to_owned())
        .map_err(|_| Error::cannot_decide("committed review: Git returned non-UTF-8 metadata"))
}

fn git(wt: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(wt)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::cannot_decide(format!("committed review cannot run Git: {e}")))?;
    if !output.status.success() {
        fail!(
            "committed review: git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}
