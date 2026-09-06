// Provider shapes and visible evidence only. Hidden reasoning is never decoded or rendered.
use super::identity::Identity;
use crate::core::error::{Error, Result};
use crate::core::mode::Provider;
use crate::handoff::types::HistoryRecord;
use serde_json::Value;
use std::collections::VecDeque;
use std::path::Path;

const MAX_RECORD: usize = 8 * 1024;
const MAX_OUTPUT: usize = 128 * 1024;
const MAX_RECORDS: usize = 200;

pub(super) struct Decoder<'a> {
    provider: Provider,
    session: &'a str,
    cwd: Identity<'a>,
    identity: bool,
    excluded: bool,
    records: VecDeque<(HistoryRecord, usize)>,
    bytes: usize,
    omitted: usize,
    truncated: usize,
}

impl<'a> Decoder<'a> {
    pub(super) fn new(provider: Provider, session: &'a str, worktree: &'a Path) -> Self {
        Self {
            provider,
            session,
            cwd: Identity::new(worktree),
            identity: false,
            excluded: false,
            records: VecDeque::new(),
            bytes: 0,
            omitted: 0,
            truncated: 0,
        }
    }

    pub(super) fn consume(&mut self, value: &Value, line: usize) -> Result<()> {
        let kind = string(value, "type", line)?;
        match self.provider {
            Provider::Claude => self.claude(value, kind, line),
            Provider::Codex => self.codex(value, kind, line),
        }
    }

    fn identity(&mut self, value: &Value, key: &str, line: usize, required: bool) -> Result<()> {
        if required || value.get(key).is_some() {
            if string(value, key, line)? != self.session {
                return Err(bad(line, "history session identity mismatch"));
            }
        }
        if required || value.get("cwd").is_some() {
            let cwd = string(value, "cwd", line)?;
            self.cwd.check(cwd, line)?;
        }
        if value.get(key).is_some() && value.get("cwd").is_some() {
            self.identity = true;
        }
        Ok(())
    }

    fn claude(&mut self, value: &Value, kind: &str, line: usize) -> Result<()> {
        if matches!(kind, "session_meta" | "response_item" | "event_msg" | "turn_context") {
            return Err(bad(line, "history provider mismatch: expected Claude"));
        }
        self.identity(value, "sessionId", line, matches!(kind, "user" | "assistant"))?;
        match kind {
            "user" | "assistant" => {
                let message = &value["message"];
                if string(message, "role", line)? != kind {
                    return Err(bad(line, "Claude message role mismatch"));
                }
                self.message(&message["content"], kind, line)
            }
            "file-history-snapshot"
            | "file-history-delta"
            | "relocated"
            | "worktree-state"
            | "mode"
            | "permission-mode"
            | "bridge-session"
            | "attachment"
            | "ai-title"
            | "atis-latch"
            | "cost-state"
            | "summary"
            | "system"
            | "progress"
            | "queue-operation"
            | "last-prompt"
            | "custom-title"
            | "agent-name"
            | "tag"
            | "pr-link" => Ok(()),
            _ => Err(bad(line, "unsupported Claude history record format")),
        }
    }

    fn codex(&mut self, value: &Value, kind: &str, line: usize) -> Result<()> {
        if value.get("sessionId").is_some()
            || matches!(kind, "user" | "assistant" | "file-history-snapshot")
        {
            return Err(bad(line, "history provider mismatch: expected Codex"));
        }
        let payload = &value["payload"];
        if kind == "session_meta" {
            if self.identity {
                return Err(bad(line, "ambiguous duplicate Codex session_meta"));
            }
            return self.identity(payload, "id", line, true);
        }
        if !self.identity {
            return Err(bad(line, "Codex history requires a session_meta header"));
        }
        // Later turn contexts can establish a cwd change; they cannot silently change ownership.
        if payload.get("cwd").is_some() || payload.get("session_id").is_some() {
            self.identity(payload, "session_id", line, false)?;
        }
        match kind {
            "event_msg"
            | "turn_context"
            | "compacted"
            | "world_state"
            | "token_usage_record"
            | "inter_agent_communication_metadata" => return Ok(()),
            "response_item" => (),
            _ => return Err(bad(line, "unsupported Codex history record format")),
        }
        if payload["channel"] == "analysis" || value["channel"] == "analysis" {
            self.excluded = true;
            return Ok(());
        }
        match string(payload, "type", line)? {
            "message" => match string(payload, "role", line)? {
                role @ ("user" | "assistant") => self.message(&payload["content"], role, line),
                _ => {
                    self.excluded = true;
                    Ok(())
                }
            },
            "function_call" | "custom_tool_call" => {
                let name = string(payload, "name", line)?;
                let id = string(payload, "call_id", line)?;
                let field = if payload["type"] == "function_call" { "arguments" } else { "input" };
                let input = text_field(payload, field, line)?;
                self.push("tool", format!("tool call {name} ({id}): {input}"), line)
            }
            "function_call_output" | "custom_tool_call_output" => {
                let id = string(payload, "call_id", line)?;
                let output = self.visible_text(&payload["output"], line)?;
                self.push("tool", format!("tool result ({id}): {output}"), line)
            }
            _ => {
                self.excluded = true;
                Ok(())
            }
        }
    }

    fn message(&mut self, content: &Value, role: &str, line: usize) -> Result<()> {
        if let Some(text) = content.as_str() {
            return self.push(role, text.to_owned(), line);
        }
        let blocks =
            content.as_array().ok_or_else(|| bad(line, "invalid history message content"))?;
        for block in blocks {
            match string(block, "type", line)? {
                "text" | "input_text" | "output_text" => {
                    self.push(role, text_field(block, "text", line)?.to_owned(), line)?;
                }
                "tool_use" if self.provider == Provider::Claude => {
                    let name = string(block, "name", line)?;
                    let id = string(block, "id", line)?;
                    let input =
                        block.get("input").ok_or_else(|| bad(line, "missing tool input"))?;
                    self.push("tool", format!("tool call {name} ({id}): {input}"), line)?;
                }
                "tool_result" if self.provider == Provider::Claude => {
                    let id = string(block, "tool_use_id", line)?;
                    let output = self.visible_text(&block["content"], line)?;
                    self.push("tool", format!("tool result ({id}): {output}"), line)?;
                }
                _ => self.excluded = true,
            }
        }
        Ok(())
    }

    fn visible_text(&mut self, content: &Value, line: usize) -> Result<String> {
        if let Some(text) = content.as_str() {
            return Ok(text.to_owned());
        }
        let blocks = content.as_array().ok_or_else(|| bad(line, "invalid history tool output"))?;
        let mut parts = Vec::new();
        for block in blocks {
            if matches!(block["type"].as_str(), Some("text" | "input_text" | "output_text")) {
                parts.push(text_field(block, "text", line)?);
            } else {
                self.excluded = true;
            }
        }
        Ok(parts.join("\n"))
    }

    fn push(&mut self, kind: &str, mut text: String, line: usize) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }
        if text.len() > MAX_RECORD {
            let marker = "\n[truncated]\n";
            let mut head = (MAX_RECORD - marker.len()) / 2;
            let mut tail = text.len() - head;
            while !text.is_char_boundary(head) {
                head -= 1;
            }
            while !text.is_char_boundary(tail) {
                tail += 1;
            }
            text = format!("{}{marker}{}", &text[..head], &text[tail..]);
            self.truncated += 1;
        }
        let record =
            HistoryRecord { reference: format!("history:{line}"), kind: kind.to_owned(), text };
        let size = serde_json::to_vec(&record)
            .map_err(|e| bad(line, &format!("cannot encode history: {e}")))?
            .len();
        self.bytes += size;
        self.records.push_back((record, size));
        while self.records.len() > MAX_RECORDS || self.bytes > MAX_OUTPUT {
            if let Some((_, size)) = self.records.pop_front() {
                self.bytes -= size;
                self.omitted += 1;
            }
        }
        Ok(())
    }

    pub(super) fn finish(self) -> Result<(Vec<HistoryRecord>, Vec<String>, usize)> {
        if !self.identity {
            return Err(Error::failed("missing history provider/session identity"));
        }
        if self.records.is_empty() {
            return Err(Error::failed("history contains no useful visible records"));
        }
        let mut warnings: Vec<_> = self.cwd.finish()?.into_iter().collect();
        if self.omitted > 0 {
            warnings.push(format!(
                "omitted {} older visible records; retained newest bounded history",
                self.omitted
            ));
        }
        if self.truncated > 0 {
            warnings.push(format!(
                "truncated {} long visible records to {MAX_RECORD} bytes",
                self.truncated
            ));
        }
        if self.excluded {
            warnings.push("excluded reasoning, analysis, encrypted or non-text content".to_owned());
        }
        Ok((self.records.into_iter().map(|(record, _)| record).collect(), warnings, self.omitted))
    }
}

fn string<'a>(value: &'a Value, key: &str, line: usize) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| bad(line, &format!("missing or invalid {key} in history record")))
}

fn text_field<'a>(value: &'a Value, key: &str, line: usize) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| bad(line, &format!("missing or invalid {key} in history record")))
}

fn bad(line: usize, message: &str) -> Error {
    Error::failed(format!("history line {line}: {message}"))
}
