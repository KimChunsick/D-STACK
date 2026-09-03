// verbs/doctor/agents.rs
// doctor section 2: the model policy every agent definition has to declare (R20, R25).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::paths::base_name;
use crate::selftest::{Selftest, Verdict};

pub fn section(ctx: &mut Context) -> Result<bool> {
    let dir = ctx.home.home.join("agents");
    say!(
        ctx,
        "agents ({}): file | model | effort | maxTurns | tools",
        dir.display()
    );
    let (mut n, mut bad) = (0, 0);
    for file in super::glob(&dir, ".md") {
        n += 1;
        if !row(ctx, &file) {
            bad += 1;
        }
    }
    say!(ctx, "  agents: {n} files, ok {}, failing {bad}", n - bad);
    Ok(bad == 0)
}

/// One table row; false when the definition breaks the model policy of R25.
fn row(ctx: &mut Context, file: &Path) -> bool {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    let (model, effort) = (fm_field(&text, "model"), fm_field(&text, "effort"));
    let (turns, tools) = (fm_field(&text, "maxTurns"), fm_field(&text, "tools"));
    let mut note = match model.as_str() {
        "sonnet" | "opus" => String::new(),
        "" => "model missing".to_string(),
        "fable" | "haiku" | "inherit" => {
            format!("model '{model}' is not allowed (only the sonnet/opus aliases)")
        }
        pinned if pinned.starts_with("claude-") || carries_a_date(pinned) => {
            format!("full model id '{model}' pins a version; use the alias")
        }
        _ => format!("model '{model}' is not sonnet or opus"),
    };
    for (value, missing) in [
        (&effort, "effort missing"),
        (&turns, "maxTurns missing"),
        (&tools, "tools allowlist missing"),
    ] {
        if value.is_empty() {
            if !note.is_empty() {
                note.push_str("; ");
            }
            note.push_str(missing);
        }
    }
    let shown = |value: &str| match value.is_empty() {
        true => "-".to_string(),
        false => value.to_string(),
    };
    say!(
        ctx,
        "  {} | {} | {} | {} | {}{}",
        base_name(file),
        shown(&model),
        shown(&effort),
        shown(&turns),
        shown(&tools),
        match note.is_empty() {
            true => String::new(),
            false => format!(" | FAIL: {note}"),
        }
    );
    note.is_empty()
}

/// The shell's `*-2[0-9][0-9][0-9]*` arm: a dated model id such as sonnet-20250219.
fn carries_a_date(model: &str) -> bool {
    let bytes = model.as_bytes();
    bytes.windows(5).any(|window| {
        window[0] == b'-' && window[1] == b'2' && window[2..].iter().all(u8::is_ascii_digit)
    })
}

/// The frontmatter reader of `_fm_field`: the block has to open on line 1 with `---`, and the
/// first row whose key matches answers. Nothing outside the block is read.
fn fm_field(text: &str, key: &str) -> String {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return String::new();
    }
    for line in lines {
        if line == "---" {
            break;
        }
        if let Some(at) = line.find(':') {
            if &line[..at] == key {
                return line[at + 1..].trim_matches([' ', '\t']).to_string();
            }
        }
    }
    String::new()
}

/// claude/lint/fixtures/doctor-agents/*.md — one agent definition per fixture.
pub struct Checker;

impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "doctor-agents"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        Ok(match super::quiet(ctx, |ctx| row(ctx, fixture)) {
            true => Verdict::Pass,
            false => Verdict::Reject,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r20__the_agents_of_this_repository_all_pass() {
        let (held, printed) = super::super::tests::printed(section);
        assert!(held, "an agent definition fails the policy:\n{printed}");
        let last = printed.lines().last().expect("the count line");
        assert!(last.starts_with("  agents: "), "unexpected line: {last}");
        assert!(last.ends_with(", failing 0"), "unexpected line: {last}");
    }

    #[test]
    fn r05__the_checker_judges_every_fixture_by_its_name() {
        let mut ctx = super::super::tests::context();
        let dir = ctx.home.home.join("lint/fixtures/doctor-agents");
        for fixture in super::super::glob(&dir, ".md") {
            let wanted = match base_name(&fixture).starts_with("bad-") {
                true => Verdict::Reject,
                false => Verdict::Pass,
            };
            let verdict = Checker.run(&mut ctx, &fixture).expect("decides");
            assert_eq!(verdict, wanted, "{}", fixture.display());
        }
    }

    #[test]
    fn r20__a_dated_model_id_is_a_pinned_version() {
        assert!(carries_a_date("sonnet-20250219"));
        assert!(!carries_a_date("sonnet"));
        assert!(!carries_a_date("gpt-5"));
        assert_eq!(fm_field("---\nmodel: opus\n---\n", "model"), "opus");
        assert_eq!(fm_field("model: opus\n", "model"), "", "no frontmatter");
        assert_eq!(fm_field("---\n---\nmodel: opus\n", "model"), "", "past it");
    }
}
