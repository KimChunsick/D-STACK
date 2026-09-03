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
    let roots = ctx.roots()?;
    let label = args.first().cloned().unwrap_or_default();
    if label.is_empty() || args.get(1).map(String::as_str) != Some("--") {
        return Err(Error::failed("usage: dstack exec <label> -- <cmd> [args]"));
    }
    let command = &args[2..];
    if command.is_empty() {
        return Err(Error::failed("no command given after --"));
    }
    // The label is a directory name, so it is checked after the command — as the shell does.
    if label.contains('/') || label.contains(' ') || label == "." || label == ".." {
        return Err(Error::failed(format!(
            "label must be a plain name (got '{label}')"
        )));
    }
    let base = roots.local.join("exec");
    mkdir(&base.join(&label))?;
    let mut dir = base.join(&label);
    let mut n = 1;
    while dir.join("exit").exists() {
        dir = base.join(format!("{label}.{n}"));
        n += 1;
    }
    mkdir(&dir)?;
    write(&dir.join("cmd"), &format!("{}\n", command.join(" ")))?;
    let started_at = utc_now();
    write(&dir.join("started_at"), &format!("{started_at}\n"))?;
    ctx.out.say(&format!("exec {label} → {}", dir.display()));

    // Our own stdout is block-buffered when it is a file, so it is flushed before the child runs.
    ctx.out.flush();
    let out = create(&dir.join("out.txt"))?;
    let err = create(&dir.join("err.txt"))?;
    let spawned = Command::new(&command[0])
        .args(&command[1..])
        .stdout(out)
        .stderr(err)
        .status();
    let code = match spawned {
        Ok(status) => status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)),
        // The shell's redirection makes bash itself report this: 127 for a command it cannot
        // find, 126 for one it finds and cannot run, and a line in err.txt that names the script
        // and the line number (`dstack: line 42: nosuch: command not found`). Neither the script
        // nor the line number exists here, so the operating system's own wording stands in.
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
    if code == 0 {
        return Ok(());
    }
    // `return $rc` through the dispatcher's `exit $?`: the child's code is the CLI's code, and
    // that code is often neither 1 nor 2.
    Err(Error::Exit(code))
}

fn size(path: &Path) -> u64 {
    file_size(path).unwrap_or(0)
}

fn create(path: &PathBuf) -> Result<fs::File> {
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
