#[path = "support/handoff.rs"]
mod support;

use dstack_cli::core::fsx::sha256_bytes;
use dstack_cli::core::mode::Provider;
use dstack_cli::core::roots::Roots;
use dstack_cli::core::target::{Target, TargetKind};
use dstack_cli::handoff::packet::{self, Packet};
use dstack_cli::handoff::snapshot::{collect, verify};
use dstack_cli::handoff::types::History;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;
use support::{success, Scratch};

const LIMIT: usize = 2 * 1024 * 1024;

fn git(tree: &Path, args: &[&str]) -> Vec<u8> {
    let out = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(tree)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

fn large_fixture() -> (Scratch, Roots, Target) {
    let s = Scratch::new("claude", "claude");
    for n in 0..4200 {
        s.write(
            &format!("tracked/{n:04}-{}", "p".repeat(210)),
            "unchanged\n",
        );
    }
    s.git(&["add", "tracked"]);
    s.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--no-verify",
        "-qm",
        "large tracked fixture",
    ]);
    let retained = s.0.join("retained");
    s.write(".git/info/exclude", "retained/\n");
    s.git(&[
        "worktree",
        "add",
        "-q",
        "--detach",
        retained.to_str().unwrap(),
        "HEAD",
    ]);
    let mut plan: Value = serde_json::from_str(&s.read(".dstack/runs/sample/plan.json")).unwrap();
    plan["plans"][0]["worktree"] = json!(retained);
    s.write(".dstack/runs/sample/plan.json", &plan.to_string());
    s.write("work.txt", "staged change\n");
    s.git(&["add", "work.txt"]);
    s.write("work.txt", "unstaged change\n");
    s.write("untracked.txt", "untracked contents\n");
    s.write("retained/work.txt", "retained change\n");
    let store = s.0.join(".dstack");
    let roots = Roots {
        main_root: s.0.clone(),
        wt_root: s.0.clone(),
        runs: store.join("runs"),
        local: store.join("local"),
        quick: store.join("quick"),
        store,
    };
    let target = Target {
        kind: TargetKind::Run,
        id: "sample".into(),
        dir: roots.runs.join("sample"),
    };
    (s, roots, target)
}

#[test]
fn r07_handoff_context_compacts_repeated_index_without_losing_changes_or_freshness() {
    let (s, roots, target) = large_fixture();
    let snapshot = collect(&roots, &target).expect("bounded snapshot");
    let mut packet = Packet {
        version: 1,
        id: "fixture".into(),
        to: Provider::Codex,
        snapshot,
        history: History {
            provider: Provider::Claude,
            session: "source".into(),
            cwd: s.0.to_string_lossy().into_owned(),
            path: "fixture-history".into(),
            sha256: sha256_bytes(b"fixture"),
            records: vec![],
            warnings: vec![],
            omitted: 0,
        },
    };
    let context = packet::context(&packet)
        .expect("redundant tracked paths must fit the unchanged context bound");
    assert!(context.len() < LIMIT);
    let git_docs: Vec<_> = packet
        .snapshot
        .documents
        .iter()
        .filter(|d| d.reference.starts_with("git:worktree:"))
        .collect();
    assert_eq!(git_docs.len(), 2);
    let mut stage_total = 0;
    for doc in &git_docs {
        let tree = Path::new(&doc.path);
        let stage = git(
            tree,
            &["ls-files", "--stage", "-z", "--", ".", ":(exclude).dstack"],
        );
        assert!(stage.len() < LIMIT, "individual Git output remains bounded");
        stage_total += stage.len();
        let value: Value = serde_json::from_str(&doc.text).unwrap();
        assert!(value.get("index_entries_z").is_none());
        assert_eq!(value["index_entries_sha256"], sha256_bytes(&stage));
        assert_eq!(value["index_entries_bytes"], stage.len());
        assert_eq!(
            value["index_entries_count"],
            stage.iter().filter(|b| **b == 0).count()
        );
        let index = git(tree, &["rev-parse", "--git-path", "index"]);
        let index = tree.join(String::from_utf8(index).unwrap().trim());
        assert_eq!(
            value["index_sha256"],
            sha256_bytes(&fs::read(index).unwrap())
        );
        let head = git(tree, &["rev-parse", "HEAD"]);
        assert_eq!(value["head"], String::from_utf8(head).unwrap().trim());
        if tree == s.0 {
            assert!(value["status_porcelain_z"]
                .as_str()
                .unwrap()
                .contains("MM work.txt\0"));
            assert!(value["status_porcelain_z"]
                .as_str()
                .unwrap()
                .contains("?? untracked.txt\0"));
            assert!(value["index_diff"]
                .as_str()
                .unwrap()
                .contains("+staged change"));
            assert!(value["worktree_diff"]
                .as_str()
                .unwrap()
                .contains("+unstaged change"));
            for (path, text) in [
                ("work.txt", "unstaged change\n"),
                ("untracked.txt", "untracked contents\n"),
            ] {
                let file = value["files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|f| f["path"] == path)
                    .unwrap();
                assert_eq!(file["text"], text);
                assert_eq!(file["sha256"], sha256_bytes(text.as_bytes()));
            }
        } else {
            assert!(value["worktree_diff"]
                .as_str()
                .unwrap()
                .contains("+retained change"));
            assert_eq!(value["files"][0]["text"], "retained change\n");
        }
    }
    assert!(
        stage_total > LIMIT,
        "unchanged listings alone must reproduce the old context overflow"
    );
    verify(&packet.snapshot, &roots, &target).expect("unchanged compact evidence stays fresh");
    s.git(&["add", "work.txt"]);
    assert!(verify(&packet.snapshot, &roots, &target)
        .unwrap_err()
        .to_string()
        .contains("stale handoff"));
    s.git(&["update-index", "--index-version", "2"]);
    let saved = collect(&roots, &target).unwrap();
    let stage_before = git(&s.0, &["ls-files", "--stage", "-z"]);
    s.git(&["update-index", "--index-version", "4"]);
    assert_eq!(stage_before, git(&s.0, &["ls-files", "--stage", "-z"]));
    assert!(verify(&saved, &roots, &target)
        .unwrap_err()
        .to_string()
        .contains("stale handoff"));
    // Reintroducing only the redundant listings must still hit the original 2 MiB bound.
    for doc in packet
        .snapshot
        .documents
        .iter_mut()
        .filter(|d| d.reference.starts_with("git:worktree:"))
    {
        let mut value: Value = serde_json::from_str(&doc.text).unwrap();
        value["index_entries_z"] = json!(String::from_utf8(git(
            Path::new(&doc.path),
            &["ls-files", "--stage", "-z"]
        ))
        .unwrap());
        doc.text = value.to_string();
    }
    assert!(packet::context(&packet)
        .unwrap_err()
        .to_string()
        .contains("exceeds 2 MiB"));
}

#[test]
fn r08_handoff_summary_compact_large_packet_reaches_destination_and_loads() {
    let (s, _, _) = large_fixture();
    let before = s.read(".dstack/runs/sample/meta.tsv");
    success(s.prepare("codex", &[]));
    assert_eq!(s.read("trace/calls"), "codex\n");
    assert!(s.read("trace/argv").contains("read-only"));
    let prompt = s.read("trace/stdin");
    assert!(prompt.contains("=== ROLE INSTRUCTIONS (stable) ==="));
    assert!(prompt.contains("index_entries_sha256"));
    assert!(!prompt.contains("index_entries_z"));
    let dir = s.packet();
    assert!(dir.join("RESUME.md").is_file());
    let packet = packet::load(&dir).expect("sealed compact packet fits unchanged load bound");
    assert_eq!(packet.to, Provider::Codex);
    assert_eq!(s.read(".dstack/runs/sample/meta.tsv"), before);
    assert_eq!(
        s.read(".dstack/runs/sample/mode.json"),
        s.read(".dstack/project/mode.json")
    );
}
