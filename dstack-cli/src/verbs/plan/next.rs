// verbs/plan/next.rs
// dstack next: ready plans, overlapping pairs with reasons, the cap and the schedulable set.

use std::path::PathBuf;

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::meta::meta_get;
use crate::core::paths::{base_name, paths_overlap, shell_int};
use crate::core::tools::policy_get;
use crate::store::plan::{self, Plan, PlanDoc};

plan_verb!(Next, "next", next);

fn next(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let mut max = String::new();
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match opt(arg, rest.get(i + 1).map(String::as_str), "max")? {
            Some((value, eaten)) => {
                max = value;
                i += eaten;
            }
            // The loop of this verb has no positional arm at all.
            None => fail!("unknown option: {arg} (usage: dstack next [--max N])"),
        }
    }
    // The cap is printed as it was written, so a policy value of "03" reads as "03".
    let (cap, cap_text, cap_source) = if !max.is_empty() {
        if !max.chars().all(|c| c.is_ascii_digit()) {
            fail!("--max must be a positive integer (got '{max}')")
        }
        // A digit-only operand no i64 can hold is where bash's `[ "$max" -ge 1 ]` gives up: the
        // test builtin fails on the number and the verb lands on the refusal a zero lands on.
        // The diagnostic bash prints next to it names the reference's next.sh, not reproduced.
        let n: i64 = max.parse().unwrap_or(0);
        if n < 1 {
            fail!("--max must be at least 1")
        }
        (n, max, "--max")
    } else {
        match policy_get(&target.roots.store, "max_concurrent") {
            // A policy value is never compared, only subtracted from, so an overflowing one is
            // not refused: it folds into an intmax_t the way every bash arithmetic literal does.
            Some(value) if !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()) => {
                (shell_int(&value), value, "PROJECT.md max_concurrent")
            }
            _ => (3, "3".to_string(), "default"),
        }
    };

    let doc = target.load()?;
    let ready = ids_with(&doc, "ready");
    let in_progress = ids_with(&doc, "in-progress");
    say!(ctx, "ready:       {}", or_none(&ready));
    say!(ctx, "in-progress: {}", or_none(&in_progress));

    // Pairwise overlap over ready ∪ in-progress: those are the plans that could run at the same
    // time, and R66 makes file overlap the only reason two plans may not.
    let mut candidates: Vec<&Plan> = ready.clone();
    candidates.extend(in_progress.iter().copied());
    let mut pairs = 0;
    say!(ctx, "overlaps:");
    for a in &candidates {
        for b in &candidates {
            // Each unordered pair once, in the order the ids sort in.
            if a.id >= b.id {
                continue;
            }
            for line in overlap_lines(a, b) {
                say!(ctx, "  {} ↔ {}: {line}", a.id, b.id);
                pairs += 1;
            }
        }
    }
    if pairs == 0 {
        say!(ctx, "  (none)");
    }
    say!(ctx, "  overlapping file pairs: {pairs}");

    let running = in_progress.len() as i64;
    // `$((cap - n_inprog))`: bash wraps at intmax_t, and a negative result is clamped to zero.
    let free = cap.wrapping_sub(running).max(0);
    say!(
        ctx,
        "cap:         {cap_text} ({cap_source}); in-progress {running}; free slots {free}"
    );

    // Greedy in array order: take a ready plan when it collides with nothing running and nothing
    // already picked. Greedy (not optimal) is deliberate — the order in plan.json is the order
    // the owner wrote, and a "better" set that reorders their intent is not better.
    let mut picked: Vec<&Plan> = Vec::new();
    for plan in &ready {
        if picked.len() as i64 >= free {
            break;
        }
        let clash = in_progress
            .iter()
            .chain(picked.iter())
            .any(|other| !overlap_lines(plan, other).is_empty());
        if !clash {
            picked.push(plan);
        }
    }
    say!(
        ctx,
        "schedulable: {} — {} of {free} free slot(s)",
        or_none(&picked),
        picked.len()
    );

    // R38: another Goal's plans are a warning, never a block — two runs may legitimately touch
    // the same file, and the real gate is the branch-containment check at close.
    let mine: Vec<&String> = doc
        .plans
        .iter()
        .filter(|plan| plan.status != "done")
        .flat_map(|plan| plan.files.iter())
        .collect();
    let mut warnings = 0;
    for other in open_runs(&target)? {
        let oid = base_name(&other);
        let odoc = plan::load(&other)?;
        for plan in odoc.plans.iter().filter(|plan| plan.status != "done") {
            for path in &plan.files {
                // One line per (other run, other plan, other path): which of OUR plans collides
                // does not change the advice, and three identical warnings read as three problems.
                if mine.iter().any(|file| paths_overlap(file, path)) {
                    say!(
                        ctx,
                        "warning: overlaps run {oid} plan {} on {path}",
                        plan.id
                    );
                    warnings += 1;
                }
            }
        }
    }
    say!(ctx, "cross-run warnings: {warnings}");
    Ok(())
}

fn ids_with<'a>(doc: &'a PlanDoc, status: &str) -> Vec<&'a Plan> {
    doc.plans.iter().filter(|p| p.status == status).collect()
}

/// _or_none(): the ids separated by a space, and "(none)" when there are none.
fn or_none(plans: &[&Plan]) -> String {
    let ids: Vec<&str> = plans.iter().map(|p| p.id.as_str()).collect();
    match ids.is_empty() {
        true => "(none)".to_string(),
        false => ids.join(" "),
    }
}

/// _overlap_lines(): every pair of declared paths that cannot be held by two workers at once.
fn overlap_lines(a: &Plan, b: &Plan) -> Vec<String> {
    let mut lines = Vec::new();
    for pa in &a.files {
        for pb in &b.files {
            if !paths_overlap(pa, pb) {
                continue;
            }
            let why = match pa == pb {
                true => "same path",
                false => "directory prefix",
            };
            lines.push(format!("{pa} overlaps {pb} ({why})"));
        }
    }
    lines
}

/// The other runs of the store that are open and carry a plan, in the order the shell's glob
/// walks them.
fn open_runs(target: &super::Target) -> Result<Vec<PathBuf>> {
    let me = base_name(&target.dir);
    let mut dirs: Vec<PathBuf> = match std::fs::read_dir(&target.roots.runs) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect(),
        Err(_) => Vec::new(),
    };
    dirs.sort();
    let mut open = Vec::new();
    for dir in dirs {
        if base_name(&dir) == me || !dir.join("plan.json").is_file() {
            continue;
        }
        if meta_get(&dir, "status")?.unwrap_or_default() == "open" {
            open.push(dir);
        }
    }
    Ok(open)
}
