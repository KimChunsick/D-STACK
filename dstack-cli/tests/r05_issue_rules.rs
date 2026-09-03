// tests/r05_issue_rules.rs
// R05: the three documents a worker reads before it starts — the two agent definitions and the
// brief template — each name `dstack issue new` and carry the filing threshold (D-04: friction
// that stopped the work or cost time, never an idea that merely occurred), and the example they
// show is one a worker can paste and one that files what the worker wrote: every option value is
// single-quoted, because every one of them is incident text — usually a command — and double
// quotes would let the shell run a substitution inside it before dstack ever saw the value.
// The issue folder is reached through the verb alone (D-02, D-05): nothing under
// claude/ aims a file-writing command at it, by its own name or through a variable holding it.
// The rule sits in one place per audience, so the two root rule files stay the byte-identical
// twins they claim to be, and doctor's verb sweep answers for the wording.
#![allow(non_snake_case)]

use std::path::PathBuf;
use std::process::{Command, Output};

/// What a worker reads before it writes anything: its own definition, and the brief it is sent.
const WORKER_DOCS: [&str; 3] = [
    "claude/agents/general-dev.md",
    "claude/agents/frontend-dev.md",
    "claude/skills/dstack-develop/SKILL.md",
];

/// The threshold of D-04, by what it has to say rather than by a sentence: each row is one thing,
/// and any word of the row says it. Lower case, because the documents are read folded.
const THRESHOLD: [(&str, &[&str]); 3] = [
    (
        "friction that stopped the work",
        &["block", "stopped", "refus"],
    ),
    ("friction that only cost time", &["cost", "slow", "detour"]),
    ("an idea is not friction", &["idea", "merely", "thought of"]),
];

/// The folder of D-05, and the commands that create or edit a file in it.
const FOLDER: &str = "Documents/dstack-issues";
const WRITES: [&str; 8] = ["mkdir", "touch", "tee", "cp", "mv", "rm", "ln", "sed -i"];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn git(args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(repo())
        .output()
        .expect("run git")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf-8")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|_| panic!("read {rel}"))
}

#[test]
fn r05__every_worker_document_names_the_verb_that_files() {
    for doc in WORKER_DOCS {
        assert!(
            read(doc).contains("dstack issue new"),
            "{doc} never tells a worker how to file"
        );
    }
}

#[test]
fn r05__every_worker_document_carries_the_filing_threshold() {
    for doc in WORKER_DOCS {
        let text = read(doc).to_lowercase();
        for (says, words) in THRESHOLD {
            assert!(
                words.iter().any(|word| text.contains(word)),
                "{doc} does not say '{says}' (any of {words:?})"
            );
        }
    }
}

/// The example is a command, so it is held to what a shell would do with it: nothing outside the
/// quotes may be expanded or refused (`[--proposal …]` dies in zsh before dstack is reached), and
/// nothing inside them may be expanded either. Values are single-quoted, because a worker fills
/// them with incident text — `the launcher runs $(echo SUBSTITUTED)` — and inside double quotes
/// the shell would run that and file its result instead of the command the worker reported.
#[test]
fn r05__the_filing_example_is_one_a_worker_can_paste() {
    for doc in WORKER_DOCS {
        for line in read(doc).lines() {
            for command in code_spans(line) {
                if !command.contains("dstack issue new") {
                    continue;
                }
                if let Some(hazard) = shell_hazard(command) {
                    panic!("{doc}: {hazard}:\n{command}");
                }
                for option in command.split(" --").skip(1) {
                    let value = option.split_once(' ').map(|(_, value)| value).unwrap_or("");
                    if value.is_empty() {
                        continue;
                    }
                    assert!(
                        value.starts_with('\''),
                        "{doc}: --{} takes a value that is not single-quoted, so the shell reads it \
                         instead of dstack:\n{command}",
                        option.split(' ').next().unwrap_or(option)
                    );
                }
            }
        }
    }
}

/// The `code spans` of one markdown line: what is between backticks, and nothing of the prose
/// around them.
fn code_spans(line: &str) -> Vec<&str> {
    line.split('`').skip(1).step_by(2).collect()
}

/// What a shell would do to the line that is not handing it to dstack: outside the quotes, the
/// brackets and braces of an optional-argument notation, a glob, a redirection; inside double
/// quotes, the substitutions that stay alive there. Inside single quotes, nothing happens at all,
/// which is why the example uses them.
fn shell_hazard(command: &str) -> Option<String> {
    const OUTSIDE: [char; 15] = [
        '[', ']', '{', '}', '*', '?', '(', ')', '<', '>', '|', '&', ';', '$', '~',
    ];
    const EXPANDED: [char; 2] = ['$', '`'];
    let mut quote: Option<char> = None;
    for c in command.chars() {
        match quote {
            Some('"') if EXPANDED.contains(&c) => return Some(format!(
                "'{c}' inside double quotes, which the shell expands before dstack sees the value"
            )),
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

#[test]
fn r05__nothing_under_claude_writes_the_issue_folder_by_hand() {
    let files: Vec<String> = stdout(&git(&["ls-files", "claude"]))
        .lines()
        .map(str::to_string)
        .collect();
    assert!(!files.is_empty(), "git ls-files claude found nothing");
    let texts: Vec<(String, String)> = files
        .into_iter()
        .map(|file| {
            let text = read(&file);
            (file, text)
        })
        .collect();

    // However the path is spelled: the folder itself, and every name a file gives it.
    let mut names: Vec<String> = vec![FOLDER.to_string()];
    for (_, text) in &texts {
        names.extend(aliases(text));
    }

    for (file, text) in &texts {
        if text.contains(FOLDER) {
            assert!(
                text.contains("dstack issue new"),
                "{file} names the issue folder without naming the verb that writes it"
            );
        }
        for (at, line) in text.lines().enumerate() {
            if !names.iter().any(|name| line.contains(name.as_str())) {
                continue;
            }
            let hand = WRITES
                .iter()
                .find(|command| word_in(line, command))
                .copied()
                .or_else(|| redirects(line).then_some(">"));
            assert!(
                hand.is_none(),
                "{file}:{}: '{}' writes the issue folder behind the verb's back:\n{line}",
                at + 1,
                hand.unwrap()
            );
        }
    }
}

/// The variables a text assigns the folder to, as they are spelled when used: `NAME=…folder…`
/// becomes `$NAME` and `${NAME}`.
fn aliases(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let Some(folder) = line.find(FOLDER) else {
            continue;
        };
        for (at, _) in line[..folder].match_indices('=') {
            let name: String = line[..at]
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect::<Vec<char>>()
                .into_iter()
                .rev()
                .collect();
            if !name.is_empty() {
                found.push(format!("${name}"));
                found.push(format!("${{{name}}}"));
            }
        }
    }
    found
}

/// `word` as a whole word: `cp` is a command, `cp` inside `scp` or `cp.md` is not.
fn word_in(line: &str, word: &str) -> bool {
    let part = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.';
    let mut from = 0;
    while let Some(at) = line[from..].find(word) {
        let (start, end) = (from + at, from + at + word.len());
        if !line[..start].chars().next_back().is_some_and(part)
            && !line[end..].chars().next().is_some_and(part)
        {
            return true;
        }
        from = end;
    }
    false
}

/// A redirection that creates a file (`> path`, `>> path`), told apart from the arrows and
/// placeholders that prose is full of (`->`, `=>`, `<one line>`) and from a quoted blockquote.
fn redirects(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    for (at, c) in chars.iter().enumerate() {
        if *c != '>' || at == 0 || matches!(chars[at - 1], '-' | '=') {
            continue;
        }
        let target = chars[at + 1..]
            .iter()
            .skip_while(|c| **c == '>' || **c == ' ')
            .next();
        if target.is_some_and(|c| {
            *c == '"' || *c == '\'' || *c == '$' || *c == '~' || *c == '/' || *c == '.'
        }) {
            return true;
        }
    }
    false
}

#[test]
fn r05__the_two_root_rule_files_differ_only_in_their_title() {
    let (claude, agents) = (read("CLAUDE.md"), read("AGENTS.md"));
    let (claude_title, claude_rest) = claude.split_once('\n').expect("a title line");
    let (agents_title, agents_rest) = agents.split_once('\n').expect("a title line");
    assert_ne!(claude_title, agents_title, "the twins lost their titles");
    assert_eq!(
        claude_rest, agents_rest,
        "CLAUDE.md and AGENTS.md drifted apart below the title line"
    );
}

#[test]
fn r05__doctor_passes_over_the_documents_the_rule_touched() {
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .arg("doctor")
        .current_dir(repo())
        .output()
        .expect("run dstack doctor");
    let printed = stdout(&out);
    assert!(
        printed
            .lines()
            .any(|line| line.trim_end().ends_with("unknown verbs: 0")),
        "the verb sweep did not report a clean count:\n{printed}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "dstack doctor:\n{printed}{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
