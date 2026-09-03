// verbs/plan/mod.rs
// dstack milestone, plan, task and next: the roadmap (ported in P10).

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, utc_now, with_lock};
use crate::core::meta::meta_get;
use crate::core::paths::base_name;
use crate::core::roots::{git_out, Roots};
use crate::core::target::{resolve_target, TargetKind};
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::plan::PlanDoc;
use crate::store::plan_graph::{refresh, render_roadmap, render_state};

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(crate::core::error::Error::failed(format!($($m)*))) }; }

/// One roster entry; the struct carries nothing but its name. Declared before the modules that
/// use it, because a macro_rules! is in scope only for what follows it.
macro_rules! plan_verb {
    ($handler:ident, $entry:literal, $body:path) => {
        pub(super) struct $handler;
        impl crate::core::verb::Verb for $handler {
            fn name(&self) -> &'static str {
                $entry
            }
            fn run(
                &self,
                ctx: &mut crate::core::context::Context,
                args: &[String],
            ) -> crate::core::error::Result<()> {
                $body(ctx, args)
            }
        }
    };
}

pub mod add;
pub mod edit;
pub mod lifecycle;
pub mod milestone;
pub mod next;
pub mod selftests;
pub mod task;

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(milestone::MilestoneAdd),
        Box::new(add::PlanAdd),
        Box::new(add::PlanInsert),
        Box::new(edit::PlanRemove),
        Box::new(edit::PlanEdit),
        Box::new(lifecycle::PlanRender),
        Box::new(lifecycle::PlanStart),
        Box::new(lifecycle::PlanDone),
        Box::new(task::TaskAdd),
        Box::new(task::TaskDone),
        Box::new(next::Next),
    ]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    selftests::all()
}

/// What _plan_target() leaves behind: the run directory that holds plan.json, and the roots the
/// lock and the generated documents need.
pub(crate) struct Target {
    pub dir: PathBuf,
    pub roots: Roots,
}

/// _plan_target(): resolve --run/--quick/CURRENT and refuse the target that structurally cannot
/// hold a plan. The verb's own arguments come back as the second half of the pair.
pub(crate) fn plan_target(ctx: &mut Context, args: &[String]) -> Result<(Target, Vec<String>)> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    if target.kind == TargetKind::Quick {
        fail!("quick tasks have no plans")
    }
    Ok((
        Target {
            dir: target.dir,
            roots,
        },
        rest,
    ))
}

impl Target {
    /// _plan_require(): every verb but `milestone add` and `plan add` needs the file to be there.
    pub(crate) fn require(&self) -> Result<()> {
        if crate::store::plan::exists(&self.dir) {
            return Ok(());
        }
        Err(Error::failed(format!(
            "no plan.json in {} — start with: dstack milestone add <slug>",
            self.dir.display()
        )))
    }

    pub(crate) fn load(&self) -> Result<PlanDoc> {
        crate::store::plan::load(&self.dir)
    }

    /// _plan_write(): the refresh, the atomic write and both documents, under the store lock —
    /// one call so no mutation path can forget half of it. What comes back is what the file now
    /// holds, which is what the counts line of every verb reports.
    pub(crate) fn write(&self, mut doc: PlanDoc) -> Result<PlanDoc> {
        refresh(&mut doc);
        let _lock = with_lock(&self.roots.local)?;
        doc.commit(&self.dir, &base_name(&self.dir), &self.worktree()?)?;
        Ok(doc)
    }

    /// _plan_regen(): ROADMAP.md and STATE.md alone. `plan render` is its only caller — every
    /// other path goes through write(), which regenerates them as part of the commit.
    pub(crate) fn regen(&self, doc: &PlanDoc) -> Result<()> {
        let _lock = with_lock(&self.roots.local)?;
        let run = base_name(&self.dir);
        write_file(&self.dir.join("ROADMAP.md"), &render_roadmap(doc, &run))?;
        let last = git_out(Some(&self.worktree()?), &["rev-parse", "--short", "HEAD"])
            .unwrap_or_else(|| "none".to_string());
        write_file(
            &self.dir.join("STATE.md"),
            &render_state(doc, &run, &last, &utc_now()),
        )
    }

    /// The checkout STATE.md's last_commit is read from: the run's own worktree while it exists,
    /// and the worktree dstack was invoked in otherwise.
    pub(crate) fn worktree(&self) -> Result<PathBuf> {
        Ok(match meta_get(&self.dir, "worktree")? {
            Some(wt) if !wt.is_empty() && Path::new(&wt).is_dir() => PathBuf::from(wt),
            _ => self.roots.wt_root.clone(),
        })
    }
}

fn write_file(file: &Path, text: &str) -> Result<()> {
    atomic_write(file, text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))
}
