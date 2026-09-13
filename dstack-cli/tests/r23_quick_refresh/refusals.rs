use super::{
    fixture, refresh,
    support::{tree, Scratch},
};
use std::fs;

fn refused(t: &Scratch, args: &[&str], message: &str) {
    let before = tree(&t.0);
    let out = t.run(args);
    assert!(!out.status.success(), "accepted {args:?}");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains(message),
        "{args:?}: expected {message}, got {text}"
    );
    assert_eq!(tree(&t.0), before, "refusal mutated files: {args:?}");
}

#[test]
fn R23__invalid_arguments_are_rejected_before_any_side_effect() {
    let t = fixture();
    for args in [
        vec!["selected", "--refresh-mode"],
        vec!["selected", "--host", "codex"],
        vec!["selected", "--refresh-mode", "--host"],
        vec!["selected", "--refresh-mode", "--host="],
        vec!["selected", "--refresh-mode", "--host", "other"],
        vec![
            "selected",
            "--refresh-mode",
            "--host",
            "codex",
            "--host=codex",
        ],
        vec![
            "selected",
            "--refresh-mode",
            "--refresh-mode",
            "--host",
            "codex",
        ],
        vec!["selected", "--refresh-mode=yes", "--host", "codex"],
        vec!["selected", "extra"],
        vec!["selected", "--unknown"],
        vec![
            "selected",
            "--refresh-mode",
            "--host",
            "codex",
            "--run",
            "active",
        ],
        vec!["--refresh-mode", "--host", "codex"],
    ] {
        let mut command = vec!["quick", "resume"];
        command.extend(args);
        refused(&t, &command, "");
        assert!(
            t.run(&command).stdout.is_empty(),
            "validation printed partial success"
        );
    }
    for slug in ["../selected", "selected/../other", "/tmp", ".", ".."] {
        refused(&t, &refresh(slug, "claude"), "plain name");
    }
    refused(&t, &refresh("absent", "claude"), "not found");
    refused(&t, &refresh("selected", "codex"), "main host mismatch");
}

#[test]
fn R23__closed_missing_or_duplicate_state_rows_refuse_without_changes() {
    let t = fixture();
    let state = ".dstack/quick/STATE.md";
    for rows in [
        "| selected | done | before | after |\n",
        "| selected | abandoned | before | after |\n",
        "| other | open | before | |\n",
        "| selected | damaged | before | |\n",
        "| selected | open | before | |\n| selected | open | before | |\n",
        "| selected | open | before | |\n| selected | done | before | after |\n",
    ] {
        t.write(state, rows);
        refused(&t, &refresh("selected", "claude"), "one open STATE row");
    }
    fs::remove_file(t.0.join(state)).unwrap();
    refused(&t, &refresh("selected", "claude"), "one open STATE row");
}

#[test]
fn R23__damaged_modes_and_unreadable_request_refuse_without_changes() {
    for relative in [
        ".dstack/project/mode.json",
        ".dstack/quick/selected/mode.json",
    ] {
        let t = fixture();
        for malformed in [
            "{",
            "{}",
            "null",
            "[]",
            r#"{"main":"codex"}"#,
            r#"{"main":"claude","sub":"unknown"}"#,
            r#"{"main":"claude","sub":"codex","extra":1}"#,
            r#"{"main":"claude","main":"codex","sub":"codex"}"#,
        ] {
            t.write(relative, malformed);
            refused(&t, &refresh("selected", "claude"), "invalid mode file");
        }
        fs::remove_file(t.0.join(relative)).unwrap();
        fs::create_dir(t.0.join(relative)).unwrap();
        refused(&t, &refresh("selected", "claude"), "mode.json");
    }
    let t = fixture();
    fs::remove_file(t.0.join(".dstack/quick/selected/request.md")).unwrap();
    refused(&t, &refresh("selected", "claude"), "request.md");
}

#[test]
fn R23__preflight_requires_target_fields_and_selected_providers_only() {
    for option in [
        None,
        Some("--research"),
        Some("--review"),
        Some("--validate"),
    ] {
        let t = Scratch::new();
        t.init();
        let mut args = vec!["quick", "new", "selected"];
        args.extend(option);
        t.ok(&args);
        t.ok(&["mode", "set", "--main", "codex", "--sub", "claude"]);
        let absent = t.0.join("absent-provider");
        let table = format!("name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\nclaude\ttest -x {}\tinstall-claude\t-\tyes\tgoal-closing\tprovider=claude\t\nvalidator\ttest -x {}\tinstall-validator\t-\tyes\tgoal-closing\te2e=cli\t\n", absent.display(), absent.display());
        t.write("deps.tsv", &table);
        if option.is_some() {
            refused(&t, &refresh("selected", "codex"), "MISSING");
        } else {
            t.ok(&refresh("selected", "codex"));
        }
        t.ok(&["mode", "set", "--main", "claude", "--sub", "codex"]);
        refused(&t, &refresh("selected", "claude"), "MISSING claude");
    }
}
