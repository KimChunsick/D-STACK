// verbs/doctor/sweep.rs
// doctor section 4: every `dstack <verb>` mention is on the roster, and every roster entry is
// answered by a registered handler (R81; D-20 counts handlers where the shell counted functions).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::registry::ROSTER;
use crate::selftest::{Selftest, Verdict};

pub fn section(ctx: &mut Context) -> Result<bool> {
    say!(
        ctx,
        "verb sweep (R81): every 'dstack <verb>' mention must be in the roster of dstack help"
    );
    let (mut total, mut unknown) = (0, 0);
    for (file, docs) in files(ctx) {
        let (mentions, unlisted) = scan(ctx, &file, docs);
        total += mentions;
        unknown += unlisted;
    }
    say!(ctx, "  mentions: {total}, unknown verbs: {unknown}");
    // The roster must also be real: every entry needs a handler the registry can dispatch to.
    let mut missing = 0;
    for (entry, _) in ROSTER.iter() {
        if !ctx.handled(entry) {
            missing += 1;
            say!(ctx, "  roster entry '{entry}' has no handler");
        }
    }
    say!(
        ctx,
        "  roster: {} entries, without a function: {missing}",
        ROSTER.len()
    );
    Ok(unknown == 0 && missing == 0)
}

/// The files the sweep reads, each with whether only backticked mentions count. Records under
/// docs/ quote request rows verbatim ("dstack makes their worktrees"), so there the backtick is
/// required; rule files are swept in full.
fn files(ctx: &Context) -> Vec<(PathBuf, bool)> {
    let (home, repo) = (&ctx.home.home, &ctx.home.repo);
    let mut files: Vec<PathBuf> = Vec::new();
    for skill in super::subdirs(&home.join("skills")) {
        files.extend(super::glob(&skill, ".md"));
    }
    files.extend(super::glob(&home.join("agents"), ".md"));
    files.push(home.join("CLAUDE.md"));
    files.extend(super::glob(&home.join("output-styles"), ".md"));
    files.extend(super::glob(&home.join("hooks"), ".sh"));
    files.extend(super::glob(&home.join("templates/request"), ".md"));
    files.extend(super::glob(&home.join("templates/prompts"), ".md"));
    files.push(home.join("prompt-caching.md"));
    for name in ["README.md", "CLAUDE.md", "AGENTS.md", "codex/AGENTS.md"] {
        files.push(repo.join(name));
    }
    for skill in super::subdirs(&repo.join("codex/skills")) {
        files.extend(super::glob(&skill, ".md"));
    }
    files.retain(|file| file.is_file());
    let mut swept: Vec<(PathBuf, bool)> = files.into_iter().map(|file| (file, false)).collect();
    swept.extend(
        markdown_under(&repo.join("docs"))
            .into_iter()
            .map(|file| (file, true)),
    );
    swept
}

/// `find <dir> -name '*.md'`: nothing at all when the directory is not there.
fn markdown_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return files,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(markdown_under(&path));
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn mention_re(docs: bool) -> &'static Regex {
    static PLAIN: OnceLock<Regex> = OnceLock::new();
    static QUOTED: OnceLock<Regex> = OnceLock::new();
    let pattern = |lead: &str| {
        Regex::new(&format!(
            "{lead}dstack [a-z][a-z-]*( [a-z][a-z-]*)?($|[^A-Za-z0-9_-])"
        ))
        .expect("a valid mention pattern")
    };
    match docs {
        true => QUOTED.get_or_init(|| pattern("`")),
        false => PLAIN.get_or_init(|| pattern("")),
    }
}

/// One file: how many mentions it carries and how many of them no roster entry answers.
fn scan(ctx: &mut Context, file: &Path, docs: bool) -> (usize, usize) {
    let bytes = match std::fs::read(file) {
        Ok(bytes) => bytes,
        Err(_) => return (0, 0),
    };
    let text = String::from_utf8_lossy(&bytes);
    let (mut total, mut unknown) = (0, 0);
    for (at, line) in text.lines().enumerate() {
        for found in mention_re(docs).find_iter(line) {
            let mention = trim_boundary(found.as_str());
            let mention = mention.strip_prefix('`').unwrap_or(mention);
            let mention = mention.strip_prefix("dstack ").unwrap_or(mention);
            total += 1;
            let (first, second) = match mention.split_once(' ') {
                Some((first, second)) => (first, second),
                None => (mention, ""),
            };
            if !known(first, second) {
                unknown += 1;
                say!(ctx, "  {}:{}: dstack {mention}", file.display(), at + 1);
            }
        }
    }
    (total, unknown)
}

/// `sed -E 's/[^A-Za-z0-9_-]$//'`: the character the pattern needed as a boundary is not part of
/// the mention.
fn trim_boundary(mention: &str) -> &str {
    match mention.chars().next_back() {
        Some(last) if !(last.is_ascii_alphanumeric() || last == '_' || last == '-') => {
            &mention[..mention.len() - last.len_utf8()]
        }
        _ => mention,
    }
}

/// `_verb_known`: the two-word entry, the one-word entry, or a bare noun in prose ("dstack run").
fn known(first: &str, second: &str) -> bool {
    let two_word = format!("{first} {second}");
    let noun = format!("{first} ");
    ROSTER.iter().any(|(entry, _)| {
        (!second.is_empty() && *entry == two_word) || *entry == first || entry.starts_with(&noun)
    })
}

/// claude/lint/fixtures/verb-sweep/*.md — a file mentioning a verb the roster does not carry.
pub struct Checker;

impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "verb-sweep"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let (_, unknown) = super::quiet(ctx, |ctx| scan(ctx, fixture, false));
        Ok(match unknown {
            0 => Verdict::Pass,
            _ => Verdict::Reject,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::paths::base_name;

    #[test]
    fn r81__this_repository_mentions_no_verb_outside_the_roster() {
        let (_, printed) = super::super::tests::printed(section);
        let counts = printed
            .lines()
            .find(|line| line.starts_with("  mentions: "))
            .expect("the mention count line");
        assert!(
            counts.ends_with(", unknown verbs: 0"),
            "the sweep found a verb the roster does not carry:\n{printed}"
        );
    }

    /// The roster half: every entry the dispatcher answers is counted, and the only entry that
    /// may still be without one is `hook`, which P14 is porting in parallel (D-01) — the registry
    /// calls such an entry a stated "not ported yet", never a silent gap.
    #[test]
    fn r13__the_roster_half_counts_the_entries_no_handler_answers() {
        let ctx = super::super::tests::context();
        let missing: Vec<&str> = ROSTER
            .iter()
            .map(|(entry, _)| *entry)
            .filter(|entry| !ctx.handled(entry))
            .collect();
        assert!(
            missing.iter().all(|entry| *entry == "hook"),
            "roster entries without a handler: {missing:?}"
        );
        let (_, printed) = super::super::tests::printed(section);
        assert_eq!(
            printed.lines().last().expect("the roster count line"),
            format!(
                "  roster: {} entries, without a function: {}",
                ROSTER.len(),
                missing.len()
            )
        );
    }

    #[test]
    fn r05__the_checker_judges_every_fixture_by_its_name() {
        let mut ctx = super::super::tests::context();
        let dir = ctx.home.home.join("lint/fixtures/verb-sweep");
        for fixture in super::super::glob(&dir, ".md") {
            let wanted = match base_name(&fixture).starts_with("bad-") {
                true => Verdict::Reject,
                false => Verdict::Pass,
            };
            assert_eq!(
                Checker.run(&mut ctx, &fixture).expect("decides"),
                wanted,
                "{}",
                fixture.display()
            );
        }
    }

    #[test]
    fn r81__what_counts_as_a_mention_of_a_known_verb() {
        assert!(known("run", "new"));
        assert!(known("run", "elsewhere"), "a bare noun in prose");
        assert!(known("status", ""));
        assert!(!known("frobnicate", "now"));
        assert_eq!(trim_boundary("dstack run new "), "dstack run new");
        assert_eq!(trim_boundary("dstack lint-ko"), "dstack lint-ko");
    }
}
