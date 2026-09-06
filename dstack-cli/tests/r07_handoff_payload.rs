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
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
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
    let (roots, target) = target(&s);
    (s, roots, target)
}

fn target(s: &Scratch) -> (Roots, Target) {
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
    (roots, target)
}

#[test]
fn r07_handoff_context_compacts_repeated_index_without_losing_changes_or_freshness() {
    let (s, roots, target) = large_fixture();
    let mut plan: Value = serde_json::from_str(&s.read(".dstack/runs/sample/plan.json")).unwrap();
    let mut completed = plan["plans"][0]["tasks"][0].clone();
    completed["id"] = json!("T2");
    completed["commit"] = json!(String::from_utf8(git(&s.0, &["rev-parse", "HEAD"])).unwrap().trim());
    completed["done_at"] = json!("2026-09-06T00:00:00Z");
    plan["plans"][0]["tasks"].as_array_mut().unwrap().push(completed);
    s.write(".dstack/runs/sample/plan.json", &plan.to_string());
    let snapshot = collect(&roots, &target).expect("bounded snapshot");
    let document = |reference| &snapshot.documents.iter().find(|d| d.reference == reference).unwrap().text;
    assert_eq!(document("state:request"), &s.read(".dstack/runs/sample/request.md"));
    let active: Value = serde_json::from_str(document("task:T1")).unwrap();
    let completed: Value = serde_json::from_str(document("task:T2")).unwrap();
    assert_eq!(active["state"], "active");
    assert_eq!(completed["state"], "completed");
    for field in ["files", "attempts", "blockers", "dependencies", "git_evidence"] {
        assert!(active.get(field).is_some(), "missing active {field}");
        assert!(completed.get(field).is_none(), "completed detail should be brief: {field}");
    }
    assert!(document("task:T1").len() > document("task:T2").len());
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
        assert_eq!(value["unmerged_index_entries_z"], "");
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
    let dir = s.packet();
    assert!(dir.join("RESUME.md").is_file());
    let packet = packet::load(&dir).expect("sealed compact packet fits unchanged load bound");
    for doc in packet.snapshot.documents.iter().filter(|d| d.reference.starts_with("git:")) {
        assert!(serde_json::from_str::<Value>(&doc.text).unwrap().get("index_entries_z").is_none());
    }
    assert_eq!(packet.to, Provider::Codex);
    assert_eq!(s.read(".dstack/runs/sample/meta.tsv"), before);
    assert_eq!(
        s.read(".dstack/runs/sample/mode.json"),
        s.read(".dstack/project/mode.json")
    );
    for scenario in ["fail", "invalid-summary"] {
        s.write("scenario", scenario);
        if scenario == "invalid-summary" { s.write("trace/summary", "{}"); }
        let out = s.prepare("codex", &[]);
        if scenario == "fail" { assert_eq!(out.status.code(), Some(23)); }
        else { assert!(String::from_utf8_lossy(&out.stderr).contains("handoff summary")); }
        assert!(!out.status.success());
        assert_eq!(s.read(".dstack/runs/sample/meta.tsv"), before);
        assert_eq!(s.read(".dstack/runs/sample/mode.json"), s.read(".dstack/project/mode.json"));
        let ready = fs::read_dir(dir.parent().unwrap()).unwrap()
            .filter(|e| e.as_ref().unwrap().path().join("ready").exists()).count();
        assert_eq!(ready, 1, "failed preparations must not seal another packet");
    }
}

#[test]
fn r07_handoff_context_r08_handoff_summary_preserves_unmerged_stage_records() {
    use std::os::unix::fs::PermissionsExt;
    let s = Scratch::new("claude", "claude");
    let path = "conflict \t\nname.txt";
    s.git(&["config", "user.name", "Fixture"]);
    s.git(&["config", "user.email", "fixture@example.invalid"]);
    s.write(path, "base\n");
    s.git(&["add", path]);
    s.git(&["commit", "--no-verify", "-qm", "base"]);
    s.git(&["checkout", "-qb", "theirs"]);
    s.write(path, "theirs\n");
    s.git(&["commit", "--no-verify", "-qam", "theirs"]);
    s.git(&["checkout", "-qb", "ours", "HEAD^"]);
    s.write(path, "ours\n");
    fs::set_permissions(s.0.join(path), fs::Permissions::from_mode(0o755)).unwrap();
    s.git(&["add", path]);
    s.git(&["commit", "--no-verify", "-qm", "ours"]);
    let merge = Command::new("git").current_dir(&s.0).args(["merge", "theirs"]).output().unwrap();
    assert!(!merge.status.success());
    let original = String::from_utf8(git(&s.0, &["ls-files", "--unmerged", "-z"])).unwrap();
    let rows: Vec<_> = original.split_terminator('\0').collect();
    assert_eq!(rows.len(), 3);
    for (row, stage) in rows.iter().zip(1..=3) {
        let (header, recorded_path) = row.split_once('\t').unwrap();
        assert!(header.ends_with(&format!(" {stage}")));
        assert_eq!(recorded_path, path);
        assert!(header.starts_with(if stage == 2 { "100755 " } else { "100644 " }));
    }
    success(s.prepare("codex", &[]));
    let packet = packet::load(&s.packet()).unwrap();
    let document = packet.snapshot.documents.iter().find(|d| d.reference.starts_with("git:")).unwrap();
    let evidence: Value = serde_json::from_str(&document.text).unwrap();
    assert_eq!(evidence["unmerged_index_entries_z"], original);
    assert!(packet::context(&packet).unwrap().contains("unmerged_index_entries_z"));
    let (roots, target) = target(&s);
    verify(&packet.snapshot, &roots, &target).unwrap();
    let ours = rows[1].split_whitespace().nth(1).unwrap();
    let changed_base = format!("100644 {ours} 1\t{path}\0");
    let mut child = Command::new("git").current_dir(&s.0)
        .args(["update-index", "-z", "--index-info"]).stdin(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(changed_base.as_bytes()).unwrap();
    assert!(child.wait().unwrap().success());
    assert_ne!(original.as_bytes(), git(&s.0, &["ls-files", "--unmerged", "-z"]));
    assert!(verify(&packet.snapshot, &roots, &target).unwrap_err().to_string().contains("stale handoff"));
}
