// tests/support/shell_ref.rs
// The shell implementation the port is compared against, taken out of git history: since P17 the
// tree carries no shell library and no bash dispatcher, and the tag `shell-final` names the last
// commit that did. Included with #[path] by every test that drives the reference, so the tree is
// extracted once per test process — cargo runs each test binary as its own process, and a
// directory named after that process is one no parallel binary can race for.
//
// Tests that use this reference are ignored unless --features shell-parity is enabled.
// Not every test that includes the module needs both halves of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The last commit that carries the shell implementation (P17).
const TAG: &str = "shell-final";

/// The extracted tree: `claude/` and `deps.tsv` of the tag, which is everything the dispatcher
/// resolves its home and its library from. It stays behind for the machine's temp sweeper, the
/// way the sandboxes of the other tests do.
fn tree() -> &'static Path {
    static ONCE: OnceLock<PathBuf> = OnceLock::new();
    ONCE.get_or_init(|| {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let dir = std::env::temp_dir().join(format!("dstack-shell-ref-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a directory for the shell reference");
        let tar = dir.join("tree.tar");
        let archived = Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["archive", "--format=tar", "-o"])
            .arg(&tar)
            .args([TAG, "claude", "deps.tsv"])
            .output()
            .expect("run git archive");
        assert!(
            archived.status.success(),
            "git archive {TAG} failed: {}",
            String::from_utf8_lossy(&archived.stderr)
        );
        let extracted = Command::new("tar")
            .arg("-xf")
            .arg(&tar)
            .arg("-C")
            .arg(&dir)
            .output()
            .expect("run tar");
        assert!(
            extracted.status.success(),
            "tar -xf failed: {}",
            String::from_utf8_lossy(&extracted.stderr)
        );
        std::fs::remove_file(&tar).expect("the archive is not part of the tree");
        std::fs::canonicalize(&dir).expect("the physical path of the shell reference")
    })
}

/// The reference dispatcher: `bash <dispatcher> <args>`.
pub fn dispatcher() -> PathBuf {
    tree().join("claude/bin/dstack")
}

/// The library directory the dispatcher resolves for itself, derived the same way it does.
pub fn lib() -> PathBuf {
    let lib = dispatcher()
        .parent()
        .expect("the dispatcher lives in a directory")
        .join("../lib");
    std::fs::canonicalize(&lib).expect("the physical path of the reference library")
}
