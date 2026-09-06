// Explicit run handoff: changing a project default never changes an active snapshot.
use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::mode::Mode;
use crate::core::tools::tool_check_for_mode;
use crate::store::request::RequestDoc;

pub fn adopt(ctx: &mut Context, dir: &Path, refresh: bool) -> Result<()> {
    let roots = ctx.roots()?;
    let mode = if refresh { Mode::project(&roots)? } else { Mode::for_run(&roots, dir)? };
    if refresh {
        let request = dir.join("request.md");
        let fields = if request.exists() {
            let doc = RequestDoc::load(&request)?;
            ["e2e", "review", "visual", "unit_tests"].iter()
                .map(|key| format!("{key}={}", doc.field(key).unwrap_or_default()))
                .collect::<Vec<_>>()
        } else { Vec::new() };
        if tool_check_for_mode(ctx, &fields, &mode, true)? != 0 {
            return Err(Error::failed("cannot refresh mode: a selected provider or required tool is missing"));
        }
        mode.snapshot(dir)?;
    }
    ctx.out.say(&format!(
        "mode: main={} sub={} ({})", mode.main, mode.sub,
        if refresh { "refreshed from project; use the selected main host" } else { "snapshot preserved" }
    ));
    Ok(())
}
