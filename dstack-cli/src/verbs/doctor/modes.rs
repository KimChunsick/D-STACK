// Mode configuration is checked by the same schema the runtime resolves.
use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::mode::Mode;
use crate::selftest::{Selftest, Verdict};

pub fn section(ctx: &mut Context) -> Result<bool> {
    let roots = match ctx.roots() {
        Ok(roots) => roots,
        Err(_) => {
            ctx.out.say("modes: skipped: no repository; defaults main=claude sub=codex");
            return Ok(true);
        }
    };
    let project = Mode::project(&roots)?;
    let effective = Mode::effective(&roots)?;
    ctx.out.say(&format!(
        "modes: project main={} sub={}; effective main={} sub={}",
        project.main, project.sub, effective.main, effective.sub
    ));
    Ok(true)
}

pub struct Checker;

impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "modes"
    }

    fn run(&self, _ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let text = std::fs::read_to_string(fixture).map_err(|e| {
            Error::cannot_decide(format!("cannot read {}: {e}", fixture.display()))
        })?;
        Ok(if serde_json::from_str::<Mode>(&text).is_ok() {
            Verdict::Pass
        } else {
            Verdict::Reject
        })
    }
}
