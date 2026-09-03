// verbs/issue/filed.rs
// What a filing left behind: the issue folder read back, entry by entry (R06).

use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::paths::base_name;
use crate::selftest::sandbox::Sandbox;

use super::asked::{matched, Asked};

/// The file `plant:` puts where the filing would land: its frontmatter is not the one dstack
/// writes, so the filing has to refuse instead of rewriting notes the maintainer keeps by hand.
pub(super) const PLANTED: &str = "# notes I keep by hand\nsightings: many\n";

/// The mkdir lock the verb serialises the folder with (fsx::with_lock): it is taken inside the
/// folder and released on the way out, so it is not something a filing wrote and is not counted
/// as one.
const LOCK: &str = "lock";

/// What the folder holds, in the words the `doctor --self` row carries.
#[derive(Debug)]
pub(super) enum Look {
    Nothing,
    Planted,
    Asked,
    Other(String),
}

impl Look {
    pub(super) fn phrase(&self) -> String {
        match self {
            Look::Nothing => "nothing".to_string(),
            Look::Planted => "the planted file, as it was planted".to_string(),
            Look::Asked => "the file the fixture asked for".to_string(),
            Look::Other(what) => what.clone(),
        }
    }
}

/// The folder read back against what the fixture asked for. Two files are as wrong as none, and
/// so is one entry that is not an issue file at all: repeated filings of one title are one file
/// (D-06).
pub(super) fn look(dir: &Path, asked: &Asked) -> Result<Look> {
    let left = entries(dir)?;
    let path = match left.as_slice() {
        [] => return Ok(Look::Nothing),
        [only] if only.extension().and_then(|ext| ext.to_str()) == Some("md") => only,
        [only] => return Ok(Look::Other(format!("one entry, {}", base_name(only)))),
        many => return Ok(Look::Other(format!("{} entries", many.len()))),
    };
    let text = read(path)?;
    if text == PLANTED {
        return Ok(Look::Planted);
    }
    Ok(match matched(&text, &base_name(path), asked) {
        Ok(()) => Look::Asked,
        Err(mismatch) => Look::Other(mismatch),
    })
}

/// Everything the folder holds, in name order, minus the lock. A folder that is not there holds
/// nothing — that is the honest reading of a refusal that stopped before the folder was made — but
/// a folder that is there and cannot be read is no answer at all (D-12), and neither is an entry
/// that cannot be listed. Every kind of entry counts, not only the issue files: a refusal that
/// dropped anything at all is not the refusal that writes nothing.
fn entries(dir: &Path) -> Result<Vec<PathBuf>> {
    let listing = match std::fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(Error::cannot_decide(format!(
                "selftest: cannot read {}: {e}",
                dir.display()
            )))
        }
    };
    let mut left: Vec<PathBuf> = Vec::new();
    for entry in listing {
        let path = entry
            .map_err(|e| {
                Error::cannot_decide(format!("selftest: cannot list {}: {e}", dir.display()))
            })?
            .path();
        if base_name(&path) != LOCK {
            left.push(path);
        }
    }
    left.sort();
    Ok(left)
}

/// D-12: a file that is there and cannot be read is not an answer.
fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| Error::cannot_decide(format!("selftest: cannot read {}: {e}", path.display())))
}

/// The folder D-05 fixes, pointed at the sandbox: $HOME/Documents/dstack-issues under a HOME of
/// this fixture's own.
pub(super) fn issues(sandbox: &Sandbox) -> PathBuf {
    sandbox.dir.join("home/Documents/dstack-issues")
}

pub(super) fn plant(dir: &Path, slug: &str) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot create {}: {e}", dir.display()))
    })?;
    let path = dir.join(format!("{slug}.md"));
    std::fs::write(&path, PLANTED).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot plant {}: {e}", path.display()))
    })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::verbs::issue::file::{render, Filing, Sighting};
    use crate::verbs::issue::slug::slug;
    use std::os::unix::fs::PermissionsExt;

    const TITLE: &str = "plan start refuses a file worktree";

    fn asked() -> Asked {
        Asked {
            filing: Filing {
                title: TITLE.to_string(),
                symptom: "it exits 1".to_string(),
                repro: "dstack plan start P4".to_string(),
                source: "lifecycle.rs".to_string(),
                proposal: String::new(),
            },
            runs: 1,
        }
    }

    /// Each of the four readings, on a folder built to carry it.
    #[test]
    fn r06__the_folder_is_read_back_entry_by_entry() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let dir = issues(&sandbox);
        let asked = asked();
        assert!(matches!(
            look(&dir, &asked).expect("a folder that is not there"),
            Look::Nothing
        ));
        plant(&dir, &slug(TITLE)).expect("plant");
        // The lock the folder is serialised with is not something a filing wrote.
        std::fs::create_dir(dir.join(LOCK)).expect("the lock");
        assert!(matches!(
            look(&dir, &asked).expect("planted"),
            Look::Planted
        ));
        let path = dir.join(format!("{}.md", slug(TITLE)));
        let filed = render(
            &asked.filing,
            &Sighting {
                stamp: "2026-09-03T05:10:22Z".to_string(),
                run: "r".to_string(),
                plan: "P2".to_string(),
            },
        );
        std::fs::write(&path, &filed).expect("the file the fixture asked for");
        assert!(matches!(look(&dir, &asked).expect("filed"), Look::Asked));
        // Anything else a filing dropped counts too, whatever it is named.
        std::fs::write(dir.join("dropped.tmp"), "x").expect("a stray file");
        assert_eq!(look(&dir, &asked).expect("two").phrase(), "2 entries");
        std::fs::remove_file(&path).expect("clean up");
        assert_eq!(
            look(&dir, &asked).expect("one stray").phrase(),
            "one entry, dropped.tmp"
        );
    }

    /// D-12: a folder that is there and cannot be read is not the empty one a refusal leaves.
    #[test]
    fn r06__a_folder_that_cannot_be_read_is_not_an_empty_one() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let dir = issues(&sandbox);
        plant(&dir, "one").expect("plant");
        let mut closed = std::fs::metadata(&dir).expect("the folder").permissions();
        closed.set_mode(0o000);
        std::fs::set_permissions(&dir, closed).expect("close the folder");
        let cannot = look(&dir, &asked()).expect_err("cannot decide");
        let mut open = std::fs::metadata(&dir).expect("the folder").permissions();
        open.set_mode(0o755);
        std::fs::set_permissions(&dir, open).expect("open it again");
        assert_eq!(cannot.code(), 2);
        assert!(
            cannot
                .message()
                .starts_with(&format!("selftest: cannot read {}", dir.display())),
            "{}",
            cannot.message()
        );
    }
}
