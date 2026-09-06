// core/target.rs
// resolve_target(): --run <id> | --quick <slug> | CURRENT, and the arguments left over.

use std::path::PathBuf;

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::meta::refresh_owner;
use crate::core::paths::is_plain_name;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TargetKind {
    Run,
    Quick,
}

#[derive(Debug)]
pub struct Target {
    pub kind: TargetKind,
    pub id: String,
    pub dir: PathBuf,
}

/// Resolve the target, renewing a heartbeat only when this session already owns the run.
pub fn resolve_target(ctx: &mut Context, args: &[String]) -> Result<(Target, Vec<String>)> {
    let mut kind: Option<TargetKind> = None;
    let mut id = String::new();
    let mut rest: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let next = args.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(&args[i], next, "run")? {
            kind = Some(TargetKind::Run);
            id = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(&args[i], next, "quick")? {
            kind = Some(TargetKind::Quick);
            id = value;
            i += eaten;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    let roots = ctx.roots()?;
    let kind = match kind {
        Some(kind) => kind,
        None => {
            id = roots.current_run_id()?.ok_or_else(|| {
                Error::failed(
                    "no current run in this worktree (dstack run new <slug>, dstack run adopt <id>, or pass --run/--quick)",
                )
            })?;
            TargetKind::Run
        }
    };
    if !is_plain_name(&id) {
        return Err(match kind {
            TargetKind::Run => Error::failed(format!("run id must be a plain name (got '{id}')")),
            TargetKind::Quick => {
                Error::failed(format!("quick slug must be a plain name (got '{id}')"))
            }
        });
    }
    let dir = match kind {
        TargetKind::Run => roots.runs.join(&id),
        TargetKind::Quick => roots.quick.join(&id),
    };
    if !dir.is_dir() {
        return Err(match kind {
            TargetKind::Run => Error::failed(format!("run not found: {id}")),
            TargetKind::Quick => Error::failed(format!("quick task not found: {id}")),
        });
    }
    if kind == TargetKind::Run {
        refresh_owner(&dir, ctx.parent_pid, &ctx.session_id)?;
    }
    Ok((Target { kind, id, dir }, rest))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::registry::Registry;
    use crate::core::roots::{Home, Roots};
    use std::rc::Rc;

    /// A store of its own under the temp directory, with one real run and one directory next to
    /// the store that nothing may ever touch.
    struct Scratch {
        base: PathBuf,
        ctx: Context,
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    fn scratch(name: &str) -> Scratch {
        let base =
            std::env::temp_dir().join(format!("dstack-target-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let store = base.join("main/.dstack");
        for dir in ["runs/20260101T000000Z_real", "quick/real", "local"] {
            std::fs::create_dir_all(store.join(dir)).expect("scratch store");
        }
        std::fs::create_dir_all(base.join("outside")).expect("outside directory");
        let home = Home::resolve().expect("repository");
        let mut ctx = Context::new(
            home,
            PathBuf::from("dstack"),
            Rc::new(Registry::new(Vec::new())),
        );
        ctx.set_roots(Roots {
            main_root: base.join("main"),
            wt_root: base.join("main"),
            runs: store.join("runs"),
            local: store.join("local"),
            quick: store.join("quick"),
            store,
        });
        Scratch { base, ctx }
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn r13__a_plain_id_resolves_without_claiming_an_unowned_run() {
        let mut s = scratch("plain");
        let (target, rest) = resolve_target(
            &mut s.ctx,
            &args(&["--run", "20260101T000000Z_real", "show"]),
        )
        .expect("resolves");
        assert_eq!(target.id, "20260101T000000Z_real");
        assert_eq!(rest, vec!["show".to_string()]);
        assert!(!target.dir.join("meta.tsv").exists());
    }

    #[test]
    fn r13__run_id_with_a_slash_is_refused() {
        let mut s = scratch("slash");
        let error =
            resolve_target(&mut s.ctx, &args(&["--run", "../../outside"])).expect_err("refused");
        assert_eq!(error.code(), 1);
        assert_eq!(
            error.message(),
            "run id must be a plain name (got '../../outside')"
        );
        assert!(
            !s.base.join("outside/meta.tsv").exists(),
            "nothing outside the store was written"
        );
    }

    #[test]
    fn r13__run_id_dotdot_is_refused() {
        let mut s = scratch("dotdot");
        for id in ["..", ".", ""] {
            let error = resolve_target(&mut s.ctx, &args(&["--run", id])).expect_err("refused");
            assert_eq!(error.code(), 1);
            assert_eq!(
                error.message(),
                format!("run id must be a plain name (got '{id}')")
            );
        }
        // A CURRENT file holding such a value is refused the same way.
        std::fs::write(s.base.join("main/.dstack/local/CURRENT"), "../outside\n").expect("CURRENT");
        let error = resolve_target(&mut s.ctx, &args(&[])).expect_err("refused");
        assert_eq!(
            error.message(),
            "run id must be a plain name (got '../outside')"
        );
        assert!(
            !s.base.join("outside/meta.tsv").exists(),
            "nothing outside the store was written"
        );
    }

    #[test]
    fn r13__quick_slug_with_a_slash_is_refused() {
        let mut s = scratch("quick");
        let error =
            resolve_target(&mut s.ctx, &args(&["--quick=../outside"])).expect_err("refused");
        assert_eq!(error.code(), 1);
        assert_eq!(
            error.message(),
            "quick slug must be a plain name (got '../outside')"
        );
        assert!(
            !s.base.join("outside/meta.tsv").exists(),
            "nothing outside the store was written"
        );
    }
}
