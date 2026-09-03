// verbs/issue/asked.rs
// What the fixture supplied, and whether the file that landed is the one it asked for (R06).

use super::file::Filing;
use super::slug::slug;

/// What the fixture supplied and how many times it filed it: the file that has to be there.
pub(super) struct Asked {
    pub filing: Filing,
    pub runs: u32,
}

impl Asked {
    /// The title as `issue new` writes it into the frontmatter: whitespace collapsed.
    fn title(&self) -> String {
        self.filing
            .title
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// The sections the filing has to carry, in the order render() writes them. The proposal is
    /// the one optional field, so its section is there exactly when the fixture passed one.
    fn sections(&self) -> Vec<(String, String)> {
        let mut wanted: Vec<(String, String)> = Vec::new();
        for (heading, body) in [
            ("Symptom", &self.filing.symptom),
            ("Reproduction", &self.filing.repro),
            ("Source", &self.filing.source),
            ("Proposal", &self.filing.proposal),
        ] {
            if !body.is_empty() {
                wanted.push((heading.to_string(), body.trim_end().to_string()));
            }
        }
        wanted.push(("Sightings".to_string(), String::new()));
        wanted
    }
}

/// The file the fixture asked for, or the first thing about this one that is not it — as the
/// phrase the `doctor --self` row carries.
///
/// The count of sightings alone is not the file: read on its own it lets a filing mishandle the
/// title, every section and every sighting line at once and still land on a passing row. So what
/// the fixture supplied is compared with what landed, field by field — the name against the slug
/// of the title, the frontmatter, each section against the value passed, and one sighting line per
/// filing, each in the shape Sighting::line() writes.
pub(super) fn matched(text: &str, name: &str, asked: &Asked) -> Result<(), String> {
    let title = asked.title();
    let named = format!("{}.md", slug(&title));
    if name != named {
        return Err(format!("a file named {name}, not {named}"));
    }
    if !text.starts_with("---\n") {
        return Err("a file that opens with no frontmatter".to_string());
    }
    frontmatter(text, "title", &title)?;
    frontmatter(text, "sightings", &asked.runs.to_string())?;
    if value(text, "first_seen").unwrap_or_default().is_empty() {
        return Err("a file whose frontmatter names no first_seen".to_string());
    }
    let found = sections(text);
    let wanted = asked.sections();
    let headings = |list: &[(String, String)]| {
        list.iter()
            .map(|(heading, _)| heading.clone())
            .collect::<Vec<String>>()
            .join(", ")
    };
    if headings(&found) != headings(&wanted) {
        return Err(format!(
            "a file whose sections are {}, not {}",
            headings(&found),
            headings(&wanted)
        ));
    }
    for ((heading, body), (_, wanted)) in found.iter().zip(&wanted) {
        if heading != "Sightings" && body != wanted {
            return Err(format!(
                "a file whose {heading} section says {body:?}, not {wanted:?}"
            ));
        }
    }
    sightings(&found, asked.runs)
}

/// One line per filing, each in the shape Sighting::line() writes: the stamp, the run and the
/// plan, two spaces apart.
fn sightings(found: &[(String, String)], runs: u32) -> Result<(), String> {
    let body = found
        .iter()
        .find(|(heading, _)| heading == "Sightings")
        .map(|(_, body)| body.as_str())
        .unwrap_or_default();
    let lines: Vec<&str> = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.len() != runs as usize {
        return Err(format!(
            "a file whose Sightings list holds {} lines, not {runs}",
            lines.len()
        ));
    }
    for line in lines {
        let fields: Vec<&str> = line.split("  ").collect();
        // `- <stamp>  run <id>  plan <id>`: the stamp is the 20 characters utc_now() writes.
        let stamped = |field: &str| {
            field
                .strip_prefix("- ")
                .map(|stamp| stamp.len() == 20 && stamp.ends_with('Z'))
        };
        let named = |field: &str, key: &str| {
            field
                .strip_prefix(key)
                .map(|id| !id.is_empty())
                .unwrap_or(false)
        };
        let shaped = fields.len() == 3
            && stamped(fields[0]).unwrap_or(false)
            && named(fields[1], "run ")
            && named(fields[2], "plan ");
        if !shaped {
            return Err(format!("a file whose sighting line reads {line:?}"));
        }
    }
    Ok(())
}

/// One frontmatter field against the value it has to carry.
fn frontmatter(text: &str, key: &str, wanted: &str) -> Result<(), String> {
    match value(text, key) {
        Some(found) if found == wanted => Ok(()),
        Some(found) => Err(format!(
            "a file whose frontmatter {key} is {found:?}, not {wanted:?}"
        )),
        None => Err(format!("a file whose frontmatter names no {key}")),
    }
}

/// The value of one frontmatter field, read between the opening and closing `---`.
fn value(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}: ");
    text.lines()
        .skip(1)
        .take_while(|line| *line != "---")
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|found| found.trim().to_string())
}

/// The `## <heading>` sections of the file with their bodies, in the order they appear.
fn sections(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        match line.strip_prefix("## ") {
            Some(heading) => found.push((heading.trim().to_string(), String::new())),
            None => {
                if let Some((_, body)) = found.last_mut() {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    for (_, body) in found.iter_mut() {
        while body.ends_with('\n') {
            body.pop();
        }
    }
    found
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::verbs::issue::file::{append, render, Sighting};

    fn filed(proposal: &str) -> Asked {
        Asked {
            filing: Filing {
                title: "plan start refuses a file worktree".to_string(),
                symptom: "it exits 1 and prints nothing at all".to_string(),
                repro: "dstack plan start P4 --worktree ./notes.txt".to_string(),
                source: "dstack-cli/src/verbs/plan/lifecycle.rs".to_string(),
                proposal: proposal.to_string(),
            },
            runs: 1,
        }
    }

    fn seen(stamp: &str) -> Sighting {
        Sighting {
            stamp: stamp.to_string(),
            run: "20260903T044827Z_dstack-issues".to_string(),
            plan: "P2".to_string(),
        }
    }

    /// The file `issue new` writes for this filing is the file the fixture asked for, and a
    /// repeat of it is too — one sighting line per filing.
    #[test]
    fn r06__the_file_the_verb_writes_is_the_file_the_fixture_asked_for() {
        let asked = filed("");
        let name = "plan-start-refuses-a-file-worktree.md";
        let first = render(&asked.filing, &seen("2026-09-03T05:10:22Z"));
        assert_eq!(matched(&first, name, &asked), Ok(()));
        let (twice, _) = append(&first, &seen("2026-09-03T06:02:41Z")).expect("a sighting");
        let repeated = Asked {
            runs: 2,
            ..filed("")
        };
        assert_eq!(matched(&twice, name, &repeated), Ok(()));
        // The proposal is the one optional field: its section is there exactly when it was passed.
        let proposed = filed("name the path that was refused");
        let with = render(&proposed.filing, &seen("2026-09-03T05:10:22Z"));
        assert_eq!(matched(&with, name, &proposed), Ok(()));
        assert_eq!(
            matched(&with, name, &asked),
            Err(
                "a file whose sections are Symptom, Reproduction, Source, Proposal, Sightings, \
                 not Symptom, Reproduction, Source, Sightings"
                    .to_string()
            )
        );
        assert_eq!(
            matched(&first, name, &proposed),
            Err(
                "a file whose sections are Symptom, Reproduction, Source, Sightings, \
                 not Symptom, Reproduction, Source, Proposal, Sightings"
                    .to_string()
            )
        );
    }

    /// Every field the fixture supplied is compared, so a filing that mishandled one is named.
    #[test]
    fn r06__a_field_the_filing_mishandled_is_the_mismatch_the_row_shows() {
        let asked = filed("");
        let name = "plan-start-refuses-a-file-worktree.md";
        let whole = render(&asked.filing, &seen("2026-09-03T05:10:22Z"));
        assert_eq!(
            matched(&whole, "elsewhere.md", &asked),
            Err("a file named elsewhere.md, not plan-start-refuses-a-file-worktree.md".to_string())
        );
        let mangled = |from: &str, to: &str| whole.replace(from, to);
        for (text, mismatch) in [
            (
                mangled("title: plan start", "title: plan stop"),
                "a file whose frontmatter title is \"plan stop refuses a file worktree\", \
                 not \"plan start refuses a file worktree\"",
            ),
            (
                mangled("sightings: 1", "sightings: 2"),
                "a file whose frontmatter sightings is \"2\", not \"1\"",
            ),
            (
                mangled("first_seen: 2026-09-03T05:10:22Z\n", ""),
                "a file whose frontmatter names no first_seen",
            ),
            (
                mangled("it exits 1 and prints nothing at all", "it exits 1"),
                "a file whose Symptom section says \"it exits 1\", \
                 not \"it exits 1 and prints nothing at all\"",
            ),
            (
                mangled("--worktree ./notes.txt", "--worktree ./other.txt"),
                "a file whose Reproduction section says \
                 \"dstack plan start P4 --worktree ./other.txt\", \
                 not \"dstack plan start P4 --worktree ./notes.txt\"",
            ),
            (
                mangled("plan/lifecycle.rs", "plan/other.rs"),
                "a file whose Source section says \"dstack-cli/src/verbs/plan/other.rs\", \
                 not \"dstack-cli/src/verbs/plan/lifecycle.rs\"",
            ),
            (
                mangled("## Sightings\n- 2026", "## Sightings\n- x 2026"),
                "a file whose sighting line reads \"- x 2026-09-03T05:10:22Z  \
                 run 20260903T044827Z_dstack-issues  plan P2\"",
            ),
            (
                mangled("run 20260903T044827Z_dstack-issues  ", ""),
                "a file whose sighting line reads \"- 2026-09-03T05:10:22Z  plan P2\"",
            ),
            (
                format!("{whole}- 2026-09-03T06:02:41Z  run r  plan P2\n"),
                "a file whose Sightings list holds 2 lines, not 1",
            ),
            (
                "# notes I keep by hand\n".to_string(),
                "a file that opens with no frontmatter",
            ),
        ] {
            assert_eq!(matched(&text, name, &asked), Err(mismatch.to_string()));
        }
    }
}
