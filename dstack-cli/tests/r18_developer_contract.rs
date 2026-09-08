#![allow(non_snake_case)]
use std::{fs, path::PathBuf};

fn read(path: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(path),
    )
    .unwrap_or_default()
}

#[test]
fn R18__developers_own_semantic_boundaries_with_proportional_judgment() {
    let common = read("claude/templates/prompts/developer.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for concept in [
        "invariants",
        "responsible module",
        "bypass",
        "state owner",
        "lifetime",
        "allowed and forbidden transitions",
        "external effects",
        "same business meaning",
        "reason to change",
        "existing abstractions",
        "misuse",
        "caller",
        "business decisions",
        "coordination",
        "I/O",
        "storage",
        "recovery",
        "presentation",
        "cohesive algorithm",
        "transient failure",
        "permanent refusal",
        "facts from inference",
        "completion from uncertainty",
        "exclusion from recovery authority",
        "identity",
        "display time",
        "derived state",
        "retry",
        "duplicate",
        "other callers",
        "prerequisite",
        "preserve",
        "compact receipt",
    ] {
        assert!(
            common.contains(concept),
            "missing shared responsibility: {concept}"
        );
    }
    for (role, concepts) in [
        (
            "frontend-dev",
            vec![
                "server state",
                "hook",
                "request identity",
                "cancellation",
                "stale responses",
                "duplicate submission",
                "loading",
                "empty",
                "permission",
                "design system",
                "keyboard",
                "focus",
                "accessibility",
                "server policy",
            ],
        ),
        (
            "general-dev",
            vec![
                "domain",
                "adapter",
                "CLI, API and batch",
                "transaction",
                "atomicity",
                "consumer consistency",
                "durable facts",
                "idempotency",
                "environment",
                "external error",
            ],
        ),
    ] {
        let text = read(&format!("claude/agents/{role}.md"));
        for concept in concepts {
            assert!(
                text.split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .contains(concept),
                "{role} missing {concept}"
            );
        }
        assert!(!text.contains("no abstraction for one use"));
        assert!(!text.contains("Tokens spent on edits are best minimised"));
        assert!(text.contains(
            "model: opus\neffort: max\nmaxTurns: 80\ntools: Read, Edit, Write, Grep, Glob, Bash"
        ));
    }
}
