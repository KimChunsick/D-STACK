#![allow(non_snake_case)]
#[path = "support/main_runtime.rs"]
mod support;
use dstack_cli::selftest::Verdict;

#[test]
fn R15__verification_dispatch_requires_receipt_and_rejects_table_only_with_good_contract_retained()
{
    // accept: Claude·Codex 공통 규칙과 관련 스킬·프롬프트에서 역할 경계와 짧은 반환 계약이 일치하고, 메인의 직접 구현·무거운 검사 및 오래된 문맥 재사용을 허용하는 잘못된 고정물을 회귀 검사가 거절해요.
    support::fixture("good-verify.md", Verdict::Pass, "");
    support::fixture(
        "bad-verify-table-only.json",
        Verdict::Reject,
        "table-only return omits compact receipt",
    );
    support::fixture(
        "bad-verify-missing-receipt.json",
        Verdict::Reject,
        "missing compact receipt",
    );
}

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
