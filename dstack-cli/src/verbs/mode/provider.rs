// Provider-specific read-only argv and successful completion normalization.
use std::fs;
use std::io::Write;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::core::error::{Error, Result};
use crate::core::mode::Provider;

#[derive(Deserialize)]
struct ClaudeResult {
    #[serde(rename = "type")]
    kind: String,
    subtype: String,
    is_error: bool,
    result: String,
}

pub fn model(provider: Provider) -> &'static str {
    match provider {
        Provider::Claude => "opus",
        Provider::Codex => "gpt-6-astra",
    }
}

pub fn command(provider: Provider, role: &str, cwd: &Path, result: &Path) -> Vec<String> {
    let web = matches!(role, "research" | "audit");
    let mut args: Vec<String> = match provider {
        Provider::Codex => [
            "codex",
            "exec",
            "--ignore-user-config",
            "-m",
            "gpt-6-astra",
            "-c",
            "model_reasoning_effort=high",
            "--sandbox",
            "read-only",
            "--json",
            "-o",
        ]
        .map(str::to_string)
        .into(),
        Provider::Claude => [
            "claude",
            "--print",
            "--model",
            "opus",
            "--effort",
            "high",
            "--output-format",
            "json",
            "--no-session-persistence",
            "--tools",
            if web {
                "Read,Glob,Grep,WebSearch,WebFetch"
            } else {
                "Read,Glob,Grep"
            },
            "--strict-mcp-config",
            "--mcp-config",
            "{\"mcpServers\":{}}",
            "--permission-mode",
            "dontAsk",
            "--permission-prompts",
            "none",
        ]
        .map(str::to_string)
        .into(),
    };
    if provider == Provider::Codex {
        args.push(result.to_string_lossy().into_owned());
        args.extend(["-C".into(), cwd.to_string_lossy().into_owned()]);
        if web {
            args.extend(["-c".into(), "tools.web_search=true".into()]);
        }
        args.push("-".into());
    }
    args
}

pub fn result(provider: Provider, dir: &Path) -> Result<String> {
    let output = text_file(&dir.join("out.txt"))?;
    let text = match provider {
        Provider::Claude => {
            // The typed object rejects missing and duplicated fields as well as extra results.
            if !output.trim_start().starts_with('{') {
                return Err(invalid("expected one Claude JSON result object"));
            }
            let value: ClaudeResult =
                serde_json::from_str(&output).map_err(|_| invalid("invalid Claude JSON result"))?;
            if value.kind != "result" || value.subtype != "success" || value.is_error {
                return Err(invalid("Claude did not return a successful result"));
            }
            value.result
        }
        Provider::Codex => {
            let mut completed = 0;
            for line in output.lines().filter(|line| !line.trim().is_empty()) {
                let value: Value =
                    serde_json::from_str(line).map_err(|_| invalid("invalid Codex JSON event"))?;
                let event = value["type"]
                    .as_str()
                    .ok_or_else(|| invalid("missing Codex event type"))?;
                if matches!(event, "error" | "turn.failed") || value["is_error"] == true {
                    return Err(invalid("Codex reported a failed turn"));
                }
                if event == "turn.completed" {
                    completed += 1;
                }
            }
            if completed != 1 {
                return Err(invalid("expected one completed Codex turn"));
            }
            text_file(&dir.join("result.txt"))?
        }
    };
    if text.trim().is_empty() {
        return Err(invalid("empty provider result"));
    }
    if provider == Provider::Claude {
        let path = dir.join("result.txt");
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|mut file| file.write_all(text.as_bytes()))
            .map_err(|e| invalid(&format!("cannot save {}: {e}", path.display())))?;
    }
    Ok(text)
}

fn text_file(path: &Path) -> Result<String> {
    let meta = fs::symlink_metadata(path)
        .map_err(|e| invalid(&format!("cannot inspect {}: {e}", path.display())))?;
    if !meta.is_file() {
        return Err(invalid(&format!(
            "result must be a regular file: {}",
            path.display()
        )));
    }
    fs::read_to_string(path).map_err(|e| invalid(&format!("cannot read {}: {e}", path.display())))
}

fn invalid(reason: &str) -> Error {
    Error::failed(format!("provider completion rejected: {reason}"))
}
