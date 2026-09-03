// verbs/request/mod.rs
// dstack request|req|check request: the request file, its R rows and its approval (R40–R48, R51).

use std::path::PathBuf;

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::target::{Target, TargetKind};
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::request::RequestDoc;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(Error::failed(format!($($m)*))) }; }

/// One roster entry of the three nouns; the struct carries nothing but its name.
macro_rules! request_verb {
    ($handler:ident, $entry:literal, $body:path) => {
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

mod add;
mod approve;
mod check;
mod marks;
mod new;
mod open_show;
mod rowfile;
mod selftests;
mod udiff;

request_verb!(RequestNew, "request new", new::new);
request_verb!(RequestOpen, "request open", open_show::open);
request_verb!(RequestApprove, "request approve", approve::approve);
request_verb!(RequestShow, "request show", open_show::show);
request_verb!(ReqAdd, "req add", add::add);
request_verb!(ReqAccept, "req accept", marks::accept);
request_verb!(ReqSplit, "req split", marks::split);
request_verb!(ReqWithdraw, "req withdraw", marks::withdraw);
request_verb!(ReqDefer, "req defer", marks::defer);
request_verb!(ReqStatus, "req status", marks::status);
request_verb!(CheckRequest, "check request", check::check);

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(RequestNew),
        Box::new(RequestOpen),
        Box::new(RequestApprove),
        Box::new(RequestShow),
        Box::new(ReqAdd),
        Box::new(ReqAccept),
        Box::new(ReqSplit),
        Box::new(ReqWithdraw),
        Box::new(ReqDefer),
        Box::new(ReqStatus),
        Box::new(CheckRequest),
    ]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    selftests::all()
}

/// core::args::opt at one position of an argument list: `--name value`, `--name=value`, or
/// Err(Exit(1)) for the operand the shell's `shift 2` would have failed on.
fn take(args: &[String], i: usize, name: &str) -> Result<Option<(String, usize)>> {
    opt(&args[i], args.get(i + 1).map(String::as_str), name)
}

fn request_file(target: &Target) -> PathBuf {
    target.dir.join("request.md")
}

fn draft_file(target: &Target) -> PathBuf {
    target.dir.join("request.agent-draft.md")
}

fn stamp_file(target: &Target) -> PathBuf {
    target.dir.join("request.approved")
}

fn is_approved(target: &Target) -> bool {
    stamp_file(target).is_file()
}

/// _req_require_file(): the request file, or the one refusal every verb of the noun shares.
fn require_file(target: &Target) -> Result<PathBuf> {
    let file = request_file(target);
    if !file.is_file() {
        fail!(
            "no request.md in {} (dstack request new --type <work_type>)",
            target.dir.display()
        );
    }
    Ok(file)
}

fn load(target: &Target) -> Result<RequestDoc> {
    RequestDoc::load(&request_file(target))
}

/// The three counts the verbs print together: rows, live rows and pending rows.
fn counts(doc: &RequestDoc) -> (usize, usize, usize) {
    let rows = doc.rows();
    let pending = rows
        .iter()
        .filter(|row| row.markers_string().contains("status=pending-approval"))
        .count();
    (rows.len(), doc.live_ids().len(), pending)
}

/// _request_target_flags(): the --run/--quick pair that names this target to a self-call.
fn target_flags(target: &Target) -> Vec<String> {
    let flag = match target.kind {
        TargetKind::Quick => "--quick",
        TargetKind::Run => "--run",
    };
    vec![flag.to_string(), target.id.clone()]
}

/// The row line as it stands in the file, which is what every verb echoes back after an edit.
fn row_line(doc: &RequestDoc, id: &str) -> String {
    match doc.row_lineno(id) {
        Some(lineno) => rowfile::lines(doc.text())
            .get(lineno - 1)
            .unwrap_or(&"")
            .to_string(),
        None => String::new(),
    }
}
