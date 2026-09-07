// The real doctor section and its fixtures use the same document checker.
// This is deterministic policy lint; passing it is not evidence of live host behavior.
use super::main_runtime_rules as rules;
use crate::core::{
    context::Context,
    error::{Error, Result},
};
use crate::selftest::{Selftest, Verdict};
use regex::Regex;
use std::{fs, path::Path};

#[derive(Debug)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
}

pub fn documents() -> Vec<&'static str> {
    let mut paths = vec!["claude/runtime.md"];
    paths.extend_from_slice(rules::REFERENCES);
    paths.extend_from_slice(rules::WORKERS);
    paths
}

// Collapse wrapping and code spans, retaining the original line for each output byte.
fn normalized(text: &str) -> (String, Vec<usize>) {
    let (mut out, mut lines, mut line, mut space) = (String::new(), Vec::new(), 1, false);
    for ch in text.chars() {
        if ch == '\n' {
            line += 1;
        }
        if ch == '`' {
            continue;
        }
        if ch.is_whitespace() {
            space = !out.is_empty();
            continue;
        }
        if space {
            out.push(' ');
            lines.push(line);
            space = false;
        }
        out.push(ch);
        lines.extend(std::iter::repeat_n(line, ch.len_utf8()));
    }
    (out, lines)
}

pub fn check_document(path: &str, text: &str) -> Vec<Diagnostic> {
    let mut required = Vec::new();
    if path == "claude/runtime.md" {
        required.extend_from_slice(rules::RUNTIME);
    } else if rules::WORKERS.contains(&path) {
        required.push(("compact receipt", rules::RECEIPT));
    } else if rules::REFERENCES.contains(&path) {
        required.push(("runtime protocol reference", rules::REFERENCE));
    } else {
        return vec![Diagnostic {
            line: 1,
            message: format!("unknown contract document {path}"),
        }];
    }
    if path == "claude/skills/dstack-develop/SKILL.md" {
        required.extend_from_slice(&[
            ("GSD provenance qualification", "Retained secondary attribution"),
            ("GSD fresh-context source", "docs/explanation/the-phase-loop.md"),
            ("GSD fresh-context quote", "Each executor gets a fresh 200k-token context window loaded with exactly what it needs"),
            ("GSD coordinator source", "skills/gsd-execute-phase/SKILL.md"),
            ("GSD coordinator quote", "Orchestrator stays lean: discover plans, analyze dependencies, group into waves, spawn subagents, collect results."),
        ]);
    }
    let (flat, lines) = normalized(text);
    let mut diagnostics = Vec::new();
    for (name, clause) in required {
        if !flat.contains(&normalized(clause).0) {
            let label = clause.split(':').next().unwrap_or(clause);
            let line = text
                .lines()
                .position(|line| line.contains(label))
                .map_or(1, |at| at + 1);
            diagnostics.push(Diagnostic {
                line,
                message: format!("missing {name}"),
            });
        }
    }
    if path == "claude/skills/dstack-develop/SKILL.md" {
        let rows: Vec<_> = text
            .lines()
            .enumerate()
            .filter(|(_, line)| line.starts_with('|'))
            .collect();
        let at = |command: &str| {
            rows.iter()
                .find(|(_, line)| line.contains(command))
                .map(|(line, _)| *line)
        };
        match (
            at("dstack review --scope plan --plan"),
            at("dstack review seal --from"),
            at("dstack plan done"),
        ) {
            (Some(review), Some(seal), Some(done)) if review < seal && seal < done => (),
            (_, _, done) => diagnostics.push(Diagnostic {
                line: done.map_or(1, |line| line + 1),
                message: "Plan checklist must seal independent review before completion".into(),
            }),
        }
    }
    for (name, pattern) in rules::FORBIDDEN {
        for found in Regex::new(pattern)
            .expect("fixed policy pattern")
            .find_iter(&flat)
        {
            // Patterns may include the preceding sentence delimiter; report the instruction.
            let at = found.start()
                + found
                    .as_str()
                    .find(|c: char| c.is_alphabetic())
                    .unwrap_or(0);
            diagnostics.push(Diagnostic {
                line: lines[at],
                message: name.to_string(),
            });
        }
    }
    diagnostics
}

fn print(ctx: &mut Context, path: &str, diagnostics: &[Diagnostic]) {
    for issue in diagnostics {
        say!(
            ctx,
            "  main-runtime: {path}:{}: {}",
            issue.line,
            issue.message
        );
    }
}

pub fn section(ctx: &mut Context) -> Result<bool> {
    say!(
        ctx,
        "main-runtime (R15/R16/R17): deterministic policy lint; live host UI/input unverified"
    );
    let mut bad = 0;
    for path in documents() {
        let issues = match fs::read_to_string(ctx.home.repo.join(path)) {
            Ok(text) => check_document(path, &text),
            Err(error) => vec![Diagnostic {
                line: 1,
                message: format!("unreadable contract: {error}"),
            }],
        };
        bad += issues.len();
        print(ctx, path, &issues);
    }
    say!(
        ctx,
        "  main-runtime documents: {}, violations: {bad}",
        documents().len()
    );
    Ok(bad == 0)
}

// good-*.md stores "document: <canonical path>" then policy text. bad-*.json mutates
// one such baseline, so a retained good clause cannot hide an appended contradiction.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Mutation {
    base: String,
    remove: Option<String>,
    append: Option<String>,
}

fn fixture_text(path: &Path) -> Result<(String, String)> {
    let read =
        |path: &Path| fs::read_to_string(path).map_err(|e| Error::cannot_decide(e.to_string()));
    let mut text = read(path)?;
    if path.extension().is_some_and(|ext| ext == "json") {
        let mutation: Mutation =
            serde_json::from_str(&text).map_err(|e| Error::cannot_decide(e.to_string()))?;
        if !mutation.base.starts_with("good-") || mutation.base.contains(['/', '\\']) {
            return Err(Error::cannot_decide(
                "fixture base must be a local good-* filename",
            ));
        }
        text = read(&path.with_file_name(mutation.base))?;
        if let Some(remove) = mutation.remove {
            if remove.is_empty() || !text.contains(&remove) {
                return Err(Error::cannot_decide(
                    "fixture removal did not match its baseline",
                ));
            }
            text = text.replacen(&remove, "", 1);
        }
        if let Some(append) = mutation.append {
            text.push('\n');
            text.push_str(&append);
        }
    }
    let (header, body) = text
        .split_once('\n')
        .ok_or_else(|| Error::cannot_decide("missing fixture document header"))?;
    let path = header
        .strip_prefix("document: ")
        .ok_or_else(|| Error::cannot_decide("missing fixture document header"))?;
    Ok((path.into(), body.into()))
}

pub struct Checker;
impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "main-runtime"
    }
    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let (path, text) = fixture_text(fixture)?;
        let issues = check_document(&path, &text);
        print(ctx, &path, &issues);
        Ok(if issues.is_empty() {
            Verdict::Pass
        } else {
            Verdict::Reject
        })
    }
}
