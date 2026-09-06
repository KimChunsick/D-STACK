#![allow(dead_code)]
use dstack_cli::core::fsx::sha256_bytes;
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
pub struct Scratch(pub PathBuf);
impl Scratch {
    pub fn new(main: &str, sub: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "dstack-handoff-exec-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        let s = Self(path.canonicalize().unwrap());
        s.write(
            ".gitignore",
            ".dstack/\nbin/\ntrace/\nhistory.jsonl\nscenario\n",
        );
        s.write("work.txt", "initial\n");
        s.git(&["init", "-q"]);
        s.git(&["add", ".gitignore", "work.txt"]);
        s.git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ]);
        s.write(".dstack/version", "2\n");
        s.write(".dstack/local/CURRENT", "sample\n");
        let mode = json!({"main":main,"sub":sub}).to_string();
        s.write(".dstack/project/mode.json", &mode);
        s.write(".dstack/runs/sample/mode.json", &mode);
        s.write(
            ".dstack/runs/sample/meta.tsv",
            &format!(
                "id\tsample\nstatus\topen\nowner_session\tsource\nworktree\t{}\nbranch\tmain\n",
                s.0.display()
            ),
        );
        let request = "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\ndesign_review: auto\nreview: on\ncodex_effort: high\ne2e: cli\nunit_tests: on\nvisual: none\nkorean_polish: off\n---\n# 인계를 확인해요\n\n- [ ] **R08** 인계를 확인해요. — accept: cargo test r08_handoff_summary로 확인해요.\n";
        s.write(".dstack/runs/sample/request.md", request);
        s.write(
            ".dstack/runs/sample/request.approved",
            &format!(
                "sha256 {}  approved_at 2026-09-06T00:00:00Z\n",
                sha256_bytes(request.as_bytes())
            ),
        );
        s.write(".dstack/runs/sample/decisions.md", "| D | Decision | Affects | Status |\n|---|---|---|---|\n| D-01 | Keep work | R08 | answered |\n");
        s.write(
            ".dstack/runs/sample/cases.tsv",
            "R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote\n",
        );
        let plan = json!({"v":2,"milestones":[{"id":"M1","slug":"handoff","order":1}],"plans":[{"id":"P1","milestone":"M1","slug":"work","files":["work.txt"],"deps":[],"status":"in-progress","worktree":s.0,"started_at":"now","done_at":"","tasks":[{"id":"T1","slug":"work","covers":["R08"],"files":["work.txt"],"deps":[],"commit":"","done_at":""}]}]});
        s.write(".dstack/runs/sample/plan.json", &plan.to_string());
        let history = if main == "claude" {
            json!({"type":"user","sessionId":"source","cwd":s.0,"message":{"role":"user","content":"Continue unfinished work"}}).to_string()
        } else {
            format!(
                "{}\n{}",
                json!({"type":"session_meta","payload":{"id":"source","cwd":s.0}}),
                json!({"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Continue unfinished work"}]}})
            )
        };
        s.write("history.jsonl", &(history + "\n"));
        s.write("trace/summary", &Self::summary().to_string());
        s.install_provider();
        s
    }
    pub fn summary() -> serde_json::Value {
        json!({"completed":[],"active":[{"id":"T1","changes":"work.txt is unchanged; work remains open.","attempts":"No recorded test run yet.","blockers":"No verified blocker.","next_steps":["Read work.txt and run the acceptance command."],"refs":["task:T1"]}],"pending":[],"uncertainties":[],"next_actions":["Resume T1 after checking source is stopped."]})
    }
    fn install_provider(&self) {
        let script = r#"#!/bin/sh
printf '%s\n' "${0##*/}" >> "$HANDOFF_TRACE/calls"
printf '%s\n' "$@" > "$HANDOFF_TRACE/argv"
/bin/cat > "$HANDOFF_TRACE/stdin"
if [ "$HANDOFF_SCENARIO" = fail ]; then printf 'quota\n' >&2; exit 23; fi
if [ "$HANDOFF_SCENARIO" = mutate ]; then printf changed > work.txt; fi
if [ "${0##*/}" = codex ]; then
  while [ "$#" -gt 0 ]; do
    if [ "$1" = -o ]; then shift; /bin/cat "$HANDOFF_TRACE/summary" > "$1"; fi
    shift
  done
  printf '{"type":"turn.completed","usage":{"input_tokens":2,"output_tokens":1}}\n'
else
  /usr/bin/env python3 -c 'import json,os; print(json.dumps({"type":"result","subtype":"success","is_error":False,"result":open(os.environ["HANDOFF_TRACE"]+"/summary").read()}))'
fi
"#;
        for name in ["claude", "codex"] {
            let path = self.write(&format!("bin/{name}"), script);
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
    pub fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_dstack"));
        c.current_dir(&self.0)
            .env("DSTACK_ROOT", &self.0)
            .env(
                "DSTACK_HOME",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../claude"),
            )
            .env("DSTACK_SESSION_ID", "destination")
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    self.0.join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("HANDOFF_TRACE", self.0.join("trace"))
            .env(
                "HANDOFF_SCENARIO",
                fs::read_to_string(self.0.join("scenario")).unwrap_or_default(),
            );
        c
    }
    pub fn prepare(&self, to: &str, extra: &[&str]) -> Output {
        self.command()
            .args(["handoff", "--to", to, "--history", "history.jsonl"])
            .args(extra)
            .output()
            .unwrap()
    }
    pub fn packet(&self) -> PathBuf {
        fs::read_dir(self.0.join(".dstack/runs/sample/handoffs"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.join("ready").is_file())
            .expect("ready handoff")
    }
    pub fn resume(&self, packet: &Path, host: &str, extra: &[&str]) -> Output {
        self.command()
            .args([
                "handoff",
                "resume",
                packet.file_name().unwrap().to_str().unwrap(),
                "--host",
                host,
            ])
            .args(extra)
            .output()
            .unwrap()
    }
    pub fn write(&self, path: &str, text: &str) -> PathBuf {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }
    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
    pub fn git(&self, args: &[&str]) {
        assert!(Command::new("git")
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub fn success(out: Output) -> String {
    assert!(
        out.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
