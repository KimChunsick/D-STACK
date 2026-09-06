// Freeze bounded, read-only workflow and Git evidence; freshness never renews ownership.
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::fsx::sha256_bytes;
use crate::core::mode::Mode;
use crate::core::paths::is_plain_name;
use crate::core::roots::Roots;
use crate::core::target::{Target, TargetKind};
use super::types::{Document, Snapshot};

#[path = "git.rs"]
mod git;
#[path = "state.rs"]
mod state;

const DOCUMENT_LIMIT: usize = 2 * 1024 * 1024;
const SNAPSHOT_LIMIT: usize = 8 * 1024 * 1024;
const ENTRY_LIMIT: usize = 2048;

pub fn collect(roots: &Roots, target: &Target) -> Result<Snapshot> {
    if target.kind != TargetKind::Run || !is_plain_name(&target.id) {
        return Err(Error::failed("handoff requires a named run"));
    }
    let run = canonical(&target.dir)?;
    if run != canonical(&roots.runs.join(&target.id))?
        || !run.starts_with(canonical(&roots.store)?)
    {
        return Err(Error::failed("handoff run directory does not match the selected run"));
    }
    let worktree = canonical(&roots.wt_root)?;
    let meta = metadata(&run)?;
    let owner = meta.get("owner_session").filter(|s| !s.trim().is_empty())
        .ok_or_else(|| Error::failed("handoff requires a nonempty run owner_session"))?;
    if meta.get("id") != Some(&target.id)
        || meta.get("worktree").map(PathBuf::from).as_ref() != Some(&worktree)
    {
        return Err(Error::failed("handoff run/worktree identity does not match run metadata"));
    }
    if !matches!(meta.get("status").map(String::as_str), Some("open" | "paused")) {
        return Err(Error::failed("handoff requires an open or paused run"));
    }
    let mut documents = vec![Document {
        reference: "state:meta".into(), path: format!("{}:1", run.join("meta.tsv").display()),
        text: meta.iter().map(|(k, v)| format!("{k}\t{v}\n")).collect(),
    }];
    state::documents(&run, &mut documents)?;
    let mode = match documents.iter().find(|d| d.reference == "state:mode") {
        Some(doc) => serde_json::from_str::<Mode>(&doc.text)
            .map_err(|e| Error::cannot_decide(format!("invalid handoff run mode: {e}")))?,
        None => Mode::default(),
    };
    let plan = state::plan(&documents)?;
    let (trees, inventory) = state::worktrees(&worktree, &plan)?;
    let inventory_source = documents.iter().find(|d| d.reference == "state:plan").unwrap_or(&documents[0]).path.clone();
    push_document(&mut documents, Document { reference: "state:worktrees".into(), path: inventory_source,
        text: serde_json::Value::Array(inventory).to_string() })?;
    let main_root = canonical(&roots.main_root)?;
    let common = git::identity(&main_root)?;
    for (index, tree) in trees.iter().enumerate() {
        if git::identity(tree)? != common {
            return Err(Error::failed(format!("handoff worktree belongs to another repository: {}", tree.display())));
        }
        let current = local_directory(tree)?.join("CURRENT");
        if let Some(text) = read_text(&current, 4096)? {
            if !text.trim().is_empty() && text.trim() != target.id {
                return Err(Error::failed(format!("handoff worktree CURRENT names another run: {}", current.display())));
            }
            push_document(&mut documents, Document { reference: format!("state:current:{index}"), path: format!("{}:1", current.display()), text })?;
        }
        push_document(&mut documents, git::capture(tree, index)?)?;
    }
    let items = state::items(&run, &main_root, &worktree, &plan, &mut documents)?;
    let mut snapshot = Snapshot {
        run_id: target.id.clone(), worktree: worktree.to_string_lossy().into_owned(),
        mode, owner_session: owner.clone(), fingerprint: String::new(), documents, items,
    };
    snapshot.fingerprint = fingerprint(&snapshot)?;
    Ok(snapshot)
}

pub fn verify(saved: &Snapshot, roots: &Roots, target: &Target) -> Result<()> {
    let fresh = collect(roots, target)?;
    if saved.run_id != fresh.run_id || saved.worktree != fresh.worktree
        || saved.owner_session != fresh.owner_session || saved.mode != fresh.mode
        || saved.fingerprint != fresh.fingerprint || saved.fingerprint != fingerprint(saved)?
    {
        return Err(Error::failed("stale handoff: run, owner, mode, workflow evidence or Git state changed; prepare again"));
    }
    Ok(())
}

pub fn check_idle(roots: &Roots, saved: &Snapshot) -> Result<()> {
    if canonical(&roots.wt_root)?.to_string_lossy() != saved.worktree {
        return Err(Error::failed("handoff execution guard worktree mismatch"));
    }
    state::check_idle(saved)
}

fn fingerprint(snapshot: &Snapshot) -> Result<String> {
    let bytes = serde_json::to_vec(&(
        &snapshot.run_id, &snapshot.worktree, snapshot.mode, &snapshot.owner_session,
        &snapshot.documents, &snapshot.items,
    )).map_err(|e| Error::cannot_decide(format!("cannot encode handoff evidence: {e}")))?;
    if bytes.len() > SNAPSHOT_LIMIT {
        return Err(Error::failed(format!("handoff snapshot exceeds {SNAPSHOT_LIMIT} bytes; narrow the run before preparing")));
    }
    Ok(sha256_bytes(&bytes))
}

fn metadata(run: &Path) -> Result<BTreeMap<String, String>> {
    let text = required(&run.join("meta.tsv"), 64 * 1024)?;
    let mut meta = BTreeMap::new();
    for line in text.lines() {
        let (key, value) = line.split_once('\t')
            .filter(|(k, v)| !k.is_empty() && !v.contains('\t'))
            .ok_or_else(|| Error::cannot_decide("invalid handoff meta.tsv row"))?;
        if meta.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(Error::cannot_decide(format!("duplicate handoff metadata key: {key}")));
        }
    }
    // owner_pid is a transient CLI parent, not session identity. Heartbeat updates move rows.
    meta.remove("owner_ts");
    meta.remove("owner_pid");
    Ok(meta)
}

fn canonical(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(|e| cannot("resolve", path, e))
}

fn push_document(docs: &mut Vec<Document>, doc: Document) -> Result<()> {
    if docs.len() >= ENTRY_LIMIT || docs.iter().map(|d| d.text.len()).sum::<usize>() + doc.text.len() > SNAPSHOT_LIMIT {
        return Err(Error::failed("handoff documents exceed the entry or byte limit"));
    }
    if docs.iter().any(|d| d.reference == doc.reference) {
        return Err(Error::cannot_decide(format!("duplicate handoff document reference: {}", doc.reference)));
    }
    docs.push(doc);
    Ok(())
}

fn local_directory(tree: &Path) -> Result<PathBuf> {
    let local = tree.join(".dstack/local");
    for path in [tree.join(".dstack"), local.clone()] {
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == ErrorKind::NotFound => break,
            Err(e) => return Err(cannot("inspect local state", &path, e)),
            Ok(meta) if !meta.is_dir() => return Err(Error::cannot_decide(format!("handoff local state must be a real directory: {}", path.display()))),
            _ => (),
        }
    }
    Ok(local)
}

fn cannot(action: &str, path: &Path, error: impl std::fmt::Display) -> Error {
    Error::cannot_decide(format!("cannot {action} {}: {error}", path.display()))
}

fn read_bytes(path: &Path, limit: usize) -> Result<Option<Vec<u8>>> {
    let meta = match fs::symlink_metadata(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(cannot("inspect", path, e)),
        Ok(meta) => meta,
    };
    if !meta.is_file() || meta.len() > limit as u64 {
        return Err(Error::cannot_decide(format!("handoff requires a regular file of at most {limit} bytes: {}", path.display())));
    }
    let mut bytes = Vec::new();
    fs::File::open(path).map_err(|e| cannot("open", path, e))?
        .take(limit as u64 + 1).read_to_end(&mut bytes).map_err(|e| cannot("read", path, e))?;
    if bytes.len() > limit {
        return Err(Error::cannot_decide(format!("handoff read exceeds {limit} bytes: {}", path.display())));
    }
    Ok(Some(bytes))
}

fn read_text(path: &Path, limit: usize) -> Result<Option<String>> {
    read_bytes(path, limit)?.map(|b| String::from_utf8(b).map_err(|e| cannot("decode UTF-8", path, e))).transpose()
}

fn required(path: &Path, limit: usize) -> Result<String> {
    read_text(path, limit)?.ok_or_else(|| Error::failed(format!("required handoff evidence is missing: {}", path.display())))
}

fn entries(path: &Path) -> Result<Vec<PathBuf>> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(cannot("inspect", path, e)),
        Ok(meta) if !meta.is_dir() => return Err(Error::cannot_decide(format!("handoff requires a real directory: {}", path.display()))),
        _ => (),
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| cannot("list", path, e))? {
        paths.push(entry.map_err(|e| cannot("list", path, e))?.path());
        if paths.len() > ENTRY_LIMIT {
            return Err(Error::cannot_decide(format!("handoff directory exceeds {ENTRY_LIMIT} entries: {}", path.display())));
        }
    }
    paths.sort();
    Ok(paths)
}
