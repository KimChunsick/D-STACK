// verbs/ask.rs
// dstack ask add|answer|assume|list: the interview question ledger (R51).
//
// The rule the ledger enforces is "no question that changes no R": every row names the R ids (or
// `design`) it moves, and an assumed question must leave an R row behind, so a default the user
// never saw cannot quietly become part of the approved request.

use std::path::{Path, PathBuf};

use crate::core::args::{is_option, opt, unknown_option};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::target::{resolve_target, TargetKind};
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::request::RequestDoc;
use crate::store::tables::{
    d_append, d_next_id, q_append, q_count, q_field, q_next_id, q_set_status, q_text_ok, questions,
};

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(Error::failed(format!($($m)*))) }; }

/// The four roster entries of the noun; the struct carries nothing but its name.
macro_rules! ask_verb {
    ($handler:ident, $entry:literal, $body:ident) => {
        struct $handler;
        impl Verb for $handler {
            fn name(&self) -> &'static str {
                $entry
            }
            fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
                $body(ctx, args)
            }
        }
    };
}

ask_verb!(AskAdd, "ask add", add);
ask_verb!(AskAnswer, "ask answer", answer);
ask_verb!(AskAssume, "ask assume", assume);
ask_verb!(AskList, "ask list", list);

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(AskAdd),
        Box::new(AskAnswer),
        Box::new(AskAssume),
        Box::new(AskList),
    ]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![]
}

fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut question, mut affects) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if let Some((value, eaten)) = opt(&arg, next(&rest, i), "affects")? {
            affects = value;
            i += eaten;
        } else if is_option(&arg) {
            return Err(unknown_option(&arg));
        } else if question.is_empty() {
            question = arg;
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if question.is_empty() {
        fail!("usage: dstack ask add \"<question>\" --affects R01,R02|design")
    }
    if affects.is_empty() {
        fail!("--affects is required: a question that changes no R is not asked (R51)")
    }
    q_text_ok("question", &question)?;
    q_text_ok("--affects", &affects)?;
    affects_note(ctx, &target.dir, &affects)?;

    let file = ask_file(&target.dir);
    let id = q_next_id(&file)?;
    q_append(&file, &id, &question, &affects, "open")?;
    say!(ctx, "questions: {}", file.display());
    say!(ctx, "  {id} | {question} | {affects} | open");
    say!(
        ctx,
        "  rows {}, {}",
        questions(&file)?.len(),
        counts(&file)?
    );
    Ok(())
}

fn answer(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (qid, given, decision) = two_and_option(&rest, "decision")?;
    if qid.is_empty() || given.is_empty() {
        fail!("usage: dstack ask answer Q-NN \"<answer>\" --decision \"<one line>\"")
    }
    if decision.is_empty() {
        fail!("--decision is required: the answer only counts once it is a decision row (R51)")
    }
    q_text_ok("answer", &given)?;
    q_text_ok("--decision", &decision)?;

    // decisions.md is read before questions.md is rewritten: the id the decision row will carry
    // is the last read that can fail, and a failed read leaves both ledgers as they were (D-12).
    let did = d_next_id(&target.dir.join("decisions.md"), false)?;
    let (file, affects) = take_open(&target.dir, &qid, "answered", "answers are recorded once")?;
    let text = format!("{decision} (from {qid}: {given})");
    record(ctx, &target.dir, &qid, &did, &text, &affects, "answered")?;
    say!(ctx, "  {}", counts(&file)?);
    Ok(())
}

fn assume(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (qid, default, accept) = two_and_option(&rest, "accept")?;
    if qid.is_empty() || default.is_empty() {
        fail!("usage: dstack ask assume Q-NN \"<default>\" --accept \"<what is observed if the default is wrong>\"")
    }
    if accept.is_empty() {
        fail!("--accept is required: an assumption is only visible if its failure is observable (R51)")
    }
    q_text_ok("default", &default)?;
    q_text_ok("--accept", &accept)?;

    // Everything this verb and its `req add --assumption` child read is read before the first
    // write: the child loads request.md only after both ledgers have moved, so an unreadable
    // request would otherwise leave a question assumed and a decision row behind (D-12). A
    // request.md that is not there at all stays the child's own checked refusal, as in the shell.
    let request = target.dir.join("request.md");
    if request.is_file() {
        RequestDoc::load(&request)?;
    }
    let did = d_next_id(&target.dir.join("decisions.md"), false)?;
    let (file, affects) = take_open(
        &target.dir,
        &qid,
        "assumed",
        "an assumption replaces an unanswered question",
    )?;
    let text = format!("adopted default: {default} (from {qid})");
    record(ctx, &target.dir, &qid, &did, &text, &affects, "assumed")?;

    // The R row is what makes the assumption survive into the approval screen (R51). The shell
    // spawns `req add` with 2>&1 and prints the capture, so there is exactly one code path that
    // appends a request row; in process that is stdout followed by stderr, printed as one block
    // whose trailing newlines the command substitution dropped.
    let flag = match target.kind {
        TargetKind::Quick => "--quick",
        TargetKind::Run => "--run",
    };
    let called = ctx.call(
        "req add",
        &[
            default,
            "--accept".to_string(),
            accept,
            "--assumption".to_string(),
            "--from".to_string(),
            qid.clone(),
            flag.to_string(),
            target.id,
        ],
    );
    let block = format!("{}{}", called.stdout, called.stderr);
    ctx.out.say(block.trim_end_matches('\n'));
    // A child that could not decide keeps its own exit code and its own `dstack: cannot read …`
    // line, which the block above already carries; folding it into the refusal below would report
    // an unreadable store as a checked failure (D-12).
    if called.code == 2 {
        return Err(Error::Exit(2));
    }
    if called.code != 0 {
        fail!("req add --assumption failed for {qid} (see above)")
    }
    say!(ctx, "  {}", counts(&file)?);
    Ok(())
}

fn list(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    let file = ask_file(&target.dir);
    say!(ctx, "questions: {}", file.display());
    ctx.out.say("Q | Question | Affects | Status");
    let rows = questions(&file)?;
    for row in &rows {
        say!(
            ctx,
            "{} | {} | {} | {}",
            row.id,
            row.question,
            row.affects,
            row.status
        );
    }
    say!(ctx, "rows {}, {}", rows.len(), counts(&file)?);
    Ok(())
}

/// The option loop answer and assume share: two positional arguments and one required option.
fn two_and_option(rest: &[String], name: &str) -> Result<(String, String, String)> {
    let (mut qid, mut given, mut value) = (String::new(), String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if let Some((found, eaten)) = opt(&arg, next(rest, i), name)? {
            value = found;
            i += eaten;
        } else if is_option(&arg) {
            return Err(unknown_option(&arg));
        } else if qid.is_empty() {
            qid = arg;
            i += 1;
        } else if given.is_empty() {
            given = arg;
            i += 1;
        } else {
            return Err(Error::failed(format!("unexpected argument: {arg}")));
        }
    }
    Ok((qid, given, value))
}

/// The status of a question moves once: an unknown or already recorded question is refused, and
/// the affects column is read before the move so the decision names the same R rows.
fn take_open(dir: &Path, qid: &str, status: &str, tail: &str) -> Result<(PathBuf, String)> {
    let file = ask_file(dir);
    if !file.is_file() {
        fail!(
            "no questions.md in {} (dstack ask add \"…\" --affects …)",
            dir.display()
        )
    }
    let was = q_field(&file, qid, 4)?.unwrap_or_default();
    if was.is_empty() {
        fail!("unknown question: {qid}")
    }
    if was != "open" {
        fail!("{qid} is {was}, not open; {tail}")
    }
    let affects = q_field(&file, qid, 3)?.unwrap_or_default();
    if q_set_status(&file, qid, status).is_err() {
        fail!("could not update {qid} in {}", file.display())
    }
    Ok((file, affects))
}

/// The decision row every recorded answer leaves behind, and the four lines that report it. The
/// affects column is copied, not re-derived: a drifting one would let check decisions pass on
/// the wrong row.
fn record(
    ctx: &mut Context,
    dir: &Path,
    qid: &str,
    did: &str,
    text: &str,
    affects: &str,
    status: &str,
) -> Result<()> {
    let decisions = dir.join("decisions.md");
    d_append(&decisions, did, text, affects, status)?;
    say!(ctx, "questions: {}", ask_file(dir).display());
    say!(ctx, "  {qid} | {status} | affects {affects}");
    say!(ctx, "decisions: {}", decisions.display());
    say!(ctx, "  {did} | {text} | {affects} | {status}");
    Ok(())
}

/// Warn, never fail about the rows: a question is usually asked before the R row it will change
/// exists. A request.md that is there and cannot be read is the one failure (D-12) — warning that
/// it "names no row" would be a verdict on a file nobody read.
fn affects_note(ctx: &mut Context, dir: &Path, affects: &str) -> Result<()> {
    let request = dir.join("request.md");
    if !request.is_file() {
        return Ok(());
    }
    let doc = Some(RequestDoc::load(&request)?);
    let mut unknown = String::new();
    for token in affects_tokens(affects) {
        let known = token == "design"
            || (is_r_token(token)
                && doc
                    .as_ref()
                    .and_then(|d| d.row(token))
                    .is_some_and(|row| !row.text.is_empty()));
        if !known {
            unknown.push(' ');
            unknown.push_str(token);
        }
    }
    if !unknown.is_empty() {
        ctx.out
            .warn(&format!("affects names no row in request.md yet:{unknown}"));
    }
    Ok(())
}

/// The shell's `for tok in $(printf '%s' "$affects" | tr ',' ' ')`: tr turns the commas into
/// spaces and the default IFS then splits on ASCII space, tab and newline only. A non-breaking
/// space belongs to the token, so Rust's Unicode-aware split_whitespace would read one name as
/// two.
fn affects_tokens(affects: &str) -> Vec<&str> {
    affects
        .split([',', ' ', '\t', '\n'])
        .filter(|token| !token.is_empty())
        .collect()
}

/// The shell's `R[0-9]*` glob: an R, a digit, then anything.
fn is_r_token(token: &str) -> bool {
    let mut chars = token.chars();
    chars.next() == Some('R') && chars.next().is_some_and(|c| c.is_ascii_digit())
}

/// The tail every ask verb prints: the three statuses of the ledger.
fn counts(file: &Path) -> Result<String> {
    Ok(format!(
        "open {}, answered {}, assumed {}",
        q_count(file, "open")?,
        q_count(file, "answered")?,
        q_count(file, "assumed")?
    ))
}

fn ask_file(dir: &Path) -> PathBuf {
    dir.join("questions.md")
}

fn next(rest: &[String], i: usize) -> Option<&str> {
    rest.get(i + 1).map(String::as_str)
}
