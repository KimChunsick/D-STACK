// verbs/doctor/hooks.rs
// doctor section 6: which hooks settings.json registers, and what they last answered (R101).

use std::fmt;
use std::path::PathBuf;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::roots::git_out;

/// jq's `to_entries` walks the object in document order; serde_json's Map is sorted unless the
/// crate is built with preserve_order, so the events are collected in the order they are read.
struct Events(Vec<(String, Vec<Group>)>);

#[derive(Deserialize)]
struct Group {
    matcher: Option<String>,
    #[serde(default)]
    hooks: Vec<Registration>,
}

#[derive(Deserialize)]
struct Registration {
    #[serde(default)]
    command: String,
}

#[derive(Deserialize)]
struct Settings {
    hooks: Option<Events>,
}

impl<'de> Deserialize<'de> for Events {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Events, D::Error> {
        struct InOrder;
        impl<'de> Visitor<'de> for InOrder {
            type Value = Events;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("the hooks object of settings.json")
            }
            fn visit_map<M: MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Events, M::Error> {
                let mut events = Vec::new();
                while let Some(entry) = map.next_entry::<String, Vec<Group>>()? {
                    events.push(entry);
                }
                Ok(Events(events))
            }
        }
        deserializer.deserialize_map(InOrder)
    }
}

pub fn section(ctx: &mut Context) -> Result<bool> {
    let settings =
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".claude/settings.json");
    say!(ctx, "hooks registered in {}:", settings.display());
    let (mut n, mut stray) = (0, 0);
    match std::fs::read_to_string(&settings) {
        Err(_) => say!(ctx, "  (no settings.json or no jq)"),
        Ok(text) => {
            let parsed: Settings = serde_json::from_str(&text).unwrap_or(Settings { hooks: None });
            for (event, groups) in parsed.hooks.map(|events| events.0).unwrap_or_default() {
                for group in groups {
                    let matcher = group.matcher.unwrap_or_else(|| "*".to_string());
                    for registration in group.hooks {
                        n += 1;
                        let command = registration.command;
                        if command.contains("dstack-hook.sh") {
                            say!(ctx, "  {event} [{matcher}] → {command}");
                        } else {
                            stray += 1;
                            say!(ctx, "  {event} [{matcher}] → {command}  | FAIL: not the dstack hook wrapper (R101 registers one script only)");
                        }
                    }
                }
            }
        }
    }
    say!(ctx, "  registered: {n}, stray: {stray}");
    say!(ctx, "hook last results (event | exit | at | note):");
    last_results(ctx)?;
    Ok(stray == 0)
}

/// The `.last` file each hook leaves behind in this worktree: its last row, as four columns.
fn last_results(ctx: &mut Context) -> Result<()> {
    if git_out(None, &["rev-parse", "--git-common-dir"]).is_none() {
        say!(ctx, "  skipped: not in a git repository");
        return Ok(());
    }
    let roots = ctx.roots()?;
    let files = super::glob(&roots.local.join("hooks"), ".last");
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        let last = text.lines().last().unwrap_or_default();
        let column: Vec<&str> = last.split('\t').collect();
        let field = |at: usize| column.get(at).copied().unwrap_or("");
        say!(
            ctx,
            "  {} | {} | {} | {}",
            field(0),
            field(1),
            field(2),
            field(3)
        );
    }
    if files.is_empty() {
        say!(ctx, "  no hook has run in this worktree yet");
    }
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r101__the_registrations_are_read_in_document_order() {
        let text = r#"{"hooks":{"UserPromptSubmit":[{"hooks":[{"command":"a dstack-hook.sh inject"}]}],
                                 "Stop":[{"matcher":"*","hooks":[{"command":"nope"}]}]}}"#;
        let parsed: Settings = serde_json::from_str(text).expect("parses");
        let events = parsed.hooks.expect("the hooks object").0;
        let names: Vec<&str> = events.iter().map(|(event, _)| event.as_str()).collect();
        assert_eq!(names, vec!["UserPromptSubmit", "Stop"], "not sorted");
        assert_eq!(events[0].1[0].matcher, None, "the default matcher is *");
        assert_eq!(events[1].1[0].hooks[0].command, "nope");
    }

    #[test]
    fn r101__the_section_counts_the_registrations_of_this_machine() {
        let (_, printed) = super::super::tests::printed(section);
        let lines: Vec<&str> = printed.lines().collect();
        assert!(lines[0].starts_with("hooks registered in "));
        assert!(
            lines.iter().any(|line| line.starts_with("  registered: ")),
            "no count line:\n{printed}"
        );
        assert!(lines
            .iter()
            .any(|line| *line == "hook last results (event | exit | at | note):"));
    }
}
