// verbs/issue/file.rs
// The shape of an issue file: the frontmatter and sections of a first filing, and the sighting a
// repeat appends to it (D-DESIGN-01).

/// What the worker filed. The optional proposal is the only field that may be empty; the other
/// three are what makes an issue actionable, and `issue new` refuses without them (D-08).
#[derive(Default)]
pub struct Filing {
    pub title: String,
    pub symptom: String,
    pub repro: String,
    pub source: String,
    pub proposal: String,
}

/// One filing of an issue: when it was filed and what it was filed from.
pub struct Sighting {
    pub stamp: String,
    pub run: String,
    pub plan: String,
}

impl Sighting {
    /// One line of the Sightings list, and the tail of what `issue new` prints.
    pub fn line(&self) -> String {
        format!("- {}  run {}  plan {}", self.stamp, self.run, self.plan)
    }
}

/// The whole file of a first filing: the frontmatter, the sections the worker filled, and the
/// Sightings list this filing opens. The list is last, so a repeat appends to the end of the file.
pub fn render(filing: &Filing, seen: &Sighting) -> String {
    let mut text = format!(
        "---\ntitle: {}\nfirst_seen: {}\nsightings: 1\n---\n",
        filing.title, seen.stamp
    );
    for (heading, body) in [
        ("Symptom", &filing.symptom),
        ("Reproduction", &filing.repro),
        ("Source", &filing.source),
        ("Proposal", &filing.proposal),
    ] {
        if !body.is_empty() {
            text.push_str(&format!("\n## {heading}\n{}\n", body.trim_end()));
        }
    }
    text.push_str(&format!("\n## Sightings\n{}\n", seen.line()));
    text
}

/// A repeat is a sighting on the file that is already there (D-06): the frontmatter count goes up
/// by one and the line joins the end of the Sightings list. The whole text and the new count come
/// back, or None when this is not a file dstack wrote — the maintainer triages these files by
/// hand, and rewriting one whose frontmatter says something else would throw that work away.
pub fn append(text: &str, seen: &Sighting) -> Option<(String, u32)> {
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    if lines.first().map(String::as_str) != Some("---") {
        return None;
    }
    let end = 1 + lines.iter().skip(1).position(|line| line == "---")?;
    let at = lines[..end]
        .iter()
        .position(|line| line.starts_with("sightings: "))?;
    let count = 1 + lines[at]
        .trim_start_matches("sightings: ")
        .parse::<u32>()
        .ok()?;
    lines[at] = format!("sightings: {count}");
    lines.push(seen.line());
    Some((format!("{}\n", lines.join("\n")), count))
}

/// What `issue list` reports about one file: the title of its frontmatter, how many sightings it
/// carries and the stamp of the last one. A file whose frontmatter says none of that still gets a
/// row, named after the file and marked "-": the list reports what is in the folder, and a file it
/// cannot read is exactly what the maintainer wants to see.
pub struct Summary {
    pub title: String,
    pub sightings: String,
    pub last: String,
}

pub fn summary(text: &str, name: &str) -> Summary {
    let (mut title, mut sightings) = (String::new(), String::new());
    if text.starts_with("---\n") {
        for line in text.lines().skip(1).take_while(|line| *line != "---") {
            if let Some(value) = line.strip_prefix("title: ") {
                title = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("sightings: ") {
                sightings = value.trim().to_string();
            }
        }
    }
    let last = text
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .last()
        .and_then(|line| line.split("  ").next())
        .unwrap_or_default();
    let or_dash = |value: &str| match value.is_empty() {
        true => "-".to_string(),
        false => value.to_string(),
    };
    Summary {
        title: match title.is_empty() {
            true => name.to_string(),
            false => title,
        },
        sightings: or_dash(&sightings),
        last: or_dash(last),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn filed() -> Filing {
        Filing {
            title: "plan start refuses a file worktree".to_string(),
            symptom: "exits 1 and prints nothing".to_string(),
            repro: "dstack plan start P4 --worktree ./notes.txt".to_string(),
            source: "dstack-cli/src/verbs/plan/lifecycle.rs".to_string(),
            proposal: String::new(),
        }
    }

    fn seen(stamp: &str) -> Sighting {
        Sighting {
            stamp: stamp.to_string(),
            run: "2026-run_x".to_string(),
            plan: "P10".to_string(),
        }
    }

    #[test]
    fn r01__a_first_filing_is_frontmatter_and_the_sections_it_filled() {
        let text = render(&filed(), &seen("2026-09-03T05:10:22Z"));
        assert_eq!(
            text,
            "---\ntitle: plan start refuses a file worktree\nfirst_seen: 2026-09-03T05:10:22Z\n\
             sightings: 1\n---\n\n## Symptom\nexits 1 and prints nothing\n\
             \n## Reproduction\ndstack plan start P4 --worktree ./notes.txt\n\
             \n## Source\ndstack-cli/src/verbs/plan/lifecycle.rs\n\
             \n## Sightings\n- 2026-09-03T05:10:22Z  run 2026-run_x  plan P10\n"
        );
        let mut with_proposal = filed();
        with_proposal.proposal = "say which path was refused and why".to_string();
        let text = render(&with_proposal, &seen("2026-09-03T05:10:22Z"));
        assert!(
            text.contains("\n## Proposal\nsay which path was refused and why\n\n## Sightings\n")
        );
    }

    #[test]
    fn r02__a_repeat_raises_the_count_and_joins_the_list() {
        let first = render(&filed(), &seen("2026-09-03T05:10:22Z"));
        let (text, count) = append(&first, &seen("2026-09-03T06:02:41Z")).expect("a sighting");
        assert_eq!(count, 2);
        assert!(text.contains("\nsightings: 2\n---\n"), "{text}");
        assert!(
            text.ends_with(
                "- 2026-09-03T05:10:22Z  run 2026-run_x  plan P10\n\
                            - 2026-09-03T06:02:41Z  run 2026-run_x  plan P10\n"
            ),
            "{text}"
        );
        let (again, count) = append(&text, &seen("2026-09-03T07:00:00Z")).expect("a sighting");
        assert_eq!(count, 3);
        assert!(again.contains("\nsightings: 3\n---\n"), "{again}");
    }

    #[test]
    fn r04__the_row_of_a_file_is_its_title_count_and_last_sighting() {
        let first = render(&filed(), &seen("2026-09-03T05:10:22Z"));
        let (text, _) = append(&first, &seen("2026-09-03T06:02:41Z")).expect("a sighting");
        let row = summary(&text, "plan-start-refuses-a-file-worktree.md");
        assert_eq!(row.title, "plan start refuses a file worktree");
        assert_eq!(row.sightings, "2");
        assert_eq!(row.last, "2026-09-03T06:02:41Z");
        // A file dstack did not write is named after itself and says nothing it cannot read.
        let stranger = summary("# notes\n", "notes.md");
        assert_eq!(stranger.title, "notes.md");
        assert_eq!(stranger.sightings, "-");
        assert_eq!(stranger.last, "-");
    }

    #[test]
    fn r02__a_file_dstack_did_not_write_is_left_alone() {
        for text in [
            "",
            "# notes\nsightings: 1\n",
            "---\ntitle: no count\n---\n\n## Symptom\nx\n",
            "---\ntitle: unopened\nsightings: many\n---\n",
            "---\ntitle: unclosed\nsightings: 1\n\n## Symptom\nx\n",
        ] {
            assert!(
                append(text, &seen("2026-09-03T06:02:41Z")).is_none(),
                "{text}"
            );
        }
    }
}
