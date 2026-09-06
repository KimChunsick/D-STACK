// Role sessions render the canonical prompt, capture independently, then publish once.
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::mode;
use crate::verbs::exec;

use super::provider;

pub fn run(ctx: &mut Context, args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;
    let roots = ctx.roots()?;
    let selected = mode::selected(&roots, options.target)?;
    let cwd = directory(
        options.worktree.map(Path::new).unwrap_or(&roots.wt_root),
        "worktree",
    )?;
    let context = fs::canonicalize(options.context)
        .map_err(|e| Error::failed(format!("cannot read context: {e}")))?;
    if !context.is_file() {
        return Err(Error::failed("context must be a regular file"));
    }
    let output = output_path(Path::new(options.output))?;
    let planned = exec::planned(ctx, options.label)?;
    let rendered = ctx.call(
        "prompt render",
        &[
            "--role".into(),
            options.role.into(),
            "--context".into(),
            context.to_string_lossy().into_owned(),
        ],
    );
    if rendered.code != 0 {
        return Err(Error::failed(rendered.stderr.trim_end()));
    }
    let sub = selected.mode.sub;
    if options.dry_run {
        ctx.out.say(
            &json!({"provider":sub, "role":options.role, "model":provider::model(sub),
            "argv":provider::command(sub, options.role, &cwd, &planned.join("result.txt")),
            "cwd":cwd, "output":output})
            .to_string(),
        );
        return Ok(());
    }
    let dir = exec::reserve(ctx, options.label)?;
    let command = provider::command(sub, options.role, &cwd, &dir.join("result.txt"));
    if !rendered.stderr.is_empty() {
        ctx.out.err_line(rendered.stderr.trim_end());
    }
    let code = exec::captured(
        ctx,
        options.label,
        &dir,
        &command,
        Some(&cwd),
        Some(rendered.stdout.as_bytes()),
    )?;
    if code != 0 {
        return Err(Error::Exit(code));
    }
    let text = provider::result(sub, &dir)?;
    publish(&output, text.as_bytes())?;
    ctx.out.say(&format!(
        "mode exec: {sub} {} result → {}",
        options.role,
        output.display()
    ));
    Ok(())
}

struct Options<'a> {
    label: &'a str,
    role: &'a str,
    context: &'a str,
    output: &'a str,
    worktree: Option<&'a str>,
    target: Option<(&'a str, &'a str)>,
    dry_run: bool,
}
impl<'a> Options<'a> {
    fn parse(args: &'a [String]) -> Result<Self> {
        let label = args
            .first()
            .filter(|a| !a.starts_with('-'))
            .ok_or_else(usage)?;
        let (mut role, mut context, mut output, mut worktree, mut target, mut dry_run) =
            (None, None, None, None, None, false);
        let mut i = 1;
        while i < args.len() {
            let key = args[i].as_str();
            if key == "--dry-run" {
                if dry_run {
                    return Err(usage());
                }
                dry_run = true;
                i += 1;
                continue;
            }
            let value = args
                .get(i + 1)
                .map(String::as_str)
                .filter(|v| !v.starts_with("--") && !v.is_empty())
                .ok_or_else(usage)?;
            let replaced = match key {
                "--role" => role.replace(value).is_some(),
                "--context" => context.replace(value).is_some(),
                "--output" => output.replace(value).is_some(),
                "--worktree" => worktree.replace(value).is_some(),
                "--run" | "--quick" => target.replace((&key[2..], value)).is_some(),
                _ => return Err(usage()),
            };
            if replaced {
                return Err(usage());
            }
            i += 2;
        }
        let role = role
            .filter(|r| matches!(*r, "review" | "research" | "audit"))
            .ok_or_else(usage)?;
        Ok(Self {
            label,
            role,
            context: context.ok_or_else(usage)?,
            output: output.ok_or_else(usage)?,
            worktree,
            target,
            dry_run,
        })
    }
}

fn directory(path: &Path, name: &str) -> Result<PathBuf> {
    let path = fs::canonicalize(path).map_err(|e| Error::failed(format!("invalid {name}: {e}")))?;
    if !path.is_dir() {
        return Err(Error::failed(format!("{name} must be a directory")));
    }
    Ok(path)
}

fn output_path(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| Error::failed("output needs a file name"))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = directory(parent, "output parent")?;
    if fs::metadata(&parent)
        .map_err(|e| Error::failed(e.to_string()))?
        .permissions()
        .readonly()
    {
        return Err(Error::failed("output parent is not writable"));
    }
    let output = parent.join(name);
    match fs::symlink_metadata(&output) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(output),
        Ok(_) => Err(Error::failed(format!(
            "output already exists: {}",
            output.display()
        ))),
        Err(e) => Err(Error::failed(format!("cannot inspect output: {e}"))),
    }
}

/// The temporary file is on the output filesystem; hard_link publishes atomically and refuses
/// existing files and symlinks, including a destination created while the provider was running.
fn publish(output: &Path, text: &[u8]) -> Result<()> {
    for n in 0.. {
        let temporary = output.with_file_name(format!(".dstack-result-{}-{n}", std::process::id()));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(Error::cannot_decide(format!("cannot prepare output: {e}"))),
        };
        let result = file
            .write_all(text)
            .and_then(|_| file.sync_all())
            .and_then(|_| fs::hard_link(&temporary, output));
        let _ = fs::remove_file(&temporary);
        return result.map_err(|e| {
            Error::cannot_decide(format!("cannot publish {}: {e}", output.display()))
        });
    }
    unreachable!()
}

fn usage() -> Error {
    Error::failed("usage: dstack mode exec <label> --role review|research|audit --context <file> --output <file> [--worktree <dir>] [--quick <slug>|--run <id>] [--dry-run]")
}
