// Strict option validation precedes all settings writes; displaying modes is read-only.
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::mode::{selected, Mode, Provider, Selection};
use crate::core::target::TargetKind;

pub fn set(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (mut main, mut sub) = (None, None);
    let mut i = 0;
    while i < args.len() {
        let (key, value) = pair(args, &mut i)?;
        let slot = match key {
            "--main" => &mut main,
            "--sub" => &mut sub,
            _ => return Err(set_usage()),
        };
        if slot.replace(Provider::parse(value)?).is_some() {
            return Err(set_usage());
        }
    }
    if main.is_none() && sub.is_none() {
        return Err(set_usage());
    }
    let roots = ctx.roots()?;
    roots.require_store()?;
    let previous = Mode::project(&roots)?;
    let active = selected(&roots, None)?;
    let mode = Mode {
        main: main.unwrap_or(previous.main),
        sub: sub.unwrap_or(previous.sub),
    };
    let apply = application(mode, &active);
    mode.write_project(&roots)?;
    ctx.out
        .say(&format!("project: main={} sub={}", mode.main, mode.sub));
    if active.target.is_some() {
        ctx.out.say(&format!(
            "active: main={} sub={} (source={})",
            active.mode.main, active.mode.sub, active.source
        ));
    }
    ctx.out.say(&format!("apply: {apply}"));
    Ok(())
}

pub fn show(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (mut json, mut host, mut target) = (false, None, None);
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--json" {
            if json {
                return Err(show_usage());
            }
            json = true;
            i += 1;
            continue;
        }
        let (key, value) = pair(args, &mut i)?;
        match key {
            "--host" if host.is_none() => host = Some(Provider::parse(value)?),
            "--run" | "--quick" if target.is_none() => {
                target = Some((key.trim_start_matches('-'), value));
            }
            _ => return Err(show_usage()),
        }
    }
    let roots = ctx.roots()?;
    let project = Mode::project(&roots)?;
    let selection = selected(&roots, target)?;
    if let Some(host) = host {
        if host != selection.mode.main {
            return Err(Error::failed(format!(
                "main host mismatch: selected {}, current {host}; {}",
                selection.mode.main,
                host_handoff(project, &selection, host)
            )));
        }
    }
    let apply = application(project, &selection);
    if json {
        let target = selection.target.as_ref().map(|target| {
            serde_json::json!({"kind": match target.kind {
                TargetKind::Run => "run", TargetKind::Quick => "quick"
            }, "id": target.id})
        });
        ctx.out.say(
            &serde_json::json!({
                "main": selection.mode.main,
                "sub": selection.mode.sub,
                "source": selection.source,
                "project": project,
                "target": target,
                "apply": apply,
            })
            .to_string(),
        );
    } else {
        ctx.out.say(&format!(
            "mode: main={} sub={} (source={})",
            selection.mode.main, selection.mode.sub, selection.source
        ));
        ctx.out.say(&format!(
            "project: main={} sub={}",
            project.main, project.sub
        ));
        if let Some(target) = &selection.target {
            ctx.out.say(&format!("target: {}", target.id));
        }
        ctx.out.say(&format!("apply: {apply}"));
    }
    Ok(())
}

fn application(project: Mode, selection: &Selection) -> String {
    let refresh = match &selection.target {
        Some(target) if target.kind == TargetKind::Run => {
            format!("dstack run adopt {} --refresh-mode", target.id)
        }
        _ => "dstack run adopt <id> --refresh-mode".to_string(),
    };
    let continuation = if selection.target.is_some() {
        format!(" Continue the selected snapshot: {}", handoff(selection))
    } else {
        String::new()
    };
    format!(
        "project settings apply to new runs and quick tasks; start {} in a new session for main={}. \
         Existing runs retain their snapshot until `{refresh}`.{continuation}",
        project.main, project.main
    )
}

fn host_handoff(project: Mode, selection: &Selection, host: Provider) -> String {
    if project.main == host {
        if let Some(target) = &selection.target {
            if target.kind == TargetKind::Run {
                return format!(
                    "project main={host} is ready in this session; run \
                     `dstack run adopt {} --refresh-mode` to apply it. \
                     An ordinary `dstack run adopt {}` preserves main={}.",
                    target.id, target.id, selection.mode.main
                );
            }
        }
    }
    handoff(selection)
}

fn handoff(selection: &Selection) -> String {
    let resume = match &selection.target {
        Some(target) => match target.kind {
            TargetKind::Run => format!("dstack run adopt {}", target.id),
            TargetKind::Quick => format!("dstack quick resume {}", target.id),
        },
        None => "dstack run new <slug> or dstack run adopt <id>".to_string(),
    };
    format!(
        "launch {} in a new session and continue with `{resume}`.",
        selection.mode.main
    )
}

fn pair<'a>(args: &'a [String], i: &mut usize) -> Result<(&'a str, &'a str)> {
    let arg = &args[*i];
    if let Some((key, value)) = arg.split_once('=') {
        *i += 1;
        if value.is_empty() {
            return Err(Error::failed(format!("missing value for {key}")));
        }
        return Ok((key, value));
    }
    let value = args
        .get(*i + 1)
        .filter(|value| !value.is_empty() && !value.starts_with('-'))
        .ok_or_else(|| Error::failed(format!("missing value for {arg}")))?;
    *i += 2;
    Ok((arg, value))
}

fn set_usage() -> Error {
    Error::failed("usage: dstack mode set [--main claude|codex] [--sub claude|codex] (each option once; at least one required)")
}

fn show_usage() -> Error {
    Error::failed("usage: dstack mode show [--json] [--host claude|codex] [--run ID | --quick SLUG] (each option once)")
}
