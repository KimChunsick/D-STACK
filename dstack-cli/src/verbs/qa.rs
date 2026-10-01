// verbs/qa.rs
// dstack qa add: record a Goal QA scenario for one usage scenario of the request, or for none,
// in the run-local QA ledger, and regenerate part 3 (R08, D-40). The text holds its preparation,
// steps and expected result under three labels.

use std::path::Path;

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{read_text, with_lock};
use crate::core::paths::base_name;
use crate::core::target::{resolve_target, TargetKind};
use crate::core::verb::Verb;
use crate::store::qa::{self, NO_SCENARIO};
use crate::store::request::RequestDoc;
use crate::store::request_sections::is_blank;
use crate::store::visible::visible;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(Error::failed(format!($($m)*))) }; }

const USAGE: &str =
    "usage: dstack qa add --scenario S<n>|none --from <file with 준비:, 단계:, 기대 결과: lines> [--run <id>]";

/// The labels that start the three parts of a QA text, in the order a refusal names them.
const PARTS: [&str; 3] = ["준비:", "단계:", "기대 결과:"];

struct QaAdd;

impl Verb for QaAdd {
    fn name(&self) -> &'static str {
        "qa add"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        add(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(QaAdd)]
}

/// Every check runs before the lock and the first write, so a refusal leaves the run as it was.
fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut scenario, mut from) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(&rest[i], next, "scenario")? {
            scenario = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(&rest[i], next, "from")? {
            from = value;
            i += eaten;
        } else if is_option(&rest[i]) {
            fail!("unknown option: {} ({USAGE})", rest[i]);
        } else {
            fail!("unexpected argument: {}", rest[i]);
        }
    }
    if scenario.is_empty() || from.is_empty() {
        fail!("{USAGE}");
    }
    if target.kind == TargetKind::Quick {
        fail!("quick tasks have no Goal QA scenarios; qa add records them for a run");
    }
    let file = target.dir.join("request.md");
    if !file.is_file() {
        fail!(
            "no request.md in {} (dstack request new --type <work_type>)",
            target.dir.display()
        );
    }
    let text = match read_text(Path::new(&from))? {
        Some(text) => text,
        None => fail!("--from file not found: {from}"),
    };
    if is_blank(&text) {
        fail!("--from file {from} is empty (or comments only); a QA scenario holds its preparation, steps and expected result");
    }
    let missing = missing_parts(&text);
    if !missing.is_empty() {
        fail!(
            "--from file {from} is missing QA parts: {} — a QA scenario holds lines starting 준비:, 단계: and 기대 결과:, each with its text",
            missing.join(", ")
        );
    }
    if scenario != NO_SCENARIO {
        let request = RequestDoc::load(&file)?;
        let known = qa::scenario_ids(request.text())
            .map_err(|e| Error::failed(format!("{}: {}", file.display(), e.message())))?;
        if !known.contains(&scenario) {
            let known = match known.is_empty() {
                true => "none".to_string(),
                false => known.join(" "),
            };
            fail!("unknown usage scenario: {scenario} (known: {known})");
        }
    }

    let run = base_name(&target.dir);
    let row = {
        let _lock = with_lock(&roots.local)?;
        qa::add(&target.dir, &run, &scenario, &text)?
    };
    let rows = qa::rows(&target.dir)?;
    let open = rows.iter().filter(|row| row.status == "open").count();
    say!(ctx, "qa add: run {run} — {} (scenario {scenario})", row.qa);
    say!(
        ctx,
        "  body {} ({} bytes, sha256 {}…)",
        row.body,
        text.len(),
        &row.body_sha256[..8]
    );
    say!(ctx, "  qa.tsv: {} scenario(s), open {open}", rows.len());
    Ok(())
}

/// Every part of a QA text a reader does not see with text, as `<label> (no line)` or
/// `<label> (no text)`. A part starts at a shown line whose text begins with its label and runs to
/// the next label, so its text may follow on the same line or below; a comment shows nothing, so a
/// label inside one starts no part.
fn missing_parts(text: &str) -> Vec<String> {
    let mut found: [Option<bool>; 3] = [None; 3];
    let mut open = None;
    for line in visible(text).lines() {
        let line = line.trim_start();
        let rest = match PARTS.iter().position(|label| line.starts_with(label)) {
            Some(at) => {
                open = Some(at);
                found[at].get_or_insert(false);
                &line[PARTS[at].len()..]
            }
            None => line,
        };
        if let (Some(at), false) = (open, rest.trim().is_empty()) {
            found[at] = Some(true);
        }
    }
    PARTS
        .iter()
        .zip(found)
        .filter_map(|(label, found)| match found {
            None => Some(format!("{label} (no line)")),
            Some(false) => Some(format!("{label} (no text)")),
            Some(true) => None,
        })
        .collect()
}
