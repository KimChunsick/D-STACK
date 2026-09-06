// verbs/doctor/mod.rs
// dstack doctor: the eight sections of the repository sweep, and --self for the fixture runner.

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::verb::Verb;
use crate::selftest::Selftest;

/// say(): one stdout line. Declared before the modules that use it, because a macro_rules! is in
/// scope only for what follows it.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

pub mod agents;
pub mod codex;
pub mod deps;
pub mod hooks;
pub mod korules;
pub mod layout;
pub mod locks;
pub mod modes;
pub mod selfrun;
pub mod sweep;

struct Doctor;

impl Verb for Doctor {
    fn name(&self) -> &'static str {
        "doctor"
    }
    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        doctor(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(Doctor)]
}

/// The doctor's own checkers need fixtures too (R100 applies to R09/R13/R20/R23/R81).
pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![
        Box::new(agents::Checker),
        Box::new(codex::Checker),
        Box::new(deps::Checker),
        Box::new(sweep::Checker),
        Box::new(layout::Checker),
        Box::new(modes::Checker),
    ]
}

/// Each section prints a table and a count line and answers whether it holds; a section fails
/// loudly, never silently. The sweep reads the repository only, so nothing here writes.
const SECTIONS: [fn(&mut Context) -> Result<bool>; 9] = [
    deps::section,
    agents::section,
    codex::section,
    sweep::section,
    locks::section,
    hooks::section,
    korules::section,
    layout::section,
    modes::section,
];

fn doctor(ctx: &mut Context, args: &[String]) -> Result<()> {
    if args.first().map(String::as_str) == Some("--self") {
        return selfrun::run(ctx);
    }
    if !args.is_empty() {
        return Err(Error::failed("usage: dstack doctor [--self]"));
    }
    let mut failing = 0;
    for section in SECTIONS {
        if !section(ctx)? {
            failing += 1;
        }
        say!(ctx, "");
    }
    say!(
        ctx,
        "doctor: {} sections, {failing} failing",
        SECTIONS.len()
    );
    // The shell's last command is the `[ "$failing" -eq 0 ]` test: exit 1, nothing printed.
    match failing {
        0 => Ok(()),
        _ => Err(Error::Exit(1)),
    }
}

/// The shell ran a checker's section function with stdout on /dev/null and read its verdict from
/// the return code; the port buffers the same output and drops it.
pub(super) fn quiet<T>(ctx: &mut Context, run: impl FnOnce(&mut Context) -> T) -> T {
    ctx.out.begin_capture();
    let answer = run(ctx);
    ctx.out.end_capture();
    answer
}

/// `<dir>/*<suffix>` in the order the shell's glob expands it: names starting with a dot are not
/// matched, and a directory that is not there expands to nothing.
pub(super) fn glob(dir: &Path, suffix: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return files,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = crate::core::paths::base_name(&path);
        if !name.starts_with('.') && name.ends_with(suffix) && path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// `<dir>/*/`: the immediate subdirectories, in name order.
pub(super) fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return dirs,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !crate::core::paths::base_name(&path).starts_with('.') && path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    dirs
}

/// `<dir>/*/<name>`: one file per immediate subdirectory, in name order.
pub(super) fn glob_sub(dir: &Path, name: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = subdirs(dir).iter().map(|sub| sub.join(name)).collect();
    files.retain(|file| file.is_file());
    files
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::registry::Registry;
    use crate::core::roots::Home;
    use std::rc::Rc;

    pub(super) fn context() -> Context {
        let home = Home::resolve().expect("the repository of this test binary");
        Context::new(
            home,
            PathBuf::new(),
            Rc::new(Registry::new(crate::verbs::all_verbs())),
        )
    }

    /// What a section printed, run on this repository.
    pub(super) fn printed(section: fn(&mut Context) -> Result<bool>) -> (bool, String) {
        let mut ctx = context();
        ctx.out.begin_capture();
        let held = section(&mut ctx).expect("the section decides");
        let (stdout, _) = ctx.out.end_capture();
        (held, stdout)
    }

    #[test]
    fn r11__doctor_refuses_an_argument_it_does_not_know() {
        let mut ctx = context();
        let refused = doctor(&mut ctx, &["--bogus".to_string()]).expect_err("refused");
        assert_eq!(refused.code(), 1);
        assert_eq!(refused.to_string(), "dstack: usage: dstack doctor [--self]");
        let refused = doctor(&mut ctx, &["one".to_string(), "two".to_string()])
            .expect_err("an extra operand is refused too");
        assert_eq!(refused.to_string(), "dstack: usage: dstack doctor [--self]");
    }

    #[test]
    fn r13__the_globs_skip_dotfiles_and_missing_directories() {
        let home = Home::resolve().expect("repository");
        let agents = glob(&home.home.join("agents"), ".md");
        assert!(agents.len() >= 5, "the agent definitions are there");
        assert!(agents.windows(2).all(|pair| pair[0] < pair[1]), "sorted");
        assert!(glob(&home.home.join("no-such-directory"), ".md").is_empty());
        let skills = glob_sub(&home.home.join("skills"), "SKILL.md");
        assert!(skills.iter().all(|file| file.ends_with("SKILL.md")));
        assert!(glob_sub(&home.home.join("no-such-directory"), "SKILL.md").is_empty());
    }
}
