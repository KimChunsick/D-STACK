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

/// What a command runs when it invokes the wrapper: the words after the script, or the whole
/// command when it equals an expected one. Nothing for any other command, even one that only
/// mentions dstack-hook.sh.
fn invocation(command: &str, expected: &[Hook]) -> Option<String> {
    runs(&shell_words(command), true).or_else(|| {
        let plain = collapse(command);
        expected
            .iter()
            .any(|want| collapse(&want.command) == plain)
            .then_some(plain)
    })
}

/// The words after the script when `words` invoke it. After an optional env prefix (`env`, its
/// options and NAME=value words) the first word is the script itself, or a shell — bash, sh, zsh,
/// dash or a path to one — whose first word after its options (words starting with -, `--`
/// included) is the script. A single-dash option holding c (`-c`, `-ec`) makes that word a
/// command string, read by this same rule when `nest` allows it: once, never deeper.
fn runs(words: &[String], nest: bool) -> Option<String> {
    let mut words = words;
    if words.first().is_some_and(|word| file_name(word) == "env") {
        let prefix = words[1..]
            .iter()
            .take_while(|word| word.starts_with('-') || word.contains('='))
            .count();
        words = &words[1 + prefix..];
    }
    let (first, rest) = words.split_first()?;
    if file_name(first) == WRAPPER {
        return Some(rest.join(" "));
    }
    if !matches!(file_name(first), "bash" | "sh" | "zsh" | "dash") {
        return None;
    }
    let (options, rest) = rest.split_at(rest.iter().take_while(|w| w.starts_with('-')).count());
    let (next, after) = rest.split_first()?;
    match options.iter().any(|o| !o.starts_with("--") && o.contains('c')) {
        true if nest => runs(&shell_words(next), false),
        true => None,
        false => (file_name(next) == WRAPPER).then(|| after.join(" ")),
    }
}

/// The words of a command as a shell splits them: unquoted whitespace separates words, and one
/// layer of single or double quotes is removed wherever it appears in a word.
fn shell_words(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quote = None;
    for c in command.chars() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => word.get_or_insert_with(String::new).push(c),
            None if c == '\'' || c == '"' => {
                quote = Some(c);
                word.get_or_insert_with(String::new);
            }
            None if c.is_whitespace() => words.extend(word.take()),
            None => word.get_or_insert_with(String::new).push(c),
        }
    }
    words.extend(word);
    words
}

fn file_name(word: &str) -> &str {
    word.rsplit('/').next().unwrap_or(word)
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

    #[test]
    fn R18__only_a_single_dash_c_makes_the_next_word_a_command_string_parsed_once() {
        let runs = |command: String| invocation(&command, &[]);
        let script = "$HOME/.claude/hooks/dstack-hook.sh";
        let stop = Some("stop".to_string());
        assert_eq!(runs(format!("sh -ec 'bash {script} stop'")), stop, "grouped -ec");
        assert_eq!(runs(format!("bash --norc {script} stop")), stop, "--norc is not -c");
        assert_eq!(runs(format!("sh -c \"sh -c 'bash {script} stop'\"")), None, "nested once");
    }
}
