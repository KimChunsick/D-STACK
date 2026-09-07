#![allow(non_snake_case)]
#[path = "support/main_runtime.rs"]
mod support;
use dstack_cli::selftest::Verdict;

#[test]
fn R15__checker_rejects_main_takeover_stale_context_and_missing_receipts() {
    for good in [
        "good-runtime.md",
        "good-worker.md",
        "good-workflow.md",
        "good-develop.md",
    ] {
        support::fixture(good, Verdict::Pass, "");
    }
    for (bad, diagnostic) in [
        ("main-edit", "forbidden main work"),
        ("main-tests", "forbidden main work"),
        ("main-heavy", "forbidden main work"),
        ("raw-log-return", "raw-log return"),
        ("stale-retry", "stale worker reuse"),
        ("failure-takeover", "failure takeover"),
        ("missing-receipt", "missing compact receipt"),
        ("tiny-exception", "direct-edit exception"),
        ("missing-reference", "missing runtime protocol reference"),
        ("premature-close", "premature Plan completion"),
        (
            "plan-checklist-order",
            "Plan checklist must seal independent review before completion",
        ),
    ] {
        support::fixture(&format!("bad-{bad}.json"), Verdict::Reject, diagnostic);
    }
}

#[test]
fn R15__actual_canonical_contracts_pass_and_wrapped_contradictions_name_the_line() {
    use dstack_cli::verbs::doctor::main_runtime::{check_document, documents};
    for path in documents() {
        let text = std::fs::read_to_string(support::repo().join(path)).unwrap();
        assert!(
            check_document(path, &text).is_empty(),
            "{path}: {:?}",
            check_document(path, &text)
        );
    }
    let source = std::fs::read_to_string(support::repo().join("claude/runtime.md")).unwrap();
    let bad = format!("{source}\nMain may directly implement\nsmall changes.\n");
    let issue = check_document("claude/runtime.md", &bad)
        .into_iter()
        .find(|issue| issue.message == "forbidden main work")
        .expect("wrapped contradiction rejected");
    assert_eq!(issue.line, source.lines().count() + 2);
    let good = format!("{source}\nNever let main implement changes; delegate to fresh workers.\n");
    assert!(check_document("claude/runtime.md", &good).is_empty());
}
