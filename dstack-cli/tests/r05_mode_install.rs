use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")).unwrap()
}

fn install(home: &Path, dry: bool) -> String {
    let mut command = Command::new("bash");
    command
        .arg(repo().join("install.sh"))
        .env("HOME", home)
        .env("DSTACK_BACKUP_TS", "mode-install-test");
    if dry {
        command.arg("--dry-run");
    }
    let out = command.output().expect("run scratch-home installer");
    assert!(
        out.status.success(),
        "installer failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn mappings() -> Vec<(String, String)> {
    let mut links = Vec::new();
    for host in ["claude", "codex"] {
        links.push(("claude/runtime.md".into(), format!(".{host}/runtime.md")));
        links.push((
            "dstack-cli/target/release/dstack".into(),
            format!(".{host}/bin/dstack"),
        ));
        for skill in [
            "dstack-workflow",
            "dstack-develop",
            "dstack-verify",
            "dstack-quick",
            "unit-test",
            "codex-review",
            "codex-research",
        ] {
            links.push((
                format!("claude/skills/{skill}"),
                format!(".{host}/skills/{skill}"),
            ));
        }
        for role in ["dstack-reviewer", "dstack-researcher"] {
            links.push((
                format!("codex/skills/{role}"),
                format!(".{host}/skills/{role}"),
            ));
        }
        for agent in [
            "recon",
            "general-dev",
            "frontend-dev",
            "e2e-runner",
            "ko-polish",
        ] {
            links.push((
                format!("claude/agents/{agent}.md"),
                format!(".{host}/agents/{agent}.md"),
            ));
        }
    }
    links
}

fn cli(home: &Path, project: &Path, host: &str, args: &[&str]) -> String {
    let out = Command::new(home.join(format!(".{host}/bin/dstack")))
        .args(args)
        .env("HOME", home)
        .current_dir(project)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "installed {host} command {args:?}: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn installed_modes(home: &Path) {
    let project = home.join("project");
    fs::create_dir(&project).unwrap();
    let git = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(git.status.success(), "initialize scratch project");
    cli(home, &project, "codex", &["init"]);
    for main in ["claude", "codex"] {
        for sub in ["claude", "codex"] {
            cli(
                home,
                &project,
                main,
                &["mode", "set", "--main", main, "--sub", sub],
            );
            let value: serde_json::Value = serde_json::from_str(&cli(
                home,
                &project,
                main,
                &["mode", "show", "--host", main, "--json"],
            ))
            .unwrap();
            assert_eq!(value["main"], main);
            assert_eq!(value["sub"], sub);
            assert_eq!(value["source"], "project");
        }
    }
}

#[test]
fn r05_mode_install_dry_run_exposes_both_complete_runtime_graphs_without_writes() {
    let home = std::env::temp_dir().join(format!("dstack-r05-dry-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    let printed = install(&home, true);
    for (_, target) in mappings() {
        assert!(
            printed.contains(&target),
            "dry-run omits installed runtime target {target}"
        );
    }
    assert_eq!(
        fs::read_dir(&home).unwrap().count(),
        0,
        "dry run changed the scratch home"
    );
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn r05_mode_install_shared_links_are_usable_idempotent_and_preserve_user_config() {
    let home = std::env::temp_dir().join(format!("dstack-r05-live-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(home.join(".codex/skills/personal")).unwrap();
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::write(
        home.join(".codex/config.toml"),
        "model = \"personal-model\"\n",
    )
    .unwrap();
    fs::write(home.join(".codex/AGENTS.md"), "previous instructions\n").unwrap();
    fs::write(
        home.join(".codex/skills/personal/SKILL.md"),
        "personal skill\n",
    )
    .unwrap();
    fs::write(
        home.join(".claude/settings.json"),
        r#"{"unrelated":{"keep":true}}"#,
    )
    .unwrap();
    for _ in 0..2 {
        install(&home, false);
        for (source, target) in mappings() {
            let link = home.join(&target);
            assert_eq!(
                fs::read_link(&link).unwrap_or_else(|e| panic!("{target}: {e}")),
                repo().join(source)
            );
            assert!(
                link.exists(),
                "installed runtime target is dangling: {target}"
            );
        }
        for host in ["claude", "codex"] {
            let out = Command::new(home.join(format!(".{host}/bin/dstack")))
                .arg("help")
                .env("HOME", &home)
                .output()
                .unwrap();
            assert!(out.status.success(), "{host} cannot run its installed CLI");
            assert!(String::from_utf8_lossy(&out.stdout).contains("mode"));
        }
        let settings: serde_json::Value =
            serde_json::from_slice(&fs::read(home.join(".claude/settings.json")).unwrap()).unwrap();
        assert_eq!(settings["unrelated"]["keep"], true);
        assert_eq!(
            fs::read_to_string(home.join(".codex/config.toml")).unwrap(),
            "model = \"personal-model\"\n"
        );
        assert_eq!(
            fs::read_to_string(home.join(".codex/skills/personal/SKILL.md")).unwrap(),
            "personal skill\n"
        );
    }
    assert_eq!(
        fs::read_to_string(home.join(".dstack-backups/mode-install-test/.codex/AGENTS.md"))
            .unwrap(),
        "previous instructions\n"
    );
    installed_modes(&home);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn r05_mode_install_dependencies_follow_provider_selection() {
    let table = fs::read_to_string(repo().join("deps.tsv")).unwrap();
    for provider in ["claude", "codex"] {
        let line = table
            .lines()
            .find(|line| line.split('\t').next() == Some(provider))
            .unwrap_or_else(|| panic!("missing dependency {provider}"));
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields[6], format!("provider={provider}"));
    }
}
