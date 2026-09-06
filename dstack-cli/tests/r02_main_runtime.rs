use std::fs;
use std::path::PathBuf;

fn document(path: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(path),
    )
    .unwrap_or_else(|error| panic!("required runtime document {path}: {error}"))
}

#[test]
fn r02_main_runtime_both_hosts_dispatch_main_and_supplied_roles_separately() {
    for (path, provider) in [("claude/CLAUDE.md", "claude"), ("codex/AGENTS.md", "codex")] {
        let entry = document(path);
        for rule in ["supplied role", "dstack-workflow", "runtime.md"] {
            assert!(entry.contains(rule), "{path} cannot dispatch {rule}");
        }
        for role in ["recon", "e2e-runner/verification", "ko-polish"] {
            assert!(
                entry.contains(role),
                "{path} must keep {role} in its supplied role"
            );
        }
        assert!(
            entry.contains(&format!("dstack mode show --host {provider}")),
            "{path} must check its actual main host before starting work"
        );
    }
    let runtime = document("claude/runtime.md");
    for rule in [
        "Agent",
        "sonnet",
        "opus",
        "spawn_agent",
        "fork_turns",
        "none",
        "inherited",
        "gpt-6-astra",
        "high",
        "dstack plan start",
        "dstack run verify",
        "main session",
        "dstack gate",
        "dstack check request",
        "dstack check coverage",
        "dstack check decisions",
        "dstack verify",
        "dstack lint-ko --changed",
        "--quick",
        "--run",
        "--refresh-mode",
        "observed engine",
        "skipped:",
    ] {
        assert!(
            runtime.contains(rule),
            "shared runtime omits executable contract: {rule}"
        );
    }
    for skill in [
        "dstack-workflow",
        "dstack-develop",
        "dstack-quick",
        "dstack-verify",
        "unit-test",
    ] {
        let text = document(&format!("claude/skills/{skill}/SKILL.md"));
        assert!(
            text.contains("runtime.md"),
            "{skill} must use the shared native runtime"
        );
        assert!(
            text.lines().count() <= 300,
            "{skill} exceeds the skill size limit"
        );
    }
}

#[test]
fn r02_main_runtime_review_and_research_instructions_select_the_snapshot_sub() {
    let review = document("claude/skills/codex-review/SKILL.md");
    assert!(review.contains("dstack mode exec review-P1-001 --role review"));
    assert!(review.contains("--context") && review.contains("--output"));
    assert!(review.contains("--quick") && review.contains("codex-review-<NNN>.md"));
    let research = document("claude/skills/codex-research/SKILL.md");
    assert!(research.contains("dstack mode exec research-001 --role research"));
    assert!(research.contains("dstack mode exec research-audit-001 --role audit"));
    for text in [&review, &research] {
        assert!(
            text.contains("sub"),
            "legacy skill names must select the configured sub"
        );
        assert!(
            text.contains("fresh"),
            "a sub pass requires independent context"
        );
        assert!(
            !text.contains("-- codex exec"),
            "orchestration must not pin a provider"
        );
    }
    let unit = document("claude/skills/unit-test/SKILL.md");
    assert!(
        !unit.contains("always `unit_tests: off`"),
        "D-STACK requires its own Rust tests"
    );
}
