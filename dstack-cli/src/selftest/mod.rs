// selftest/mod.rs
// The Selftest trait every checker registers: one verdict per fixture (R100).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::Result;

pub mod sandbox;
pub mod writers;

/// What a fixture proved. The runner prints the word and compares it to the fixture's name:
/// bad-* must read reject, good-* must read pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Pass,
    Reject,
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Reject => "reject",
        }
    }
}

pub trait Selftest {
    /// The fixture directory name under claude/lint/fixtures, e.g. "evidence-add".
    fn checker(&self) -> &'static str;

    /// Runs the checker against one fixture file. An Err is the runner's "cannot decide", not a
    /// rejection: a fixture that provokes a refusal returns Ok(Verdict::Reject).
    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict>;
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r05__verdicts_read_as_the_runner_prints_them() {
        assert_eq!(Verdict::Pass.as_str(), "pass");
        assert_eq!(Verdict::Reject.as_str(), "reject");
    }
}
