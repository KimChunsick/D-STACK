// verbs/exec.rs
// dstack exec <label> -- <cmd…>: run a long external command and keep stdout/stderr/exit under
// .dstack/local/exec/<label>/ so a background Bash call can end the turn (R98). Exit passes through.

use std::fs;
use std::io::ErrorKind;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{file_size, utc_now};
use crate::core::verb::Verb;
use crate::selftest::Selftest;

struct Exec;

impl Verb for Exec {
    fn name(&self) -> &'static str {
        "exec"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        exec(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(Exec)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![]
}

fn exec(ctx: &mut Context, args: &[String]) -> Result<()> {
    let label = args.first().cloned().unwrap_or_default();
    if label.is_empty() || args.get(1).map(String::as_str) != Some("--") {
        return Err(Error::failed("usage: dstack exec <label> -- <cmd> [args]"));
    }
    let command = &args[2..];
    if command.is_empty() {
        return Err(Error::failed("no command given after --"));
    }
    let dir = reserve(ctx, &label)?;
    let code = captured(ctx, &label, &dir, command, None, None)?;
    if code == 0 {
        Ok(())
    } else {
        Err(Error::Exit(code))
    }
}

/// Read-only capture planning is also used by `mode exec --dry-run`.
pub fn planned(ctx: &mut Context, label: &str) -> Result<PathBuf> {
    if label.is_empty()
        || label.contains('/')
        || label.chars().any(char::is_whitespace)
        || label == "."
        || label == ".."
    {
        return Err(Error::failed(format!(
            "label must be a plain name (got '{label}')"
        )));
    }
    let base = ctx.roots()?.local.join("exec");
    for n in 0.. {
        let dir = base.join(if n == 0 {
            label.into()
        } else {
            format!("{label}.{n}")
        });
        match fs::symlink_metadata(&dir) {
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(dir),
            Ok(m) if m.is_dir() && dir.join("exit").is_file() => continue,
            Ok(_) => {
                return Err(Error::failed(format!(
                    "exec capture already exists without a completed exit: {}",
                    dir.display()
                )))
            }
            Err(e) => {
                return Err(Error::cannot_decide(format!(
                    "cannot inspect {}: {e}",
                    dir.display()
                )))
            }
        }
    }
    unreachable!()
}

/// Exclusive creation prevents two in-flight calls from overwriting one another's evidence.
pub fn reserve(ctx: &mut Context, label: &str) -> Result<PathBuf> {
    loop {
        let dir = planned(ctx, label)?;
        mkdir(dir.parent().expect("capture parent"))?;
        match fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(Error::cannot_decide(format!(
                    "cannot create {}: {e}",
                    dir.display()
                )))
            }
        }
    }
}

/// The only generic process launcher. None preserves `dstack exec`'s inherited cwd/stdin;
/// role calls supply a finite input file so EOF is deterministic even for a long prompt.
pub fn captured(
    ctx: &mut Context,
    label: &str,
    dir: &Path,
    command: &[String],
    cwd: Option<&Path>,
    input: Option<&[u8]>,
) -> Result<i32> {
    if command.is_empty() {
        return Err(Error::failed("no command given"));
    }
    write(&dir.join("cmd"), &format!("{}\n", command.join(" ")))?;
    let started_at = utc_now();
    write(&dir.join("started_at"), &format!("{started_at}\n"))?;
    ctx.out.say(&format!("exec {label} → {}", dir.display()));
    ctx.out.flush();
    let out = create(&dir.join("out.txt"))?;
    let err = create(&dir.join("err.txt"))?;
    let mut child = Command::new(&command[0]);
    child.args(&command[1..]).stdout(out).stderr(err);
    if let Some(cwd) = cwd {
        child.current_dir(cwd);
    }
    if let Some(bytes) = input {
        let path = dir.join("stdin.txt");
        fs::write(&path, bytes)
            .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))?;
        let file = fs::File::open(&path)
            .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", path.display())))?;
        child.stdin(file);
    }
    let code = match child.status() {
        Ok(status) => status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)),
        Err(error) => {
            let code = if error.kind() == ErrorKind::NotFound {
                127
            } else {
                126
            };
            write(&dir.join("err.txt"), &format!("{}: {error}\n", command[0]))?;
            code
        }
    };
    write(&dir.join("exit"), &format!("{code}\n"))?;
    let finished_at = utc_now();
    write(&dir.join("finished_at"), &format!("{finished_at}\n"))?;
    ctx.out.say(&format!(
        "exec {label}: exit {code}, stdout {}B, stderr {}B ({started_at} → {finished_at})",
        size(&dir.join("out.txt")),
        size(&dir.join("err.txt"))
    ));
    super::prompt::usage::capture(ctx, command, dir);
    Ok(code)
}

fn size(path: &Path) -> u64 {
    file_size(path).unwrap_or(0)
}

fn create(path: &Path) -> Result<fs::File> {
    fs::File::create(path)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

fn mkdir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)
        .map_err(|e| Error::cannot_decide(format!("cannot create {}: {e}", dir.display())))
}

fn write(path: &Path, text: &str) -> Result<()> {
    fs::write(path, text)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}
