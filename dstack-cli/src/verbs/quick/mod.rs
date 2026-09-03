// verbs/quick/mod.rs
// dstack quick: the track for work that is not a Goal (R99) — new, list, status, resume, close.
//
// A quick task uses the SAME request format, the same ledger and the same checkers as a run; the
// only difference is the frontmatter (most stages off) and that there is no plan.json, which is
// why `review: off` is the one place in the pipeline where a skipped review is real. It never
// touches CURRENT: a Goal run and a quick task coexist in one worktree, and the Stop gate checks
// both (R33) through state::open_slugs.

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::paths::is_plain_name;
use crate::core::verb::Verb;
use crate::selftest::Selftest;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(crate::core::error::Error::failed(format!($($m)*))) }; }

/// One roster entry; the struct carries nothing but its name. Declared before the modules that
/// use it, because a macro_rules! is in scope only for what follows it.
macro_rules! quick_verb {
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

pub mod close;
pub mod new;
pub mod selftests;
pub mod state;
pub mod view;

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(new::QuickNew),
        Box::new(view::QuickList),
        Box::new(view::QuickStatus),
        Box::new(view::QuickResume),
        Box::new(close::QuickClose),
    ]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(selftests::QuickNew)]
}

/// _quick_require_dir(): the directory of one quick task, and the one place every quick verb
/// resolves a positional slug.
///
/// D-10: the shell joins the slug into `$QUICK` unchecked, so `/tmp` replaces the quick root
/// outright and `.` or `..` reads a directory outside the quick tree — `quick status /tmp`
/// answers 0 wherever /tmp exists. That defect is not reproduced. A slug that is not a plain
/// name is refused here with the wording resolve_target already gives `--run` and `--quick`,
/// before anything is read or written; step 33 declares the divergence per call.
fn require_dir(quick: &Path, slug: &str, verb: &str) -> Result<PathBuf> {
    if slug.is_empty() {
        return Err(Error::failed(format!("usage: dstack quick {verb} <slug>")));
    }
    if !is_plain_name(slug) {
        return Err(Error::failed(format!(
            "quick slug must be a plain name (got '{slug}')"
        )));
    }
    let dir = quick.join(slug);
    if !dir.is_dir() {
        return Err(Error::failed(format!(
            "quick task not found: {} (dstack quick list)",
            dir.display()
        )));
    }
    Ok(dir)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r11__a_slug_that_is_a_path_never_reaches_the_filesystem() {
        let quick = Path::new("/nowhere/.dstack/quick");
        for slug in ["/tmp", "../x", "a/b", ".", "..", "/"] {
            let refused = require_dir(quick, slug, "status").expect_err("refused");
            assert_eq!(refused.code(), 1);
            assert_eq!(
                refused.message(),
                format!("quick slug must be a plain name (got '{slug}')")
            );
        }
        let usage = require_dir(quick, "", "resume").expect_err("refused");
        assert_eq!(usage.message(), "usage: dstack quick resume <slug>");
        // A plain name still reaches the "not found" answer, not the refusal above.
        let missing = require_dir(quick, "real", "close").expect_err("refused");
        assert!(
            missing.message().starts_with("quick task not found:"),
            "{}",
            missing.message()
        );
    }
}
