// verbs/issue/slug.rs
// The slug of an issue title: the whole identity of an issue file (D-DESIGN-01).

/// The normalised title: ASCII letters and digits in lower case, every other run of characters
/// folded into one dash, and no dash at either end. Two filings whose titles differ only in
/// capitals or punctuation land on the same file, which is what turns a repeat into a sighting
/// instead of a second file (D-06); an empty answer is a title with nothing to name a file after.
///
/// core::paths::valid_slug is the rule this has to satisfy, and the test below holds the two
/// together. It stays a validator of a slug someone typed: a title is prose that was never typed
/// as a slug, so building one out of it is a different job and lives here.
pub fn slug(title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::paths::valid_slug;

    #[test]
    fn r01__a_title_becomes_the_name_of_its_file() {
        assert_eq!(
            slug("plan start refuses a file worktree"),
            "plan-start-refuses-a-file-worktree"
        );
        assert_eq!(slug("R64 rejects ../x"), "r64-rejects-x");
        assert!(valid_slug(&slug("plan start refuses a file worktree")));
    }

    #[test]
    fn r02__capitals_and_punctuation_land_on_the_same_slug() {
        let plain = slug("plan start refuses a file worktree");
        for title in [
            "Plan start, refuses a file worktree!",
            "  plan   start — refuses a file worktree  ",
            "PLAN START: REFUSES A FILE WORKTREE",
        ] {
            assert_eq!(slug(title), plain, "{title}");
        }
    }

    #[test]
    fn r03__a_title_with_nothing_to_name_a_file_after_is_empty() {
        for title in ["", "!!!", "   ", "한글"] {
            assert_eq!(slug(title), "", "{title}");
        }
    }
}
