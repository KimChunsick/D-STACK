// verbs/ledger/mod.rs
// dstack cases, evidence, check coverage and worker report: the one requirement ledger (R73).

use crate::core::error::{Error, Result};
use crate::core::target::TargetKind;
use crate::core::verb::Verb;
use crate::selftest::{Selftest, Verdict};

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(crate::core::error::Error::failed(format!($($m)*))) }; }

/// One roster entry; the struct carries nothing but its name. Declared before the modules that
/// use it, because a macro_rules! is in scope only for what follows it.
macro_rules! ledger_verb {
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

pub mod artifact;
pub mod cases;
pub mod coverage;
pub mod evidence;
pub mod evidence_selftests;
pub mod retire;
pub mod worker;

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(cases::CasesSync),
        Box::new(cases::CasesRender),
        Box::new(evidence::EvidenceAdd),
        Box::new(retire::EvidenceRetire),
        Box::new(coverage::CheckCoverage),
        Box::new(worker::WorkerReport),
    ]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![
        Box::new(evidence_selftests::EvidenceAdd),
        Box::new(evidence_selftests::EvidenceRetire),
        Box::new(coverage::CoverageSelftest),
        Box::new(worker::WorkerSelftest),
    ]
}

/// What a fixture run proved, read off the exit code of the one call that judges it: 0 is the
/// checker passing, 1 is the refusal a bad-* fixture has to provoke. Anything else — a die()
/// that could not decide, a sandbox that never came up — is a runner failure, never a
/// rejection: reporting it as reject would let a broken checker read as a working one.
fn verdict(code: i32, what: &str) -> Result<Verdict> {
    match code {
        0 => Ok(Verdict::Pass),
        1 => Ok(Verdict::Reject),
        code => Err(Error::cannot_decide(format!(
            "selftest: {what} exited {code} instead of deciding"
        ))),
    }
}

/// "$TARGET_KIND": the word every line of these verbs prints next to the target id.
fn kind_word(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Run => "run",
        TargetKind::Quick => "quick",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r05_a_checker_that_could_not_run_is_not_a_rejection() {
        assert_eq!(
            verdict(0, "dstack cases sync").expect("pass"),
            Verdict::Pass
        );
        assert_eq!(
            verdict(1, "dstack cases sync").expect("reject"),
            Verdict::Reject
        );
        for code in [2, 5, 127] {
            let error = verdict(code, "dstack cases sync").expect_err("cannot decide");
            assert_eq!(error.code(), 2);
            assert_eq!(
                error.message(),
                format!("selftest: dstack cases sync exited {code} instead of deciding")
            );
        }
    }
}
