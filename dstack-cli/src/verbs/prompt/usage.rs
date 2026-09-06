// Only completion summaries count: Claude assistant/stream events overlap its result totals.
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::core::context::Context;
use crate::core::error::{Error, Result};

#[derive(Default)]
struct Totals {
    samples: u64,
    input: u64,
    read: u64,
    write: u64,
    unknown_writes: bool,
    output: u64,
}

pub fn run(ctx: &mut Context, args: &[String]) -> Result<()> {
    if args.len() < 3 || args[0] != "--provider" {
        return Err(Error::failed(
            "usage: dstack prompt usage --provider codex|claude <jsonl-file>...",
        ));
    }
    let files: Vec<PathBuf> = args[2..].iter().map(PathBuf::from).collect();
    let report = summarize(&args[1], &files)?;
    ctx.out.say(&report.to_string());
    Ok(())
}

pub fn summarize(provider: &str, files: &[PathBuf]) -> Result<Value> {
    if !matches!(provider, "codex" | "claude") {
        return Err(Error::failed("usage provider must be codex or claude"));
    }
    let mut totals = Totals::default();
    let mut seen = HashSet::new();
    for path in files {
        let canonical = path
            .canonicalize()
            .map_err(|e| unavailable(path, &e.to_string()))?;
        if !seen.insert(canonical) {
            return Err(unavailable(path, "duplicate input file"));
        }
        let reader =
            BufReader::new(File::open(path).map_err(|e| unavailable(path, &e.to_string()))?);
        let before = totals.samples;
        for (line_no, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| unavailable(path, &e.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }
            let event: Value = serde_json::from_str(&line)
                .map_err(|_| unavailable(path, &format!("invalid JSON at line {}", line_no + 1)))?;
            let expected = if provider == "codex" {
                "turn.completed"
            } else {
                "result"
            };
            if event["type"] != expected {
                continue;
            }
            if provider == "claude" && totals.samples > before {
                return Err(unavailable(
                    path,
                    "multiple Claude results; supply one invocation per file",
                ));
            }
            add_usage(&mut totals, provider, &event["usage"]).map_err(|e| {
                unavailable(path, &format!("line {}: {}", line_no + 1, e.message()))
            })?;
        }
        if totals.samples == before {
            return Err(unavailable(
                path,
                "no supported completion usage; enable CLI JSON output",
            ));
        }
    }
    if totals.samples == 0 {
        return Err(Error::cannot_decide("skipped: no usage samples"));
    }
    let ratio = if totals.input == 0 {
        Value::Null
    } else {
        json!(totals.read as f64 / totals.input as f64)
    };
    Ok(json!({
        "status": "measured", "provider": provider, "samples": totals.samples,
        "input_tokens": totals.input, "cache_read_tokens": totals.read,
        "cache_write_tokens": if totals.unknown_writes { Value::Null } else { json!(totals.write) },
        "output_tokens": totals.output, "cache_read_ratio": ratio,
        "scope": if provider == "claude" {
            "Claude result.usage: main agent only, excludes subagents; token-weighted"
        } else { "Codex completed turns; token-weighted, not request hit rate" }
    }))
}

fn add_usage(t: &mut Totals, provider: &str, usage: &Value) -> Result<()> {
    let input = number(usage, "input_tokens")?;
    let output = number(usage, "output_tokens")?;
    let (input, read, write) = if provider == "codex" {
        let read = number(usage, "cached_input_tokens")?;
        let write = match usage.get("cache_write_input_tokens") {
            None => None,
            Some(_) => Some(number(usage, "cache_write_input_tokens")?),
        };
        if read > input || write.is_some_and(|w| w > input - read) {
            return Err(Error::cannot_decide(
                "cache token counts exceed input tokens",
            ));
        }
        (input, read, write)
    } else {
        let read = number(usage, "cache_read_input_tokens")?;
        let write = number(usage, "cache_creation_input_tokens")?;
        (sum(sum(input, read)?, write)?, read, Some(write))
    };
    t.input = sum(t.input, input)?;
    t.read = sum(t.read, read)?;
    t.output = sum(t.output, output)?;
    t.samples = sum(t.samples, 1)?;
    match write {
        Some(n) => t.write = sum(t.write, n)?,
        None => t.unknown_writes = true,
    }
    Ok(())
}

fn number(value: &Value, key: &str) -> Result<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::cannot_decide(format!("missing or invalid {key}")))
}
fn sum(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or_else(|| Error::cannot_decide("usage token count overflow"))
}
fn unavailable(path: &Path, why: &str) -> Error {
    Error::cannot_decide(format!(
        "skipped: cache usage unavailable for {}: {why}",
        path.display()
    ))
}

/// Telemetry must never replace a child command's exit status or fail an otherwise valid run.
pub fn capture(ctx: &mut Context, command: &[String], dir: &Path) {
    let provider = Path::new(&command[0])
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !matches!(provider, "codex" | "claude") {
        return;
    }
    let report = match summarize(provider, &[dir.join("out.txt")]) {
        Ok(report) => report,
        Err(error) => json!({"status": "skipped", "provider": provider, "reason": error.message()}),
    };
    if let Err(error) = std::fs::write(dir.join("usage.json"), format!("{report}\n")) {
        ctx.out
            .warn(&format!("skipped: could not save cache usage: {error}"));
    } else {
        ctx.out.say(&format!(
            "cache usage: {} ({})",
            dir.join("usage.json").display(),
            report["status"]
        ));
    }
}
