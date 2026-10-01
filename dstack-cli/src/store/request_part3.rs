// store/request_part3.rs
// Part 3 of a Goal request (D-38): the marker line under `# 3부 계획과 검증` and the bytes the
// approval hash covers. Everything below the marker is the CLI's to regenerate from the plan.

use std::path::Path;

use crate::core::error::{Error, Result};
use crate::core::fsx::sha256_bytes;

/// The line that ends what the approval hash covers. Only the CLI writes it (the templates put it
/// right under the visible heading), so a request without it is a legacy one, hashed whole.
pub const MARKER: &str = "<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->";

/// What the approval covers: every byte above the first line that is the marker once trimmed,
/// or the whole text when no line is. A marker with anything else on its line is no marker, so
/// an edit that reaches it falls back to the whole text and no longer matches the stamp.
pub fn approval_bytes(text: &str) -> &str {
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        if line.trim() == MARKER {
            return &text[..at];
        }
        at += line.len();
    }
    text
}

/// The sha256 a stamp records for this text, in the hex form `request.approved` holds.
pub fn approval_hash(text: &str) -> String {
    sha256_bytes(approval_bytes(text).as_bytes())
}

/// The approval hash of the request file at `path`, as approve writes it and every check reads it.
pub fn sha256_approved(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map(|text| approval_hash(&text))
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", path.display())))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn R15_part3_hash_bytes_stop_above_the_marker_line() {
        let text = format!("위\n# 3부 계획과 검증\n{MARKER}\n아래\n");
        assert_eq!(approval_bytes(&text), "위\n# 3부 계획과 검증\n");
        let crlf = format!("위\r\n{MARKER}\r\n아래\r\n");
        assert_eq!(approval_bytes(&crlf), "위\r\n");
        let last = format!("위\n  {MARKER}");
        assert_eq!(approval_bytes(&last), "위\n");
        assert_eq!(approval_bytes(&format!("{MARKER}\n아래\n")), "");
        let two = format!("위\n{MARKER}\n가운데\n{MARKER}\n아래\n");
        assert_eq!(approval_bytes(&two), "위\n");
    }

    #[test]
    fn R15_part3_hash_bytes_are_the_whole_text_without_a_whole_marker_line() {
        for text in [
            "위\n# 3부 계획과 검증\n아래\n".to_string(),
            format!("위\n{MARKER}x\n아래\n"),
            format!("위\n앞 {MARKER}\n아래\n"),
            String::new(),
        ] {
            assert_eq!(approval_bytes(&text), text);
        }
        assert_eq!(approval_hash("위\n"), sha256_bytes("위\n".as_bytes()));
    }
}
