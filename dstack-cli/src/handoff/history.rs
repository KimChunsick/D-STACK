// Read only the selected transcript; preserve its original bytes' digest and line references.
use super::types::History;
use crate::core::error::{Error, Result};
use crate::core::mode::Provider;
use sha2::{Digest, Sha256};
use std::fs::{self, File, Metadata};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[path = "history_format.rs"]
mod format;

const MAX_FILE: u64 = 64 * 1024 * 1024;
const MAX_LINE: u64 = 2 * 1024 * 1024;
const MAX_SEARCH: usize = 100_000;

pub fn load(path: &Path, provider: Provider, session: &str, worktree: &Path) -> Result<History> {
    validate_session(session)?;
    let worktree = canonical_directory(worktree)?;
    let original = fs::canonicalize(path).map_err(|e| io_error("resolve history", path, e))?;
    let before = fs::metadata(&original).map_err(|e| io_error("inspect history", &original, e))?;
    if !before.is_file() {
        return Err(Error::failed("history must be a regular file"));
    }
    if before.len() > MAX_FILE {
        return Err(Error::failed(format!("oversized history file (limit {MAX_FILE} bytes)")));
    }
    let file = File::open(&original).map_err(|e| io_error("read history", &original, e))?;
    let result = read(&file, provider, session, &worktree, before.len());
    // Check even after a parse error: concurrent writes must not masquerade as corruption.
    unchanged(&file, path, &original, &before)?;
    let (records, warnings, omitted, sha256) = result?;
    Ok(History {
        provider,
        session: session.to_owned(),
        cwd: utf8(&worktree)?,
        path: utf8(&original)?,
        sha256,
        records,
        warnings,
        omitted,
    })
}

type ReadHistory = (Vec<super::types::HistoryRecord>, Vec<String>, usize, String);

fn read(
    file: &File,
    provider: Provider,
    session: &str,
    worktree: &Path,
    size: u64,
) -> Result<ReadHistory> {
    let mut reader = BufReader::new(file);
    let mut parser = format::Decoder::new(provider, session, worktree);
    let mut digest = Sha256::new();
    let (mut line, mut total) = (0, 0);
    let mut bytes = Vec::new();
    let mut warning = None;
    loop {
        bytes.clear();
        let n = Read::take(&mut reader, MAX_LINE + 1)
            .read_until(b'\n', &mut bytes)
            .map_err(|e| Error::failed(format!("cannot read history: {e}")))?;
        if n == 0 {
            break;
        }
        line += 1;
        total += n as u64;
        if total > size {
            return Err(Error::failed("history changed while reading"));
        }
        if n as u64 > MAX_LINE {
            return Err(Error::failed(format!(
                "oversized history line {line} (limit {MAX_LINE} bytes)"
            )));
        }
        digest.update(&bytes);
        match serde_json::from_slice(&bytes) {
            Ok(value) => parser.consume(&value, line)?,
            Err(error) if error.is_eof() && !bytes.ends_with(b"\n") => {
                warning = Some(format!(
                    "skipped incomplete trailing history JSONL record at line {line}"
                ));
            }
            Err(error) => {
                return Err(Error::failed(format!("malformed history JSONL line {line}: {error}")))
            }
        }
    }
    if total != size {
        return Err(Error::failed("history changed while reading"));
    }
    let (records, mut warnings, omitted) = parser.finish()?;
    if let Some(warning) = warning {
        warnings.push(warning);
    }
    Ok((records, warnings, omitted, format!("{:x}", digest.finalize())))
}

fn unchanged(file: &File, supplied: &Path, original: &Path, before: &Metadata) -> Result<()> {
    let stable = file.metadata().ok().filter(|after| same_file(before, after)).is_some()
        && fs::metadata(original).ok().filter(|after| same_file(before, after)).is_some()
        && fs::canonicalize(supplied).ok().as_deref() == Some(original);
    if stable {
        Ok(())
    } else {
        Err(Error::failed("history changed while reading"))
    }
}

fn same_file(before: &Metadata, after: &Metadata) -> bool {
    let same = before.len() == after.len() && before.modified().ok() == after.modified().ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        same && before.dev() == after.dev()
            && before.ino() == after.ino()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec()
    }
    #[cfg(not(unix))]
    {
        same && before.created().ok() == after.created().ok()
    }
}

/// Find by the exact session filename, never by age or transcript contents.
pub fn locate(provider: Provider, session: &str, worktree: &Path, home: &Path) -> Result<PathBuf> {
    validate_session(session)?;
    let cwd = canonical_directory(worktree)?;
    let path = match provider {
        Provider::Claude => {
            let encoded: String = utf8(&cwd)?
                .encode_utf16()
                .map(|c| {
                    if c < 128 && (c as u8).is_ascii_alphanumeric() {
                        c as u8 as char
                    } else {
                        '-'
                    }
                })
                .collect();
            home.join(".claude/projects").join(encoded).join(format!("{session}.jsonl"))
        }
        Provider::Codex => {
            let root = home.join(".codex/sessions");
            let mut found = None;
            search(&root, 0, &format!("-{session}.jsonl"), &mut found, &mut 0)?;
            found.ok_or_else(|| {
                Error::failed(format!("missing Codex history for session {session}"))
            })?
        }
    };
    let canonical =
        fs::canonicalize(&path).map_err(|e| io_error("resolve history candidate", &path, e))?;
    if !fs::metadata(&canonical)
        .map_err(|e| io_error("inspect history candidate", &path, e))?
        .is_file()
    {
        return Err(Error::failed(format!(
            "history candidate is not a regular file: {}",
            path.display()
        )));
    }
    File::open(&canonical).map_err(|e| io_error("read history candidate", &path, e))?;
    Ok(canonical)
}

fn search(
    dir: &Path,
    depth: usize,
    suffix: &str,
    found: &mut Option<PathBuf>,
    count: &mut usize,
) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| io_error("read history directory", dir, e))? {
        let entry = entry.map_err(|e| io_error("read history directory entry", dir, e))?;
        *count += 1;
        if *count > MAX_SEARCH {
            return Err(Error::failed("history filename search exceeded its bound"));
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if depth < 3 {
            if !date_part(name, depth) {
                continue;
            }
            let path = entry.path();
            if !fs::metadata(&path)
                .map_err(|e| io_error("inspect history directory", &path, e))?
                .is_dir()
            {
                return Err(Error::failed(format!(
                    "invalid history date directory: {}",
                    path.display()
                )));
            }
            search(&path, depth + 1, suffix, found, count)?;
        } else if name.starts_with("rollout-") && name.ends_with(suffix) {
            if found.is_some() {
                return Err(Error::failed(format!(
                    "ambiguous Codex history filename suffix {suffix}"
                )));
            }
            *found = Some(entry.path());
        }
    }
    Ok(())
}

fn date_part(name: &str, depth: usize) -> bool {
    if name.len() != if depth == 0 { 4 } else { 2 } || !name.bytes().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let number = name.parse::<u16>().unwrap_or(0);
    match depth {
        0 => number > 0,
        1 => (1..=12).contains(&number),
        _ => (1..=31).contains(&number),
    }
}

fn validate_session(session: &str) -> Result<()> {
    if session.is_empty()
        || session.len() > 256
        || session == "."
        || session == ".."
        || !session.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
    {
        return Err(Error::failed("history session must be an exact nonempty plain session id"));
    }
    Ok(())
}

fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let canonical =
        fs::canonicalize(path).map_err(|e| io_error("resolve history worktree", path, e))?;
    if !canonical.is_dir() {
        return Err(Error::failed("history worktree must be a directory"));
    }
    Ok(canonical)
}

fn utf8(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::failed("history path must be valid UTF-8"))
}

fn io_error(action: &str, path: &Path, error: std::io::Error) -> Error {
    Error::failed(format!("cannot {action} {}: {error}", path.display()))
}
