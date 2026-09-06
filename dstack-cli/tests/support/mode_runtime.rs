use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
pub struct Scratch(pub PathBuf);
impl Scratch {
    pub fn new(main: &str, sub: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "dstack-sub-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        for dir in [
            ".dstack/project",
            ".dstack/local",
            ".dstack/runs/sample",
            "bin",
            "trace",
            "work",
        ] {
            fs::create_dir_all(p.join(dir)).unwrap();
        }
        fs::write(p.join(".dstack/local/CURRENT"), "sample\n").unwrap();
        let mode = format!(r#"{{"main":"{main}","sub":"{sub}"}}"#);
        fs::write(p.join(".dstack/runs/sample/mode.json"), &mode).unwrap();
        fs::write(p.join(".dstack/project/mode.json"), &mode).unwrap();
        fs::write(p.join("context.md"), "Frozen R03 task context\n").unwrap();
        let script = r#"#!/bin/sh
printf '%s\n' "${0##*/}" > "$MODE_TRACE/provider"
printf 'call\n' >> "$MODE_TRACE/calls"
printf '%s\n' "$@" > "$MODE_TRACE/argv"
printf '%s\n' "$CLAUDECODE" "$CLAUDE_CODE_CHILD_SESSION" > "$MODE_TRACE/markers"
pwd -P > "$MODE_TRACE/cwd"
/bin/cat > "$MODE_TRACE/stdin"
if [ "$MODE_FAKE" = block ]; then
    : > "$MODE_TRACE/started"
    while [ ! -f "$MODE_TRACE/release" ]; do /bin/sleep 0.01; done
fi
if [ "$MODE_FAKE" = exit ]; then printf 'provider refused\n' >&2; exit 23; fi
if [ "$MODE_FAKE" = race ]; then printf keep > "$MODE_TRACE/../result.md"; fi
if [ "${0##*/}" = codex ]; then
    while [ "$#" -gt 0 ]; do
        if [ "$1" = -o ]; then
            shift
            case "$MODE_FAKE" in
                missing) ;;
                empty) : > "$1" ;;
                symlink) /bin/ln -s "$MODE_TRACE/../context.md" "$1" ;;
                *) printf 'R03 raw result\nVERDICT: PASS\n' > "$1" ;;
            esac
        fi
        shift
    done
    result='{"type":"turn.completed","usage":{"input_tokens":10,"cached_input_tokens":5,"output_tokens":1}}'
    case "$MODE_FAKE" in
        malformed) result='invalid-json' ;;
        failed) result='{"type":"turn.failed","error":{"message":"refused"}}' ;;
        no-completion) result='{"type":"turn.started"}' ;;
    esac
else
    result='{"type":"result","subtype":"success","is_error":false,"result":"R03 raw result\nVERDICT: PASS\n","usage":{"input_tokens":5,"cache_read_input_tokens":5,"cache_creation_input_tokens":0,"output_tokens":1}}'
    case "$MODE_FAKE" in
        malformed) result='invalid-json' ;;
        failed) result='{"type":"result","subtype":"success","is_error":true,"result":"failed"}' ;;
        missing) result='{"type":"result","subtype":"success","is_error":false}' ;;
        empty) result='{"type":"result","subtype":"success","is_error":false,"result":" "}' ;;
        subtype) result='{"type":"result","subtype":"error_max_turns","is_error":false,"result":"partial"}' ;;
        marker) result='{"type":"result","subtype":"success","result":"missing is_error"}' ;;
        duplicate-key) result='{"type":"result","subtype":"success","is_error":false,"result":"one","result":"two"}' ;;
    esac
fi
printf '%s\n' "$result"
if [ "$MODE_FAKE" = multiple ]; then printf '%s\n' "$result"; fi
"#;
        for name in ["codex", "claude"] {
            let path = p.join("bin").join(name);
            fs::write(&path, script).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        Self(p.canonicalize().unwrap())
    }
    pub fn scenario(&self, value: &str) {
        fs::write(self.0.join("scenario"), value).unwrap();
    }
    pub fn tree(&self) -> Vec<(PathBuf, Vec<u8>, std::time::SystemTime)> {
        fn visit(
            path: &std::path::Path,
            entries: &mut Vec<(PathBuf, Vec<u8>, std::time::SystemTime)>,
        ) {
            let meta = fs::symlink_metadata(path).unwrap();
            let bytes = if meta.is_file() {
                fs::read(path).unwrap()
            } else {
                Vec::new()
            };
            entries.push((path.into(), bytes, meta.modified().unwrap()));
            if meta.is_dir() {
                for child in fs::read_dir(path).unwrap() {
                    visit(&child.unwrap().path(), entries);
                }
            }
        }
        let mut entries = Vec::new();
        visit(&self.0, &mut entries);
        entries.sort();
        entries
    }
    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_dstack"));
        command
            .current_dir(&self.0)
            .env("DSTACK_ROOT", &self.0)
            .env(
                "DSTACK_HOME",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"),
            )
            .env("PATH", self.0.join("bin"))
            .env("MODE_TRACE", self.0.join("trace"))
            .env(
                "MODE_FAKE",
                fs::read_to_string(self.0.join("scenario")).unwrap_or_default(),
            );
        command
    }
    pub fn run(&self, role: &str, extra: &[&str]) -> Output {
        self.command()
            .args([
                "mode",
                "exec",
                "check",
                "--role",
                role,
                "--context",
                "context.md",
                "--output",
                "result.md",
                "--worktree",
                "work",
            ])
            .args(extra)
            .output()
            .unwrap()
    }
    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
