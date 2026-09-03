// verbs/plan/milestone.rs
// dstack milestone add: append a milestone, or take a decimal id when it is not the last (R67).

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::paths::valid_slug;
use crate::store::plan::{ensure, Milestone};
use crate::store::plan_graph::counts_line;
use crate::store::plan_ids::{next_decimal_id, next_int_id};

plan_verb!(MilestoneAdd, "milestone add", add);

fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    let (mut slug, mut after) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(arg, next, "after")? {
            after = value;
            i += eaten;
        } else if is_option(arg) {
            fail!("unknown option: {arg} (usage: dstack milestone add <slug> [--after M<n>])")
        } else if slug.is_empty() {
            slug = arg.to_string();
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if slug.is_empty() {
        fail!("usage: dstack milestone add <slug> [--after M<n>]")
    }
    if !valid_slug(&slug) {
        fail!("slug must match [a-z0-9][a-z0-9-]* (got '{slug}')")
    }
    ensure(&target.dir)?;

    let mut doc = target.load()?;
    let ids: Vec<String> = doc.milestones.iter().map(|m| m.id.clone()).collect();
    let last = ids.last().cloned().unwrap_or_default();
    if !after.is_empty() && !ids.contains(&after) {
        fail!("milestone not found: {after} (known: {})", ids.join(" "))
    }
    // Appending to the end keeps whole numbers; inserting in the middle takes a decimal so the
    // ids of everything after it never shift (R67).
    let id = if after.is_empty() || after == last {
        after.clear();
        next_int_id(&doc, "M")
    } else {
        next_decimal_id(&after, &ids)?
    };

    let new = Milestone {
        id: id.clone(),
        slug: slug.clone(),
        order: 0,
    };
    let mut milestones: Vec<Milestone> = Vec::new();
    for milestone in doc.milestones {
        let here = milestone.id == after;
        milestones.push(milestone);
        if here {
            milestones.push(new.clone());
        }
    }
    if after.is_empty() {
        milestones.push(new);
    }
    for (index, milestone) in milestones.iter_mut().enumerate() {
        milestone.order = index as u32 + 1;
    }
    doc.milestones = milestones;

    let doc = target.write(doc)?;
    say!(ctx, "milestone {id}: {slug}");
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}
