// store/rows.rs
// One R row: its id, state, text and accept criterion.

use crate::core::paths::parse_rid;

/// The segment separator design.md §4.2 fixes: U+2014 with a space on each side.
pub const REQ_SEP: &str = " — ";

/// One `- [ ] **R<NN>** <text> — accept: <criterion>[ — key: value]*` line.
///
/// `markers` holds the segments after the accept in file order; the shell keeps the same data
/// as one `key=value;key=value` string, which `markers_string` reproduces for the checks that
/// search it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub text: String,
    pub accept: String,
    pub markers: Vec<(String, String)>,
    pub ticked: bool,
    pub lineno: usize,
}

impl Row {
    /// req_marker(): the value of one marker, or None.
    ///
    /// The shell does not look the key up among the parsed segments: it joins every marker into
    /// one `key=value;key=value` string, splits that on `;` and takes the first token whose text
    /// before the first `=` is the key. A value carrying a semicolon therefore ends at it and can
    /// hide a further `key=value` behind it — reproduced rather than fixed, because verbs print
    /// what this returns.
    pub fn marker(&self, key: &str) -> Option<String> {
        for token in self.markers_string().split(';') {
            let (found, value) = match token.find('=') {
                Some(at) => (&token[..at], &token[at + 1..]),
                None => (token, token),
            };
            if found == key {
                return Some(value.to_string());
            }
        }
        None
    }

    /// The markers as parse.sh joins them, `ticked=yes` included.
    pub fn markers_string(&self) -> String {
        let mut out = String::new();
        for (key, value) in &self.markers {
            if !out.is_empty() {
                out.push(';');
            }
            out.push_str(key);
            out.push('=');
            out.push_str(value);
        }
        if self.ticked {
            if !out.is_empty() {
                out.push(';');
            }
            out.push_str("ticked=yes");
        }
        out
    }

    pub fn is_pending(&self) -> bool {
        format!(";{};", self.markers_string()).contains(";status=pending-approval;")
    }

    /// req_live_ids(): approved, not withdrawn or deferred, and not a split parent.
    pub fn is_live(&self) -> bool {
        let joined = format!(";{};", self.markers_string());
        !(joined.contains(";status=pending-approval;")
            || joined.contains(";withdrawn=")
            || joined.contains(";deferred=")
            || joined.contains(";superseded-by="))
    }

    /// The row line as the CLI writes it. A row whose accept is empty renders without the
    /// accept segment: `check request` refuses such a row, so the shape never reaches a file.
    pub fn render(&self) -> String {
        let mut line = format!(
            "- [{}] **{}** {}",
            if self.ticked { "x" } else { " " },
            self.id,
            self.text
        );
        if !self.accept.is_empty() {
            line.push_str(REQ_SEP);
            line.push_str("accept: ");
            line.push_str(&self.accept);
        }
        for (key, value) in &self.markers {
            line.push_str(REQ_SEP);
            line.push_str(key);
            line.push_str(": ");
            line.push_str(value);
        }
        line
    }
}

/// One line → a Row, or None when the line is not a row (the awk match of parse.sh).
///
/// The id is checked with core::paths::parse_rid — the one rule for ids — and then kept as the
/// string the line carries: awk's `\*\*R[0-9]+\*\*` puts no limit on the digits, so a row
/// `req add --id` minted past u32 reads back as a row and renders unchanged.
pub fn parse_line(lineno: usize, line: &str) -> Option<Row> {
    let after_box = line.strip_prefix("- [")?;
    let box_char = after_box.chars().next()?;
    if !matches!(box_char, ' ' | 'x' | 'X') {
        return None;
    }
    let rest = after_box[box_char.len_utf8()..].strip_prefix("] **")?;
    let (id, rest) = rest.split_once("**")?;
    parse_rid(id)?;
    let rest = rest.strip_prefix(' ')?;

    let mut segments = rest.split(REQ_SEP);
    let text = segments.next().unwrap_or_default().to_string();
    let mut accept = String::new();
    let mut markers = Vec::new();
    for segment in segments {
        if let Some(value) = segment.strip_prefix("accept:") {
            accept = value.trim_start_matches(' ').to_string();
        } else {
            let (key, value) = match segment.find(':') {
                Some(at) => (&segment[..at], segment[at + 1..].trim_start_matches(' ')),
                None => (segment, segment),
            };
            markers.push((key.to_string(), value.to_string()));
        }
    }
    Some(Row {
        id: id.to_string(),
        text,
        accept,
        markers,
        ticked: box_char != ' ',
        lineno,
    })
}
