// core/meta.rs
// meta.tsv of a run: the key/value reader and writer plus the owner heartbeat.

use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, epoch_now, read_bytes, utc_now, utc_to_epoch};

/// The rows of a meta.tsv as raw bytes. The shell's awk works on bytes, so a value that is not
/// valid UTF-8 is carried through instead of failing the parse.
fn rows(bytes: &[u8]) -> Vec<&[u8]> {
    let mut rows: Vec<&[u8]> = bytes.split(|byte| *byte == b'\n').collect();
    if rows.last().map(|last| last.is_empty()).unwrap_or(false) {
        rows.pop();
    }
    rows
}

fn field(row: &[u8], at: usize) -> &[u8] {
    row.split(|byte| *byte == b'\t').nth(at).unwrap_or(b"")
}

/// meta_get(): the value of the first row with this key, absent rows read as None. Only a table
/// that is not there is an absent row; one that cannot be read is a cannot-decide (D-12), because
/// every verdict computed from a missing status, owner or branch would be computed from air.
pub fn meta_get(dir: &Path, key: &str) -> Result<Option<String>> {
    let path = dir.join("meta.tsv");
    let bytes = match read_bytes(&path)? {
        Some(bytes) => bytes,
        None => return Ok(None),
    };
    for row in rows(&bytes) {
        if field(row, 0) == key.as_bytes() {
            return Ok(Some(String::from_utf8_lossy(field(row, 1)).into_owned()));
        }
    }
    Ok(None)
}

/// meta_set(): every row of the key is dropped and one row is appended, written atomically. Only
/// a missing file is an empty table — a table that exists but cannot be read stops the write,
/// because rewriting it would drop every row it holds.
pub fn meta_set(dir: &Path, key: &str, value: &str) -> Result<()> {
    let path = dir.join("meta.tsv");
    let existing = read_bytes(&path)?.unwrap_or_default();
    let mut kept: Vec<u8> = Vec::new();
    for row in rows(&existing) {
        if field(row, 0) == key.as_bytes() {
            continue;
        }
        kept.extend_from_slice(row);
        kept.push(b'\n');
    }
    kept.extend_from_slice(key.as_bytes());
    kept.push(b'\t');
    kept.extend_from_slice(value.as_bytes());
    kept.push(b'\n');
    atomic_write(&path, &kept)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

/// touch_owner(): who is driving this run right now (R31).
pub fn touch_owner(dir: &Path, parent_pid: u32, session_id: &str) -> Result<()> {
    meta_set(dir, "owner_pid", &parent_pid.to_string())?;
    meta_set(dir, "owner_session", session_id)?;
    meta_set(dir, "owner_ts", &utc_now())
}

/// Ordinary target resolution may renew only its existing, nonempty session owner.
/// Claiming an unowned or foreign run remains explicit in run new/adopt and handoff resume.
pub fn refresh_owner(dir: &Path, parent_pid: u32, session_id: &str) -> Result<()> {
    if !session_id.trim().is_empty()
        && meta_get(dir, "owner_session")?.as_deref() == Some(session_id)
    {
        touch_owner(dir, parent_pid, session_id)?;
    }
    Ok(())
}

/// True when the owner heartbeat is older than 600 s, unparseable or absent (R31).
pub fn owner_is_stale(dir: &Path) -> Result<bool> {
    let stamp = match meta_get(dir, "owner_ts")? {
        Some(stamp) if !stamp.is_empty() => stamp,
        _ => return Ok(true),
    };
    Ok(match utc_to_epoch(&stamp) {
        Some(epoch) => epoch_now() - epoch > 600,
        None => true,
    })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dstack-meta-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn r13__set_replaces_every_row_of_the_key() {
        let dir = scratch("set");
        meta_set(&dir, "status", "open").expect("write");
        meta_set(&dir, "slug", "sandbox").expect("write");
        meta_set(&dir, "status", "closed").expect("rewrite");
        assert_eq!(meta_get(&dir, "status").expect("read").as_deref(), Some("closed"));
        assert_eq!(meta_get(&dir, "slug").expect("read").as_deref(), Some("sandbox"));
        assert_eq!(meta_get(&dir, "missing").expect("read"), None);
        let text = std::fs::read_to_string(dir.join("meta.tsv")).expect("read");
        assert_eq!(text, "slug\tsandbox\nstatus\tclosed\n");
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13__meta_set_refuses_an_unreadable_table() {
        let dir = scratch("unreadable");
        // A directory where meta.tsv belongs: it exists, so it is not an empty table, and it
        // cannot be read either.
        std::fs::create_dir(dir.join("meta.tsv")).expect("directory in the way");
        let error = meta_set(&dir, "status", "open").expect_err("refused");
        assert_eq!(error.code(), 2);
        assert!(
            error
                .message()
                .starts_with(&format!("cannot read {}", dir.join("meta.tsv").display())),
            "unexpected message: {}",
            error.message()
        );
        assert!(dir.join("meta.tsv").is_dir(), "nothing was written over it");
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13__meta_tolerates_non_utf8_bytes_in_values() {
        let dir = scratch("bytes");
        let mut table: Vec<u8> = b"slug\t".to_vec();
        table.extend_from_slice(&[0xff, 0xfe]);
        table.extend_from_slice(b"\nstatus\topen\n");
        std::fs::write(dir.join("meta.tsv"), &table).expect("write");
        assert_eq!(meta_get(&dir, "status").expect("read").as_deref(), Some("open"));
        assert!(
            meta_get(&dir, "slug").expect("read").is_some(),
            "a value that is not UTF-8 still reads"
        );
        meta_set(&dir, "status", "closed").expect("rewrite");
        let after = std::fs::read(dir.join("meta.tsv")).expect("read");
        assert!(
            after.windows(2).any(|pair| pair == [0xff, 0xfe]),
            "the bytes of the other row survived the rewrite"
        );
        assert_eq!(meta_get(&dir, "status").expect("read").as_deref(), Some("closed"));
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13__owner_without_a_heartbeat_is_stale() {
        let dir = scratch("owner");
        assert!(owner_is_stale(&dir).expect("read"));
        touch_owner(&dir, 4242, "session-1").expect("write");
        assert!(!owner_is_stale(&dir).expect("read"));
        assert_eq!(meta_get(&dir, "owner_pid").expect("read").as_deref(), Some("4242"));
        assert_eq!(
            meta_get(&dir, "owner_session").expect("read").as_deref(),
            Some("session-1")
        );
        meta_set(&dir, "owner_ts", "2020-01-01T00:00:00Z").expect("write");
        assert!(owner_is_stale(&dir).expect("read"));
        std::fs::remove_dir_all(&dir).expect("clean up");
    }
}
