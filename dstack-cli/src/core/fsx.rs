// core/fsx.rs
// Files, hashes and UTC time: atomic_write, the mkdir lock, stat, sha256 and the timestamps.

use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::core::error::{Error, Result};

const STAMP: &[BorrowedFormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

pub fn utc_now() -> String {
    OffsetDateTime::now_utc().format(&STAMP).unwrap_or_default()
}

pub fn epoch_now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

/// utc_to_epoch(): None when the stamp does not parse, as the shell prints nothing.
pub fn utc_to_epoch(stamp: &str) -> Option<i64> {
    PrimitiveDateTime::parse(stamp, &STAMP)
        .ok()
        .map(|dt| dt.assume_utc().unix_timestamp())
}

pub fn file_mtime(path: &Path) -> Option<i64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(modified.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64)
}

pub fn file_size(path: &Path) -> Option<u64> {
    Some(fs::metadata(path).ok()?.len())
}

/// The store's read idiom (D-12): a file that is not there reads as the empty answer the shell's
/// `[ -f ] && awk … || true` also gives, and every other failure — a permission, a directory, a
/// byte sequence that is not UTF-8 — is a cannot-decide naming the path, never an empty result.
pub fn read_text(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::cannot_decide(format!(
            "cannot read {}: {e}",
            path.display()
        ))),
    }
}

/// read_text for a table the shell's awk reads as bytes: meta.tsv carries whatever a value held,
/// so a byte sequence that is not UTF-8 is data here, not a reason to refuse.
pub fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::cannot_decide(format!(
            "cannot read {}: {e}",
            path.display()
        ))),
    }
}

/// The file streams through the digest: an artifact can be large and none of it needs to be held.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// atomic_write(): through a temp file next to the target, so a crash never leaves a half file.
pub fn atomic_write(dst: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = dst.parent().unwrap_or_else(|| Path::new("."));
    let name = dst
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let (tmp, mut file) = create_temp(dir, &name)?;
    let written = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .and_then(|()| {
            drop(file);
            fs::rename(&tmp, dst)
        });
    if written.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    written
}

/// The temp file is opened with create_new (O_EXCL): an existing path — a file, or a symlink
/// someone planted in the directory — makes the open fail instead of being followed and
/// truncated. The nonce in the name means a lost race is just another attempt.
fn create_temp(dir: &Path, name: &str) -> std::io::Result<(PathBuf, fs::File)> {
    let mut taken = None;
    for _ in 0..16 {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.subsec_nanos())
            .unwrap_or(0);
        let tmp = dir.join(format!(".{name}.tmp.{}.{nonce}", std::process::id()));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
        {
            Ok(file) => return Ok((tmp, file)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => taken = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(taken
        .unwrap_or_else(|| std::io::Error::new(ErrorKind::AlreadyExists, "no free temp file name")))
}

/// The lock directory is removed on every path out, including an error return.
pub struct LockGuard {
    dir: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.dir);
    }
}

/// with_lock(): mkdir is atomic everywhere. 30 s of waiting, then a loud give-up — a lock that
/// waits forever would hang a hook, and a hook that hangs is a silent pass (R101).
pub fn with_lock(local: &Path) -> Result<LockGuard> {
    let lock = local.join("lock");
    fs::create_dir_all(local)
        .map_err(|e| Error::cannot_decide(format!("cannot create {}: {e}", local.display())))?;
    let mut waited = 0;
    loop {
        if fs::create_dir(&lock).is_ok() {
            return Ok(LockGuard { dir: lock });
        }
        waited += 1;
        if waited > 300 {
            return Err(Error::cannot_decide(format!(
                "lock held for 30s: {} (remove it if no dstack is running)",
                lock.display()
            )));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__stamps_round_trip() {
        let now = utc_now();
        assert_eq!(now.len(), 20);
        assert!(now.ends_with('Z'));
        assert_eq!(utc_to_epoch("1970-01-01T00:00:42Z"), Some(42));
        assert_eq!(utc_to_epoch("not a stamp"), None);
        let drift = (epoch_now() - utc_to_epoch(&now).expect("parses")).abs();
        assert!(drift <= 1, "utc_now and epoch_now disagree by {drift}s");
    }

    #[test]
    fn r13__sha256_matches_shasum() {
        assert_eq!(
            sha256_bytes(b"dstack\n"),
            "99a7d833854849ca38a138b6204f8b32745fcd5a4af90c149d629cda66ddd78a"
        );
    }

    #[test]
    fn r13__atomic_write_replaces_the_file() {
        let dir = std::env::temp_dir().join(format!("dstack-fsx-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let target = dir.join("meta.tsv");
        atomic_write(&target, b"one\n").expect("write");
        atomic_write(&target, b"two\n").expect("rewrite");
        assert_eq!(fs::read_to_string(&target).expect("read"), "two\n");
        assert_eq!(
            fs::read_dir(&dir).expect("list").count(),
            1,
            "no temp file left behind"
        );
        assert!(file_size(&target).expect("size") == 4);
        assert!(file_mtime(&target).is_some());
        fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13__sha256_file_streams_large_input() {
        let dir = std::env::temp_dir().join(format!("dstack-sha-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let target = dir.join("large.bin");
        let content: Vec<u8> = (0..3 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
        fs::write(&target, &content).expect("write");
        assert_eq!(
            sha256_file(&target).expect("hash the file"),
            sha256_bytes(&content)
        );
        fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13__atomic_write_opens_an_exclusive_temp_file() {
        let dir = std::env::temp_dir().join(format!("dstack-temp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        let (first, _first_file) = create_temp(&dir, "meta.tsv").expect("first temp file");
        let (second, _second_file) = create_temp(&dir, "meta.tsv").expect("second temp file");
        assert_ne!(first, second, "two writers never share a temp name");
        assert!(first.is_file() && second.is_file());
        // What create_new buys: an existing path is refused, so its target is never written.
        let planted = dir.join("planted");
        let victim = dir.join("victim");
        std::os::unix::fs::symlink(&victim, &planted).expect("plant a symlink");
        let refused = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&planted)
            .expect_err("an existing path is not opened");
        assert_eq!(refused.kind(), ErrorKind::AlreadyExists);
        assert!(!victim.exists(), "the symlink was not followed");
        fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn r13__lock_is_released_when_the_guard_drops() {
        let dir = std::env::temp_dir().join(format!("dstack-lock-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        {
            let _guard = with_lock(&dir).expect("lock");
            assert!(dir.join("lock").is_dir());
        }
        assert!(!dir.join("lock").exists());
        fs::remove_dir_all(&dir).expect("clean up");
    }
}
