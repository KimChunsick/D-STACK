// verbs/ledger/artifact.rs
// The file an evidence row points at: its physical path and whether it names the R (R104).

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};

/// (3) the artifact must exist, as an absolute path whose directory is the physical one.
pub(super) fn resolve_artifact(artifact: &str) -> Result<PathBuf> {
    let mut abs = PathBuf::from(artifact);
    if !abs.is_absolute() {
        let cwd = std::env::current_dir()
            .map_err(|e| Error::cannot_decide(format!("cannot read the cwd: {e}")))?;
        abs = cwd.join(artifact);
    }
    let parent = abs.parent().unwrap_or(Path::new("/")).to_path_buf();
    let base = abs.file_name().unwrap_or_default().to_os_string();
    let physical = match std::fs::canonicalize(&parent) {
        Ok(physical) if physical.is_dir() => physical,
        _ => fail!(
            "artifact not found: {artifact} (no directory {})",
            parent.display()
        ),
    };
    let abs = physical.join(base);
    if !abs.is_file() {
        fail!("artifact not found: {}", abs.display())
    }
    Ok(abs)
}

/// `grep -qw -- "$r" "$abs"`: the id surrounded by anything that is not a word constituent.
/// grep reads the file in the caller's locale, where a letter like é is a word character, so the
/// boundary is asked of decoded characters; a byte sequence that is not UTF-8 decodes to the
/// replacement character, which is not one and therefore reads as a boundary.
///
/// grep scans line by line and so does this: an artifact can be a capture of any size, an R id
/// carries no newline, and a newline is not a word constituent either — so the verdict is the
/// same one the whole file gave, with only the longest line ever held (P8 round 031).
pub(super) fn names_word(path: &Path, r: &str) -> bool {
    if r.is_empty() {
        return false;
    }
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    BufReader::new(file)
        .split(b'\n')
        .map_while(std::result::Result::ok)
        .any(|line| names_word_in(&String::from_utf8_lossy(&line), r))
}

/// One line of the scan above.
fn names_word_in(text: &str, r: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(r).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + r.len()..].chars().next();
        !before.is_some_and(word) && !after.is_some_and(word)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// grep counts é as a word character under a UTF-8 locale, so éR01é is not a mention of R01.
    #[test]
    fn r13_the_r_id_has_to_stand_as_a_whole_word() {
        let dir = std::env::temp_dir().join(format!("dstack-p8-word-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let cases: [(&str, &[u8], bool); 9] = [
            ("spaces", b" R01 ", true),
            ("parens", b"(R01)", true),
            ("bare", b"R01", true),
            ("glued", b"R011", false),
            ("under", b"_R01", false),
            ("accent", "\u{e9}R01\u{e9}".as_bytes(), false),
            ("binary", &[0xff, 0xfe, b' ', b'R', b'0', b'1', b'\n'], true),
            // The scan is per line, as grep's is: a boundary at the end of one line is still a
            // boundary, and a glued id on another line is still not a mention.
            ("lines", b"first\nxR01x\n R01\nlast", true),
            ("glued lines", b"first\nxR01\nR01x\n", false),
        ];
        for (name, text, wanted) in cases {
            let path = dir.join(name);
            std::fs::write(&path, text).expect("write");
            assert_eq!(names_word(&path, "R01"), wanted, "{name}");
        }
        std::fs::remove_dir_all(&dir).expect("clean up");
    }
}
