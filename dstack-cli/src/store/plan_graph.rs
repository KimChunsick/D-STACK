// store/plan_graph.rs
// The plan graph and the documents it generates: readiness, cycles, dependents, ROADMAP, STATE.

use crate::store::plan::{Plan, PlanDoc};

/// _PLAN_REFRESH: `ready` is derived, never authoritative — every write recomputes it. Plans that
/// are in-progress or done are owned by plan start/done and are never downgraded here.
pub fn refresh(doc: &mut PlanDoc) {
    let done: Vec<String> = doc
        .plans
        .iter()
        .filter(|p| p.status == "done")
        .map(|p| p.id.clone())
        .collect();
    for plan in doc.plans.iter_mut() {
        if plan.status != "pending" && plan.status != "ready" {
            continue;
        }
        let ready = plan.deps.iter().all(|d| done.contains(d));
        plan.status = if ready { "ready" } else { "pending" }.to_string();
    }
}

pub fn acyclic_plans(doc: &PlanDoc) -> bool {
    acyclic(doc.plans.iter().map(|p| (&p.id, &p.deps)).collect())
}

pub fn acyclic_tasks(doc: &PlanDoc) -> bool {
    acyclic(
        doc.plans
            .iter()
            .flat_map(|p| p.tasks.iter().map(|t| (&t.id, &t.deps)))
            .collect(),
    )
}

/// Peel the nodes that have no remaining dependency; whatever survives is in a cycle. Deps that
/// name an id outside the graph are dropped first, as the jq prelude does.
fn acyclic(nodes: Vec<(&String, &Vec<String>)>) -> bool {
    let ids: Vec<&String> = nodes.iter().map(|(id, _)| *id).collect();
    let mut graph: Vec<(&String, Vec<&String>)> = nodes
        .iter()
        .map(|(id, deps)| (*id, deps.iter().filter(|d| ids.contains(d)).collect()))
        .collect();
    loop {
        let free: Vec<&String> = graph
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(id, _)| *id)
            .collect();
        if free.is_empty() {
            return graph.is_empty();
        }
        graph = graph
            .into_iter()
            .filter(|(_, deps)| !deps.is_empty())
            .map(|(id, deps)| {
                let kept = deps.into_iter().filter(|d| !free.contains(d)).collect();
                (id, kept)
            })
            .collect();
    }
}

/// The seed plus every plan that transitively depends on it — the affected subtree R67 protects.
/// Sorted and unique, because the jq program ends in `unique`.
pub fn dependents(doc: &PlanDoc, seed: &str) -> Vec<String> {
    let mut found = vec![seed.to_string()];
    loop {
        let grown: Vec<String> = doc
            .plans
            .iter()
            .filter(|p| !found.contains(&p.id))
            .filter(|p| p.deps.iter().any(|d| found.contains(d)))
            .map(|p| p.id.clone())
            .collect();
        if grown.is_empty() {
            break;
        }
        found.extend(grown);
    }
    found.sort();
    found.dedup();
    found
}

/// The ids of that subtree that are in-progress, in plan order — empty means the subtree is free.
pub fn subtree_busy(doc: &PlanDoc, seed: &str) -> String {
    let ids = dependents(doc, seed);
    doc.plans
        .iter()
        .filter(|p| ids.contains(&p.id) && p.status == "in-progress")
        .map(|p| p.id.clone())
        .collect::<Vec<String>>()
        .join(", ")
}

pub fn plan_covers(doc: &PlanDoc, plan_id: &str) -> Vec<String> {
    covers(doc.plans.iter().filter(|p| p.id == plan_id))
}

pub fn milestone_covers(doc: &PlanDoc, mid: &str) -> Vec<String> {
    covers(doc.plans.iter().filter(|p| p.milestone == mid))
}

/// `sort -u` over the covers of the selected plans.
fn covers<'a>(plans: impl Iterator<Item = &'a Plan>) -> Vec<String> {
    let mut all: Vec<String> = plans
        .flat_map(|p| p.tasks.iter().flat_map(|t| t.covers.iter().cloned()))
        .collect();
    all.sort();
    all.dedup();
    all
}

/// The tasks covering an R id, as `P1/T3`.
pub fn tasks_covering(doc: &PlanDoc, r: &str) -> Vec<String> {
    doc.plans
        .iter()
        .flat_map(|p| {
            p.tasks
                .iter()
                .filter(|t| t.covers.iter().any(|c| c == r))
                .map(move |t| format!("{}/{}", p.id, t.id))
        })
        .collect()
}

/// _plan_counts: the one line every plan verb echoes after a mutation.
pub fn counts_line(doc: &PlanDoc) -> String {
    let with = |status: &str| doc.plans.iter().filter(|p| p.status == status).count();
    let tasks: Vec<&crate::store::plan::Task> =
        doc.plans.iter().flat_map(|p| p.tasks.iter()).collect();
    format!(
        "milestones {}, plans {} (pending {}, ready {}, in-progress {}, done {}), tasks {} (committed {})",
        doc.milestones.len(),
        doc.plans.len(),
        with("pending"),
        with("ready"),
        with("in-progress"),
        with("done"),
        tasks.len(),
        tasks.iter().filter(|t| !t.commit.is_empty()).count()
    )
}

/// _plan_table: one section per milestone, the body of ROADMAP.md and of `plan render`.
pub fn render_table(doc: &PlanDoc) -> String {
    let mut out = String::new();
    for milestone in &doc.milestones {
        out.push_str(&format!("## {} — {}\n\n", milestone.id, milestone.slug));
        out.push_str("| plan | slug | status | deps | files | tasks (id: covers) |\n");
        out.push_str("|---|---|---|---|---|---|\n");
        let plans: Vec<&Plan> = doc
            .plans
            .iter()
            .filter(|p| p.milestone == milestone.id)
            .collect();
        if plans.is_empty() {
            out.push_str("| — | (no plans yet) | — | — | — | — |\n");
        }
        for plan in plans {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                plan.id,
                plan.slug,
                plan.status,
                dash_or(&plan.deps.join(", ")),
                plan.files.join(", "),
                dash_or(
                    &plan
                        .tasks
                        .iter()
                        .map(|t| format!("{}: {}", t.id, t.covers.join("+")))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            ));
        }
        out.push('\n');
    }
    out
}

fn dash_or(text: &str) -> String {
    if text.is_empty() {
        "—".to_string()
    } else {
        text.to_string()
    }
}

pub fn render_roadmap(doc: &PlanDoc, run_id: &str) -> String {
    format!(
        "# Roadmap — {run_id}\n\nGenerated by dstack from plan.json. Never hand-edit: the next mutation overwrites it.\n\n{}{}\n",
        render_table(doc),
        counts_line(doc)
    )
}

pub fn render_state(doc: &PlanDoc, run_id: &str, last_commit: &str, updated_at: &str) -> String {
    let ids_of = |status: &str| {
        doc.plans
            .iter()
            .filter(|p| p.status == status)
            .map(|p| p.id.clone())
            .collect::<Vec<String>>()
            .join(", ")
    };
    let current = doc
        .plans
        .iter()
        .find(|p| p.status == "in-progress")
        .map(|p| p.id.as_str())
        .unwrap_or("");
    format!(
        "# State — {run_id}\n\nGenerated by dstack from plan.json. Never hand-edit.\n\ncurrent_plan: {current}\nready: {}\nin_progress: {}\nblocked: {}\ndone: {}\nlast_commit: {last_commit}\nupdated_at: {updated_at}\n",
        ids_of("ready"),
        ids_of("in-progress"),
        ids_of("pending"),
        ids_of("done"),
    )
}
