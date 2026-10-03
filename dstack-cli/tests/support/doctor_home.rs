// tests/support/doctor_home.rs
// `dstack doctor` run under a scratch home, so no test reads the settings.json of this machine.
// HOME is the only thing doctor resolves through it that a test can see: the deps probes are
// `command -v` lookups on PATH and every other section reads the repository, so only the hooks
// section moves.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub fn repo() -> PathBuf {
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
        .expect("the physical path of the repository")
}

/// The four dstack hooks exactly as install.sh registers them: the enforced settings themselves.
pub fn dstack_only() -> String {
    fs::read_to_string(repo().join("claude/settings.enforced.json"))
        .expect("claude/settings.enforced.json")
}

/// A fixture of the doctor-hooks checker, used as a whole settings.json.
pub fn fixture(name: &str) -> String {
    fs::read_to_string(repo().join("claude/lint/fixtures/doctor-hooks").join(name))
        .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

/// A home whose .claude/settings.json holds the given text; it goes away with the value.
pub struct ScratchHome(pub PathBuf);

impl ScratchHome {
    pub fn new(settings: &str) -> Self {
        let home = Self::without_settings();
        fs::write(home.settings(), settings).expect("write settings.json");
        home
    }

    /// A home whose .claude directory holds no settings.json at all.
    pub fn without_settings() -> Self {
        let path = std::env::temp_dir().join(format!(
            "dstack-doctor-home-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join(".claude")).expect("a scratch home");
        Self(fs::canonicalize(path).expect("the physical path of the scratch home"))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn settings(&self) -> PathBuf {
        self.0.join(".claude/settings.json")
    }

    /// `dstack doctor` in the repository with HOME pointed here; the caller adds what it needs.
    pub fn doctor(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_dstack"));
        command.arg("doctor").current_dir(repo()).env("HOME", self.path());
        command
    }
}

impl Drop for ScratchHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
