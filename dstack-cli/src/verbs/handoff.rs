// Prepare with the destination provider; only an explicit new main session may resume.
use super::{exec, mode::provider};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::meta::meta_get;
use crate::core::mode::{self, Provider};
use crate::core::paths::is_plain_name;
use crate::core::target::{Target, TargetKind};
use crate::core::verb::Verb;
use crate::handoff::{history, packet, recover, resume, snapshot, summary};
use std::path::{Path, PathBuf};

struct Handoff(&'static str);
impl Verb for Handoff {
    fn name(&self) -> &'static str {
        self.0
    }
    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        if self.0 == "handoff recover-owner" {
            recover_command(ctx, args)
        } else if self.0 == "handoff resume" {
            resume_command(ctx, args)
        } else {
            prepare(ctx, args)
        }
    }
}
pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(Handoff("handoff")),
        Box::new(Handoff("handoff resume")),
        Box::new(Handoff("handoff recover-owner")),
    ]
}

fn prepare(ctx: &mut Context, args: &[String]) -> Result<()> {
    let options = Options::parse(args, false)?;
    let to = Provider::parse(options.to.ok_or_else(usage)?)?;
    let roots = ctx.roots()?;
    roots.require_store()?;
    let target = target(ctx, options.run)?;
    let state = snapshot::collect(&roots, &target)?;
    if to == state.mode.main {
        return Err(Error::failed(
            "handoff destination must differ from the current main",
        ));
    }
    let session = options.session.unwrap_or(&state.owner_session);
    if session.trim().is_empty() || session != state.owner_session {
        return Err(Error::failed("--session must match the saved run owner_session; use the source run/session, not the latest history"));
    }
    let history_path = match options.history {
        Some(path) => PathBuf::from(path),
        None => match meta_get(&target.dir, "transcript_path")?.filter(|p| !p.is_empty()) {
            Some(path) => PathBuf::from(path),
            None => history::locate(
                state.mode.main,
                session,
                Path::new(&state.worktree),
                &user_home()?,
            )?,
        },
    };
    let history = history::load(
        &history_path,
        state.mode.main,
        session,
        Path::new(&state.worktree),
    )?;
    snapshot::check_idle(&roots, &state)?;
    let label = format!("handoff-{}-{}", target.id, to);
    let planned = exec::planned(ctx, &label)?;
    if options.dry_run {
        let value = serde_json::json!({"provider":to,"source":state.mode.main,"role":"handoff","model":provider::model(to),"effort":"high","run":target.id,"worktree":state.worktree,"session":history.session,"history":history.path,"warnings":history.warnings,"argv":provider::command(to,"handoff",Path::new(&state.worktree),&planned.join("result.txt")),"applies":false});
        ctx.out
            .say(&serde_json::to_string_pretty(&value).map_err(packet::io)?);
        return Ok(());
    }
    let dir = packet::create(&target.dir)?;
    let data = packet::Packet {
        version: 1,
        id: dir.file_name().unwrap().to_string_lossy().into_owned(),
        to,
        snapshot: state,
        history,
    };
    let context = packet::context(&data)?;
    packet::write_new(&dir.join("context.md"), &context)?;
    let rendered = ctx.call(
        "prompt render",
        &[
            "--role".into(),
            "handoff".into(),
            "--context".into(),
            dir.join("context.md").to_string_lossy().into_owned(),
        ],
    );
    if !rendered.stderr.is_empty() {
        ctx.out.err_line(rendered.stderr.trim_end());
    }
    if rendered.code != 0 {
        return Err(Error::Exit(rendered.code));
    }
    let capture = exec::reserve(ctx, &label)?;
    let argv = provider::command(
        to,
        "handoff",
        Path::new(&data.snapshot.worktree),
        &capture.join("result.txt"),
    );
    let code = exec::captured(
        ctx,
        &label,
        &capture,
        &argv,
        Some(Path::new(&data.snapshot.worktree)),
        Some(rendered.stdout.as_bytes()),
    )?;
    if code != 0 {
        ctx.out.err_line(&format!(
            "handoff summary failed; state was not adopted. Capture: {}",
            capture.display()
        ));
        return Err(Error::Exit(code));
    }
    let result = provider::result(to, &capture)?;
    let summary = summary::validate(&result, &data.snapshot, &data.history)?;
    packet::verify_history(&data)?;
    snapshot::verify(&data.snapshot, &roots, &target)?;
    snapshot::check_idle(&roots, &data.snapshot)?;
    packet::seal(&dir, &data, &summary)?;
    ctx.out.say(&format!("handoff ready: {}", data.id));
    ctx.out.say(&format!(
        "open a new {to} main session in {} and read {}",
        data.snapshot.worktree,
        dir.join("RESUME.md").display()
    ));
    ctx.out.say(&format!(
        "resume: dstack handoff resume {} --run {} --host {to} --source-stopped",
        data.id, target.id
    ));
    Ok(())
}

fn resume_command(ctx: &mut Context, args: &[String]) -> Result<()> {
    let id = args
        .first()
        .filter(|s| is_plain_name(s) && !s.starts_with('-'))
        .ok_or_else(usage)?;
    let options = Options::parse(&args[1..], true)?;
    let host = Provider::parse(options.host.ok_or_else(usage)?)?;
    let roots = ctx.roots()?;
    let target = target(ctx, options.run)?;
    let dir = target.dir.join("handoffs").join(id);
    resume::apply(ctx, &roots, &target, &dir, host, options.stopped)
}
fn recover_command(ctx: &mut Context, args: &[String]) -> Result<()> {
    // Recovery shares the strict option parser, with source identity required explicitly.
    let mut rewritten = args.to_vec();
    let stopped = rewritten
        .iter()
        .position(|v| v == "--source-stopped")
        .map(|i| {
            rewritten.remove(i);
            true
        })
        .unwrap_or(false);
    let host_at = rewritten
        .iter()
        .position(|v| v == "--host")
        .ok_or_else(usage)?;
    rewritten[host_at] = "--to".into();
    let options = Options::parse(&rewritten, false)?;
    if options.dry_run {
        return Err(usage());
    }
    let host = Provider::parse(options.to.ok_or_else(usage)?)?;
    let source = options.session.ok_or_else(usage)?;
    let path = Path::new(options.history.ok_or_else(usage)?);
    let id = options.run.ok_or_else(usage)?;
    let roots = ctx.roots()?;
    roots.require_store()?;
    let target = target(ctx, Some(id))?;
    recover::apply(ctx, &roots, &target, host, source, path, stopped)
}
fn target(ctx: &mut Context, id: Option<&str>) -> Result<Target> {
    let roots = ctx.roots()?;
    let selected = mode::selected(&roots, id.map(|id| ("run", id)))?;
    selected
        .target
        .filter(|t| t.kind == TargetKind::Run)
        .ok_or_else(|| Error::failed("handoff requires an existing run (--run <id>)"))
}
fn user_home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| Error::cannot_decide("HOME unavailable; pass --history <path>"))
}

#[derive(Default)]
struct Options<'a> {
    to: Option<&'a str>,
    run: Option<&'a str>,
    session: Option<&'a str>,
    history: Option<&'a str>,
    host: Option<&'a str>,
    dry_run: bool,
    stopped: bool,
}
impl<'a> Options<'a> {
    fn parse(args: &'a [String], resume: bool) -> Result<Self> {
        let mut out = Self::default();
        let mut i = 0;
        while i < args.len() {
            let key = args[i].as_str();
            let flag = match (key, resume) {
                ("--dry-run", false) => Some(&mut out.dry_run),
                ("--source-stopped", true) => Some(&mut out.stopped),
                _ => None,
            };
            if let Some(flag) = flag {
                if *flag {
                    return Err(usage());
                }
                *flag = true;
                i += 1;
                continue;
            }
            let slot = match (key, resume) {
                ("--run", _) => &mut out.run,
                ("--host", true) => &mut out.host,
                ("--to", false) => &mut out.to,
                ("--session", false) => &mut out.session,
                ("--history", false) => &mut out.history,
                _ => return Err(usage()),
            };
            let value = args
                .get(i + 1)
                .filter(|s| !s.is_empty() && !s.starts_with("--"))
                .ok_or_else(usage)?;
            if slot.replace(value).is_some() {
                return Err(usage());
            }
            i += 2;
        }
        Ok(out)
    }
}
fn usage() -> Error {
    Error::failed("usage: dstack handoff --to claude|codex [--run ID] [--session ID] [--history FILE] [--dry-run] | dstack handoff resume ID --host claude|codex --source-stopped [--run ID] | dstack handoff recover-owner --run ID --host claude|codex --session SOURCE --history FILE --source-stopped")
}
