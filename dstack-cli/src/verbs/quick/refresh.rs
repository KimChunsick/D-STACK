// Explicit quick snapshot transition; ordinary resume remains a read-only missing-work report.
use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::mode::{Mode, Provider};
use crate::core::tools::tool_check_for_mode;
use crate::store::request::RequestDoc;

use super::{require_dir, state};

pub(super) fn parse(args: &[String]) -> Result<(String, Option<Provider>)> {
    let (mut slug, mut refresh, mut host) = (None, false, None);
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--refresh-mode" if !refresh => refresh = true,
            "--host" if host.is_none() => {
                i += 1;
                host = Some(Provider::parse(args.get(i).ok_or_else(usage)?)?);
            }
            _ if arg.starts_with("--host=") && host.is_none() => {
                host = Some(Provider::parse(&arg[7..])?);
            }
            _ if !arg.is_empty() && !arg.starts_with('-') && slug.is_none() => {
                slug = Some(arg.clone());
            }
            _ => return Err(usage()),
        }
        i += 1;
    }
    if refresh != host.is_some() {
        return Err(usage());
    }
    Ok((slug.ok_or_else(usage)?, host))
}

pub(super) fn run(ctx: &mut Context, slug: &str, host: Provider) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let dir = require_dir(&roots.quick, slug, "resume")?;
    // Quick state belongs to this worktree, even though project defaults live in the main
    // store. Reject redirected roots and target directories before reading their state.
    let worktree = canonical(&roots.wt_root)?;
    let quick = worktree.join(".dstack/quick");
    require_path(&roots.quick, &quick)?;
    require_path(&dir, &quick.join(slug))?;
    let rows = state::rows(&roots.quick)?;
    let matching: Vec<_> = rows.iter().filter(|row| row.slug == slug).collect();
    if matching.len() != 1 || matching[0].status != "open" {
        fail!("quick mode refresh requires exactly one open STATE row for {slug}");
    }
    let previous = Mode::for_run(&roots, &dir)?;
    let mode = Mode::project(&roots)?;
    if mode.main != host {
        fail!("main host mismatch: project {}, current {host}; launch {} in a new session before quick mode refresh", mode.main, mode.main);
    }
    let doc = RequestDoc::load(&dir.join("request.md"))?;
    let fields: Vec<String> = ["e2e", "review", "visual", "unit_tests"]
        .iter()
        .map(|field| format!("{field}={}", doc.field(field).unwrap_or_default()))
        .collect();
    let need_sub = doc.field("review").as_deref() == Some("on")
        || doc.field("external_research").as_deref() == Some("one-pass");
    if tool_check_for_mode(ctx, &fields, &mode, need_sub)? != 0 {
        fail!("quick mode refresh refused: a tool required by the selected providers or request fields is missing");
    }

    // All validation precedes the single atomic mode.json replacement. There is no quick
    // session owner or shared close lock: this is not a Goal handoff or close serialization.
    mode.snapshot(&dir)?;
    say!(ctx, "quick mode refreshed: {slug}");
    say!(
        ctx,
        "  previous: main={} sub={}",
        previous.main,
        previous.sub
    );
    say!(ctx, "  current: main={} sub={}", mode.main, mode.sub);
    say!(ctx, "  snapshot: {}", dir.join("mode.json").display());
    ctx.out.say(
        "  engine unchanged; Goal ownership unchanged; CURRENT and other task files preserved",
    );
    say!(ctx, "  next: dstack mode show --host {host} --quick {slug}");
    say!(ctx, "  then: dstack quick resume {slug}");
    Ok(())
}

fn canonical(path: &Path) -> Result<PathBuf> {
    std::fs::canonicalize(path).map_err(|error| {
        Error::cannot_decide(format!(
            "cannot resolve quick path {}: {error}",
            path.display()
        ))
    })
}

fn require_path(path: &Path, expected: &Path) -> Result<()> {
    if canonical(path)? != expected {
        fail!(
            "quick mode refresh path must stay in this worktree: {}",
            path.display()
        );
    }
    Ok(())
}

fn usage() -> Error {
    Error::failed(
        "usage: dstack quick resume <slug> [--refresh-mode --host claude|codex] (each option once)",
    )
}
