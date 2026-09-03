// core/verb.rs
// The Verb trait: one struct per roster entry, found by its roster name.

use crate::core::context::Context;
use crate::core::error::Result;

pub trait Verb {
    /// The roster name of the shell reference's help.sh: "run new", "status", "lint-ko", "hook".
    fn name(&self) -> &'static str;

    /// The arguments are what the shell handler received: the roster name is already consumed.
    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()>;
}
