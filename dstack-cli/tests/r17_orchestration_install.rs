#![allow(non_snake_case)]
#[path = "support/main_runtime.rs"]
mod support;
use dstack_cli::selftest::Verdict;
use std::{fs, process::Command};

#[test]
fn R17__doctor_self_runs_positive_and_negative_main_runtime_fixtures() {
    let output = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .args(["doctor", "--self"])
        .env("DSTACK_HOME", support::repo().join("claude"))
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{out}");
    assert!(
        out.contains("main-runtime | good-runtime.md | pass | pass | ok"),
        "{out}"
    );
    assert!(
        out.contains("main-runtime | bad-main-edit.json | reject | reject | ok"),
        "{out}"
    );
    support::fixture(
        "bad-missing-provenance.json",
        Verdict::Reject,
        "missing GSD provenance",
    );
}

#[test]
fn R17__scratch_install_resolves_updated_shared_contract_and_provenance() {
    let home = std::env::temp_dir().join(format!("dstack-r17-install-{}", std::process::id()));
    fs::create_dir(&home).expect("exclusive scratch home");
    let out = Command::new("bash")
        .arg(support::repo().join("install.sh"))
        .env("HOME", &home)
        .env("DSTACK_BACKUP_TS", "r17-scratch")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for (target, source) in [
        (".claude/CLAUDE.md", "claude/CLAUDE.md"),
        (".codex/AGENTS.md", "codex/AGENTS.md"),
    ] {
        assert_eq!(
            fs::read_link(home.join(target)).unwrap(),
            support::repo().join(source)
        );
        let text = fs::read_to_string(home.join(target)).unwrap();
        assert!(dstack_cli::verbs::doctor::main_runtime::check_document(source, &text).is_empty());
    }
    for host in ["claude", "codex"] {
        for (target, source) in [
            ("runtime.md", "claude/runtime.md"),
            ("skills/dstack-workflow", "claude/skills/dstack-workflow"),
            ("skills/codex-review", "claude/skills/codex-review"),
            ("skills/dstack-develop", "claude/skills/dstack-develop"),
            ("skills/dstack-verify", "claude/skills/dstack-verify"),
            ("skills/dstack-quick", "claude/skills/dstack-quick"),
            ("skills/unit-test", "claude/skills/unit-test"),
            ("agents/general-dev.md", "claude/agents/general-dev.md"),
            ("agents/frontend-dev.md", "claude/agents/frontend-dev.md"),
            ("agents/e2e-runner.md", "claude/agents/e2e-runner.md"),
        ] {
            assert_eq!(
                fs::read_link(home.join(format!(".{host}/{target}"))).unwrap(),
                support::repo().join(source)
            );
        }
        let runtime = fs::read_to_string(home.join(format!(".{host}/runtime.md"))).unwrap();
        let issues =
            dstack_cli::verbs::doctor::main_runtime::check_document("claude/runtime.md", &runtime);
        assert!(issues.is_empty(), "installed {host} runtime: {issues:?}");
        let develop =
            fs::read_to_string(home.join(format!(".{host}/skills/dstack-develop/SKILL.md")))
                .unwrap();
        let issues = dstack_cli::verbs::doctor::main_runtime::check_document(
            "claude/skills/dstack-develop/SKILL.md",
            &develop,
        );
        assert!(
            issues.is_empty(),
            "installed {host} provenance/protocol: {issues:?}"
        );
        let output = Command::new(home.join(format!(".{host}/bin/dstack")))
            .args(["doctor", "--self"])
            .env("HOME", &home)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    fs::remove_dir_all(home).unwrap();
}
