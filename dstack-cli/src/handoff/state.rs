// Workflow documents and task evidence; completion markers alone never prove a commit.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::core::error::{Error, Result};
use crate::core::fsx::{sha256_bytes, utc_to_epoch};
use crate::handoff::types::{Document, Snapshot, WorkItem};
use crate::store::cases::{CaseRow, CASES_HEADER};
use crate::store::plan::PlanDoc;
use super::{canonical, cannot, entries, git, local_directory, push_document, read_bytes, read_text, required, DOCUMENT_LIMIT, ENTRY_LIMIT};

pub(super) fn documents(run: &Path, docs: &mut Vec<Document>) -> Result<()> {
    for (name, reference, mandatory) in [
        ("request.md", "state:request", true), ("request.approved", "state:approval", true),
        ("decisions.md", "state:decisions", true), ("plan.json", "state:plan", false),
        ("cases.tsv", "state:cases", false), ("questions.md", "state:questions", false),
        ("mode.json", "state:mode", false), ("ROADMAP.md", "state:roadmap", false),
        ("STATE.md", "state:state", false), ("accepts.tsv", "state:accepts", false),
        ("metrics.tsv", "state:metrics", false),
    ] {
        let path = run.join(name);
        let text = if mandatory { Some(required(&path, DOCUMENT_LIMIT)?) } else { read_text(&path, DOCUMENT_LIMIT)? };
        if let Some(text) = text { add(docs, reference.into(), &path, text)?; }
    }
    let request = text(docs, "state:request").expect("required request");
    let stamp: Vec<_> = text(docs, "state:approval").expect("required approval").split_whitespace().collect();
    if stamp.len() != 4 || stamp[0] != "sha256" || stamp[1] != sha256_bytes(request.as_bytes())
        || stamp[2] != "approved_at" || utc_to_epoch(stamp[3]).is_none()
    {
        return Err(Error::failed("handoff requires the current request to match its valid approval stamp"));
    }
    let mut manifest = Vec::new();
    let mut review_bytes = 0;
    for path in entries(&run.join("review"))? {
        let name = path.file_name().and_then(|v| v.to_str())
            .ok_or_else(|| Error::cannot_decide("handoff review filename is not UTF-8"))?;
        let reference = match name {
            "index.tsv" => Some("state:reviews".to_owned()),
            "closed.tsv" => Some("state:review-closed".to_owned()),
            _ if name.starts_with("codex-review-") && name.ends_with(".md") => Some(format!("review:{name}")),
            _ => None,
        };
        let bytes = read_bytes(&path, DOCUMENT_LIMIT)?.ok_or_else(|| Error::cannot_decide("handoff review file disappeared"))?;
        review_bytes += bytes.len();
        if review_bytes > 64 * 1024 * 1024 { return Err(Error::failed("handoff review files exceed 64 MiB")); }
        manifest.push(json!({"path":path,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),"reference":reference}));
        if let Some(reference) = reference {
            let text = String::from_utf8(bytes).map_err(|e| cannot("decode review UTF-8", &path, e))?;
            add(docs, reference, &path, text)?;
        }
    }
    if !manifest.is_empty() {
        add(docs, "state:review-files".into(), &run.join("review"), json!(manifest).to_string())?;
    }
    validate_reviews(docs)
}

fn add(docs: &mut Vec<Document>, reference: String, path: &Path, text: String) -> Result<()> {
    push_document(docs, Document { reference, path: format!("{}:1", path.display()), text })
}

fn validate_reviews(docs: &[Document]) -> Result<()> {
    for (index, row) in text(docs, "state:reviews").unwrap_or("").lines().enumerate() {
        let cells: Vec<_> = row.split('\t').collect();
        if cells.len() != 8 || cells[0].parse::<u32>().is_err()
            || !matches!(cells[1], "plan" | "milestone" | "quick") || cells[2].is_empty()
            || utc_to_epoch(cells[4]).is_none() || cells[5..].iter().any(|v| v.parse::<u32>().is_err())
            || text(docs, &format!("review:{}", cells[3])).is_none()
        {
            return Err(Error::cannot_decide(format!("invalid handoff review index or missing sealed review at row {}", index + 1)));
        }
    }
    Ok(())
}

fn text<'a>(docs: &'a [Document], reference: &str) -> Option<&'a str> {
    docs.iter().find(|d| d.reference == reference).map(|d| d.text.as_str())
}

pub(super) fn plan(docs: &[Document]) -> Result<PlanDoc> {
    let plan: PlanDoc = serde_json::from_str(text(docs, "state:plan").unwrap_or(crate::store::plan::SEED))
        .map_err(|e| Error::cannot_decide(format!("invalid handoff plan.json: {e}")))?;
    let mut ids = BTreeSet::new();
    if plan.v != 2 { return Err(Error::cannot_decide("unsupported handoff plan version")); }
    for p in &plan.plans {
        if !ids.insert(p.id.clone()) || !matches!(p.status.as_str(), "pending" | "ready" | "in-progress" | "done" | "blocked") {
            return Err(Error::cannot_decide("invalid or duplicate handoff plan"));
        }
        for task in &p.tasks {
            if !ids.insert(task.id.clone()) || !task.id.starts_with('T') || task.id[1..].parse::<u32>().is_err() {
                return Err(Error::cannot_decide("invalid or duplicate handoff task id"));
            }
        }
    }
    if ids.len() > ENTRY_LIMIT { return Err(Error::failed("handoff plans/tasks exceed the entry limit")); }
    Ok(plan)
}

pub(super) fn worktrees(root: &Path, plan: &PlanDoc) -> Result<(Vec<PathBuf>, Vec<Value>)> {
    let mut trees = BTreeSet::from([root.to_path_buf()]);
    let mut inventory = vec![json!({"role":"run","worktree":root,"status":"inspected"})];
    for p in plan.plans.iter().filter(|p| !p.worktree.is_empty()) {
        let path = Path::new(&p.worktree);
        if !path.is_absolute() || path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(Error::failed(format!("Plan {} worktree must be a canonical absolute path", p.id)));
        }
        match fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && p.status == "done" => {
                missing_checkout(path)?;
                inventory.push(json!({"plan":p.id,"plan_status":p.status,"worktree":path,"status":"removed",
                    "note":"skipped: removed historical checkout of a completed Plan; no live files or exec captures to inspect"}));
                continue;
            }
            Err(e) => return Err(cannot(&format!("inspect required Plan {} worktree", p.id), path, e)),
            Ok(meta) if !meta.is_dir() => return Err(Error::failed(format!("Plan {} worktree is not a real directory: {}", p.id, path.display()))),
            _ => (),
        }
        let tree = canonical(path)?;
        if tree != path {
            return Err(Error::failed(format!("Plan {} worktree must be its canonical path", p.id)));
        }
        inventory.push(json!({"plan":p.id,"plan_status":p.status,"worktree":tree,"status":"inspected"}));
        trees.insert(tree);
    }
    if trees.len() > 64 { return Err(Error::failed("handoff retained worktrees exceed 64")); }
    Ok((trees.into_iter().collect(), inventory))
}

fn missing_checkout(path: &Path) -> Result<()> {
    // ENOENT below a dangling ancestor link does not prove that a checkout was removed.
    for ancestor in path.ancestors().skip(1) {
        match fs::symlink_metadata(ancestor) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(cannot("inspect removed checkout ancestor", ancestor, e)),
            Ok(meta) if meta.is_dir() && canonical(ancestor)? == ancestor => return Ok(()),
            _ => return Err(Error::failed(format!("cannot establish removed historical checkout: {}", path.display()))),
        }
    }
    Err(Error::failed(format!("cannot establish removed historical checkout: {}", path.display())))
}

struct Attempt { row: CaseRow, line: usize, observed: Value }

fn attempts(run: &Path, main_root: &Path, docs: &mut Vec<Document>) -> Result<Vec<Attempt>> {
    let Some(cases) = text(docs, "state:cases") else { return Ok(Vec::new()); };
    if cases.lines().next() != Some(CASES_HEADER) {
        return Err(Error::cannot_decide("invalid handoff cases.tsv header"));
    }
    let mut result = Vec::new();
    let mut cache = BTreeMap::<PathBuf, Value>::new();
    let mut total = 0;
    for (line, raw) in cases.lines().enumerate().skip(1) {
        let cells: Vec<String> = raw.split('\t').map(str::to_owned).collect();
        if cells.len() != 9 || !matches!(cells[3].as_str(), "open" | "unreported" | "met" | "abstain" | "blocked" | "skipped" | "retired") {
            return Err(Error::cannot_decide(format!("invalid handoff cases.tsv row {}", line + 1)));
        }
        let row = CaseRow::parse(&cells);
        let observed = if row.artifact.is_empty() || row.artifact == "-" { json!({"status":"not recorded"}) }
        else {
            // evidence add stores relative artifact paths against the shared repository root.
            let path = main_root.join(&row.artifact);
            if !cache.contains_key(&path) {
                let value = match read_bytes(&path, 32 * 1024 * 1024)? {
                    None => json!({"path":path,"status":"missing"}),
                    Some(bytes) => {
                        total += bytes.len();
                        if total > 64 * 1024 * 1024 { return Err(Error::failed("handoff evidence artifacts exceed 64 MiB")); }
                        json!({"path":path,"status":"read","bytes":bytes.len(),"sha256":sha256_bytes(&bytes)})
                    }
                };
                cache.insert(path.clone(), value);
            }
            let mut value = cache[&path].clone();
            value["matches_recorded_sha256"] = json!(value.get("sha256").and_then(Value::as_str) == Some(&row.sha256));
            value
        };
        result.push(Attempt { row, line: line + 1, observed });
    }
    let manifest: Vec<_> = cache.values().cloned().collect();
    add(docs, "state:evidence".into(), &run.join("cases.tsv"), json!(manifest).to_string())?;
    Ok(result)
}

pub(super) fn items(run: &Path, main_root: &Path, tree: &Path, plan: &PlanDoc, docs: &mut Vec<Document>) -> Result<Vec<WorkItem>> {
    let attempts = attempts(run, main_root, docs)?;
    let raw_plan = text(docs, "state:plan").unwrap_or("").to_owned();
    let mut items = Vec::new();
    for p in &plan.plans {
        let inspected = docs.iter().any(|d| d.reference.starts_with("git:worktree:") && d.path == p.worktree);
        let commit_tree = if inspected { Path::new(&p.worktree) } else { tree };
        for task in &p.tasks {
            let committed = git::committed(commit_tree, &task.commit)?;
            let complete = committed.is_some() && utc_to_epoch(&task.done_at).is_some();
            let state = if complete { "completed" } else if p.status == "in-progress" || p.status == "done" || p.status == "blocked" || !task.commit.is_empty() || !task.done_at.is_empty() { "active" } else { "pending" };
            let relevant: Vec<_> = attempts.iter().filter(|a| task.covers.contains(&a.row.r)).collect();
            let mut gaps = Vec::new();
            for r in &task.covers {
                let rows: Vec<_> = relevant.iter().filter(|a| &a.row.r == r && a.row.status != "retired").collect();
                if rows.is_empty() { gaps.push(format!("{r}: no verification evidence recorded")); }
                for a in rows {
                    if a.row.status != "met" || a.observed.get("matches_recorded_sha256").and_then(Value::as_bool) != Some(true) {
                        gaps.push(format!("{} {}: verification {} (artifact integrity {})", a.row.r, a.row.case_id, a.row.status, a.observed));
                    }
                }
            }
            let location = task_location(run, &raw_plan, &task.id)?;
            let implementation = if complete { "committed; verified in Git ancestry" } else if committed.is_some() { "commit verified; completion timestamp missing or invalid" } else if !task.commit.is_empty() { "recorded commit is not verified in Git ancestry" } else { "no committed implementation recorded" };
            let mut details = json!({"source":location,"id":task.id,"plan":p.id,"state":state,
                "title":task.slug,"covers":task.covers,"implementation":implementation,
                "commit":committed,"verification_gaps":gaps});
            if state == "active" {
                let mut blockers: Vec<String> = relevant.iter().filter(|a| matches!(a.row.status.as_str(), "blocked" | "abstain" | "skipped"))
                    .map(|a| format!("{} {}: {} — {}", a.row.r, a.row.case_id, a.row.status, a.row.note)).collect();
                if !task.commit.is_empty() && committed.is_none() { blockers.push(implementation.into()); }
                details["files"] = json!(task.files);
                details["worktree"] = json!(commit_tree);
                details["dependencies"] = json!({"tasks":task.deps,"plans":p.deps});
                details["blockers"] = json!(blockers);
                details["attempts"] = json!(relevant.iter().map(|a| json!({"source":format!("{}:{}",run.join("cases.tsv").display(),a.line),
                    "R":a.row.r,"case":a.row.case_id,"kind":a.row.kind,"status":a.row.status,"artifact":a.row.artifact,
                    "recorded_sha256":a.row.sha256,"produced_by":a.row.produced_by,"recorded_at":a.row.recorded_at,
                    "note":a.row.note,"artifact_observation":a.observed})).collect::<Vec<_>>());
                details["git_evidence"] = json!(docs.iter().filter(|d| d.reference.starts_with("git:")).map(|d| &d.reference).collect::<Vec<_>>());
            } else {
                details["verification_gaps"] = json!({"count":gaps.len(),"source":"state:cases","details":"state:evidence"});
            }
            let reference = format!("task:{}", task.id);
            push_document(docs, Document { reference: reference.clone(), path: location, text: details.to_string() })?;
            items.push(WorkItem { id:task.id.clone(),plan:p.id.clone(),state:state.into(),title:task.slug.clone(),
                covers:task.covers.clone(),files:task.files.clone(),deps:task.deps.clone(),commit:task.commit.clone(),refs:vec![reference] });
        }
    }
    Ok(items)
}

fn task_location(run: &Path, raw: &str, id: &str) -> Result<String> {
    let pattern = format!(r#""id"\s*:\s*"{}""#, regex::escape(id));
    let found = regex::Regex::new(&pattern).expect("escaped task id").find(raw)
        .ok_or_else(|| Error::cannot_decide(format!("cannot locate handoff task {id} in plan.json")))?;
    Ok(format!("{}:{}", run.join("plan.json").display(), raw[..found.start()].bytes().filter(|b| *b == b'\n').count() + 1))
}

pub(super) fn check_idle(saved: &Snapshot) -> Result<()> {
    let common = git::identity(Path::new(&saved.worktree))?;
    let (trees, inventory) = worktrees(Path::new(&saved.worktree), &plan(&saved.documents)?)?;
    let observed = Value::Array(inventory).to_string();
    if text(&saved.documents, "state:worktrees") != Some(observed.as_str()) {
        return Err(Error::failed("stale handoff: retained or removed Plan worktree inventory changed; prepare again"));
    }
    let mut trees: BTreeSet<_> = trees.into_iter().collect();
    for doc in saved.documents.iter().filter(|d| d.reference.starts_with("git:worktree:")) {
        trees.insert(git::worktree_from_document(doc)?);
    }
    for tree in trees {
        if git::identity(&tree)? != common { return Err(Error::failed("handoff execution guard repository mismatch")); }
        let dir = local_directory(&tree)?.join("exec");
        for capture in entries(&dir)? {
            let meta = fs::symlink_metadata(&capture).map_err(|e| cannot("inspect exec capture", &capture, e))?;
            if !meta.is_dir() { return Err(Error::failed(format!("invalid handoff exec capture: {}", capture.display()))); }
            let exit = read_text(&capture.join("exit"), 4096)?;
            let finished = read_text(&capture.join("finished_at"), 4096)?;
            let code = exit.as_deref().and_then(|s| s.trim().parse::<u16>().ok());
            let time = finished.as_deref().and_then(|s| utc_to_epoch(s.trim()));
            if !code.is_some_and(|v| v <= 255) || time.is_none() {
                return Err(Error::failed(format!("handoff refused: active or unresolved exec capture (valid exit and finished_at required): {}", capture.display())));
            }
            if let Some(started) = read_text(&capture.join("started_at"), 4096)? {
                if !utc_to_epoch(started.trim()).is_some_and(|v| Some(v) <= time) {
                    return Err(Error::failed(format!("invalid exec capture timestamps: {}", capture.display())));
                }
            }
        }
    }
    Ok(())
}
