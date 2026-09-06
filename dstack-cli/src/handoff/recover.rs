// Explicit repair of a legacy query heartbeat; never a force-adopt or session impersonation.
use super::{history, packet, snapshot};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, sha256_bytes, utc_now, with_lock};
use crate::core::meta::meta_get;
use crate::core::mode::Provider;
use crate::core::roots::Roots;
use crate::core::target::Target;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn apply(
    ctx: &mut Context,
    roots: &Roots,
    target: &Target,
    host: Provider,
    source: &str,
    path: &Path,
    stopped: bool,
) -> Result<()> {
    if !stopped {
        return Err(Error::failed(
            "confirm the source session and every native worker are stopped with --source-stopped",
        ));
    }
    let _lock = with_lock(&roots.local)?;
    let receipt = target.dir.join("owner-recovery");
    if present(&receipt)? {
        return Err(Error::failed("prior owner recovery exists; inspect its receipt and actual metadata; automatic retry is refused"));
    }
    handoff_guard(&target.dir)?;
    let state = snapshot::collect(roots, target)?;
    if ctx.session_id.trim().is_empty()
        || ctx.session_id != state.owner_session
        || ctx.session_id.contains(['\t', '\n', '\r'])
    {
        return Err(Error::failed(
            "owner recovery requires the actual caller to equal the current saved owner_session",
        ));
    }
    if source == ctx.session_id {
        return Err(Error::failed(
            "owner recovery source must differ from the actual caller",
        ));
    }
    if state.mode.main == host {
        return Err(Error::failed(
            "owner recovery requires a saved main different from the actual --host",
        ));
    }
    let stored = meta_get(&target.dir, "transcript_path")?
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| Error::failed("owner recovery requires saved transcript_path provenance"))?;
    let source_path = anchored_path(Path::new(&stored), path, state.mode.main, source)?;
    let history = history::load(
        &source_path,
        state.mode.main,
        source,
        Path::new(&state.worktree),
    )?;
    complete_history(&history)?;
    snapshot::check_idle(roots, &state)?;
    let meta_path = target.dir.join("meta.tsv");
    let original = packet::read(&meta_path, 64 * 1024)?;
    // The legacy caller's PID/time cannot describe the stopped source. Remove those fields;
    // preserve all other rows byte-for-byte except verified source owner/transcript provenance.
    let mut proposed = original
        .split_inclusive('\n')
        .filter(|line| {
            !matches!(
                line.split('\t').next(),
                Some("owner_session" | "owner_pid" | "owner_ts" | "transcript_path")
            )
        })
        .collect::<String>();
    if !proposed.is_empty() && !proposed.ends_with('\n') {
        proposed.push('\n');
    }
    proposed.push_str(&format!(
        "owner_session\t{source}\ntranscript_path\t{}\n",
        history.path
    ));
    let intent = serde_json::json!({
        "version": 1, "run": target.id, "worktree": state.worktree,
        "caller_session": ctx.session_id, "caller_host": host, "at": utc_now(),
        "source_session": source, "source_provider": state.mode.main,
        "source_path": history.path, "source_sha256": history.sha256, "source_stopped": true,
        "original_meta": original, "original_sha256": sha256_bytes(original.as_bytes()),
        "proposed_meta": proposed, "proposed_sha256": sha256_bytes(proposed.as_bytes()),
        "snapshot_fingerprint": state.fingerprint,
    });
    // Validate twice before reserving the journal, then once more after its durable intent.
    let fresh = || -> Result<()> {
        handoff_guard(&target.dir)?;
        if anchored_path(Path::new(&stored), path, state.mode.main, source)? != source_path {
            return Err(Error::failed(
                "source history path changed before owner recovery",
            ));
        }
        let reread = history::load(
            &source_path,
            state.mode.main,
            source,
            Path::new(&state.worktree),
        )?;
        complete_history(&reread)?;
        if reread.sha256 != history.sha256 {
            return Err(Error::failed(
                "source history changed before owner recovery",
            ));
        }
        snapshot::verify(&state, roots, target)?;
        snapshot::check_idle(roots, &state)?;
        if packet::read(&meta_path, 64 * 1024)? != original {
            return Err(Error::failed("run metadata changed before owner recovery"));
        }
        Ok(())
    };
    fresh()?;
    // create_dir is exclusive. Even a crash before intent.json leaves an uncertainty guard.
    fs::create_dir(&receipt).map_err(packet::io)?;
    sync_dir(&target.dir)?;
    packet::write_new(
        &receipt.join("intent.json"),
        &serde_json::to_string_pretty(&intent).map_err(packet::io)?,
    )?;
    sync_dir(&receipt)?;
    fresh()?;
    atomic_write(&meta_path, proposed.as_bytes()).map_err(packet::io)?;
    fs::File::open(&meta_path)
        .and_then(|f| f.sync_all())
        .map_err(packet::io)?;
    sync_dir(&target.dir)?;
    packet::write_new(
        &receipt.join("completed"),
        &format!(
            "at={}\nmeta_sha256={}\n",
            utc_now(),
            sha256_bytes(proposed.as_bytes())
        ),
    )?;
    sync_dir(&receipt)?;
    ctx.out.say(&format!(
        "source owner recovered: {} (run {}); receipt: {}",
        source,
        target.id,
        receipt.display()
    ));
    Ok(())
}

fn anchored_path(
    stored: &Path,
    supplied: &Path,
    provider: Provider,
    source: &str,
) -> Result<PathBuf> {
    let name = stored.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let matches = match provider {
        Provider::Claude => name == format!("{source}.jsonl"),
        Provider::Codex => {
            name.starts_with("rollout-")
                && name.ends_with(&format!("-{source}.jsonl"))
                && name.len() > "rollout-".len() + source.len() + ".jsonl".len() + 1
        }
    };
    if !matches {
        return Err(Error::failed(
            "saved transcript filename does not encode the requested source session",
        ));
    }
    let supplied = fs::canonicalize(supplied).map_err(packet::io)?;
    if supplied.file_name() != stored.file_name() {
        return Err(Error::failed(
            "supplied history filename must match saved transcript filename",
        ));
    }
    match fs::canonicalize(stored) {
        Ok(original) if original != supplied => {
            return Err(Error::failed(
                "history must equal the existing stored transcript path",
            ))
        }
        Ok(_) => (),
        Err(e) if e.kind() == ErrorKind::NotFound => (),
        Err(e) => return Err(packet::io(e)),
    }
    if supplied
        .to_str()
        .is_none_or(|s| s.contains(['\t', '\n', '\r']))
    {
        return Err(Error::failed(
            "source history path must be valid UTF-8 without metadata separators",
        ));
    }
    Ok(supplied)
}
fn complete_history(history: &super::types::History) -> Result<()> {
    if history
        .warnings
        .iter()
        .any(|s| s.contains("incomplete trailing"))
    {
        return Err(Error::failed(
            "owner recovery refuses incomplete trailing history; verify the stopped source file",
        ));
    }
    Ok(())
}
fn handoff_guard(run: &Path) -> Result<()> {
    let dir = run.join("handoffs");
    if present(&dir)? {
        packet::regular_dir(&dir)?;
        if fs::read_dir(&dir)
            .map_err(packet::io)?
            .next()
            .transpose()
            .map_err(packet::io)?
            .is_some()
        {
            return Err(Error::failed("existing handoff attempt blocks owner recovery; inspect prepared, resuming or consumed receipts"));
        }
    }
    Ok(())
}
fn present(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(packet::io(e)),
    }
}
fn sync_dir(path: &Path) -> Result<()> {
    fs::File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(packet::io)
}
