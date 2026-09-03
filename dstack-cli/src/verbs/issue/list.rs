// verbs/issue/list.rs
// dstack issue list: one row per filed issue, the friction hit most recently first.

use std::io::ErrorKind;
use std::path::Path;

use crate::core::args::{is_option, unknown_option};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::read_text;
use crate::core::paths::base_name;

use super::file::{summary, Summary};

issue_verb!(IssueList, "issue list", list);

fn list(ctx: &mut Context, args: &[String]) -> Result<()> {
    if let Some(arg) = args.first() {
        if is_option(arg) {
            return Err(unknown_option(arg));
        }
        fail!("unexpected argument: {arg} (usage: dstack issue list)")
    }
    let dir = super::folder()?;
    let mut rows = read(&dir)?;
    // The friction hit last is the one worth reading first; two files seen at the same second
    // sort by title, so the order never depends on what the directory happens to return.
    rows.sort_by(|a, b| b.last.cmp(&a.last).then(a.title.cmp(&b.title)));
    say!(ctx, "issues: {}", dir.display());
    for row in &rows {
        say!(
            ctx,
            "  {} | sightings {} | last {}",
            row.title,
            row.sightings,
            row.last
        );
    }
    say!(ctx, "issues {}", rows.len());
    Ok(())
}

/// One row per .md file of the folder. D-12: a folder that is not there is nothing filed yet, and
/// stays a count of 0; a folder or an entry that IS there and cannot be read is a cannot-decide,
/// because a row silently dropped would make the closing count say the folder holds less than it
/// does. Nothing is printed on that path, so there is no count line to disbelieve.
fn read(dir: &Path) -> Result<Vec<Summary>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(Error::cannot_decide(format!(
                "cannot read {}: {e}",
                dir.display()
            )))
        }
    };
    let mut rows: Vec<Summary> = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", dir.display())))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        rows.push(summary(
            &read_text(&path)?.unwrap_or_default(),
            &base_name(&path),
        ));
    }
    Ok(rows)
}
