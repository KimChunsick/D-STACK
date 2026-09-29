// verbs/request/section.rs
// dstack request section: one prose section of an unapproved request, every other byte kept (R02).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::read_text;
use crate::core::target::resolve_target;
use crate::store::request_sections::{
    check_prose, label, replace, Place, REQUIREMENTS, SECTION_KEYS,
};

use super::{counts, is_approved, load, require_file, rowfile, take};

pub fn section(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut key, mut from) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if let Some((value, eaten)) = take(&rest, i, "from")? {
            from = value;
            i += eaten;
        } else if arg.starts_with('-') {
            fail!("unknown option: {arg}");
        } else if key.is_empty() {
            key = arg;
            i += 1;
        } else {
            fail!("unexpected argument: {arg}");
        }
    }
    if key.is_empty() || from.is_empty() {
        fail!(
            "usage: dstack request section <key> --from <file> (keys: {})",
            keys()
        );
    }
    if key == "requirements" || key == REQUIREMENTS {
        fail!("## {REQUIREMENTS} holds the R rows; they are written with dstack req add and the req verbs, never as prose");
    }
    let place = match Place::of_key(&key) {
        Some(place) => place,
        None => fail!("unknown section key: {key} (keys: {})", keys()),
    };
    let file = require_file(&target)?;
    // The approval hash covers the prose too: once stamped, a rewrite here would be a change
    // nobody approved, so the writer refuses instead of leaving a hash mismatch behind.
    if is_approved(&target) {
        fail!(
            "{} is approved; request section writes only before approval",
            file.display()
        );
    }
    let content = match read_text(Path::new(&from))? {
        Some(content) => content,
        None => fail!("--from file not found: {from}"),
    };
    check_prose(&content, &place)?;
    let doc = load(&target)?;
    let text = replace(doc.text(), &place, &content)
        .map_err(|e| Error::failed(format!("{}: {}", file.display(), e.message())))?;
    rowfile::write(&file, &text)?;
    let doc = load(&target)?;
    let (rows, _live, pending) = counts(&doc);
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  wrote section {key}: {}", label(doc.text(), &place));
    say!(
        ctx,
        "  rows {rows}, pending {pending}, lines {}",
        doc.line_count()
    );
    Ok(())
}

fn keys() -> String {
    let mut keys = vec!["summary"];
    keys.extend(SECTION_KEYS.iter().map(|(key, _)| *key));
    keys.join(", ")
}
