#![allow(non_snake_case)]
use std::{fs, path::PathBuf, process::Command};
fn repo() -> PathBuf {
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")).unwrap()
}

#[test]
fn R20__eight_runnable_cases_have_behavior_and_design_rubrics() {
    let dir = repo().join("claude/evals/developer-contracts");
    let rubric = fs::read_to_string(dir.join("README.md")).unwrap_or_default();
    for id in [
        "01-policy",
        "02-meaning",
        "03-state",
        "04-requests",
        "05-publication",
        "06-recurrence",
        "07-prerequisite",
        "08-cohesion",
    ] {
        assert!(rubric.contains(id), "missing rubric: {id}");
        assert!(dir.join(format!("{id}.md")).is_file(), "missing task: {id}");
        assert!(
            dir.join(format!("{id}.py")).is_file(),
            "missing runnable specimen: {id}"
        );
    }
    assert!(rubric.contains("not proof of model behavior"));
    assert!(rubric.contains("fresh"));
    let result = Command::new("python3")
        .arg("-B")
        .arg(dir.join("check.py"))
        .arg("--baseline")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn R19_R20__scratch_install_delivers_both_developers_through_both_installed_binaries() {
    let home = std::env::temp_dir().join(format!("dstack-r20-install-{}", std::process::id()));
    fs::create_dir(&home).unwrap();
    for _ in 0..2 {
        let out = Command::new("bash")
            .arg(repo().join("install.sh"))
            .env("HOME", &home)
            .env("DSTACK_BACKUP_TS", "r20-scratch")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let context = home.join("task.md");
    fs::write(&context, "Only this bounded task\n").unwrap();
    for host in ["claude", "codex"] {
        let common_path = home.join(format!(".{host}/templates/prompts/developer.md"));
        assert_eq!(
            fs::read_link(&common_path).unwrap(),
            repo().join("claude/templates/prompts/developer.md")
        );
        let common = fs::read_to_string(common_path).unwrap();
        assert!(!home.join(format!(".{host}/agents/developer.md")).exists());
        for role in ["frontend-dev", "general-dev"] {
            let agent = fs::read_to_string(home.join(format!(".{host}/agents/{role}.md"))).unwrap();
            let body = agent.split_once("\n---\n").unwrap().1;
            let out = Command::new(home.join(format!(".{host}/bin/dstack")))
                .env("HOME", &home)
                .env_remove("DSTACK_HOME")
                .args(["prompt", "render", "--role", role, "--context"])
                .arg(&context)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let text = String::from_utf8(out.stdout).unwrap();
            assert_eq!(text.matches(&common).count(), 1);
            assert!(text.contains(body));
        }
        for role in ["recon", "e2e-runner", "ko-polish"] {
            let installed =
                fs::read_to_string(home.join(format!(".{host}/agents/{role}.md"))).unwrap();
            assert!(!installed.contains("# Developer implementation responsibility"));
        }
    }
    fs::remove_dir_all(home).unwrap();
}
