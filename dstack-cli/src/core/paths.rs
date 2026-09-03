// core/paths.rs
// Path and id helpers shared by the verbs: slugs, R ids, repo-relative paths, overlap, names.

/// valid_slug: non-empty, never opens with a dash, and carries nothing outside [a-z0-9-].
pub fn valid_slug(slug: &str) -> bool {
    if slug.is_empty() || slug.starts_with('-') {
        return false;
    }
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// An id names one directory inside RUNS or QUICK and nothing else. The shell joined the id
/// unchecked, so `--run ../../elsewhere` reached touch_owner and rewrote a meta.tsv outside the
/// store; D-10 says that defect is not reproduced.
pub fn is_plain_name(id: &str) -> bool {
    !id.is_empty() && !id.contains('/') && id != "." && id != ".."
}

/// `basename "$path"`: the last component of a path, empty when there is none.
pub fn base_name(path: &std::path::Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// fmt_rid: the shell's `R%02d`. The width of 2 counts the sign, so 7 is written R07 and a
/// negative id prints as R-5, exactly as the shell's printf does.
pub fn fmt_rid(n: i64) -> String {
    format!("R{n:02}")
}

/// parse_rid: the number an R id carries, padded (R07) or bare (R7). Anything else is not an id.
// The shell's wrap is reproduced: R9223372036854775808 parses as the negative value bash computes.
pub fn parse_rid(id: &str) -> Option<i64> {
    let digits = id.strip_prefix('R')?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(shell_int(digits))
}

/// shell_int: bash's `$((10#$digits))`. The digits fold into an intmax_t, so a value past 2^63
/// comes back negative and one past 2^64 starts from zero again. Callers pass ASCII digits only.
pub fn shell_int(digits: &str) -> i64 {
    let mut n: u64 = 0;
    for b in digits.bytes() {
        n = n.wrapping_mul(10).wrapping_add(u64::from(b - b'0'));
    }
    n as i64
}

/// valid_rel_path (R64): no absolute path, no `..` segment, no glob character, not empty.
pub fn valid_rel_path(path: &str) -> bool {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('*')
        || path.contains('?')
        || path.contains('[')
    {
        return false;
    }
    !(path == ".." || path.starts_with("../") || path.ends_with("/..") || path.contains("/../"))
}

/// paths_overlap: equal, or one is a directory prefix of the other. Undirected, unlike path_within.
pub fn paths_overlap(a: &str, b: &str) -> bool {
    a == b || dir_prefix(a, b) || dir_prefix(b, a)
}

/// The shell's `case "$inner/" in "$outer"/*)`: outer must end on a `/` boundary of inner.
fn dir_prefix(outer: &str, inner: &str) -> bool {
    inner.len() > outer.len() && inner.as_bytes()[outer.len()] == b'/' && inner.starts_with(outer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_valid_slug_accepts_lowercase_digits_and_dashes() {
        assert!(valid_slug("dstack-rust"));
        assert!(valid_slug("p3"));
        assert!(valid_slug("a"));
        assert!(valid_slug("0-9-a"));
        assert!(valid_slug("trailing-"));
    }

    #[test]
    fn r13_valid_slug_rejects_empty_and_leading_dash() {
        assert!(!valid_slug(""));
        assert!(!valid_slug("-lead"));
        assert!(!valid_slug("-"));
    }

    #[test]
    fn r13_valid_slug_rejects_characters_outside_the_set() {
        assert!(!valid_slug("Upper"));
        assert!(!valid_slug("with space"));
        assert!(!valid_slug("under_score"));
        assert!(!valid_slug("dot.slug"));
        assert!(!valid_slug("한글"));
    }

    #[test]
    fn r13_fmt_rid_pads_to_two_digits() {
        assert_eq!(fmt_rid(0), "R00");
        assert_eq!(fmt_rid(7), "R07");
        assert_eq!(fmt_rid(13), "R13");
        assert_eq!(fmt_rid(123), "R123");
    }

    #[test]
    fn r13_parse_rid_reads_padded_and_bare_numbers() {
        assert_eq!(parse_rid("R07"), Some(7));
        assert_eq!(parse_rid("R7"), Some(7));
        assert_eq!(parse_rid("R123"), Some(123));
        assert_eq!(parse_rid("R00"), Some(0));
    }

    #[test]
    fn r13_fmt_rid_accepts_the_shell_integer_range() {
        assert_eq!(fmt_rid(7), "R07");
        assert_eq!(fmt_rid(123), "R123");
        assert_eq!(fmt_rid(4294967296), "R4294967296");
        assert_eq!(parse_rid(&fmt_rid(4294967296)), Some(4294967296));
        assert_eq!(fmt_rid(i64::MAX), "R9223372036854775807");
        assert_eq!(parse_rid(&fmt_rid(i64::MAX)), Some(i64::MAX));
    }

    #[test]
    fn r13_parse_rid_accepts_the_shell_integer_range() {
        assert_eq!(parse_rid("R4294967296"), Some(4294967296));
        assert_eq!(parse_rid("R9223372036854775807"), Some(i64::MAX));
        assert_eq!(shell_int("07"), 7);
        assert_eq!(shell_int("9223372036854775807"), i64::MAX);
    }

    #[test]
    fn r13_parse_rid_wraps_like_bash() {
        assert_eq!(
            parse_rid("R9223372036854775808"),
            Some(-9223372036854775808)
        );
        assert_eq!(parse_rid("R18446744073709551616"), Some(0));
        assert_eq!(parse_rid("R18446744073709551617"), Some(1));
        assert_eq!(parse_rid("R18446744073709551615"), Some(-1));
        assert_eq!(
            parse_rid("R99999999999999999999"),
            Some(7766279631452241919)
        );
        assert_eq!(fmt_rid(-5), "R-5");
        assert_eq!(fmt_rid(-9223372036854775808), "R-9223372036854775808");
    }

    #[test]
    fn r13_parse_rid_rejects_anything_else() {
        assert_eq!(parse_rid(""), None);
        assert_eq!(parse_rid("R"), None);
        assert_eq!(parse_rid("7"), None);
        assert_eq!(parse_rid("r7"), None);
        assert_eq!(parse_rid("R7a"), None);
        assert_eq!(parse_rid("RR7"), None);
        assert_eq!(parse_rid("R 7"), None);
        assert_eq!(parse_rid("R-7"), None);
    }

    #[test]
    fn r13_valid_rel_path_accepts_repo_relative_paths() {
        assert!(valid_rel_path("src/main.rs"));
        assert!(valid_rel_path("a"));
        assert!(valid_rel_path("a/b/c.txt"));
        assert!(valid_rel_path("..hidden"));
        assert!(valid_rel_path("a/..b"));
    }

    #[test]
    fn r13_valid_rel_path_rejects_empty_and_absolute() {
        assert!(!valid_rel_path(""));
        assert!(!valid_rel_path("/etc/passwd"));
        assert!(!valid_rel_path("/"));
    }

    #[test]
    fn r13_valid_rel_path_rejects_glob_characters() {
        assert!(!valid_rel_path("src/*.rs"));
        assert!(!valid_rel_path("src/f?.rs"));
        assert!(!valid_rel_path("src/[ab].rs"));
    }

    #[test]
    fn r13_valid_rel_path_rejects_dotdot_in_every_position() {
        assert!(!valid_rel_path(".."));
        assert!(!valid_rel_path("../etc"));
        assert!(!valid_rel_path("src/.."));
        assert!(!valid_rel_path("src/../etc"));
    }

    #[test]
    fn r13_paths_overlap_on_equal_and_prefix_in_both_directions() {
        assert!(paths_overlap("src", "src"));
        assert!(paths_overlap("src", "src/core/paths.rs"));
        assert!(paths_overlap("src/core/paths.rs", "src"));
    }

    #[test]
    fn r13_paths_overlap_is_false_for_neighbours() {
        assert!(!paths_overlap("a/b", "a/bc"));
        assert!(!paths_overlap("a/bc", "a/b"));
        assert!(!paths_overlap("src", "tests"));
        assert!(!paths_overlap("", "a"));
    }
}
