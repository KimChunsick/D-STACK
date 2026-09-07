#![allow(non_snake_case)]
#[path = "support/main_runtime.rs"]
mod support;
use dstack_cli::selftest::Verdict;

// Policy/checker evidence only: these do not simulate or prove a host UI's live input.
#[test]
fn R16__checker_requires_each_input_branch_and_rejects_fake_completion() {
    support::fixture("good-runtime.md", Verdict::Pass, "");
    for (bad, diagnostic) in [
        ("question", "missing question handling"),
        ("addition", "missing addition handling"),
        ("conflict", "missing conflict handling"),
        ("stop", "missing stop handling"),
        ("unsupported", "missing unsupported-host handling"),
        ("duplicate", "duplicate launch"),
        ("false-completion", "false completion"),
        ("question-contradiction", "question restart"),
        ("addition-contradiction", "addition blanket restart"),
        ("conflict-contradiction", "manual state reset"),
        ("stop-contradiction", "stop treated as done"),
        ("unsupported-contradiction", "unsupported input promise"),
    ] {
        support::fixture(&format!("bad-{bad}.json"), Verdict::Reject, diagnostic);
    }
}

#[test]
fn R16__busy_plan_guards_preserve_state_and_never_advise_fake_completion() {
    use dstack_cli::selftest::sandbox::Sandbox;
    let ctx = support::context();
    let sandbox = Sandbox::new(&ctx).unwrap();
    for args in [
        vec!["milestone", "add", "core"],
        vec![
            "plan",
            "add",
            "first",
            "--milestone",
            "M1",
            "--files",
            "a.rs",
        ],
    ] {
        let (exit, output) = sandbox.dsx(&ctx, &args).unwrap();
        assert_eq!(exit, 0, "{output}");
    }
    let path = sandbox.run_dir().unwrap().join("plan.json");
    let mut plan: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    plan["plans"][0]["status"] = "in-progress".into();
    let before = serde_json::to_vec_pretty(&plan).unwrap();
    std::fs::write(&path, &before).unwrap();
    for args in [
        vec!["plan", "remove", "P1"],
        vec!["plan", "edit", "P1", "--files", "b.rs"],
        vec![
            "plan", "insert", "urgent", "--after", "P1", "--files", "b.rs",
        ],
    ] {
        let (exit, output) = sandbox.dsx(&ctx, &args).unwrap();
        assert_eq!(exit, 1, "{args:?}: {output}");
        assert!(
            output.contains("in-progress") || output.contains("in progress"),
            "{output}"
        );
        assert!(!output.contains("dstack plan done P1 first"), "{output}");
        assert!(!output.contains("or reset it by hand"), "{output}");
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}

// accept: 질문·추가 요구·충돌 변경·중지·호스트 기능 부재의 처리 규칙이 실행 흐름에 연결되고 회귀 검사로 확인돼요. 호스트 화면이나 실제 동시 대화를 시험하지 않은 경우 그 한계를 명시하며 지원하지 않는 입력 동작을 보장하지 않아요.
#[test]
fn R16__busy_subtree_insert_preserves_state_and_advises_supported_transition() {
    use dstack_cli::selftest::sandbox::Sandbox;
    let ctx = support::context();
    let sandbox = Sandbox::new(&ctx).unwrap();
    for args in [
        vec!["milestone", "add", "core"],
        vec![
            "plan",
            "add",
            "first",
            "--milestone",
            "M1",
            "--files",
            "a.rs",
        ],
        vec![
            "plan",
            "add",
            "dependent",
            "--milestone",
            "M1",
            "--files",
            "b.rs",
            "--deps",
            "P1",
        ],
    ] {
        let (exit, output) = sandbox.dsx(&ctx, &args).unwrap();
        assert_eq!(exit, 0, "{output}");
    }
    let path = sandbox.run_dir().unwrap().join("plan.json");
    let mut plan: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    plan["plans"][1]["status"] = "in-progress".into();
    let before = serde_json::to_vec_pretty(&plan).unwrap();
    std::fs::write(&path, &before).unwrap();

    let (exit, output) = sandbox
        .dsx(
            &ctx,
            &[
                "plan", "insert", "urgent", "--after", "P1", "--files", "c.rs",
            ],
        )
        .unwrap();
    println!("{output}");
    assert_eq!(exit, 1, "{output}");
    assert!(
        output.contains("refused: the affected subtree of P1 is in progress (P2)"),
        "{output}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    for advisory in [
        "stop the affected workers",
        "keep the Plans pending/blocked until a supported CLI transition is available",
        "never mark unfinished work done or reset state by hand",
    ] {
        assert!(output.contains(advisory), "missing {advisory}: {output}");
    }
    assert!(!output.contains("finish or reset those plans"), "{output}");
}
