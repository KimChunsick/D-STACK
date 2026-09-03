// verbs/request/add.rs
// dstack req add: mint the next R row, its markers and the decision an assumption leaves (R42, R45, R48, R51).

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::paths::{fmt_rid, parse_rid};
use crate::core::target::{resolve_target, Target};
use crate::store::request::{req_text_ok, RequestDoc};
use crate::store::rows::REQ_SEP;
use crate::store::tables::{d_append, d_has_for_q, d_next_id, q_field, q_set_status};

use super::{counts, is_approved, load, require_file, rowfile, take, target_flags};

pub fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut text, mut accept, mut want_id, mut qid) =
        (String::new(), String::new(), String::new(), String::new());
    let (mut from_answer, mut assumption) = (false, false);
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if arg == "--from-answer" {
            from_answer = true;
            i += 1;
        } else if arg == "--assumption" {
            assumption = true;
            i += 1;
        } else if let Some((value, eaten)) = take(&rest, i, "accept")? {
            accept = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "id")? {
            want_id = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "from")? {
            qid = value;
            i += eaten;
        } else if arg.starts_with('-') {
            fail!("unknown option: {arg}");
        } else if text.is_empty() {
            text = arg;
            i += 1;
        } else {
            fail!("unexpected argument: {arg}");
        }
    }
    let file = require_file(&target)?;
    if text.is_empty() {
        fail!("usage: dstack req add \"<one line>\" --accept \"<observable criterion>\" [--id R<NN>] [--from-answer] [--assumption --from Q-NN]");
    }
    req_text_ok("row text", &text)?;
    if accept.is_empty() {
        // R45: the free-text answer becomes a row immediately, even before anyone can say how it
        // will be observed. The row is unmistakably incomplete until `req accept` fills it.
        if !from_answer {
            fail!("--accept is required (or use --from-answer, which parks the criterion as \"pending: agent to propose\")");
        }
        accept = "pending: agent to propose".to_string();
    } else {
        req_text_ok("--accept", &accept)?;
    }
    if assumption && qid.is_empty() {
        fail!("--assumption needs --from Q-NN (the question whose default this row records)");
    }
    if !assumption && !qid.is_empty() {
        fail!("--from Q-NN is only meaningful with --assumption");
    }

    let doc = load(&target)?;
    let id = mint(&doc, &want_id)?;
    if doc.row_lineno(&id).is_some() {
        fail!("row {id} already exists");
    }

    // Every read that can refuse happens before the first write: the question has to exist and
    // to be open or already assumed before its status, the request row and the decision row are
    // written, because the three files have no transaction between them.
    let question = match assumption {
        true => Some(read_question(&target, &qid)?),
        false => None,
    };
    let mut markers = String::new();
    if assumption {
        markers = format!("{REQ_SEP}from: {qid}");
    }
    // R48: appending to an already approved request must not silently change what was approved.
    // The row is visible but marked, `check request` fails while it stands, and only a fresh
    // `request approve` clears it and re-stamps the hash.
    let pending = is_approved(&target);
    if pending {
        markers.push_str(&format!("{REQ_SEP}status: pending-approval"));
    }
    let row = format!("- [ ] **{id}** {text}{REQ_SEP}accept: {accept}{markers}");

    // The undo is planned before the first write, so every file it may put back has been read
    // while it was still whole.
    let undo = Undo::plan(&target, &doc, &question)?;
    let written = write_all(&target, &doc, &row, &question, &text, &id);
    let (note, (rows, live, pend)) = match written {
        Ok(written) => written,
        Err(error) => {
            undo.revert();
            return Err(error);
        }
    };
    say!(ctx, "request: {}", file.display());
    say!(ctx, "  {row}");
    if !note.is_empty() {
        say!(ctx, "  {note}");
    }
    say!(ctx, "  rows {rows}, live {live}, pending {pend}");
    if pending {
        say!(ctx, "  marked pending-approval: dstack request approve {} re-stamps the hash and syncs the ledger",
             target_flags(&target).join(" "));
    }
    Ok(())
}

/// D-08: an explicit id is allowed so a request can carry an external numbering, but only
/// forwards; gaps are fine, renumbering never is.
fn mint(doc: &RequestDoc, want_id: &str) -> Result<String> {
    let highest = doc
        .rows()
        .iter()
        .filter_map(|row| parse_rid(&row.id))
        .max()
        .unwrap_or(0);
    if want_id.is_empty() {
        return Ok(fmt_rid(highest + 1));
    }
    // The shell lets `RR7` through this gate and then dies inside `$((10#RR7))` with a bash
    // arithmetic error; that defect is not reproduced (D-09), the shape is refused here.
    let number = match parse_rid(want_id) {
        Some(number) => number,
        None => fail!("--id must look like R07 (got '{want_id}')"),
    };
    if number <= highest {
        fail!("--id {want_id} is not greater than the highest existing id {}; ids are never reused or renumbered (R42)", fmt_rid(highest));
    }
    Ok(fmt_rid(number))
}

/// The question the assumption adopts, read before anything is written: its status has to allow
/// the adoption, and its affects column becomes the decision row's.
fn read_question(target: &Target, qid: &str) -> Result<Question> {
    let file = target.dir.join("questions.md");
    if !file.is_file() {
        fail!(
            "no questions.md in {} (dstack ask add \"…\" --affects …)",
            target.dir.display()
        );
    }
    let status = match q_field(&file, qid, 4)? {
        Some(status) if !status.is_empty() => status,
        _ => fail!("unknown question: {qid}"),
    };
    match status.as_str() {
        "open" | "assumed" => {}
        other => fail!("{qid} is {other}; only an open or already assumed question turns into an assumption row"),
    }
    let affects = q_field(&file, qid, 3)?.unwrap_or_default();
    Ok(Question {
        file,
        id: qid.to_string(),
        was_open: status == "open",
        affects,
    })
}

/// The three writes, in the shell's order: the question's status, the request row, the decision
/// row. Each one records how to take itself back, so a failure in the middle leaves the three
/// files as they were instead of half an assumption.
fn write_all(
    target: &Target,
    doc: &RequestDoc,
    row: &str,
    question: &Option<Question>,
    text: &str,
    id: &str,
) -> Result<(String, (usize, usize, usize))> {
    if let Some(question) = question {
        if question.was_open {
            q_set_status(&question.file, &question.id, "assumed").map_err(|_| {
                Error::failed(format!(
                    "could not mark {} assumed in {}",
                    question.id,
                    question.file.display()
                ))
            })?;
        }
    }
    let with_row = rowfile::insert_after(doc.text(), rowfile::last_row_lineno(doc.text()), row);
    rowfile::write(&doc.path, &with_row)?;

    let note = match question {
        Some(question) => decision_note(target, question, text, id)?,
        None => String::new(),
    };
    let doc = load(target)?;
    Ok((note, counts(&doc)))
}

/// What `req add --assumption` is about to write, and how to take it back. Reverting is
/// best-effort: it restores the bytes each file had, and a failure inside it changes nothing
/// about the error the caller reports.
#[derive(Default)]
struct Undo {
    question: Option<(PathBuf, String)>,
    files: Vec<(PathBuf, Option<String>)>,
}

impl Undo {
    /// Every file the verb may touch, read before the first write. A file that is not there is
    /// None and gets unlinked by a revert; a file that exists and cannot be read stops the verb
    /// here (D-12), because treating it as absent would let a revert delete it.
    fn plan(target: &Target, doc: &RequestDoc, question: &Option<Question>) -> Result<Undo> {
        let mut undo = Undo::default();
        undo.files
            .push((doc.path.clone(), Some(doc.text().to_string())));
        if let Some(question) = question {
            if question.was_open {
                undo.question = Some((question.file.clone(), question.id.clone()));
            }
            let decisions = target.dir.join("decisions.md");
            let before = snapshot(&decisions)?;
            undo.files.push((decisions, before));
        }
        Ok(undo)
    }

    fn revert(&self) {
        if let Some((file, id)) = &self.question {
            let _ = q_set_status(file, id, "open");
        }
        for (file, before) in self.files.iter().rev() {
            match before {
                Some(text) => {
                    let _ = rowfile::write(file, text);
                }
                None => {
                    let _ = std::fs::remove_file(file);
                }
            }
        }
    }
}

/// The question row an assumption adopts.
struct Question {
    file: PathBuf,
    id: String,
    was_open: bool,
    affects: String,
}

/// The assumption's decision row. `ask assume` writes it first and this skips the duplicate, so
/// either entry point leaves exactly one D row per adopted default.
fn decision_note(target: &Target, question: &Question, text: &str, id: &str) -> Result<String> {
    let file = target.dir.join("decisions.md");
    let qid = &question.id;
    if let Some(existing) = d_has_for_q(&file, qid)? {
        return Ok(format!("decision: {existing} (already recorded for {qid})"));
    }
    let decision = format!("adopted default: {text} (from {qid})");
    let affects = match question.affects.is_empty() {
        true => id,
        false => &question.affects,
    };
    let did = d_next_id(&file, false)?;
    d_append(&file, &did, &decision, affects, "assumed")?;
    Ok(format!(
        "decision: {did} | {decision} | {affects} | assumed"
    ))
}

/// The bytes a file carries before the first write. Only ErrorKind::NotFound is None: any other
/// read error of a file that is there is a store file the verb cannot parse (D-12).
fn snapshot(file: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(file) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::cannot_decide(format!(
            "cannot read {}: {error}",
            file.display()
        ))),
    }
}
