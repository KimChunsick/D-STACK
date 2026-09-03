// verbs/report/metrics.rs
// dstack report --metrics: the five numbers R01 asks a run for, plus the coverage rate.
//
// Every metric is read, never estimated: what cannot be read is written as `unavailable: <why>`
// and keeps the command from exiting 0, because a metric nobody measured is not a measurement.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::fsx::{epoch_now, utc_to_epoch};
use crate::core::meta::meta_get;
use crate::core::roots::Roots;
use crate::core::target::{Target, TargetKind};
use crate::store::cases::{self, MetricRow};
use crate::verbs::run::run_dirs;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// The four usage fields a transcript line contributes to a token sum.
const USAGE_FIELDS: [&str; 4] = [
    "input_tokens",
    "output_tokens",
    "cache_read_input_tokens",
    "cache_creation_input_tokens",
];

fn dur_human(seconds: i64) -> String {
    format!("{}h {:02}m {:02}s", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
}

/// The rounds of a review directory, as `ls review/codex-review-*.md | wc -l` counts them.
fn round_count(dir: &Path) -> usize {
    std::fs::read_dir(dir.join("review"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with("codex-review-") && name.ends_with(".md")
        })
        .count()
}

/// The five metrics R01 asks a closed run for, plus the coverage rate of the table above.
/// False when a metric could not be read, which is what keeps `--metrics` from reporting a
/// number nobody measured.
pub fn run_metrics(ctx: &mut Context, roots: &Roots, target: &Target, rate: &str) -> Result<bool> {
    let dir = &target.dir;
    let mut rows: Vec<MetricRow> = Vec::new();
    let mut add = |metric: &str, value: String, source: String| {
        rows.push(MetricRow { metric: metric.to_string(), value, source })
    };
    let runs = roots.runs.display().to_string();
    if target.kind != TargetKind::Run {
        // R01 measures a run: a quick task has no meta.tsv, so there is nothing to read rather
        // than something to guess.
        let per_run = "unavailable: metrics are recorded per run (R01)".to_string();
        add("wall-clock", format!("{per_run}; a quick task has no meta.tsv"), dir.display().to_string());
        add("main-loop-tokens", per_run.clone(), dir.display().to_string());
        add("subagent-tokens", per_run.clone(), dir.display().to_string());
        add("review-rounds", round_count(dir).to_string(), format!("{}/review/", dir.display()));
        add("concurrent-runs", per_run, runs.clone());
        add("coverage-rate", rate.to_string(), "dstack report (R79 table above)".to_string());
    } else {
        let meta = |key: &str| -> Result<String> { Ok(meta_get(dir, key)?.unwrap_or_default()) };
        let (started, closed) = (meta("started_at")?, meta("closed_at")?);
        let from = utc_to_epoch(&started);
        let table = format!("{}/meta.tsv", dir.display());
        match from {
            None => add("wall-clock", "unavailable: started_at not readable in meta.tsv".to_string(), table.clone()),
            Some(from) => {
                let (until, open) = match closed.is_empty() {
                    true => (epoch_now(), " (run still open)"),
                    false => (utc_to_epoch(&closed).unwrap_or(0), ""),
                };
                add("wall-clock", format!("{}{open}", dur_human(until - from)), table.clone());
            }
        }
        let transcript = PathBuf::from(meta("transcript_path")?);
        if meta("transcript_path")?.is_empty() {
            let why = "unavailable: transcript_path not recorded (the Stop hook writes it)";
            add("main-loop-tokens", why.to_string(), table.clone());
            add("subagent-tokens", why.to_string(), table);
        } else if !transcript.is_file() {
            let why = format!("unavailable: transcript file is gone: {}", transcript.display());
            let source = transcript.display().to_string();
            add("main-loop-tokens", why.clone(), source.clone());
            add("subagent-tokens", why, source);
        } else {
            add("main-loop-tokens", usage_sum(&[transcript.clone()]).to_string(), transcript.display().to_string());
            let session = transcript.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            let sub = transcript.parent().unwrap_or(Path::new("")).join(session).join("subagents");
            let mut found: Vec<PathBuf> = Vec::new();
            let present = sub.is_dir();
            if present {
                collect_jsonl(&sub, &mut found);
            }
            let (value, note) = match (present, found.len()) {
                (false, _) => ("0".to_string(), " (absent: this run delegated to no subagent)".to_string()),
                (true, 0) => ("0".to_string(), " (no transcripts)".to_string()),
                (true, n) => (usage_sum(&found).to_string(), format!(" ({n} file(s))")),
            };
            add("subagent-tokens", value, format!("{}{note}", sub.display()));
        }
        add("review-rounds", round_count(dir).to_string(), format!("{}/review/", dir.display()));
        // Overlap, not "open now": two runs that never ran at the same time were never concurrent.
        let (mine_from, mine_until) = (
            from.unwrap_or(0),
            match closed.is_empty() {
                true => epoch_now(),
                false => utc_to_epoch(&closed).unwrap_or(0),
            },
        );
        let mut concurrent = 1;
        for other in run_dirs(roots) {
            if !other.join("meta.tsv").is_file() || other.file_name().unwrap_or_default().to_string_lossy() == target.id {
                continue;
            }
            let started = match utc_to_epoch(&meta_get(&other, "started_at")?.unwrap_or_default()) {
                Some(started) => started,
                None => continue,
            };
            let ended = match meta_get(&other, "closed_at")?.unwrap_or_default() {
                stamp if stamp.is_empty() => epoch_now(),
                stamp => utc_to_epoch(&stamp).unwrap_or_else(epoch_now),
            };
            if started <= mine_until && mine_from <= ended {
                concurrent += 1;
            }
        }
        add("concurrent-runs", concurrent.to_string(), format!("{runs} (intervals overlapping this run, including it)"));
        add("coverage-rate", rate.to_string(), "dstack report (R79 table above)".to_string());
    }
    let unavailable = rows.iter().filter(|row| row.value.starts_with("unavailable:")).count();
    ctx.out.say("| metric | value | source |");
    ctx.out.say("|---|---|---|");
    for row in &rows {
        say!(ctx, "| {} | {} | {} |", row.metric, row.value, row.source);
    }
    cases::metrics_write(dir, &rows)?;
    say!(ctx, "metrics: {} rows, unavailable {unavailable} → {}/metrics.tsv", rows.len(), dir.display());
    Ok(unavailable == 0)
}

/// `find <dir> -type f -name '*.jsonl'`, which walks the subtree and follows no symlink.
///
/// find without -L reads every path with lstat: a symlinked directory is neither entered nor
/// listed, a symlink to a .jsonl file is type l and not type f, and a symlink given as the start
/// point is printed but never descended. `test -d` above does follow, which is why presence is
/// asked there and the files are counted here: a subagents directory that is a symlink reads as
/// present and holds nothing find would list. Following any of them would let a link planted in
/// a transcript directory pull an unrelated tree — or a cycle — into the sum.
fn collect_jsonl(dir: &Path, found: &mut Vec<PathBuf>) {
    match std::fs::symlink_metadata(dir) {
        Ok(meta) if !meta.file_type().is_symlink() => (),
        _ => return,
    }
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(_) => continue,
        };
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            collect_jsonl(&entry.path(), found);
        } else if kind.is_file() && entry.file_name().to_string_lossy().ends_with(".jsonl") {
            found.push(entry.path());
        }
    }
}

/// The usage sum over JSONL transcripts. A line without `.message.usage` contributes 0; a line
/// that is not JSON at all, or whose fields are not numbers, is skipped rather than fatal,
/// because a transcript is written by another process and may be mid-write. The shell says the
/// same thing as `jq … try … catch empty`, so this tolerance is the ported behaviour and not the
/// erasure D-12 forbids: a transcript is not a store file.
fn usage_sum(paths: &[PathBuf]) -> i64 {
    let mut total = 0f64;
    for path in paths {
        // A transcript is a log that grows for as long as the session runs, so it is read one
        // line at a time and never held whole in memory.
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => continue,
        };
        for line in BufReader::new(file).lines() {
            // Bytes that are not UTF-8 end that line the way a line that is not JSON ends: jq
            // drops it and reads on, so one damaged line never costs the rest of the file.
            let line = match line {
                Ok(line) => line,
                Err(_) => continue,
            };
            total += line_usage(&line).unwrap_or(0.0);
        }
    }
    total as i64
}

fn line_usage(line: &str) -> Option<f64> {
    let value: Value = serde_json::from_str(line).ok()?;
    let usage = member(&member(&value, "message")?, "usage")?;
    let mut sum = 0f64;
    for field in USAGE_FIELDS {
        sum += match member(&usage, field)? {
            Value::Null => 0.0,
            Value::Number(number) => number.as_f64()?,
            _ => return None,
        };
    }
    Some(sum)
}

/// jq's `.key`: null indexes to null, an object to its member, and anything else is the error
/// the shell's `catch empty` drops the whole line on.
fn member(value: &Value, key: &str) -> Option<Value> {
    match value {
        Value::Null => Some(Value::Null),
        Value::Object(map) => Some(map.get(key).cloned().unwrap_or(Value::Null)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r01_a_transcript_line_sums_only_what_jq_would_sum() {
        assert_eq!(line_usage(r#"{"message":{"usage":{"input_tokens":2,"output_tokens":3}}}"#), Some(5.0));
        assert_eq!(line_usage(r#"{"type":"user"}"#), Some(0.0));
        assert_eq!(line_usage("not json"), None);
        assert_eq!(line_usage(r#"{"message":"a string"}"#), None);
        assert_eq!(line_usage(r#"{"message":{"usage":{"input_tokens":"two"}}}"#), None);
    }

    #[test]
    fn r01_a_duration_reads_as_the_shell_prints_it() {
        assert_eq!(dur_human(3723), "1h 02m 03s");
        assert_eq!(dur_human(0), "0h 00m 00s");
    }
}
