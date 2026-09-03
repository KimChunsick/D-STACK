// verbs/lint/run.rs
// The lint-ko verb itself: its modes, its options, its counters and the lines it prints.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::core::args;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::roots::git_out;
use crate::core::verb::Verb;

use super::rules::{hits, Table};
use super::scope::{self, Scopes};

const USAGE: &str = "dstack lint-ko [paths] | --stdin --path p [--fragment] | --stdin --scope commit-msg | --changed | --report";

struct LintKo;

impl Verb for LintKo {
    fn name(&self) -> &'static str {
        "lint-ko"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        lint_ko(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(LintKo)]
}

enum Mode {
    File,
    Stdin,
    Changed,
}

/// cmd_lint_ko(). The Stop gate asks for `--changed`, which reads no stdin at all.
pub fn lint_ko(ctx: &mut Context, argv: &[String]) -> Result<()> {
    lint_ko_stdin(ctx, argv, None)
}

/// cmd_lint_ko() with the body of a `--stdin` call handed over instead of read from fd 0: the
/// PreToolUse hook spends its own stdin on the payload before anything can fail, so it holds the
/// pending content in memory where the shell had a fresh subprocess to pipe it into.
pub fn lint_ko_stdin(ctx: &mut Context, argv: &[String], body: Option<&str>) -> Result<()> {
    let (wt_root, main_root) = ko_roots(ctx);
    let mut mode = Mode::File;
    let mut stdin_path = String::new();
    let mut scope_arg = String::new();
    let mut rules = String::new();
    let (mut fragment, mut report) = (false, false);
    let mut paths: Vec<String> = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        let next = argv.get(i + 1).map(String::as_str);
        if arg == "--stdin" {
            mode = Mode::Stdin;
            i += 1;
        } else if arg == "--changed" {
            mode = Mode::Changed;
            i += 1;
        } else if arg == "--fragment" {
            fragment = true;
            i += 1;
        } else if arg == "--report" {
            report = true;
            i += 1;
        } else if let Some((value, ate)) = args::opt(arg, next, "path")? {
            stdin_path = value;
            i += ate;
        } else if let Some((value, ate)) = args::opt(arg, next, "scope")? {
            scope_arg = value;
            i += ate;
        } else if arg == "--rules" {
            // The shell option loop has no --rules=value arm, so that form is an unknown option.
            rules = next.ok_or(Error::Exit(1))?.to_string();
            i += 2;
        } else if args::is_option(arg) {
            return Err(Error::failed(format!("unknown option: {arg} ({USAGE})")));
        } else {
            paths.push(arg.to_string());
            i += 1;
        }
    }

    let table = Table::load(&table_path(ctx, &rules))?;
    let mut lint = Lint {
        scopes: Scopes::load(&main_root, &ctx.home.home),
        table,
        wt_root,
        report,
        files: 0,
        hits: 0,
        s1: 0,
        unclassified: 0,
    };
    match mode {
        Mode::Stdin => lint.stdin(ctx, body, &stdin_path, &scope_arg, fragment)?,
        Mode::Changed => lint.changed(ctx),
        Mode::File => lint.files(ctx, &paths)?,
    }

    ctx.out.say(&format!(
        "files {}, hits {} (S1 {}), unclassified {}",
        lint.files, lint.hits, lint.s1, lint.unclassified
    ));
    if report {
        ctx.out.say(&format!(
            "rules {} ({} regex, {} judgment) from {}",
            lint.table.rules.len(),
            lint.table.regex_n,
            lint.table.judgment_n,
            lint.table.path.display()
        ));
        let table = match &lint.scopes.path {
            Some(path) => path.display().to_string(),
            None => "<none — every path is unclassified and nothing blocks>".to_string(),
        };
        ctx.out.say(&format!("scope table: {table}"));
    }
    // Only an S1 in a blocking scope reaches here: the scan returns for en, exempt and ko-data.
    match lint.s1 {
        0 => Ok(()),
        _ => Err(Error::Exit(1)),
    }
}

/// What one run counts and what it needs to count it.
struct Lint {
    table: Table,
    scopes: Scopes,
    wt_root: PathBuf,
    report: bool,
    files: usize,
    hits: usize,
    s1: usize,
    unclassified: usize,
}

impl Lint {
    /// The body of a pending Write, Edit or Bash arrives on stdin under the path it is meant for,
    /// or as a commit message, which belongs to no path at all.
    fn stdin(
        &mut self,
        ctx: &mut Context,
        body: Option<&str>,
        stdin_path: &str,
        scope_arg: &str,
        fragment: bool,
    ) -> Result<()> {
        let text = match body {
            Some(body) => body.to_string(),
            None => {
                let mut raw = Vec::new();
                let _ = std::io::stdin().read_to_end(&mut raw);
                String::from_utf8_lossy(&raw).to_string()
            }
        };
        self.files = 1;
        if scope_arg == "commit-msg" {
            self.scan(ctx, &text, "<commit-msg>", "commit-msg", fragment);
            return Ok(());
        }
        if stdin_path.is_empty() {
            return Err(Error::failed(
                "--stdin needs --path <path> or --scope commit-msg",
            ));
        }
        let rel = scope::rel(&self.wt_root, stdin_path);
        let scope = self.scopes.of(&rel).to_string();
        match scope.is_empty() {
            true => self.unclassified = 1,
            false => self.scan(ctx, &text, &rel, &scope, fragment),
        }
        Ok(())
    }

    /// HEAD-relative changes plus untracked files: the Stop gate asks what this turn wrote.
    fn changed(&mut self, ctx: &mut Context) {
        let mut list: Vec<String> = Vec::new();
        for command in [
            ["diff", "--name-only", "HEAD"].as_slice(),
            ["ls-files", "--others", "--exclude-standard"].as_slice(),
        ] {
            if let Some(answer) = git_out(Some(&self.wt_root), command) {
                list.extend(answer.lines().map(str::to_string));
            }
        }
        // `sort -u`, by bytes. The shell sorts in the UTF-8 locale _ko_locale exports, whose
        // collation orders a mixed-case set of paths differently (reported with P9).
        list.sort();
        list.dedup();
        for path in list {
            if path.is_empty() {
                continue;
            }
            let absolute = self.wt_root.join(&path);
            if !absolute.is_file() {
                continue;
            }
            self.do_file(ctx, &absolute.display().to_string());
        }
    }

    fn files(&mut self, ctx: &mut Context, paths: &[String]) -> Result<()> {
        if paths.is_empty() {
            return Err(Error::failed(
                "nothing to check: pass paths, --stdin, or --changed",
            ));
        }
        for path in paths {
            if !Path::new(path).is_file() {
                ctx.out
                    .say(&format!("{path}: skipped (not a regular file)"));
                continue;
            }
            self.do_file(ctx, path);
        }
        Ok(())
    }

    /// _ko_do_file(): resolve the scope of one path, count it, scan it.
    fn do_file(&mut self, ctx: &mut Context, path: &str) {
        let rel = scope::rel(&self.wt_root, path);
        let scope = self.scopes.of(&rel).to_string();
        self.files += 1;
        if scope.is_empty() {
            self.unclassified += 1;
            if self.report {
                ctx.out
                    .say(&format!("{rel}: unclassified (no ko-scope.tsv row)"));
            }
            return;
        }
        if self.report {
            ctx.out.say(&format!("{rel}: scope {scope}"));
        }
        let text = std::fs::read(path)
            .map(|raw| String::from_utf8_lossy(&raw).to_string())
            .unwrap_or_default();
        self.scan(ctx, &text, &rel, &scope, false);
    }

    /// _ko_scan(): every regex rule over every line, rule by rule as the shell ran one grep per
    /// rule. A fragment is an Edit's new_string, where only word-level rules survive: a sentence
    /// rule judging half a sentence would reject correct prose (R93).
    fn scan(&mut self, ctx: &mut Context, text: &str, label: &str, scope: &str, fragment: bool) {
        if !scope::blocks(scope) {
            return;
        }
        let (mut found, mut severe) = (0, 0);
        for rule in &self.table.rules {
            let matcher = match &rule.matcher {
                Some(matcher) if !fragment || rule.level == "word" => matcher,
                _ => continue,
            };
            for (line, matched) in hits(matcher, text) {
                ctx.out.say(&format!(
                    "{label}:{line}: {} ({}) matched '{matched}' → {} [ko-rules.tsv]",
                    rule.id, rule.severity, rule.replacement
                ));
                found += 1;
                if rule.severity == "S1" {
                    severe += 1;
                }
            }
        }
        self.hits += found;
        self.s1 += severe;
    }
}

/// `${1:-${DSTACK_KO_RULES:-$DSTACK_HOME/lint/ko-rules.tsv}}`.
fn table_path(ctx: &Context, rules: &str) -> PathBuf {
    if !rules.is_empty() {
        return PathBuf::from(rules);
    }
    match std::env::var("DSTACK_KO_RULES") {
        Ok(path) if !path.is_empty() => PathBuf::from(path),
        _ => ctx.home.home.join("lint/ko-rules.tsv"),
    }
}

/// _ko_roots(): roots without dying. lint-ko runs outside a repository too — a commit message
/// from a bare checkout, a fixture in $TMPDIR — where resolve_roots would die(2) and a hook turns
/// a 2 into a block. `[ -n "${WT_ROOT:-}" ] && return 0` comes first: the Stop gate and the
/// PreToolUse hook resolved the roots before they called, and asking git again is the spawn that
/// costs most of a hook-path call (R10). The shell asked git twice; one rev-parse answers both.
fn ko_roots(ctx: &Context) -> (PathBuf, PathBuf) {
    if let Some(roots) = ctx.resolved_roots() {
        return (roots.wt_root, roots.main_root);
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    let answer = match git_out(None, &["rev-parse", "--show-toplevel", "--git-common-dir"]) {
        Some(answer) => answer,
        None => return (cwd.clone(), cwd),
    };
    let mut lines = answer.lines();
    let wt_root = PathBuf::from(lines.next().unwrap_or_default());
    let common = PathBuf::from(lines.next().unwrap_or_default());
    let common = match common.is_absolute() {
        true => common,
        false => cwd.join(common),
    };
    let main_root = std::fs::canonicalize(common.join("..")).unwrap_or_else(|_| wt_root.clone());
    (wt_root, main_root)
}
