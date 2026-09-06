// verbs/run.rs
// dstack run: mint, adopt, close and pause the execution unit (R30–R38).

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, thread, time::Duration};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{read_text, utc_now};
use crate::core::meta::{meta_get, meta_set, owner_is_stale, touch_owner};
use crate::core::{mode::Mode, mode_run};
use crate::core::paths::{base_name, is_plain_name, valid_slug};
use crate::core::roots::{git_out, Roots};
use crate::core::tools::{team_style_lookup, tool_check_for_mode, NO_TEAM_STYLE_LINE};
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::request::{field_default, req_enum};
use crate::store::plan;

macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

macro_rules! fail { ($($m:tt)*) => { return Err(Error::failed(format!($($m)*))) }; }

macro_rules! run_verb {
    ($handler:ident, $entry:literal, $body:ident) => {
        struct $handler;
        impl Verb for $handler {
            fn name(&self) -> &'static str { $entry }
            fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> { $body(ctx, args) }
        }
    };
}

run_verb!(RunNew, "run new", new);
run_verb!(RunAdopt, "run adopt", adopt);
run_verb!(RunClose, "run close", close);
run_verb!(RunPause, "run pause", pause);

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(RunNew), Box::new(RunAdopt), Box::new(RunClose), Box::new(RunPause)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> { vec![] }

fn new(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (mut slug, mut worktree) = (String::new(), String::new());
    let mut work_type = "cli".to_string();
    let mut i = 0;
    while i < args.len() {
        // The option loop of the shell takes only the two-word form, so `--type=cli` falls
        // through to the unknown-option arm; core::args::opt would have accepted it.
        let arg = args[i].clone();
        match arg.as_str() {
            "--worktree" => worktree = eat(args, &mut i)?,
            "--type" => work_type = eat(args, &mut i)?,
            _ if arg.starts_with('-') => fail!("unknown option: {arg}"),
            _ if slug.is_empty() => { slug = arg; i += 1 }
            _ => fail!("unexpected argument: {arg}"),
        }
    }
    if slug.is_empty() { fail!("usage: dstack run new <slug> [--type <work_type>] [--worktree <path>]") }
    if !valid_slug(&slug) { fail!("slug must match [a-z0-9][a-z0-9-]* (got '{slug}')") }
    let types = req_enum("work_type");
    if !types.contains(&work_type.as_str()) { fail!("--type must be one of: {}", types.join(" ")) }
    refuse_second_run(ctx, &roots, &slug, &worktree)?;

    let mode = Mode::project(&roots)?;
    let fields: Vec<String> = ["e2e", "review", "visual", "unit_tests"].iter()
        .map(|f| format!("{f}={}", field_default(&work_type, f))).collect();
    if tool_check_for_mode(ctx, &fields, &mode, true)? != 0 {
        say!(ctx, "refused: a goal-closing tool is missing for work_type={work_type} (see lines above)");
        return Err(Error::Exit(1));
    }

    let git = |args: &[&str], or: &str| git_out(Some(&roots.wt_root), args).unwrap_or(or.to_string());
    let base_branch = git(&["rev-parse", "--abbrev-ref", "HEAD"], "detached");
    let base_head = git(&["rev-parse", "HEAD"], "none");
    let mut branch = base_branch.clone();
    let mut target_wt = roots.wt_root.clone();
    let overlap = overlap_files(&roots)?;
    if !worktree.is_empty() {
        branch = format!("goal/{slug}");
        target_wt = add_worktree(&roots, &worktree, &branch)?;
    }
    let (id, dir) = mint(&roots, &slug)?;
    mode.snapshot(&dir)?;
    let target = target_wt.to_string_lossy().into_owned();
    let started_at = utc_now();
    for (key, value) in [
        ("id", id.as_str()), ("slug", &slug), ("status", "open"),
        ("started_at", &started_at), ("closed_at", ""), ("worktree", &target),
        ("branch", &branch), ("base_branch", &base_branch), ("base_head", &base_head),
        ("work_type", &work_type), ("transcript_path", ""),
    ] { meta_set(&dir, key, value)?; }
    touch_owner(&dir, ctx.parent_pid, &ctx.session_id)?;
    let style = team_style_lookup(ctx).map(|p| p.to_string_lossy().into_owned());
    meta_set(&dir, "team_style", style.as_deref().unwrap_or(""))?;
    match &style {
        Some(style) => say!(ctx, "team style: {style}"),
        None => ctx.out.say(NO_TEAM_STYLE_LINE),
    }
    let current = target_wt.join(".dstack/local/CURRENT");
    mkdir(&target_wt.join(".dstack/local"))?;
    write(&current, &format!("{id}\n"))?;
    let short: String = base_head.chars().take(8).collect();
    say!(ctx, "run: {id}");
    say!(ctx, "  dir: {}", dir.display());
    say!(ctx, "  CURRENT: {}", current.display());
    say!(ctx, "  branch: {branch} (base {base_branch} @ {short})");
    say!(ctx, "  other open runs declare {overlap} file(s); overlaps are warned by dstack next and checked at close");
    Ok(())
}

/// One Goal run per worktree (R33, R37). A second run here is refused with the way out.
fn refuse_second_run(ctx: &mut Context, roots: &Roots, slug: &str, worktree: &str) -> Result<()> {
    let current = match roots.current_run_id()? {
        Some(current) if worktree.is_empty() => current,
        _ => return Ok(()),
    };
    if meta_get(&roots.runs.join(&current), "status")?.as_deref() != Some("open") { return Ok(()) }
    let main = base_name(&roots.main_root);
    say!(ctx, "refused: this worktree already runs '{current}' (status open)");
    ctx.out.say("  start the second Goal in its own worktree:");
    say!(ctx, "    dstack run new {slug} --worktree ../{main}-{slug}");
    ctx.out.say("  or close/pause the current one: dstack run close | dstack run pause");
    Err(Error::Exit(1))
}

/// `git worktree add -b goal/<slug>`: the checkout the second Goal runs in, and its local store.
fn add_worktree(roots: &Roots, worktree: &str, branch: &str) -> Result<PathBuf> {
    let mut path = worktree.to_string();
    if !path.starts_with('/') {
        let cwd = std::env::current_dir().map_err(|e| cannot("read the cwd", &e))?;
        path = format!("{}/{path}", cwd.display());
    }
    if Path::new(&path).exists() { fail!("worktree path exists: {path}") }
    // git prints its progress on stderr, which the shell lets through; only stdout is muted.
    let added = Command::new("git")
        .args(["-C", &roots.wt_root.to_string_lossy()])
        .args(["worktree", "add", "-b", branch, &path, "HEAD"])
        .stdout(Stdio::null()).status().map(|s| s.success()).unwrap_or(false);
    if !added { fail!("git worktree add failed") }
    let target = fs::canonicalize(&path).map_err(|e| cannot(&format!("read {path}"), &e))?;
    let local = target.join(".dstack/local");
    mkdir(&local)?;
    mkdir(&target.join(".dstack/quick"))?;
    // `chmod 700` under set -e ends the command; the message names the path as chmod does,
    // without chmod's own prefix and mode.
    fs::set_permissions(&local, fs::Permissions::from_mode(0o700))
        .map_err(|e| Error::failed(format!("chmod: {}: {e}", local.display())))?;
    write(&target.join(".dstack/.gitignore"), "*\n")?;
    Ok(target)
}

/// The run id is the UTC second plus the slug, so a collision waits for the next second.
fn mint(roots: &Roots, slug: &str) -> Result<(String, PathBuf)> {
    for attempt in 0..=21 {
        let id = format!("{}_{slug}", utc_now().replace(['-', ':'], ""));
        let dir = roots.runs.join(&id);
        if fs::create_dir(&dir).is_ok() {
            mkdir(&dir.join("review"))?;
            return Ok((id, dir));
        }
        if attempt < 21 { thread::sleep(Duration::from_secs(1)) }
    }
    Err(Error::cannot_decide(format!("cannot create run dir under {}", roots.runs.display())))
}

/// Other open runs whose plans declare files: warn only (R38), counted before this run is minted:
/// a plan.json that cannot be read is a cannot decide (D-12) and must not leave a run behind it.
fn overlap_files(roots: &Roots) -> Result<usize> {
    let mut files = 0;
    for other in run_dirs(roots) {
        if field(&other, "status")? != "open" || !plan::exists(&other) { continue }
        files += plan::load(&other)?.plans.iter().map(|p| p.files.len()).sum::<usize>();
    }
    Ok(files)
}

fn adopt(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (mut id, mut force, mut refresh) = (String::new(), false, false);
    for arg in args {
        match arg.as_str() {
            "--force" => force = true,
            "--refresh-mode" => refresh = true,
            other if other.starts_with('-') => fail!("unknown option: {other}"),
            other => id = other.to_string(),
        }
    }
    if id.is_empty() { id = roots.current_run_id()?.unwrap_or_default() }
    if id.is_empty() { id = only_candidate(&roots)? }
    if !is_plain_name(&id) { fail!("run id must be a plain name (got '{id}')") }
    let dir = roots.runs.join(&id);
    if !dir.is_dir() { fail!("run not found: {id}") }
    let status = field(&dir, "status")?;
    if status == "closed" || status == "abandoned" { fail!("run {id} is {status}; cannot adopt") }
    let before = owner_stamp(&dir)?;
    let mine = !ctx.session_id.is_empty() && field(&dir, "owner_session")? == ctx.session_id;
    if !owner_is_stale(&dir)? && !mine && !force {
        fail!("run {id} has a live owner ({before}, refreshed <10 min ago); pass --force to take it")
    }
    mode_run::adopt(ctx, &dir, refresh)?;
    touch_owner(&dir, ctx.parent_pid, &ctx.session_id)?;
    meta_set(&dir, "status", "open")?;
    mkdir(&roots.local)?;
    write(&roots.local.join("CURRENT"), &format!("{id}\n"))?;
    say!(ctx, "adopted {id}");
    say!(ctx, "  owner before: {before}");
    say!(ctx, "  owner after:  {}", owner_stamp(&dir)?);
    Ok(())
}

/// After /clear or a pause CURRENT is empty; the one open or paused run of THIS worktree is the
/// unambiguous candidate. Two candidates need an explicit id.
fn only_candidate(roots: &Roots) -> Result<String> {
    let mut listed = String::new();
    for dir in run_dirs(roots) {
        let status = field(&dir, "status")?;
        let mine = field(&dir, "worktree")? == roots.wt_root.to_string_lossy();
        if dir.join("meta.tsv").is_file() && (status == "open" || status == "paused") && mine {
            listed.push_str(&format!(" {}", base_name(&dir)));
        }
    }
    let names: Vec<&str> = listed.split_whitespace().collect();
    if names.len() == 1 { return Ok(names[0].to_string()) }
    let candidates = if listed.is_empty() { " none" } else { &listed };
    fail!("no run id given and CURRENT is empty; candidates for this worktree:{candidates} (dstack run list)")
}

fn close(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (mut abandon, mut id) = (String::new(), String::new());
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].clone();
        match arg.as_str() {
            "--abandon" => abandon = eat(args, &mut i)?,
            _ if arg.starts_with('-') => fail!("unknown option: {arg}"),
            _ => { id = arg; i += 1 }
        }
    }
    if id.is_empty() { id = roots.current_run_id()?.unwrap_or_default() }
    if id.is_empty() { fail!("no current run") }
    if !is_plain_name(&id) { fail!("run id must be a plain name (got '{id}')") }
    let dir = roots.runs.join(&id);
    if !dir.is_dir() { fail!("run not found: {id}") }
    if abandon.is_empty() {
        verify_before_close(ctx, &id)?;
        meta_set(&dir, "status", "closed")?;
    } else {
        meta_set(&dir, "status", "abandoned")?;
        meta_set(&dir, "closed_reason", &abandon)?;
    }
    meta_set(&dir, "closed_at", &utc_now())?;
    let in_worktree = PathBuf::from(field(&dir, "worktree")?).join(".dstack/local/CURRENT");
    if read_id(&in_worktree)? == id { write(&in_worktree, "")? }
    if roots.current_run_id()?.unwrap_or_default() == id { write(&roots.local.join("CURRENT"), "")? }
    let (status, at) = (field(&dir, "status")?, field(&dir, "closed_at")?);
    say!(ctx, "closed {id} ({status}) at {at}; CURRENT cleared");
    Ok(())
}

/// The shell sources verify.sh and calls cmd_verify in process, so a fail()/die() inside verify
/// ends the whole dstack run with that code and that one stderr line, while a plain `return 1|2`
/// reaches the `if !` branch and becomes the refusal below. The captured stderr tells the two
/// apart: only fail()/die() leave a trailing `dstack: …` line.
fn verify_before_close(ctx: &mut Context, id: &str) -> Result<()> {
    let called = ctx.call("verify", &["--run".to_string(), id.to_string()]);
    let mut lines: Vec<String> = called.stderr.lines().map(String::from).collect();
    let died = match lines.last() {
        Some(last) if last.starts_with("dstack: ") => lines.pop(),
        _ => None,
    };
    ctx.out.raw(&called.stdout);
    for line in lines { ctx.out.err_line(&line) }
    if let Some(last) = died {
        let message = last["dstack: ".len()..].to_string();
        return Err(match called.code {
            2 => Error::cannot_decide(message),
            _ => Error::failed(message),
        });
    }
    if called.code != 0 {
        say!(ctx, "refused: dstack verify failed for {id} — fix the lines above, or close with --abandon <why>");
        return Err(Error::Exit(1));
    }
    Ok(())
}

fn pause(ctx: &mut Context, _args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let current = roots.local.join("CURRENT");
    let id = roots.current_run_id()?.unwrap_or_default();
    if id.is_empty() { fail!("no current run in {}", current.display()) }
    if !is_plain_name(&id) { fail!("run id must be a plain name (got '{id}')") }
    meta_set(&roots.runs.join(&id), "status", "paused")?;
    write(&current, "")?;
    say!(ctx, "paused {id}");
    say!(ctx, "  cleared: {}", current.display());
    say!(ctx, "  resume:  dstack run adopt {id}");
    Ok(())
}

/// `--name value`: the value, with the index moved past both words. Without a value the shell's
/// `shift 2` fails under `set -e`, which ends the command with 1 and prints nothing at all.
fn eat(args: &[String], i: &mut usize) -> Result<String> {
    let value = args.get(*i + 1).ok_or(Error::Exit(1))?.clone();
    *i += 2;
    Ok(value)
}

/// The `"$RUNS"/*/` glob: every directory of the store, in name order.
pub fn run_dirs(roots: &Roots) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(&roots.runs).into_iter().flatten().flatten()
        .map(|entry| entry.path()).filter(|path| path.is_dir()).collect();
    dirs.sort();
    dirs
}

fn owner_stamp(dir: &Path) -> Result<String> {
    Ok(format!("{}:{}@{}", field(dir, "owner_session")?, field(dir, "owner_pid")?, field(dir, "owner_ts")?))
}

pub fn field(dir: &Path, key: &str) -> Result<String> { Ok(meta_get(dir, key)?.unwrap_or_default()) }

/// `cat "$file"`: the id a CURRENT file names, empty when it is not there. One that is there and
/// cannot be read is a cannot-decide (D-12), not a worktree left pointing at a run it just closed.
fn read_id(path: &Path) -> Result<String> {
    Ok(read_text(path)?.unwrap_or_default().trim_end_matches('\n').to_string())
}

fn mkdir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).map_err(|e| cannot(&format!("create {}", dir.display()), &e))
}

fn write(path: &Path, text: &str) -> Result<()> {
    fs::write(path, text).map_err(|e| cannot(&format!("write {}", path.display()), &e))
}

pub fn cannot(what: &str, error: &std::io::Error) -> Error {
    Error::cannot_decide(format!("cannot {what}: {error}"))
}
