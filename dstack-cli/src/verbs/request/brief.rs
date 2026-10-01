// verbs/request/brief.rs
// dstack request brief: the 한눈에 보기 section of an unapproved Goal request, generated (R14).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::read_text;
use crate::core::target::{resolve_target, TargetKind};
use crate::store::request::RequestDoc;
use crate::store::request_sections::{body, check_prose, replace, Place};
use crate::store::tables::{decisions, Decision};

use super::design_gate::{dec_file, SKIP_PREFIX};
use super::{is_approved, load, require_file, rowfile};

/// The section this verb owns; every other byte of request.md is kept.
const BRIEF: Place = Place::Section("한눈에 보기");

const NOTE: &str = "<!-- dstack request brief가 decisions.md, recon.md, 비목표 절, R 행과 모듈의 대응 절에서 모아 만든 절이에요. 고칠 때는 원본을 바꾼 뒤 다시 만들어요. -->";

/// What an empty group shows, so a reader tells "nothing to judge" from "not generated".
const NONE: &str = "없음";

/// Whitespace as store::request_sections counts it (twin of its `BLANK`); U+00A0 is text.
const BLANK: [char; 4] = [' ', '\t', '\n', '\r'];

pub fn brief(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    if let Some(arg) = rest.first() {
        match arg.starts_with('-') {
            true => fail!("unknown option: {arg}"),
            false => fail!("unexpected argument: {arg}"),
        }
    }
    if target.kind == TargetKind::Quick {
        fail!("a quick request has no 한눈에 보기 section; request brief serves Goal runs only");
    }
    let file = require_file(&target)?;
    // The approval hash covers this section: a rewrite after the stamp would be a change nobody
    // approved, so the brief is generated only before approval.
    if is_approved(&target) {
        fail!(
            "{} is approved; request brief writes only before approval",
            file.display()
        );
    }
    let doc = load(&target)?;
    let content = render(&target.dir, &doc)?;
    let failed = |e: Error| Error::failed(format!("{}: {}", file.display(), e.message()));
    check_prose(&content, &BRIEF).map_err(failed)?;
    let text = replace(doc.text(), &BRIEF, &content).map_err(failed)?;
    rowfile::write(&file, &text)?;
    say!(ctx, "request: {}", file.display());
    ctx.out.say("  wrote section ## 한눈에 보기");
    for line in content.lines() {
        ctx.out.say(line);
    }
    Ok(())
}

/// The section body: the note, then the four groups in their fixed order.
fn render(dir: &Path, doc: &RequestDoc) -> Result<String> {
    let rows = decisions(&dec_file(dir))?;
    let assumed: Vec<String> = rows
        .iter()
        .filter(|row| row.status == "assumed")
        .map(|row| match question(row, doc) {
            Some(q) => format!("- {} ({q}): {}", row.id, row.text),
            None => format!("- {}: {}", row.id, row.text),
        })
        .collect();
    let design: Vec<String> = rows
        .iter()
        .filter_map(|row| match row.text.strip_prefix(SKIP_PREFIX) {
            Some(reason) => Some(format!("- {} 설계를 건너뛴 사유: {}", row.id, reason.trim())),
            None if row.id.starts_with("D-DESIGN-") => Some(format!("- {}: {}", row.id, row.text)),
            None => None,
        })
        .map(|line| line.trim_end().to_string())
        .collect();
    let non_goals = section(doc.text(), "비목표")?;
    let non_goals: Vec<String> = match visible(non_goals).trim_matches(BLANK).is_empty() {
        true => Vec::new(),
        false => as_written(non_goals),
    };
    let mut files = Vec::new();
    if let Some(recon) = read_text(&dir.join("recon.md"))? {
        for line in blast_radius(&recon) {
            add_spans(&mut files, line);
        }
    }
    for line in visible(section(doc.text(), "R 행과 모듈의 대응")?).lines() {
        add_spans(&mut files, line);
    }
    let files: Vec<String> = files.iter().map(|path| format!("- `{path}`")).collect();
    let mut out = format!("{NOTE}\n");
    for (label, lines) in [
        ("대신 정한 가정", assumed),
        ("설계 선택지와 버린 대안", design),
        ("비목표", non_goals),
        ("영향 파일", files),
    ] {
        out.push_str(&format!("\n**{label}**\n\n"));
        match lines.is_empty() {
            true => out.push_str(NONE),
            false => out.push_str(&lines.join("\n")),
        }
        out.push('\n');
    }
    Ok(out)
}

/// The Q id an assumed decision stems from: the one its text names (`ask assume` writes
/// `(from Q-NN)`), else the `from:` marker of an R row it affects.
fn question(row: &Decision, doc: &RequestDoc) -> Option<String> {
    q_id(&row.text).map(str::to_string).or_else(|| {
        row.affects
            .split(',')
            .filter_map(|id| doc.row(id.trim())?.marker("from"))
            .find(|from| q_id(from) == Some(from.as_str()))
    })
}

/// The first `Q-<digits>` token of a text, standing on its own.
fn q_id(text: &str) -> Option<&str> {
    let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_');
    text.match_indices("Q-").find_map(|(at, _)| {
        let digits = text[at + 2..].bytes().take_while(u8::is_ascii_digit).count();
        let end = at + 2 + digits;
        let alone = !word(text[..at].chars().next_back()) && !word(text[end..].chars().next());
        (digits > 0 && alone).then(|| &text[at..end])
    })
}

/// A prose section's body, or nothing when the request has no such heading. A heading that is
/// there but cannot be read for certain (repeated, hidden) refuses, as the writer would.
fn section<'a>(text: &'a str, heading: &'static str) -> Result<&'a str> {
    let present = text.lines().any(|line| {
        line.strip_prefix("##")
            .filter(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
            .is_some_and(|rest| rest.trim_matches([' ', '\t']) == heading)
    });
    match present {
        true => body(text, &Place::Section(heading)),
        false => Ok(""),
    }
}

/// What a renderer shows of a body: its HTML comments left out as store::request_sections reads
/// them (twin of its `comment_end`). A `<!--` runs to the next `-->` looked for from its own
/// `--`, so `<!-->` and `<!--->` close themselves; an unclosed comment runs to the end.
fn visible(body: &str) -> String {
    let mut kept = String::new();
    let mut rest = body;
    while let Some(at) = rest.find("<!--") {
        kept.push_str(&rest[..at]);
        rest = rest[at + 2..].split_once("-->").map_or("", |(_, next)| next);
    }
    kept.push_str(rest);
    kept
}

/// The lines of a body as written, comments included, less its leading and trailing blank lines.
fn as_written(body: &str) -> Vec<String> {
    let lines: Vec<&str> = body.lines().collect();
    let blank = |line: &&str| line.trim_matches(BLANK).is_empty();
    let start = lines.iter().position(|line| !blank(line)).unwrap_or(lines.len());
    let end = lines.iter().rposition(|line| !blank(line)).map_or(start, |at| at + 1);
    lines[start..end].iter().map(|line| line.to_string()).collect()
}

/// The table lines of recon.md's `## Blast radius` section. recon.md is English working notes,
/// so it is read leniently: the section runs to the next `#` or `##` heading, and nothing in it
/// refuses the brief.
fn blast_radius(recon: &str) -> Vec<&str> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in recon.lines() {
        let level = line.bytes().take_while(|b| *b == b'#').count();
        let rest = &line[level..];
        if (1..=2).contains(&level) && (rest.is_empty() || rest.starts_with([' ', '\t'])) {
            inside = level == 2 && rest.trim_matches([' ', '\t']) == "Blast radius";
        } else if inside && line.trim_start_matches(' ').starts_with('|') {
            lines.push(line);
        }
    }
    lines
}

/// Every backticked span of one line, exactly as written, that holds no whitespace and is not yet
/// in `files`, in the order found. Nothing is guessed: an identifier is listed as a path is.
fn add_spans(files: &mut Vec<String>, line: &str) {
    let spans: Vec<&str> = line.split('`').collect();
    for span in spans.iter().skip(1).take(spans.len().saturating_sub(2)).step_by(2) {
        let named = !span.is_empty() && !span.contains(char::is_whitespace);
        if named && !files.iter().any(|seen| seen == span) {
            files.push(span.to_string());
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn R14_spans_are_listed_as_written_without_guessing() {
        let mut files = Vec::new();
        add_spans(&mut files, "| R01 | `a/b.rs:3-9`; `Procfile`; `cargo test`; ` x `; ``; `R01`; `--run` | `open");
        add_spans(&mut files, "`Procfile` and `c/d`");
        assert_eq!(files, ["a/b.rs:3-9", "Procfile", "R01", "--run", "c/d"]);
    }

    #[test]
    fn R14_q_ids_stand_on_their_own() {
        assert_eq!(q_id("adopted default: x (from Q-07)"), Some("Q-07"));
        assert_eq!(q_id("FAQ-1 and Q-"), None);
        assert_eq!(q_id("Q-12x"), None);
    }

    #[test]
    fn R14_visible_text_reads_comments_as_the_section_reader_does() {
        assert_eq!(visible("1. A\n\n<!-->\n\n2. B"), "1. A\n\n\n\n2. B");
        assert_eq!(visible("<!---> a <!-- b\nc --> d"), " a  d");
        assert_eq!(visible("<!-- 안내예요. -->\n<!--> \n").trim_matches(BLANK), "");
        assert_eq!(visible("e <!-- open"), "e ");
    }

    #[test]
    fn R14_as_written_trims_only_the_outer_blank_lines() {
        let body = "\n \n1. one\n<!-- a -->\n\n\n---\t\n\t\n";
        assert_eq!(as_written(body), ["1. one", "<!-- a -->", "", "", "---\t"]);
        assert!(as_written("\n \n").is_empty());
    }
}
