// verbs/hook/command.rs
// What a Bash call makes visible: heredoc bodies, the message of a `git commit`, and the file a
// redirect names. The shell did this with awk, grep -oE and sed; the patterns are theirs.

use regex::Regex;

/// `printf '%s' "$c" | tr '\n' ' '`: grep is line-oriented, so the whole command is read as one
/// line before any pattern is asked about it.
fn spaced(command: &str) -> String {
    command.replace('\n', " ")
}

fn compiled(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a literal pattern of this file")
}

/// The `git ... commit` a Bash call is about to run. R66's --no-verify and Codex's own commits
/// skip git's hooks, so the message can only be checked here, on the Bash argument itself.
pub(super) fn is_git_commit(command: &str) -> bool {
    compiled(r"git( +--?[A-Za-z-]+( +[^ ]+)?)* +commit").is_match(&spaced(command))
}

/// Heredoc bodies are the only content a Bash call makes visible; a bare `> file` says nothing
/// about what will land there, so it is allowed and the Stop gate's `lint-ko --changed` catches it.
pub(super) fn heredoc_bodies(command: &str) -> String {
    let opener = compiled(r#"<<-?[ ]*["']?([A-Za-z_][A-Za-z0-9_]*)"#);
    let text = format!("{command}\n");
    // `printf '%s\n'` ends the last line, and awk drops the empty record behind the final newline.
    let mut lines: Vec<&str> = text.split('\n').collect();
    lines.pop();
    let mut out = String::new();
    let mut closing: Option<Regex> = None;
    for line in lines {
        match &closing {
            None => {
                if let Some(found) = opener.captures(line) {
                    closing = Some(compiled(&format!(r"^[ \t]*{}[ \t]*$", &found[1])));
                }
            }
            Some(end) => match end.is_match(line) {
                true => closing = None,
                false => {
                    out.push_str(line);
                    out.push('\n');
                }
            },
        }
    }
    out
}

/// The -m arguments and the heredoc body of a `git commit`. Newlines are folded to \001 first
/// because a commit message argument routinely spans lines.
pub(super) fn commit_text(command: &str) -> String {
    let folded = command.replace('\n', "\u{1}");
    let mut out = String::new();
    for pattern in [r#"-m +"[^"]*""#, r"-m +'[^']*'"] {
        for found in compiled(pattern).find_iter(&folded) {
            let argument = found.as_str();
            let opened = argument
                .find(['"', '\''])
                .expect("the pattern matched a quote");
            out.push_str(&argument[opened + 1..argument.len() - 1].replace('\u{1}', "\n"));
            out.push('\n');
        }
    }
    // A heredoc is the message only when it feeds the git commit itself (`git commit -F - <<EOF`).
    // A heredoc elsewhere in the same command (writing a file, then committing) is that file's
    // content and is judged by the file's own scope, not as a commit message.
    if compiled(r"git( +--?[A-Za-z-]+( +[^ ]+)?)* +commit[^;|&]*<<").is_match(&spaced(command)) {
        out.push_str(&heredoc_bodies(command));
    }
    out
}

/// The first `> path` / `>> path` / `tee path` of the command. `2>&1` and `>&2` cannot match: the
/// character before `>` is a digit and the character after is `&`, both excluded. /dev/* and `-`
/// are no file creation at all.
pub(super) fn redirect_path(command: &str) -> String {
    let spaced = spaced(command);
    let path = first(
        r"(?:^|[^0-9&<>])>>?[[:space:]]*([^[:space:];&|<>]+)",
        &spaced,
    )
    .or_else(|| first(r"tee(?: +-a)? +([^[:space:];&|]+)", &spaced))
    .unwrap_or_default();
    match path.starts_with("/dev/") || path == "-" {
        true => String::new(),
        false => path,
    }
}

/// `grep -oE '<pattern>' | head -1 | sed -E 's/.*//'`: the capture of the first match.
fn first(pattern: &str, text: &str) -> Option<String> {
    compiled(pattern)
        .captures(text)
        .map(|found| found[1].to_string())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r07__a_heredoc_body_is_the_content_between_its_tags() {
        assert_eq!(
            heredoc_bodies("cat > a.md <<'KO'\n한 줄\n또 한 줄\nKO\ncat b\n"),
            "한 줄\n또 한 줄\n"
        );
        // <<- strips nothing here: the tag may be indented and the body is taken as it stands.
        assert_eq!(
            heredoc_bodies("cat <<-EOF\n  body\n  EOF\nrest"),
            "  body\n"
        );
        // A second heredoc in the same command is body too, and a command without one is empty.
        assert_eq!(
            heredoc_bodies("cat <<A\none\nA\ncat <<\"B\"\ntwo\nB"),
            "one\ntwo\n"
        );
        assert_eq!(heredoc_bodies("printf 'hi\\n' > out.txt"), "");
    }

    #[test]
    fn r07__a_commit_message_is_read_from_its_own_argument() {
        assert_eq!(
            commit_text("git commit --no-verify -m \"첫 줄\n\n본문\""),
            "첫 줄\n\n본문\n"
        );
        assert_eq!(commit_text("git commit -m 'one' -m 'two'"), "one\ntwo\n");
        // The heredoc counts only when it feeds the commit itself.
        assert_eq!(
            commit_text("git commit -F - <<MSG\n메시지\nMSG\n"),
            "메시지\n"
        );
        assert_eq!(
            commit_text("cat > a.md <<MSG\n파일 내용\nMSG\ngit commit -m 'x'"),
            "x\n"
        );
        assert_eq!(commit_text("git commit --amend --no-edit"), "");
    }

    #[test]
    fn r07__the_commit_is_recognised_through_its_options() {
        assert!(is_git_commit("git commit -m x"));
        assert!(is_git_commit("git -c commit.gpgsign=false commit -m x"));
        assert!(is_git_commit("git -C /tmp/repo commit"));
        assert!(!is_git_commit("git status"));
        assert!(!is_git_commit("printf 'commit\\n' > out.txt"));
    }

    #[test]
    fn r07__the_redirect_target_is_the_first_file_a_command_names() {
        assert_eq!(redirect_path("printf hi > out.txt"), "out.txt");
        assert_eq!(redirect_path("printf hi >>  logs/a.md 2>&1"), "logs/a.md");
        assert_eq!(redirect_path("cat a | tee -a b.md"), "b.md");
        // A stream that goes nowhere is no file creation, and neither is a bare command.
        assert_eq!(redirect_path("cmd > /dev/null 2>&1"), "");
        assert_eq!(redirect_path("ls -l"), "");
        assert_eq!(redirect_path("cmd 2>&1 | grep x"), "");
    }
}
