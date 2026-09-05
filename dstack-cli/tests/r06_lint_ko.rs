// tests/r06_lint_ko.rs
// R06: the Korean rule table under the Rust regex engine. grep -nEo is the reference — every
// regex row of the live table is run over one corpus by both engines and the printed matches are
// compared byte for byte — and a row that cannot be run is reported by its id.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::PathBuf;
use std::process::Command;

use dstack_cli::verbs::lint::rules::{hits, Matcher, Table};
use dstack_cli::verbs::lint::scope::{glob, Scopes};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn live_table() -> Table {
    Table::load(&repo().join("claude/lint/ko-rules.tsv")).expect("the live rule table loads")
}

/// Lines that put the engine where POSIX and the regex crate disagree: alternations whose
/// branches match different lengths at the same position, anchors, `.*` and repeated matches.
const CRAFTED: &str = "\
게이트\n\
게이트웨이는 게이트가 아니에요\n\
이 게이트는 통과 조건이에요.\n\
에서의 에의 으로의 으로부터의 에로의\n\
필요한 것은 방향이다. 중요한 것은 속도입니다.\n\
핵심은 속도가 아니라 방향이다.\n\
fail-closed 와 `fail-open` 과 x-fail-closed\n\
즉, 시작해요. 그리고즉, 끝이에요.\n\
정본 정본 정본\n\
판별유니언과 판별 유니언\n\
AI에 의해 만들어요. 사람에 의해 고쳐요.\n\
단순한 도구를 넘어 단순히 손을 넘어\n\
되어진다 되어집니다 지게 된다\n\
과제도 남아있다 과제도 남아 있다\n\
이 값은 — 참고로 — 기본값이에요.\n\
크게 세 가지로 나눠요. 다음과 같은 몇 가지가 있어요.\n\
";

/// One scratch file with the text both engines are asked about.
fn write_corpus(text: &str, name: &str) -> (PathBuf, String) {
    let path = std::env::temp_dir().join(format!("dstack-r06-{name}-{}.txt", std::process::id()));
    std::fs::write(&path, text).expect("write the corpus");
    (path, text.to_string())
}

/// Every example of the table plus the crafted lines: one file both engines scan.
fn corpus(table: &Table) -> (PathBuf, String) {
    let mut text = String::new();
    for rule in &table.rules {
        text.push_str(&rule.example);
        text.push('\n');
    }
    text.push_str(CRAFTED);
    write_corpus(&text, "corpus")
}

/// What `grep -nEo -e <pattern> <file>` prints, line by line. _ko_locale forces a UTF-8 locale
/// on grep, so the reference is measured in exactly that locale.
fn grep(pattern: &str, file: &PathBuf) -> Vec<String> {
    let out = Command::new("grep")
        .args(["-nEo", "-e", pattern])
        .arg(file)
        .env("LC_ALL", "en_US.UTF-8")
        .output()
        .expect("run grep");
    let text = String::from_utf8(out.stdout).expect("utf-8");
    text.lines().map(str::to_string).collect()
}

#[test]
fn r06__every_regex_row_prints_what_grep_prints() {
    let table = live_table();
    let (path, text) = corpus(&table);
    let mut checked = 0;
    for rule in &table.rules {
        let matcher = match &rule.matcher {
            Some(matcher) => matcher,
            None => continue,
        };
        let mine: Vec<String> = hits(matcher, &text)
            .iter()
            .map(|(line, matched)| format!("{line}:{matched}"))
            .collect();
        // Every example is in the corpus, so a regex row that matches nothing there would make
        // the comparison meaningless.
        assert!(
            !mine.is_empty(),
            "rule {} matches nothing in the corpus",
            rule.id
        );
        assert_eq!(
            mine,
            grep(&rule.pattern, &path),
            "rule {} scans the corpus differently from grep",
            rule.id
        );
        checked += 1;
    }
    std::fs::remove_file(&path).expect("clean up");
    assert_eq!(checked, 46, "every regex row of the live table is compared");
}

#[test]
fn r06__the_live_table_counts_its_rows_the_way_the_shell_counts_them() {
    let table = live_table();
    assert_eq!(table.rules.len(), 67);
    assert_eq!(table.regex_n, 46);
    assert_eq!(table.judgment_n, 21);
    assert!(table
        .rules
        .iter()
        .all(|rule| { (rule.kind == "regex") == rule.matcher.is_some() }));
}

#[test]
fn r06__a_missing_or_empty_table_is_reported_by_path() {
    let missing = repo().join("claude/lint/no-such-table.tsv");
    let error = Table::load(&missing)
        .err()
        .expect("a missing table cannot be decided");
    assert_eq!(error.code(), 2);
    assert_eq!(
        error.message(),
        format!("rule table missing: {} (R91)", missing.display())
    );

    let empty = std::env::temp_dir().join(format!("dstack-r06-empty-{}.tsv", std::process::id()));
    std::fs::write(&empty, "# only a comment\nid\tkind\tpattern\n").expect("write");
    let error = Table::load(&empty)
        .err()
        .expect("a table with no rows cannot be decided");
    assert_eq!(
        error.message(),
        format!("rule table has no rows: {}", empty.display())
    );
    std::fs::remove_file(&empty).expect("clean up");
}

#[test]
fn r06__a_pattern_that_does_not_compile_is_reported_by_id() {
    let error = Matcher::compile("K99", "(unclosed")
        .err()
        .expect("an uncompilable pattern");
    assert_eq!(error.code(), 2);
    assert!(
        error
            .message()
            .starts_with("rule K99: pattern does not compile: "),
        "the report names the rule id: {}",
        error.message()
    );
}

/// The three shapes where leftmost-first and leftmost-longest disagree, each checked against the
/// engine that decides the question.
#[test]
fn r06__an_alternation_answers_with_the_longest_match_at_every_depth() {
    let cases = [
        ("K87", "가나|가나다", "가나다"),
        ("K88", "가(나|나다)", "가나다"),
        // A branch that is no literal prefix of the other and still matches longer.
        ("K89", "(a|[a]b?)", "ab"),
        ("K90", "(ab)?a|ab", "aba"),
        // The two live rows where the question is not academic: a five-branch alternation whose
        // branches share a prefix, and a rule with a `.*` between two alternations.
        (
            "K61",
            "에서의|에로의|으로의|에의|으로부터의",
            "회의에서의 결정과 회의에의 결정",
        ),
        (
            "K68",
            "(필요한|중요한) 것은 .*(이다|입니다|이에요|예요)\\.",
            "필요한 것은 방향이다. 중요한 것은 속도입니다.",
        ),
    ];
    for (id, pattern, line) in cases {
        let matcher = Matcher::compile(id, pattern).expect("compiles");
        let (path, _) = write_corpus(&format!("{line}\n"), id);
        let mine: Vec<String> = hits(&matcher, &format!("{line}\n"))
            .iter()
            .map(|(number, matched)| format!("{number}:{matched}"))
            .collect();
        assert_eq!(mine, grep(pattern, &path), "{id}: {pattern} over {line}");
        std::fs::remove_file(&path).expect("clean up");
    }
}

#[test]
fn r06__the_scope_glob_follows_the_bash_case_semantics() {
    assert!(glob("claude/lint/**", "claude/lint/a/b.md"));
    // No FNM_PATHNAME in a case pattern: * covers / too.
    assert!(glob("*.ko.json", "a/b.ko.json"));
    assert!(glob("docs/**/*.md", "docs/a/b.md"));
    assert!(!glob("docs/**/*.md", "docs/x.md"));
    assert!(glob("?.md", "a.md"));
    assert!(!glob("?.md", "ab.md"));
    assert!(glob("[ab].md", "b.md"));
    assert!(!glob("[!ab].md", "b.md"));
    assert!(!glob("[^ab].md", "b.md"));
    assert!(glob("[a-c].md", "c.md"));
    assert!(glob("[]a].md", "].md"));
    assert!(glob("a\\*b", "a*b"));
    assert!(!glob("a\\*b", "axb"));
    // An unterminated [ is a literal [ for bash.
    assert!(glob("a[", "a["));
    assert!(glob("*", ""));
}

#[test]
fn r06__the_scope_table_answers_the_repository_paths() {
    let scopes = Scopes::load(&repo(), &repo().join("claude"));
    assert!(scopes.path.is_some(), "this checkout has a scope table");
    assert_eq!(scopes.of("README.md"), "ko-haeyo");
    assert_eq!(scopes.of("claude/lint/ko-rules.tsv"), "exempt");
    assert_eq!(scopes.of("claude/skills/a/b.md"), "en");
    assert_eq!(scopes.of("codex/x"), "en");
    assert_eq!(scopes.of("app/ko/messages.ko.json"), "ko-data");
    assert_eq!(scopes.of("weird/x.txt"), "");
}

// ── the verb ───────────────────────────────────────────────────────────────────────────

use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::verbs::lint;

/// A Context whose self_exe is the binary this test run built, as the fixture runner builds one.
fn context() -> Context {
    let home = Home::resolve().expect("the repository of this test binary");
    let registry = Rc::new(Registry::new(dstack_cli::verbs::all_verbs()));
    Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::clone(&registry),
    )
}

/// A directory outside any repository: lint-ko needs neither a store nor a git repository.
fn elsewhere(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-r06-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    std::fs::canonicalize(&dir).expect("its physical path")
}

/// One call of a dispatcher: (stdout, stderr, exit code).
fn call(command: &mut Command, dir: &PathBuf, args: &[&str]) -> (String, String, i32) {
    let out = command
        .args(args)
        .current_dir(dir)
        .env("DSTACK_KO_RULES", repo().join("claude/lint/ko-rules.tsv"))
        .output()
        .expect("run the dispatcher");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().expect("an exit code"),
    )
}

fn rust(dir: &PathBuf, args: &[&str]) -> (String, String, i32) {
    call(&mut Command::new(env!("CARGO_BIN_EXE_dstack")), dir, args)
}

fn shell(dir: &PathBuf, args: &[&str]) -> (String, String, i32) {
    let mut command = Command::new("bash");
    command.arg(shell_ref::dispatcher());
    call(&mut command, dir, args)
}

#[test]
fn r13__lint_ko_keeps_the_exit_code_contract_without_a_store() {
    let dir = elsewhere("contract");
    std::fs::write(dir.join("clean.md"), "이 도구는 저장소 안에서 돌아요.\n").expect("write");
    std::fs::write(dir.join("README.md"), "정본은 이 파일이에요.\n").expect("write");

    // 0: a path no scope row claims is passed and counted as unclassified.
    assert_eq!(rust(&dir, &["lint-ko", "clean.md"]).2, 0);
    // 1: a checked condition failed — an S1 hit in a blocking scope.
    let (stdout, stderr, code) = rust(&dir, &["lint-ko", "README.md"]);
    assert_eq!((code, stderr.as_str()), (1, ""));
    assert!(
        stdout.contains("README.md:1: K01 (S1) matched '정본'"),
        "{stdout}"
    );
    // 2: cannot decide — the rule table is not there.
    let (_, stderr, code) = rust(
        &dir,
        &["lint-ko", "--rules", "/no/such/table.tsv", "README.md"],
    );
    assert_eq!(code, 2);
    assert_eq!(
        stderr,
        "dstack: rule table missing: /no/such/table.tsv (R91)\n"
    );
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r11__the_refusals_read_exactly_as_the_shell_writes_them() {
    let dir = elsewhere("usage");
    std::fs::write(dir.join("README.md"), "정본은 이 파일이에요.\n").expect("write");
    let wrong: [&[&str]; 8] = [
        &["lint-ko"],
        &["lint-ko", "--bogus"],
        &["lint-ko", "-r"],
        &["lint-ko", "--rules=x", "README.md"],
        &["lint-ko", "--path"],
        &["lint-ko", "--scope"],
        &["lint-ko", "--rules"],
        &["lint-ko", "--rules", "/no/such/table.tsv", "README.md"],
    ];
    for args in wrong {
        assert_eq!(
            rust(&dir, args),
            shell(&dir, args),
            "dstack {}",
            args.join(" ")
        );
    }
    std::fs::remove_dir_all(&dir).expect("clean up");
}

/// bad-* must be rejected and good-* must pass, for both checkers of this noun.
fn run_fixtures(checker: &str) {
    let mut ctx = context();
    let selftests = lint::selftests();
    let selftest = selftests
        .iter()
        .find(|s| s.checker() == checker)
        .unwrap_or_else(|| panic!("{checker} is registered"));
    let dir = repo().join("claude/lint/fixtures").join(checker);
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).expect("the fixture directory") {
        let fixture = entry.expect("a fixture").path();
        let name = fixture
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .to_string();
        let wanted = match name.starts_with("bad-") {
            true => Verdict::Reject,
            false => Verdict::Pass,
        };
        let verdict = selftest
            .run(&mut ctx, &fixture)
            .unwrap_or_else(|e| panic!("{checker}/{name}: {e}"));
        assert_eq!(verdict, wanted, "{checker}/{name}");
        seen += 1;
    }
    assert!(seen >= 2, "{checker} has fixtures on both sides");
}

#[test]
fn r05__the_lint_ko_fixtures_are_judged_the_way_they_are_named() {
    run_fixtures("lint-ko");
}

#[test]
fn r05__the_lint_ko_rules_fixtures_are_judged_the_way_they_are_named() {
    run_fixtures("lint-ko-rules");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r04__the_lint_ko_step_of_the_parity_harness_reports_no_difference() {
    let out = Command::new("bash")
        .arg(repo().join("dstack-cli/parity/run.sh"))
        .args(["--shell-ref", "shell-final"])
        .args(["--rust", env!("CARGO_BIN_EXE_dstack")])
        .args(["--only", "24-lint-ko"])
        .output()
        .expect("run the parity harness");
    let report = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        String::from_utf8_lossy(&out.stderr).is_empty(),
        "the harness aborted: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let last = report.lines().last().expect("a report line");
    let (steps, differing) = last
        .strip_prefix("steps ")
        .and_then(|rest| rest.split_once(", differing "))
        .expect("the report ends with the steps line");
    assert!(steps.parse::<usize>().expect("a count") >= 30, "{report}");
    assert_eq!(differing, "0", "{report}");
}

#[test]
fn r05__a_checker_that_cannot_run_is_not_a_verdict() {
    let mut ctx = context();
    let selftests = lint::selftests();
    let by = |name: &str| {
        selftests
            .iter()
            .find(|s| s.checker() == name)
            .unwrap_or_else(|| panic!("{name} is registered"))
    };
    // A fixture that cannot be put in front of the checker is a failure of the runner.
    let error = by("lint-ko")
        .run(&mut ctx, std::path::Path::new("/no/such/fixture.md"))
        .err()
        .expect("the fixture cannot be copied");
    assert_eq!(error.code(), 2);
    // selftest_lint_ko_rules answers for a missing table itself, and its answer is reject.
    let verdict = by("lint-ko-rules")
        .run(&mut ctx, std::path::Path::new("/no/such/table.tsv"))
        .expect("a verdict");
    assert_eq!(verdict, Verdict::Reject);
}

#[test]
fn r06__the_rule_checker_names_every_row_it_rejects() {
    let mut ctx = context();
    let selftests = lint::selftests();
    let checker = selftests
        .iter()
        .find(|s| s.checker() == "lint-ko-rules")
        .expect("lint-ko-rules is registered");

    // The fixture whose K02 example stopped matching its own pattern.
    let fixture = repo().join("claude/lint/fixtures/lint-ko-rules/bad-example-mismatch.tsv");
    ctx.out.begin_capture();
    let verdict = checker.run(&mut ctx, &fixture).expect("a verdict");
    let (said, _) = ctx.out.end_capture();
    assert_eq!(verdict, Verdict::Reject);
    assert_eq!(said, "rule K02: example does not match its pattern\n");

    // A row this engine cannot run at all is a rejection too, named by its id — never a refusal
    // to decide, because an unrunnable table is what this checker exists to catch.
    let broken = std::env::temp_dir().join(format!("dstack-r06-broken-{}.tsv", std::process::id()));
    std::fs::write(
        &broken,
        "id\tkind\tpattern\tseverity\treplacement\texample\tsource\tlevel\n\
         K99\tregex\t(unclosed\tS1\tx\t(unclosed\tv1\tword\n",
    )
    .expect("write the broken table");
    ctx.out.begin_capture();
    let verdict = checker.run(&mut ctx, &broken).expect("a verdict");
    let (said, _) = ctx.out.end_capture();
    assert_eq!(verdict, Verdict::Reject);
    assert!(
        said.starts_with("rule K99: pattern does not compile: "),
        "{said}"
    );
    assert_eq!(said.lines().count(), 1, "one line per failing row: {said}");
    std::fs::remove_file(&broken).expect("clean up");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r06__both_dispatchers_print_the_same_lines_for_every_fixture() {
    let dirs = [elsewhere("fixtures-shell"), elsewhere("fixtures-rust")];
    for dir in &dirs {
        let done = Command::new("git")
            .args(["init", "-q"])
            .current_dir(dir)
            .output()
            .expect("run git");
        assert!(done.status.success(), "git init in the scratch repository");
    }
    let mut seen = 0;
    for entry in std::fs::read_dir(repo().join("claude/lint/fixtures/lint-ko")).expect("fixtures") {
        let fixture = entry.expect("a fixture").path();
        let name = fixture
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .to_string();
        for dir in &dirs {
            std::fs::copy(&fixture, dir.join("README.md")).expect("copy the fixture");
        }
        let args = ["lint-ko", "--report", "README.md"];
        assert_eq!(shell(&dirs[0], &args), rust(&dirs[1], &args), "{name}");
        seen += 1;
    }
    assert_eq!(seen, 4, "every fixture of the checker was run through both");
    for dir in &dirs {
        std::fs::remove_dir_all(dir).expect("clean up");
    }
}
