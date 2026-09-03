// verbs/issue/new.rs
// dstack issue new: file the friction, or add a sighting to the file that already carries it.

use crate::core::args::{is_option, opt, unknown_option};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, read_text, utc_now, with_lock};

use super::file::{self, Filing, Sighting};
use super::slug::slug;

/// A worker that hits a refusal copies this line out of the message and runs it, so the text after
/// "usage: " is a command a shell hands to dstack unchanged. Two things make that true, and the
/// unit test below holds both:
///
/// - every value is in SINGLE quotes. Double quotes still run `$(…)` and backticks, so a repro
///   like `the launcher runs $(rm -rf x)` would be run by the worker's own shell and the filed
///   issue would record the result instead of the command it meant to report. Inside single quotes
///   nothing happens at all. Text holding an apostrophe closes and reopens around it: 'it'\''s'.
/// - no `[...]`. The other 25 usage strings of this CLI mark an optional option that way and are
///   right to: they are notation, not something anyone pastes. Here `[--proposal` is a glob
///   pattern in zsh, which fails before dstack is reached, and a literal argument in bash, which
///   the verb then refuses — so the one line that tells a worker how to file could not be filed
///   with. Nothing is lost by showing --proposal plainly, because which fields are required is
///   what the refusals below say, one by one, in their own words.
///
/// This is the line claude/agents/*.md carry, character for character, so a worker that reads the
/// refusal and a worker that reads its agent definition are told the same thing.
/// tests/r05_issue_rules.rs holds those documents to the same rule (R05) by sweeping claude/;
/// this file's test covers the CLI's own message.
const USAGE: &str = "usage: dstack issue new '<short symptom>' --symptom '<what happened>' --repro '<how to make it happen>' --source '<the command or file>' --proposal '<one line>'";

issue_verb!(IssueNew, "issue new", new);

fn new(ctx: &mut Context, args: &[String]) -> Result<()> {
    let mut filing = Filing::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].clone();
        let next = args.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(&arg, next, "symptom")? {
            filing.symptom = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(&arg, next, "repro")? {
            filing.repro = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(&arg, next, "source")? {
            filing.source = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(&arg, next, "proposal")? {
            filing.proposal = value;
            i += eaten;
        } else if is_option(&arg) {
            return Err(unknown_option(&arg));
        } else if filing.title.is_empty() {
            filing.title = arg;
            i += 1;
        } else {
            fail!("unexpected argument: {arg} ({USAGE})")
        }
    }

    // The title is written into a frontmatter line, so its whitespace collapses to single spaces.
    filing.title = filing
        .title
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    if filing.title.is_empty() {
        fail!("{USAGE}")
    }
    // D-08: an issue nobody can act on is refused, and nothing at all is written. The wording is
    // not judged and the reproduction need not be a command — only that each field says something.
    for (name, value) in [
        ("--symptom", &filing.symptom),
        ("--repro", &filing.repro),
        ("--source", &filing.source),
    ] {
        if value.trim().is_empty() {
            fail!("{name} is required: an issue without it is not actionable ({USAGE})")
        }
    }
    let slug = slug(&filing.title);
    if slug.is_empty() {
        fail!(
            "the title carries no letter or digit to name a file after: {}",
            filing.title
        )
    }

    let (run, plan) = super::origin(ctx);
    let seen = Sighting {
        stamp: utc_now(),
        run,
        plan,
    };
    let dir = super::folder()?;
    let path = dir.join(format!("{slug}.md"));
    // Reading the count and writing it back is one step for everybody or it is nothing: a wave of
    // workers hitting the same friction at once is what this verb is for, and two of them reading
    // the same count would drop a sighting however atomic each write is. with_lock is the mkdir
    // lock the store already serialises its read-modify-writes with; it takes the issue folder
    // rather than a run's local dir, because the folder is the whole of what is being changed —
    // and it makes the folder on the way, which is the only place the folder is ever created.
    let _lock = with_lock(&dir)?;
    let (text, count) = match read_text(&path)? {
        Some(existing) => file::append(&existing, &seen).ok_or_else(|| {
            Error::failed(format!(
                "{} is not a dstack issue file: its frontmatter carries no sightings count",
                path.display()
            ))
        })?,
        None => (file::render(&filing, &seen), 1),
    };
    atomic_write(&path, text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))?;
    say!(ctx, "issue: {}", path.display());
    say!(
        ctx,
        "  sighting {count}  {}  run {}  plan {}",
        seen.stamp,
        seen.run,
        seen.plan
    );
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// The usage line is advice a worker runs verbatim, so it is held to what a shell does with
    /// it: every value single-quoted, and nothing anywhere that the shell expands or refuses.
    #[test]
    fn r01__the_usage_line_quotes_every_value_it_names() {
        let command = USAGE
            .strip_prefix("usage: ")
            .expect("the usage line names the command it is about");
        assert!(
            command.starts_with("dstack issue new '<short symptom>'"),
            "{command}"
        );
        for option in ["--symptom", "--repro", "--source", "--proposal"] {
            let value = command
                .split_once(&format!("{option} "))
                .unwrap_or_else(|| panic!("{option} is not in the usage line: {command}"))
                .1;
            assert!(
                value.starts_with("'<"),
                "the value of {option} is not single-quoted, so the shell reads it before dstack does: {value}"
            );
        }
        assert_eq!(
            shell_hazard(command),
            None,
            "the usage line is not one a worker can paste: {command}"
        );
    }

    /// The rule catches each hazard on its own, so a usage line that regresses says which one.
    #[test]
    fn r01__the_rule_names_what_a_shell_would_do_to_the_line() {
        assert!(shell_hazard("dstack issue new '<short symptom>' --repro '<how>'").is_none());
        let brackets = shell_hazard("dstack issue new '<t>' [--proposal '<one line>']")
            .expect("a bracket is not something a shell takes");
        assert!(brackets.starts_with("'[' outside quotes"), "{brackets}");
        let substitution = shell_hazard("dstack issue new '<t>' --repro \"it runs $(id)\"")
            .expect("a substitution inside double quotes runs before dstack sees it");
        assert!(
            substitution.starts_with("'$' inside double quotes"),
            "{substitution}"
        );
        let backtick = shell_hazard("dstack issue new '<t>' --repro \"it runs `id`\"")
            .expect("a backtick inside double quotes runs too");
        assert!(
            backtick.starts_with("'`' inside double quotes"),
            "{backtick}"
        );
        // A single-quoted value is inert, substitution characters and all.
        assert!(shell_hazard("dstack issue new '<t>' --repro 'it runs $(id)'").is_none());
    }

    /// What a shell would do to the line that is not handing it to dstack: outside the quotes, the
    /// brackets and braces of an optional-argument notation, a glob, a redirection; inside double
    /// quotes, the substitutions that stay alive there. Inside single quotes nothing happens at
    /// all, which is why the usage line uses them.
    ///
    /// The same list and the same reading as shell_hazard in tests/r05_issue_rules.rs, which holds
    /// the agent documents to this rule. They are copies on purpose: that one walks claude/ and
    /// this one reads a const of the lib, and a shell-safety helper is not behaviour to export.
    fn shell_hazard(command: &str) -> Option<String> {
        const OUTSIDE: [char; 15] = [
            '[', ']', '{', '}', '*', '?', '(', ')', '<', '>', '|', '&', ';', '$', '~',
        ];
        const EXPANDED: [char; 2] = ['$', '`'];
        let mut quote: Option<char> = None;
        for c in command.chars() {
            match quote {
                Some('"') if EXPANDED.contains(&c) => {
                    return Some(format!(
                        "'{c}' inside double quotes, which the shell expands before dstack sees the value"
                    ))
                }
                Some(open) if c == open => quote = None,
                Some(_) => {}
                None if c == '"' || c == '\'' => quote = Some(c),
                None if OUTSIDE.contains(&c) => {
                    return Some(format!("'{c}' outside quotes, which a shell does not take"))
                }
                None => {}
            }
        }
        None
    }
}
