// verbs/init.rs
// dstack init: bootstrap the .dstack store of a repository (R34, D-01). Never expands cases.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::verb::Verb;
use crate::selftest::Selftest;

const QUICK_STATE: &str = "# Quick tasks state\n\n## Quick tasks\n\n\
                           | slug | status | opened | closed |\n|---|---|---|---|\n";

struct Init;

impl Verb for Init {
    fn name(&self) -> &'static str {
        "init"
    }

    /// The shell handler reads no argument at all, so `init --bogus` is an ordinary init.
    fn run(&self, ctx: &mut Context, _args: &[String]) -> Result<()> {
        init(ctx)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(Init)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![]
}

fn init(ctx: &mut Context) -> Result<()> {
    let roots = ctx.roots()?;
    let mut created = 0;
    let mut existing = 0;
    for dir in [
        roots.store.clone(),
        roots.store.join("project"),
        roots.runs.clone(),
        roots.local.clone(),
        roots.local.join("hooks"),
        roots.local.join("exec"),
        roots.quick.clone(),
    ] {
        if dir.is_dir() {
            existing += 1;
        } else {
            fs::create_dir_all(&dir).map_err(|e| cannot_write(&dir, e))?;
            created += 1;
        }
    }
    let wt_store = roots.wt_root.join(".dstack");
    chmod_700(&roots.local);
    chmod_700(&wt_store);

    // .dstack is local to the machine (owner decision D-01): ignored at the root and
    // self-ignoring so a product repo never carries a request or a ledger into a PR by accident.
    let store_ignore = roots.store.join(".gitignore");
    match fs::read_to_string(&store_ignore) {
        Ok(text) if text.trim_end_matches('\n') == "*" => existing += 1,
        _ => {
            write(&store_ignore, "*\n")?;
            created += 1;
        }
    }
    let gitignore = roots.main_root.join(".gitignore");
    let already = fs::read_to_string(&gitignore)
        .map(|text| {
            text.lines()
                .any(|line| line == ".dstack" || line == ".dstack/")
        })
        .unwrap_or(false);
    let ignore_line = if already {
        "already present".to_string()
    } else {
        append(&gitignore, ".dstack/\n")
            .map_err(|_| Error::failed(format!("cannot write {}", gitignore.display())))?;
        format!("added .dstack/ to {}", base_name(&gitignore))
    };
    if roots.wt_root != roots.main_root {
        fs::create_dir_all(&wt_store).map_err(|e| cannot_write(&wt_store, e))?;
        write(&wt_store.join(".gitignore"), "*\n")?;
    }

    let project = roots.store.join("project/PROJECT.md");
    if project.is_file() {
        existing += 1;
    } else {
        write(&project, &project_template(&base_name(&roots.main_root)))?;
        created += 1;
    }
    let version = roots.store.join("version");
    if !version.is_file() {
        write(&version, "2\n")?;
        created += 1;
    }
    let quick_state = roots.quick.join("STATE.md");
    if !quick_state.is_file() {
        write(&quick_state, QUICK_STATE)?;
        created += 1;
    }

    ctx.out
        .say(&format!("dstack init: store {}", roots.store.display()));
    ctx.out
        .say(&format!("  local (per worktree): {}", roots.local.display()));
    ctx.out.say(&format!(
        "  created {created}, existing {existing}, root .gitignore: {ignore_line}"
    ));
    ctx.out.say(&format!(
        "  policy: {} (edit the ## Verification policy block)",
        project.display()
    ));
    Ok(())
}

fn project_template(name: &str) -> String {
    format!(
        "# {name}\n\n\
         One paragraph describing this repository for a worker that starts with an empty \
         context.\n\n\
         ## Verification policy\n\
         surfaces: cli\n\
         e2e_evidence: cli\n\
         visual_diff: forbidden\n\
         team_style:\n\
         max_concurrent: 3\n\
         why: default policy written by dstack init — edit to match what this repository can \
         actually verify\n"
    )
}

/// `chmod 700 … 2>/dev/null || true`: a store on a filesystem without modes is still a store.
fn chmod_700(dir: &Path) {
    let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
}

fn base_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn write(path: &Path, text: &str) -> Result<()> {
    fs::write(path, text).map_err(|e| cannot_write(path, e))
}

fn append(path: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(text.as_bytes())
}

fn cannot_write(path: &Path, error: std::io::Error) -> Error {
    Error::cannot_decide(format!("cannot write {}: {error}", path.display()))
}
