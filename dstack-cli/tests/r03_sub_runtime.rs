mod support {
    pub mod mode_runtime;
}
use std::fs;
use support::mode_runtime::Scratch;

#[test]
fn r03_sub_runtime_all_combinations_use_the_selected_independent_role() {
    for main in ["claude", "codex"] {
        for sub in ["claude", "codex"] {
            for role in ["review", "research", "audit"] {
                let t = Scratch::new(main, sub);
                let output = t.run(role, &[]);
                assert!(
                    output.status.success(),
                    "{main}/{sub}/{role}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(t.read("trace/provider"), format!("{sub}\n"));
                assert_eq!(
                    t.read("trace/cwd").trim(),
                    t.0.join("work").canonicalize().unwrap().to_str().unwrap()
                );
                assert_eq!(t.read("result.md"), "R03 raw result\nVERDICT: PASS\n");
                let prompt = t.read("trace/stdin");
                let canonical = t
                    .command()
                    .args([
                        "prompt",
                        "render",
                        "--role",
                        role,
                        "--context",
                        "context.md",
                    ])
                    .output()
                    .unwrap();
                assert!(canonical.status.success());
                assert_eq!(prompt.as_bytes(), canonical.stdout);
                let args = t.read("trace/argv");
                assert!(!args.contains("--resume") && !args.contains("--continue"));
                let argv: Vec<&str> = args.lines().collect();
                if sub == "codex" {
                    assert_eq!(
                        &argv[..11],
                        &[
                            "exec",
                            "--ignore-user-config",
                            "-m",
                            "gpt-6-astra",
                            "-c",
                            "model_reasoning_effort=high",
                            "--sandbox",
                            "read-only",
                            "--json",
                            "-o",
                            t.0.join(".dstack/local/exec/check/result.txt")
                                .to_str()
                                .unwrap()
                        ]
                    );
                    assert_eq!(argv[11], "-C");
                    assert_eq!(
                        argv[12],
                        t.0.join("work").canonicalize().unwrap().to_str().unwrap()
                    );
                    assert_eq!(argv.last(), Some(&"-"));
                    assert_eq!(args.contains("tools.web_search=true"), role != "review");
                } else {
                    let tools = if role == "review" {
                        "Read,Glob,Grep"
                    } else {
                        "Read,Glob,Grep,WebSearch,WebFetch"
                    };
                    assert_eq!(
                        argv,
                        [
                            "--print",
                            "--model",
                            "opus",
                            "--effort",
                            "high",
                            "--output-format",
                            "json",
                            "--no-session-persistence",
                            "--tools",
                            tools,
                            "--strict-mcp-config",
                            "--mcp-config",
                            "{\"mcpServers\":{}}",
                            "--permission-mode",
                            "dontAsk",
                            "--permission-prompts",
                            "none"
                        ]
                    );
                }
                let usage: serde_json::Value =
                    serde_json::from_str(&t.read(".dstack/local/exec/check/usage.json")).unwrap();
                assert_eq!(usage["provider"], sub);
                assert_eq!(usage["status"], "measured");
            }
        }
    }
}

#[test]
fn r03_sub_runtime_dry_run_is_read_only_and_resolves_snapshots() {
    let t = Scratch::new("codex", "claude");
    fs::create_dir_all(t.0.join(".dstack/quick/other")).unwrap();
    fs::write(
        t.0.join(".dstack/quick/other/mode.json"),
        r#"{"main":"claude","sub":"codex"}"#,
    )
    .unwrap();
    fs::write(
        t.0.join(".dstack/project/mode.json"),
        r#"{"main":"codex","sub":"codex"}"#,
    )
    .unwrap();
    for (selector, expected) in [
        (vec![], "claude"),
        (vec!["--run", "sample"], "claude"),
        (vec!["--quick", "other"], "codex"),
    ] {
        let before = t.tree();
        let mut args = vec!["--dry-run"];
        args.extend(selector);
        let output = t.run("audit", &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(plan["provider"], expected);
        assert_eq!(plan["role"], "audit");
        assert_eq!(plan["argv"][0], expected);
        assert_eq!(plan["output"], t.0.join("result.md").to_str().unwrap());
        assert_eq!(
            plan["cwd"],
            t.0.join("work").canonicalize().unwrap().to_str().unwrap()
        );
        assert_eq!(before, t.tree(), "dry-run changed files or metadata");
    }
    let output = t.run("research", &["--quick", "other"]);
    assert!(output.status.success());
    assert_eq!(t.read("trace/provider"), "codex\n");
}

#[test]
fn r03_sub_runtime_rejects_failed_missing_ambiguous_and_unstructured_results() {
    for sub in ["claude", "codex"] {
        let mut scenarios = vec![
            "exit",
            "malformed",
            "failed",
            "missing",
            "empty",
            "multiple",
        ];
        scenarios.extend(if sub == "claude" {
            vec!["subtype", "marker", "duplicate-key"]
        } else {
            vec!["no-completion", "symlink"]
        });
        for scenario in scenarios {
            let t = Scratch::new(sub, sub);
            t.scenario(scenario);
            let output = t.run("review", &[]);
            assert!(!output.status.success(), "accepted {sub}/{scenario}");
            assert!(
                !t.0.join("result.md").exists(),
                "published {sub}/{scenario}"
            );
            assert_eq!(
                t.read("trace/calls"),
                "call\n",
                "unexpected retry or fallback"
            );
            assert_eq!(
                t.read(".dstack/local/exec/check/exit"),
                if scenario == "exit" { "23\n" } else { "0\n" }
            );
            if scenario == "exit" {
                assert_eq!(output.status.code(), Some(23));
                assert!(t
                    .read(".dstack/local/exec/check/err.txt")
                    .contains("refused"));
            }
            assert!(t.0.join(".dstack/local/exec/check/usage.json").is_file());
        }
    }
}

#[test]
fn r03_sub_runtime_existing_outputs_and_invalid_paths_stop_before_execution() {
    use std::os::unix::fs::symlink;
    for scenario in ["existing", "dangling", "directory", "context", "worktree"] {
        let t = Scratch::new("claude", "claude");
        match scenario {
            "existing" => fs::write(t.0.join("result.md"), "keep").unwrap(),
            "dangling" => symlink("missing", t.0.join("result.md")).unwrap(),
            "directory" => fs::create_dir(t.0.join("result.md")).unwrap(),
            "context" => fs::remove_file(t.0.join("context.md")).unwrap(),
            "worktree" => fs::remove_dir(t.0.join("work")).unwrap(),
            _ => unreachable!(),
        }
        let before = t.tree();
        assert!(
            !t.run("review", &[]).status.success(),
            "accepted {scenario}"
        );
        assert_eq!(before, t.tree(), "mutated state for {scenario}");
    }
    for extra in [
        vec!["--run", "missing"],
        vec!["--run", "../sample"],
        vec!["--run", "sample", "--quick", "other"],
        vec!["--output", "other.md"],
        vec!["--wat"],
        vec!["--role", "worker"],
    ] {
        let t = Scratch::new("claude", "codex");
        let before = t.tree();
        assert!(!t.run("review", &extra).status.success());
        assert_eq!(before, t.tree());
    }
}

#[test]
fn r03_sub_runtime_never_falls_back_or_overwrites_a_late_output() {
    for sub in ["claude", "codex"] {
        let t = Scratch::new(sub, sub);
        fs::remove_file(t.0.join("bin").join(sub)).unwrap();
        let output = t.run("review", &[]);
        assert_eq!(output.status.code(), Some(127));
        assert!(!t.0.join("trace/calls").exists());
        assert!(!t.0.join("result.md").exists());
        let t = Scratch::new(sub, sub);
        t.scenario("race");
        assert!(!t.run("review", &[]).status.success());
        assert_eq!(t.read("result.md"), "keep");
        assert!(!fs::read_dir(&t.0).unwrap().any(|f| f
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".dstack-result-")));
    }
}

#[test]
fn r03_sub_runtime_generic_capture_refuses_inflight_collision_and_keeps_suffixes() {
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let t = Scratch::new("claude", "codex");
    let fake = t.0.join("bin/codex");
    let args = ["exec", "queued", "--", fake.to_str().unwrap()];
    let first = t
        .command()
        .env("MODE_FAKE", "block")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !t.0.join("trace/started").is_file() {
        assert!(Instant::now() < deadline, "fake provider failed to start");
        std::thread::sleep(Duration::from_millis(10));
    }
    let first_cmd = t.read(".dstack/local/exec/queued/cmd");
    assert!(!t.command().args(args).output().unwrap().status.success());
    assert_eq!(t.read(".dstack/local/exec/queued/cmd"), first_cmd);
    assert_eq!(t.read("trace/calls"), "call\n");
    fs::write(t.0.join("trace/release"), "").unwrap();
    assert!(first.wait_with_output().unwrap().status.success());
    assert!(t.command().args(args).output().unwrap().status.success());
    assert_eq!(t.read(".dstack/local/exec/queued/exit"), "0\n");
    assert_eq!(t.read(".dstack/local/exec/queued.1/exit"), "0\n");
}

#[test]
fn r03_sub_runtime_preserves_nested_markers_and_repeated_sessions_are_fresh() {
    let t = Scratch::new("claude", "claude");
    for n in 0..2 {
        let output_name = format!("result-{n}.md");
        let output = t
            .command()
            .env("CLAUDECODE", "1")
            .env("CLAUDE_CODE_CHILD_SESSION", "1")
            .args([
                "mode",
                "exec",
                "fresh",
                "--role",
                "review",
                "--context",
                "context.md",
                "--output",
                &output_name,
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(t.read("trace/markers"), "1\n1\n");
        assert!(!t.read("trace/argv").contains("--resume"));
    }
    assert_eq!(t.read("trace/calls"), "call\ncall\n");
    assert_eq!(
        t.read(".dstack/local/exec/fresh/stdin.txt"),
        t.read(".dstack/local/exec/fresh.1/stdin.txt")
    );
}
