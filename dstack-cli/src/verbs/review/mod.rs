// verbs/review/mod.rs
// dstack review, check review-bundle and the review rounds: the bundle and its verdicts (R69, R70).
//
// The reviewer is Codex (the `codex-review` skill calls it, R23/R96); nothing here invokes a
// model. It only assembles what the reviewer is allowed to see and re-reads what came back.
// The whole point of R69 is that the request travels WITH the diff, so a bundle that does not
// name the same R ids in its REQUEST section, in the plan it cites and in its body is not a
// bundle: `check review-bundle` refuses it and `review` deletes what it just built.

use crate::core::args::opt;
use crate::core::error::Result;
use crate::core::verb::Verb;
use crate::selftest::Selftest;

/// R70's ceiling: past this, the answer is "split the plan", not "truncate".
pub const MAX_BUNDLE: usize = 512000;

/// One file may not drown the other files' diffs.
pub const MAX_FILE_DIFF: usize = 65536;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail {
    ($($m:tt)*) => { return Err(crate::core::error::Error::failed(format!($($m)*))) };
}

/// One roster entry; the struct carries nothing but its name. Declared before the modules that
/// use it, because a macro_rules! is in scope only for what follows it.
macro_rules! review_verb {
    ($handler:ident, $entry:literal, $body:path) => {
        struct $handler;
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

mod bundle;
mod check_bundle;
mod emit_diff;
mod findings;
mod rounds;
mod selftests;

review_verb!(Review, "review", bundle::review);
review_verb!(CheckReviewBundle, "check review-bundle", check_bundle::check);
review_verb!(ReviewSeal, "review seal", rounds::seal);
review_verb!(ReviewClose, "review close", rounds::close);

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(Review),
        Box::new(CheckReviewBundle),
        Box::new(ReviewSeal),
        Box::new(ReviewClose),
    ]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    selftests::all()
}

/// core::args::opt at one position of an argument list: `--name value`, `--name=value`, or
/// Err(Exit(1)) for the operand the shell's `shift 2` would have failed on.
fn take(args: &[String], i: usize, name: &str) -> Result<Option<(String, usize)>> {
    opt(&args[i], args.get(i + 1).map(String::as_str), name)
}

/// The records of a text as awk counts them: a trailing newline ends the last line and does not
/// start an empty one.
fn lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&str> = text.split('\n').collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_lines_count_records_the_way_awk_counts_them() {
        assert_eq!(lines("one\ntwo\n"), vec!["one", "two"]);
        assert_eq!(lines("one\ntwo"), vec!["one", "two"]);
        assert!(lines("").is_empty());
    }
}
