use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use dstack_cli::core::fsx::sha256_bytes;
use dstack_cli::core::roots::Roots;
use dstack_cli::core::target::{Target, TargetKind};
use dstack_cli::handoff::snapshot::{check_idle, collect, verify};
use dstack_cli::handoff::types::Snapshot;
use serde_json::{json, Value};

const REQUEST: &str = "# 인계를 준비해요\n\n- [ ] **R07** 원문을 보존해요. — accept: 원문과 같아요.\n- [ ] **R09** 상태를 확인해요. — accept: 변경을 거부해요.\n";
const DECISIONS: &str = "# Decisions\n\n| D-01 | Keep the approved wording exactly. | R07,R09 | answered |\n";
const STAMP: &str = "2026-09-06T09:00:00Z";

struct Fixture { root: PathBuf, roots: Roots, target: Target, head: String }

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("handoff-snapshot-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        git(&root, &["init", "-q", "-b", "main"]);
        fs::write(root.join(".gitignore"), ".dstack/\n").unwrap();
        fs::write(root.join("tracked.txt"), "original\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--no-verify", "-qm", "fixture"]);
        let head = git(&root, &["rev-parse", "HEAD"]);
        let store = root.join(".dstack");
        let roots = Roots { main_root: root.clone(), wt_root: root.clone(), runs: store.join("runs"), local: store.join("local"), quick: store.join("quick"), store };
        let target = Target { kind: TargetKind::Run, id: "fixture-run".into(), dir: roots.runs.join("fixture-run") };
        fs::create_dir_all(target.dir.join("review")).unwrap();
        fs::create_dir_all(&roots.local).unwrap();
        let s = Self { root, roots, target, head };
        s.write("meta.tsv", &format!("id\tfixture-run\nstatus\topen\nworktree\t{}\nowner_session\tsource\nowner_pid\t42\nowner_ts\t{STAMP}\n", s.root.display()));
        s.write("request.md", REQUEST);
        s.write("request.approved", &format!("sha256 {}  approved_at {STAMP}\n", sha256_bytes(REQUEST.as_bytes())));
        s.write("decisions.md", DECISIONS);
        s.write("mode.json", "{\"main\":\"codex\",\"sub\":\"claude\"}\n");
        s.write("cases.tsv", "R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote\nR07\tc1\tcli\topen\t-\t-\t-\t-\tverification still pending\nR09\tc-test-red\ttest\tmet\tattempt.log\tunused\tcargo test r09_handoff_resume\t2026-09-06T09:00:00Z\tRed: expected failure\nR09\tc1\tcli\tblocked\t-\t-\t-\t-\tprovider unavailable\n");
        fs::write(s.root.join("attempt.log"), "r09_handoff_resume expected failure\n").unwrap();
        s.write("plan.json", &s.plan().to_string());
        s
    }
    fn write(&self, name: &str, text: &str) { fs::write(self.target.dir.join(name), text).unwrap(); }
    fn plan(&self) -> Value {
        let task = |id, commit: &str, covers| json!({"id":id,"slug":id,"covers":[covers],"files":["tracked.txt"],"deps":[],"commit":commit,"done_at":if commit.is_empty() { "" } else { STAMP }});
        let plan = |id, status, tasks| json!({"id":id,"milestone":"M1","slug":id,"files":["tracked.txt"],"deps":[],"status":status,"worktree":"","started_at":"","done_at":"","tasks":tasks});
        json!({"v":2,"milestones":[{"id":"M1","slug":"test","order":1}],"plans":[plan("P1","in-progress",vec![task("T1",&self.head,"R07"),task("T2","","R09")]),plan("P2","ready",vec![task("T3","","R07")])]})
    }
    fn snapshot(&self) -> Snapshot { collect(&self.roots, &self.target).expect("bounded snapshot") }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); } }
fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git").arg("-C").arg(root).args(args).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap().trim_end().into()
}
fn doc<'a>(snapshot: &'a Snapshot, name: &str) -> &'a str {
    &snapshot.documents.iter().find(|d| d.reference == name).unwrap_or_else(|| panic!("missing {name}")).text
}

#[test]
fn r07_handoff_context_preserves_frozen_documents_and_task_categories() {
    let s = Fixture::new("categories");
    fs::write(s.root.join("tracked.txt"), "uncommitted change\n").unwrap();
    let before = fs::read(s.target.dir.join("meta.tsv")).unwrap();
    let snap = s.snapshot();
    assert_eq!(doc(&snap, "state:request"), REQUEST);
    assert_eq!(doc(&snap, "state:decisions"), DECISIONS);
    assert_eq!(snap.items.iter().map(|i| i.state.as_str()).collect::<Vec<_>>(), ["completed", "active", "pending"]);
    for item in &snap.items { assert!(snap.documents.iter().any(|d| d.reference == format!("task:{}", item.id) && d.path.contains("plan.json:"))); }
    let active: Value = serde_json::from_str(doc(&snap, "task:T2")).unwrap();
    for field in ["files", "attempts", "blockers", "dependencies", "verification_gaps"] { assert!(active.get(field).is_some(), "missing {field}"); }
    assert!(doc(&snap, "task:T2").contains("expected failure"));
    assert!(doc(&snap, "task:T2").contains("provider unavailable"));
    assert!(doc(&snap, "task:T1").contains("verification"));
    assert!(doc(&snap, "task:T2").len() > doc(&snap, "task:T3").len());
    assert!(snap.documents.iter().any(|d| d.reference.starts_with("git:") && d.text.contains("uncommitted change")));
    assert_eq!(before, fs::read(s.target.dir.join("meta.tsv")).unwrap(), "collection must not take ownership");
}

#[test]
fn r07_handoff_context_fingerprints_dirty_untracked_index_and_symlink_contents() {
    let s = Fixture::new("contents");
    let original = s.snapshot();
    fs::write(s.root.join("tracked.txt"), "dirty one\n").unwrap();
    let first = s.snapshot();
    fs::write(s.root.join("tracked.txt"), "dirty two\n").unwrap();
    assert_ne!(first.fingerprint, s.snapshot().fingerprint);
    fs::write(s.root.join("odd\n\" name.txt"), "untracked one").unwrap();
    let first = s.snapshot();
    fs::write(s.root.join("odd\n\" name.txt"), "untracked two").unwrap();
    assert_ne!(first.fingerprint, s.snapshot().fingerprint);
    git(&s.root, &["add", "tracked.txt"]);
    let first = s.snapshot();
    git(&s.root, &["reset", "-q", "HEAD", "--", "tracked.txt"]);
    assert_ne!(first.fingerprint, s.snapshot().fingerprint);
    std::os::unix::fs::symlink("/outside/first", s.root.join("link")).unwrap();
    let first = s.snapshot();
    fs::remove_file(s.root.join("link")).unwrap();
    std::os::unix::fs::symlink("/outside/other", s.root.join("link")).unwrap();
    assert_ne!(first.fingerprint, s.snapshot().fingerprint);
    assert!(verify(&original, &s.roots, &s.target).is_err());
}

#[test]
fn r07_handoff_context_does_not_claim_unverified_commits_completed() {
    let s = Fixture::new("commit");
    let mut plan = s.plan();
    plan["plans"][0]["tasks"][0]["commit"] = json!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef");
    s.write("plan.json", &plan.to_string());
    assert_eq!(s.snapshot().items[0].state, "active");
    plan["plans"][0]["tasks"][0]["commit"] = json!(s.head);
    plan["plans"][0]["tasks"][0]["done_at"] = json!("");
    s.write("plan.json", &plan.to_string());
    assert_eq!(s.snapshot().items[0].state, "active");
}

#[test]
fn r09_handoff_resume_rejects_stale_state_owner_mode_and_worktree() {
    let s = Fixture::new("stale");
    for name in ["decisions.md", "cases.tsv", "request.approved", "plan.json", "review/index.tsv", "questions.md"] {
        let saved = s.snapshot();
        let file = s.target.dir.join(name);
        let old = fs::read(&file).ok();
        let mut bytes = old.clone().unwrap_or_default(); bytes.extend_from_slice(b"\nchanged\n");
        fs::write(&file, bytes).unwrap();
        assert!(verify(&saved, &s.roots, &s.target).is_err(), "stale {name}");
        match old { Some(bytes) => fs::write(file, bytes).unwrap(), None => fs::remove_file(file).unwrap() }
    }
    let saved = s.snapshot();
    let meta = fs::read_to_string(s.target.dir.join("meta.tsv")).unwrap();
    s.write("meta.tsv", &meta.replace("owner_pid\t42", "owner_pid\t99").replace(STAMP, "2026-09-06T10:00:00Z"));
    verify(&saved, &s.roots, &s.target).expect("owner heartbeat is not state drift");
    s.write("meta.tsv", &meta.replace("owner_session\tsource", "owner_session\tother"));
    assert!(verify(&saved, &s.roots, &s.target).is_err());
    s.write("meta.tsv", &meta);
    s.write("mode.json", "{\"main\":\"claude\",\"sub\":\"claude\"}");
    assert!(verify(&saved, &s.roots, &s.target).is_err());
    s.write("meta.tsv", &meta.replace(&s.root.to_string_lossy().to_string(), "/tmp"));
    assert!(collect(&s.roots, &s.target).is_err());
}

#[test]
fn r09_handoff_resume_ignores_finished_capture_logs_and_guards_active_exec() {
    let s = Fixture::new("exec");
    let saved = s.snapshot();
    check_idle(&s.roots, &saved).unwrap();
    let cap = s.roots.local.join("exec/worker");
    fs::create_dir_all(&cap).unwrap();
    fs::write(cap.join("started_at"), STAMP).unwrap();
    assert!(check_idle(&s.roots, &saved).is_err());
    fs::write(cap.join("exit"), "0\n").unwrap();
    assert!(check_idle(&s.roots, &saved).is_err());
    fs::write(cap.join("finished_at"), "invalid").unwrap();
    assert!(check_idle(&s.roots, &saved).is_err());
    fs::write(cap.join("finished_at"), STAMP).unwrap();
    check_idle(&s.roots, &saved).unwrap();
    fs::write(cap.join("exit"), "not a number").unwrap();
    assert!(check_idle(&s.roots, &saved).is_err());
    fs::write(cap.join("exit"), "1\n").unwrap();
    fs::write(cap.join("out.txt"), "new log data").unwrap();
    fs::create_dir_all(s.target.dir.join("handoffs/new")).unwrap();
    verify(&saved, &s.roots, &s.target).expect("handoff/capture output is not frozen state");
}

#[test]
fn r09_handoff_resume_includes_separate_active_plan_worktrees() {
    let s = Fixture::new("worktrees");
    let worker = s.root.join("worker");
    git(&s.root, &["worktree", "add", "-q", "--detach", worker.to_str().unwrap(), "HEAD"]);
    fs::write(s.root.join(".git/info/exclude"), "worker/\n").unwrap();
    let mut plan = s.plan(); plan["plans"][0]["worktree"] = json!(worker);
    s.write("plan.json", &plan.to_string());
    let saved = s.snapshot();
    fs::write(worker.join("tracked.txt"), "worker dirty\n").unwrap();
    assert!(verify(&saved, &s.roots, &s.target).is_err());
    let cap = worker.join(".dstack/local/exec/task"); fs::create_dir_all(cap).unwrap();
    assert!(check_idle(&s.roots, &saved).is_err());
    fs::remove_dir_all(&worker).unwrap();
    assert!(collect(&s.roots, &s.target).is_err(), "cannot omit inaccessible active worktree");
}

#[test]
fn r07_handoff_context_refuses_invalid_or_overflowing_required_evidence() {
    let s = Fixture::new("bounds");
    fs::write(s.target.dir.join("decisions.md"), [0xff, 0xfe]).unwrap();
    assert!(collect(&s.roots, &s.target).is_err());
    s.write("decisions.md", DECISIONS);
    let file = fs::File::create(s.root.join("huge.bin")).unwrap(); file.set_len(64 * 1024 * 1024).unwrap();
    assert!(collect(&s.roots, &s.target).is_err(), "large untracked data cannot silently disappear");
}

#[test]
fn r07_handoff_context_compacts_review_bundles_but_tracks_their_contents() {
    let s = Fixture::new("review-manifest");
    let body = "historical request and diff data ".repeat(16000);
    s.write("review/bundle-P1.txt", &body);
    s.write("review/codex-review-001.md", "| R07 | partial | verification pending |\nVERDICT: reject\n");
    s.write("review/index.tsv", &format!("001\tplan\tP1\tcodex-review-001.md\t{STAMP}\t0\t1\t0\n"));
    let saved = s.snapshot();
    assert!(doc(&saved, "state:review-files").contains(&sha256_bytes(body.as_bytes())));
    assert!(doc(&saved, "review:codex-review-001.md").contains("VERDICT: reject"));
    assert!(!serde_json::to_string(&saved).unwrap().contains("historical request and diff data"));
    s.write("review/bundle-P1.txt", &(body + "changed"));
    assert!(verify(&saved, &s.roots, &s.target).is_err());
}

#[test]
fn r09_handoff_resume_guards_head_branch_packet_and_local_identity() {
    let s = Fixture::new("identity");
    let saved = s.snapshot();
    git(&s.root, &["checkout", "-qb", "other"]);
    assert!(verify(&saved, &s.roots, &s.target).is_err());
    git(&s.root, &["checkout", "-q", "main"]);
    git(&s.root, &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "--no-verify", "-qm", "next"]);
    assert!(verify(&saved, &s.roots, &s.target).is_err());
    let mut fresh = s.snapshot(); fresh.documents[0].text.push_str("tampered\n");
    assert!(verify(&fresh, &s.roots, &s.target).is_err());
    fs::remove_dir_all(&s.roots.local).unwrap();
    std::os::unix::fs::symlink("/nonexistent/outside", &s.roots.local).unwrap();
    assert!(check_idle(&s.roots, &saved).is_err());
    assert!(collect(&s.roots, &s.target).is_err());
}

#[test]
fn r07_handoff_context_hashes_ignored_evidence_without_external_diff_drivers() {
    let s = Fixture::new("artifact-driver");
    git(&s.root, &["config", "diff.external", "false"]);
    git(&s.root, &["config", "diff.proof.textconv", "false"]);
    fs::write(s.root.join(".gitattributes"), "tracked.txt diff=proof\n").unwrap();
    fs::write(s.root.join("tracked.txt"), "raw changed contents\n").unwrap();
    let proof = s.roots.local.join("proof.log");
    fs::write(&proof, "r09_handoff_resume red attempt\n").unwrap();
    let cases = fs::read_to_string(s.target.dir.join("cases.tsv")).unwrap();
    s.write("cases.tsv", &cases.replace("attempt.log\tunused", &format!(".dstack/local/proof.log\t{}", sha256_bytes(&fs::read(&proof).unwrap()))));
    let saved = s.snapshot();
    assert!(doc(&saved, "state:evidence").contains(&sha256_bytes(&fs::read(&proof).unwrap())));
    assert!(doc(&saved, "task:T2").contains("\"matches_recorded_sha256\":true"));
    fs::write(proof, "r09_handoff_resume altered evidence\n").unwrap();
    assert!(verify(&saved, &s.roots, &s.target).is_err());
}

fn completed_worktree(s: &Fixture) -> PathBuf {
    let worker = s.root.join("completed-worker");
    git(&s.root, &["worktree", "add", "-q", "--detach", worker.to_str().unwrap(), "HEAD"]);
    fs::write(s.root.join(".git/info/exclude"), "completed-worker/\n").unwrap();
    let mut plan = s.plan();
    plan["plans"][0]["status"] = json!("done"); plan["plans"][0]["worktree"] = json!(worker);
    plan["plans"][0]["tasks"][1]["commit"] = json!(s.head); plan["plans"][0]["tasks"][1]["done_at"] = json!(STAMP);
    s.write("plan.json", &plan.to_string());
    worker
}

#[test]
fn r07_handoff_context_review_fix_resolves_linked_run_evidence_from_main() {
    let mut s = Fixture::new("review-linked-evidence");
    let linked = s.root.join("linked-run");
    git(&s.root, &["worktree", "add", "-q", "--detach", linked.to_str().unwrap(), "HEAD"]);
    let main_proof = s.roots.local.join("proof.log");
    fs::write(&main_proof, "R09 main-checkout proof\n").unwrap();
    let before_sha = sha256_bytes(&fs::read(&main_proof).unwrap());
    let cases = fs::read_to_string(s.target.dir.join("cases.tsv")).unwrap();
    s.write("cases.tsv", &cases.replace("attempt.log\tunused", &format!(".dstack/local/proof.log\t{before_sha}")));
    let meta = fs::read_to_string(s.target.dir.join("meta.tsv")).unwrap();
    s.write("meta.tsv", &meta.replace(&s.root.to_string_lossy().to_string(), linked.to_str().unwrap()));
    s.roots.wt_root = linked.clone(); s.roots.local = linked.join(".dstack/local"); s.roots.quick = linked.join(".dstack/quick");
    fs::create_dir_all(&s.roots.local).unwrap();
    fs::write(s.roots.local.join("proof.log"), "R09 wrong run-local proof\n").unwrap();
    let saved = s.snapshot();
    fs::write(&main_proof, "R09 changed main-checkout proof\n").unwrap();
    assert!(verify(&saved, &s.roots, &s.target).is_err(), "main-checkout evidence mutation must invalidate a linked-run snapshot");
    assert!(doc(&saved, "state:evidence").contains(&before_sha));
    assert!(doc(&saved, "task:T2").contains("\"matches_recorded_sha256\":true"));
}

#[test]
fn r09_handoff_resume_review_fix_checks_retained_completed_worktrees() {
    let s = Fixture::new("review-retained-done");
    let worker = completed_worktree(&s);
    let saved = s.snapshot();
    fs::write(worker.join("tracked.txt"), "uncommitted after plan completion\n").unwrap();
    let stale = verify(&saved, &s.roots, &s.target).is_err();
    fs::create_dir_all(worker.join(".dstack/local/exec/unfinished")).unwrap();
    let guarded = check_idle(&s.roots, &saved).is_err();
    assert!(stale && guarded, "retained done checkout: stale Git rejected={stale}, active exec rejected={guarded}");
}

#[test]
fn r09_handoff_resume_review_fix_records_removed_historical_worktrees() {
    let s = Fixture::new("review-removed-done");
    let worker = completed_worktree(&s);
    let retained = s.snapshot();
    git(&s.root, &["worktree", "remove", worker.to_str().unwrap()]);
    assert!(verify(&retained, &s.roots, &s.target).is_err(), "checkout removal must invalidate its previous snapshot");
    assert!(check_idle(&s.roots, &retained).is_err());
    let removed = s.snapshot();
    let inventory: Value = serde_json::from_str(doc(&removed, "state:worktrees")).unwrap();
    assert!(inventory.as_array().unwrap().iter().any(|v| v["plan"] == "P1" && v["status"] == "removed"));
    check_idle(&s.roots, &removed).expect("removed historical checkout has no live captures to inspect");
    std::os::unix::fs::symlink("/nonexistent/outside", &worker).unwrap();
    assert!(collect(&s.roots, &s.target).is_err(), "a dangling link is not a removed checkout");
    assert!(check_idle(&s.roots, &removed).is_err());
    fs::remove_file(&worker).unwrap();
    git(&s.root, &["worktree", "add", "-q", "--detach", worker.to_str().unwrap(), "HEAD"]);
    assert!(verify(&removed, &s.roots, &s.target).is_err(), "a restored historical checkout changes the inventory");
    assert!(check_idle(&s.roots, &removed).is_err());
}
