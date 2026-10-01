// tests/r16_language_rules.rs
// R16 (rule-text half): the six documents that state the language rule — the repository's root
// twins, the two installed global instructions, the Korean output style and the shared runtime —
// carry one identical rule line: request.md holds the request, the design and the plan and
// verification in one file, parts 2 and 3 are Korean 해요체 as well, and other workflow
// artifacts stay English. The root twins stay byte-identical below their title lines.
#![allow(non_snake_case)]

use std::path::PathBuf;

const RULE: &str = "`request.md` is one file holding the request (part 1), the design (part 2) and the plan and verification (part 3); parts 2 and 3 are Korean 해요체 as well, and other workflow artifacts stay English.";

const DOCUMENTS: [&str; 6] = [
    "CLAUDE.md",
    "AGENTS.md",
    "claude/CLAUDE.md",
    "codex/AGENTS.md",
    "claude/output-styles/dstack-korean.md",
    "claude/runtime.md",
];

fn read(path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Whether a whole line, list marker aside, is the rule: a sentence that merely contains it, or
/// a wrapped copy split over two lines, is not the shared line.
fn carries_rule(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        line.strip_prefix("- ").unwrap_or(line) == RULE
    })
}

#[test]
fn R16_every_language_rule_document_carries_the_one_file_rule() {
    let missing: Vec<&str> = DOCUMENTS
        .into_iter()
        .filter(|path| !carries_rule(&read(path)))
        .collect();
    assert!(missing.is_empty(), "no shared rule line in {missing:?}");
}

#[test]
fn R16_root_twins_differ_only_in_the_title_line() {
    let (claude, agents) = (read("CLAUDE.md"), read("AGENTS.md"));
    let below = |text: &str| text.split_once('\n').map(|(_, rest)| rest.to_string());
    assert!(below(&claude).is_some(), "CLAUDE.md has no line below its title");
    assert_eq!(
        below(&claude),
        below(&agents),
        "CLAUDE.md and AGENTS.md drifted apart below the title line"
    );
}
