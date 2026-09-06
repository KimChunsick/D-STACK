// Actual Git evidence, with bounded pipes, NUL paths, no textconv or external diff drivers.
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

use crate::core::error::{Error, Result};
use crate::core::fsx::sha256_bytes;
use crate::handoff::types::Document;
use super::{canonical, cannot, read_bytes, DOCUMENT_LIMIT, ENTRY_LIMIT};

const FILE_LIMIT: usize = 8 * 1024 * 1024;
const CONTENT_LIMIT: usize = 32 * 1024 * 1024;

pub(super) fn identity(tree: &Path) -> Result<PathBuf> {
    let top = line(tree, &["rev-parse", "--show-toplevel"])?;
    if canonical(Path::new(&top))? != tree {
        return Err(Error::failed(format!("handoff path is not the actual Git worktree root: {}", tree.display())));
    }
    let common = line(tree, &["rev-parse", "--git-common-dir"])?;
    canonical(&tree.join(common))
}

pub(super) fn capture(tree: &Path, index: usize) -> Result<Document> {
    let head = line(tree, &["rev-parse", "--verify", "HEAD"])?;
    let branch = optional(tree, &["symbolic-ref", "--quiet", "HEAD"])?
        .map(|b| decode(b, tree)).transpose()?.unwrap_or_else(|| "[detached]".into());
    let status = output(tree, &["status", "--porcelain=v1", "-z", "--untracked-files=all", "--ignore-submodules=none", "--", ".", ":(exclude).dstack"])?;
    let flags = output(tree, &["ls-files", "-v", "-z", "--", ".", ":(exclude).dstack"])?;
    if flags.split(|b| *b == 0).filter_map(|row| row.first()).any(|b| b.is_ascii_lowercase() || *b == b'S') {
        return Err(Error::failed("handoff cannot inspect assume-unchanged or skip-worktree entries; expose those files before preparing"));
    }
    let staged = output(tree, &["ls-files", "--stage", "-z", "--", ".", ":(exclude).dstack"])?;
    let index_path = tree.join(line(tree, &["rev-parse", "--git-path", "index"])?);
    let index_hash = read_bytes(&index_path, FILE_LIMIT)?.map(|bytes| sha256_bytes(&bytes));
    let diff = |cached| {
        let mut args = vec!["diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "--no-renames"];
        if cached { args.push("--cached"); }
        args.extend(["--", ".", ":(exclude).dstack"]);
        line(tree, &args)
    };
    let mut files = Vec::new();
    let mut total = 0;
    for path in changed_paths(&status)? {
        let full = tree.join(&path);
        let parent = full.parent().expect("a Git path has a parent");
        // A deleted path can have a deleted parent. Existing parents must stay in the tree.
        if parent.exists() && !canonical(parent)?.starts_with(tree) {
            return Err(Error::failed(format!("handoff changed path traverses an outside symlink: {path}")));
        }
        let meta = match fs::symlink_metadata(&full) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                files.push(json!({"path":path,"kind":"missing"})); continue;
            }
            Err(e) => return Err(cannot("inspect changed path", &full, e)),
            Ok(meta) => meta,
        };
        let (kind, bytes) = if meta.file_type().is_symlink() {
            let target = fs::read_link(&full).map_err(|e| cannot("read symlink", &full, e))?;
            let text = target.to_str().ok_or_else(|| Error::cannot_decide("handoff symlink target is not UTF-8"))?;
            ("symlink", text.as_bytes().to_vec())
        } else {
            ("file", read_bytes(&full, FILE_LIMIT)?.ok_or_else(|| Error::failed(format!("Git file disappeared during handoff: {path}")))?)
        };
        total += bytes.len();
        if total > CONTENT_LIMIT {
            return Err(Error::failed(format!("handoff changed contents exceed {CONTENT_LIMIT} bytes")));
        }
        files.push(json!({"path":path,"kind":kind,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),
            "permissions":meta.permissions().mode(),"text":std::str::from_utf8(&bytes).ok(),
            "binary":std::str::from_utf8(&bytes).is_err()}));
    }
    // HEAD and diffs carry the actionable state; hash the bounded stage listing for freshness.
    let text = json!({"worktree":tree,"head":head,"branch":branch,
        "status_porcelain_z":decode(status,tree)?,"index_entries_sha256":sha256_bytes(&staged),
        "index_entries_bytes":staged.len(),"index_entries_count":staged.iter().filter(|b| **b == 0).count(),
        "index_sha256":index_hash,"worktree_diff":diff(false)?,"index_diff":diff(true)?,"files":files}).to_string();
    Ok(Document { reference: format!("git:worktree:{index}"), path: tree.to_string_lossy().into_owned(), text })
}

fn changed_paths(status: &[u8]) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    let mut rows = status.split(|b| *b == 0).peekable();
    while let Some(row) = rows.next() {
        if row.is_empty() && rows.peek().is_none() { break; }
        if row.len() < 4 || row[2] != b' ' {
            return Err(Error::cannot_decide("invalid Git NUL status record during handoff"));
        }
        paths.insert(git_path(&row[3..])?);
        if row[..2].iter().any(|b| *b == b'R' || *b == b'C') {
            paths.insert(git_path(rows.next().ok_or_else(|| Error::cannot_decide("missing Git rename source"))?)?);
        }
        if paths.len() > ENTRY_LIMIT {
            return Err(Error::failed(format!("handoff changed files exceed {ENTRY_LIMIT} entries")));
        }
    }
    Ok(paths)
}

fn git_path(bytes: &[u8]) -> Result<String> {
    let path = std::str::from_utf8(bytes).map_err(|_| Error::cannot_decide("handoff Git path is not UTF-8"))?;
    if path.is_empty() || Path::new(path).components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(Error::cannot_decide("invalid relative Git path during handoff"));
    }
    Ok(path.to_owned())
}

pub(super) fn committed(tree: &Path, commit: &str) -> Result<Option<String>> {
    if !(7..=64).contains(&commit.len()) || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Ok(None);
    }
    let revision = format!("{commit}^{{commit}}");
    let Some(bytes) = optional(tree, &["rev-parse", "--verify", &revision])? else { return Ok(None); };
    let resolved = decode(bytes, tree)?.trim_end_matches('\n').to_owned();
    if optional(tree, &["merge-base", "--is-ancestor", &resolved, "HEAD"])?.is_none() {
        return Ok(None);
    }
    Ok(Some(resolved))
}

fn decode(bytes: Vec<u8>, tree: &Path) -> Result<String> {
    String::from_utf8(bytes).map_err(|e| cannot("decode Git output for", tree, e))
}

fn line(tree: &Path, args: &[&str]) -> Result<String> {
    let text = decode(output(tree, args)?, tree)?;
    Ok(text.strip_suffix('\n').unwrap_or(&text).to_owned())
}

fn output(tree: &Path, args: &[&str]) -> Result<Vec<u8>> {
    optional(tree, args)?.ok_or_else(|| Error::cannot_decide(format!("Git evidence command failed in {}: git {}", tree.display(), args.join(" "))))
}

fn optional(tree: &Path, args: &[&str]) -> Result<Option<Vec<u8>>> {
    let mut child = Command::new("git");
    child.args(["--no-pager", "--no-optional-locks", "-c", "core.fsmonitor=false", "-c", "core.untrackedCache=false"])
        .arg("-C").arg(tree).args(args).env("GIT_NO_REPLACE_OBJECTS", "1")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    for key in ["GIT_DIR", "GIT_COMMON_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES"] { child.env_remove(key); }
    let mut child = child.spawn().map_err(|e| cannot("launch Git in", tree, e))?;
    let out = child.stdout.take().expect("piped stdout");
    let err = child.stderr.take().expect("piped stderr");
    let (status, out, err) = std::thread::scope(|scope| {
        let stdout = scope.spawn(move || pipe(out, DOCUMENT_LIMIT));
        let stderr = scope.spawn(move || pipe(err, 64 * 1024));
        (child.wait(), stdout.join(), stderr.join())
    });
    let status = status.map_err(|e| cannot("wait for Git in", tree, e))?;
    let out = out.map_err(|_| Error::cannot_decide("Git stdout reader panicked"))?
        .map_err(|e| cannot("read Git stdout in", tree, e))?;
    let err = err.map_err(|_| Error::cannot_decide("Git stderr reader panicked"))?
        .map_err(|e| cannot("read Git stderr in", tree, e))?;
    if out.len() > DOCUMENT_LIMIT || err.len() > 64 * 1024 {
        return Err(Error::failed(format!("handoff Git output exceeds its byte limit: git {}", args.join(" "))));
    }
    if status.success() { Ok(Some(out)) } else { Ok(None) }
}

fn pipe(reader: impl Read, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub(super) fn worktree_from_document(doc: &Document) -> Result<PathBuf> {
    let value: Value = serde_json::from_str(&doc.text)
        .map_err(|e| Error::cannot_decide(format!("invalid handoff Git document: {e}")))?;
    let path = value.get("worktree").and_then(Value::as_str)
        .ok_or_else(|| Error::cannot_decide("handoff Git document lacks worktree"))?;
    canonical(Path::new(path))
}
