#![allow(dead_code)]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct Scratch(pub PathBuf);

impl Scratch {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "dstack-mode-settings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        let scratch = Self(fs::canonicalize(path).unwrap());
        scratch.write(
            "deps.tsv",
            "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n",
        );
        scratch
    }

    pub fn init(&self) {
        self.ok(&["init"]);
    }

    pub fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dstack"))
            .current_dir(&self.0)
            .env("DSTACK_ROOT", &self.0)
            .env(
                "DSTACK_HOME",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"),
            )
            .env("DSTACK_DEPS", self.0.join("deps.tsv"))
            .env("CLAUDE_CODE_SESSION_ID", "mode-settings-test")
            .args(args)
            .output()
            .unwrap()
    }

    pub fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    pub fn json(&self, args: &[&str]) -> serde_json::Value {
        serde_json::from_str(&self.ok(args)).expect("structured mode output")
    }

    pub fn write(&self, relative: &str, text: &str) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.0.join(relative)).unwrap()
    }

    pub fn project(&self) -> PathBuf {
        self.0.join(".dstack/project/mode.json")
    }

    pub fn run_fixture(&self, id: &str, mode: Option<&str>, current: bool) -> PathBuf {
        let dir = self.0.join(".dstack/runs").join(id);
        self.write(
            &format!(".dstack/runs/{id}/meta.tsv"),
            "status\topen\nowner_session\tfixture\n",
        );
        if let Some(mode) = mode {
            fs::write(dir.join("mode.json"), mode).unwrap();
        }
        if current {
            self.write(".dstack/local/CURRENT", &format!("{id}\n"));
        }
        dir
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn tree(path: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(root: &Path, dir: &Path, rows: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, rows);
            } else {
                rows.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                ));
            }
        }
    }
    let mut rows = Vec::new();
    visit(path, path, &mut rows);
    rows.sort();
    rows
}
