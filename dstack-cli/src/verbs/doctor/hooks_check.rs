// verbs/doctor/hooks_check.rs
// The registration judgement of the hooks section: each dstack hook once, other programs noted (R18).

use std::fmt;
use std::path::Path;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::{Selftest, Verdict};

/// The script every dstack registration runs; any other command belongs to another program.
const WRAPPER: &str = "dstack-hook.sh";
const FOREIGN: &str = "  | note: registered by another program; dstack does not manage it";
const STALE: &str =
    "  | FAIL: no dstack hook has this event, matcher and argument (stale or wrong registration)";

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

/// One registered command, under its event and matcher (no matcher reads `*`).
pub struct Hook {
    pub event: String,
    pub matcher: String,
    pub command: String,
}

impl Hook {
    /// The dstack hook this registration is: its event, its matcher and what its command runs.
    /// Nothing for a command of another program, even one that only mentions the wrapper.
    fn key(&self, expected: &[Hook]) -> Option<(&str, &str, String)> {
        let runs = invocation(&self.command, expected)?;
        Some((&self.event, &self.matcher, runs))
    }

    fn row(&self) -> String {
        format!("{} [{}] → {}", self.event, self.matcher, self.command)
    }
}

/// What a command runs when it invokes the wrapper: every argument after the script when its
/// first token runs dstack-hook.sh (a shell — bash, sh or an absolute path to one — followed by
/// the script, or the script itself), the whole command when it equals an expected one. Words
/// lose one layer of surrounding quotes. Nothing for any other command.
fn invocation(command: &str, expected: &[Hook]) -> Option<String> {
    let words: Vec<&str> = command.split_whitespace().map(unquote).collect();
    let is_wrapper = |word: &str| word.rsplit('/').next() == Some(WRAPPER);
    let is_shell = |word: &str| {
        matches!(word, "bash" | "sh")
            || (word.starts_with('/') && (word.ends_with("/bash") || word.ends_with("/sh")))
    };
    let script = match words.as_slice() {
        [shell, script, ..] if is_shell(shell) && is_wrapper(script) => 2,
        [script, ..] if is_wrapper(script) => 1,
        _ => {
            let plain = collapse(command);
            return expected
                .iter()
                .any(|want| collapse(&want.command) == plain)
                .then_some(plain);
        }
    };
    Some(words[script..].join(" "))
}

fn unquote(word: &str) -> &str {
    for quote in ['\'', '"'] {
        if let Some(inner) = word.strip_prefix(quote).and_then(|w| w.strip_suffix(quote)) {
            return inner;
        }
    }
    word
}

fn collapse(command: &str) -> String {
    command.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every registration of a settings.json text, in document order.
fn registrations(text: &str) -> std::result::Result<Vec<Hook>, serde_json::Error> {
    let settings: Settings = serde_json::from_str(text)?;
    let mut hooks = Vec::new();
    for (event, groups) in settings.hooks.map(|events| events.0).unwrap_or_default() {
        for group in groups {
            let matcher = group.matcher.unwrap_or_else(|| "*".to_string());
            for registration in group.hooks {
                hooks.push(Hook {
                    event: event.clone(),
                    matcher: matcher.clone(),
                    command: registration.command,
                });
            }
        }
    }
    Ok(hooks)
}

/// The dstack registrations install.sh writes: those of claude/settings.enforced.json.
pub fn expected(home: &Path) -> Result<Vec<Hook>> {
    let path = home.join("settings.enforced.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", path.display())))?;
    registrations(&text)
        .map_err(|e| Error::cannot_decide(format!("cannot parse {}: {e}", path.display())))
}

/// What the section prints between its header and its count line, and whether it holds.
pub struct Judgement {
    pub lines: Vec<String>,
    pub registered: usize,
    pub dstack: usize,
    pub other: usize,
    pub holds: bool,
}

/// A settings.json text, None when there is no such file, against the expected dstack
/// registrations. Text that is not a readable hooks table fails rather than reading as "no
/// hooks"; no file, no hooks key and an empty table all leave every dstack hook missing.
pub fn judge(text: Option<&str>, expected: &[Hook]) -> Judgement {
    let mut judgement = Judgement {
        lines: Vec::new(),
        registered: 0,
        dstack: 0,
        other: 0,
        holds: true,
    };
    let actual = match text.map(registrations) {
        None => {
            judgement
                .lines
                .push("  (no settings.json: no hook is registered on this machine)".to_string());
            Vec::new()
        }
        Some(Err(e)) => {
            judgement.holds = false;
            judgement.lines.push(format!(
                "  FAIL: settings.json is not a hooks table doctor can read: {e}"
            ));
            return judgement;
        }
        Some(Ok(actual)) if actual.is_empty() => {
            judgement.lines.push("  (settings.json registers no hook)".to_string());
            actual
        }
        Some(Ok(actual)) => actual,
    };
    let wanted: Vec<_> = expected.iter().map(|want| want.key(expected)).collect();
    let keys: Vec<_> = actual.iter().map(|hook| hook.key(expected)).collect();
    for (hook, key) in actual.iter().zip(&keys) {
        judgement.registered += 1;
        let (note, held) = match key {
            None => {
                judgement.other += 1;
                (FOREIGN, true)
            }
            Some(_) => {
                judgement.dstack += 1;
                match wanted.contains(key) {
                    true => ("", true),
                    false => (STALE, false),
                }
            }
        };
        judgement.holds &= held;
        judgement.lines.push(format!("  {}{note}", hook.row()));
    }
    for (want, key) in expected.iter().zip(&wanted) {
        let times = keys.iter().filter(|hook| *hook == key).count();
        let fail = match times {
            1 => continue,
            0 => format!("  FAIL: dstack hook missing: {}", want.row()),
            n => format!("  FAIL: dstack hook registered {n} times: {} (expected once)", want.row()),
        };
        judgement.holds = false;
        judgement.lines.push(fail);
    }
    judgement
}

/// claude/lint/fixtures/doctor-hooks/*.json — one settings.json per fixture, judged as the
/// section judges this machine's.
pub struct Checker;

impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "doctor-hooks"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let text = std::fs::read_to_string(fixture).map_err(|e| {
            Error::cannot_decide(format!("cannot read {}: {e}", fixture.display()))
        })?;
        Ok(match judge(Some(&text), &expected(&ctx.home.home)?).holds {
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
    fn r101__the_registrations_are_read_in_document_order() {
        let text = r#"{"hooks":{"UserPromptSubmit":[{"hooks":[{"command":"a dstack-hook.sh inject"}]}],
                                 "Stop":[{"matcher":"*","hooks":[{"command":"nope"}]}]}}"#;
        let hooks = registrations(text).expect("parses");
        let names: Vec<&str> = hooks.iter().map(|hook| hook.event.as_str()).collect();
        assert_eq!(names, vec!["UserPromptSubmit", "Stop"], "not sorted");
        assert_eq!(hooks[0].matcher, "*", "the default matcher is *");
        assert_eq!(hooks[1].command, "nope");
    }
}
