#![allow(non_snake_case)]
#[path = "support/mode_settings.rs"]
mod support;

use dstack_cli::core::fsx::sha256_bytes;
use serde_json::{json, Value};
use std::process::Output;
use support::Scratch;

const RUN: &str = ".dstack/runs/sample";
const REQUEST: &str = "# 요청해요\n\n\
- [ ] **R01** 첫 동작을 확인해요. — accept: 첫 검사가 통과해요.\n\
- [ ] **R02** 둘째 동작을 확인해요. — accept: 둘째 검사가 통과해요.\n\
- [ ] **R03** 셋째 동작을 확인해요. — accept: 셋째 검사가 통과해요.\n\
- [ ] **R04** 넷째 동작을 확인해요. — accept: 넷째 검사가 통과해요.\n";

fn plan(id: &str, milestone: &str, covers: &[&str]) -> Value {
    json!({"id":id,"milestone":milestone,"slug":"fixture","files":[],"deps":[],
        "status":"done","worktree":"","started_at":"","done_at":"",
        "tasks":[{"id":format!("T{id}"),"slug":"fixture","covers":covers,"files":[],
            "deps":[],"commit":"","done_at":""}]})
}

fn fixture(findings: &str) -> Scratch {
    let s = Scratch::new();
    s.write(".dstack/version", "2\n");
    s.write(&format!("{RUN}/meta.tsv"), "status\topen\n");
    s.write(&format!("{RUN}/request.md"), REQUEST);
    s.write(&format!("{RUN}/request.approved"), "fixture\n");
    s.write(&format!("{RUN}/findings.md"), findings);
    let doc = json!({"v":2,
        "milestones":[{"id":"M1","slug":"one","order":1},
            {"id":"M2","slug":"two","order":2}],
        "plans":[plan("P1", "M1", &["R01", "R02"]),
            plan("P2", "M2", &["R03", "R04"]), plan("P10", "M2", &["R04"])]});
    s.write(&format!("{RUN}/plan.json"), &doc.to_string());
    s
}

fn record(args: &[&str], output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!(
        "$ dstack {}\nstdout:\n{stdout}stderr:\n{stderr}exit: {:?}",
        args.join(" "),
        output.status.code()
    );
    format!("{stdout}{stderr}")
}

fn bundle(s: &Scratch, mid: &str) -> Output {
    let before = sha256_bytes(s.read(&format!("{RUN}/findings.md")).as_bytes());
    let args = [
        "review",
        "--run",
        "sample",
        "--scope",
        "milestone",
        "--milestone",
        mid,
        "--out",
        "bundle.txt",
    ];
    let output = s.run(&args);
    record(&args, &output);
    assert_eq!(
        before,
        sha256_bytes(s.read(&format!("{RUN}/findings.md")).as_bytes()),
        "bundle generation must preserve the original findings hash"
    );
    output
}

fn checked(s: &Scratch, mid: &str, ids: &[&str]) -> String {
    let output = bundle(s, mid);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let args = ["check", "review-bundle", "bundle.txt", "--run", "sample"];
    let check = s.run(&args);
    let report = record(&args, &check);
    assert!(check.status.success(), "{report}");
    let text = s.read("bundle.txt");
    let frozen = text
        .split("=== REQUEST (frozen) ===\n")
        .nth(1)
        .unwrap()
        .split("\n=== FINDINGS (open) ===")
        .next()
        .unwrap();
    let expected: String = REQUEST
        .lines()
        .filter(|line| {
            ids.iter()
                .any(|id| line.starts_with(&format!("- [ ] **{id}**")))
        })
        .map(|line| format!("{line}\n"))
        .collect();
    assert_eq!(
        frozen, expected,
        "frozen rows must be exact and in request order"
    );
    let body = text.split("=== FINDINGS (open) ===").nth(1).unwrap();
    let re = regex::Regex::new(r"R[0-9]+").unwrap();
    let mut actual: Vec<_> = re.find_iter(body).map(|m| m.as_str()).collect();
    actual.sort();
    actual.dedup();
    assert_eq!(actual, ids, "body IDs must exactly equal selected coverage");
    text
}

#[test]
fn R13__both_milestones_keep_only_their_open_findings_and_integration() {
    let first = "- [P1, codex-review-003, goal achievement] HIGH: R01 remains open.";
    let future = "- [P1, codex-review-003] R02 remains open; to be resolved by P1.";
    let second = "- [P2 / round 019, scoping] R03 needs integration with Plan P10 (R04).";
    let no_r = "- [P10 / round 021, security] process boundary remains open.";
    let r_only = "* R02 has an unresolved edge case.";
    let m_only = "- [M2, integration] milestone relationship remains open.";
    let resolved = "- [P1] R99 old issue. resolved: fixed in P1.";
    let s = fixture(&[first, future, second, no_r, r_only, m_only, resolved].join("\n"));
    for (mid, ids, included, excluded) in [
        (
            "M1",
            vec!["R01", "R02"],
            vec![first, future, r_only],
            vec![second, no_r, m_only, resolved],
        ),
        (
            "M2",
            vec!["R03", "R04"],
            vec![second, no_r, m_only],
            vec![first, future, r_only, resolved],
        ),
    ] {
        let text = checked(&s, mid, &ids);
        for line in included {
            assert!(text.contains(line), "missing: {line}");
        }
        for line in excluded {
            assert!(!text.contains(line), "unexpected: {line}");
        }
        let integration = text.split("=== INTEGRATION ===\n").nth(1).unwrap();
        assert_eq!(integration.contains("P1 fixture status:"), mid == "M1");
        assert_eq!(integration.contains("P2 fixture status:"), mid == "M2");
        assert_eq!(integration.contains("P10 fixture status:"), mid == "M2");
    }
}

#[test]
fn R13__followup_coverage_carries_relevant_prior_plan_findings() {
    let shared = "- [P2 / round 019] R03 remains open from the prior Plan.";
    let inherited = "- [P2, security] prior Plan finding has no explicit requirement.";
    let other = "- [P2, goal achievement] R04 belongs only to the other milestone.";
    let selected = "- [P2.1, integration] R03 remains open in the follow-up.";
    let s = fixture(&[shared, inherited, other, selected].join("\n"));
    let mut doc: Value = serde_json::from_str(&s.read(&format!("{RUN}/plan.json"))).unwrap();
    let mut followup = plan("P2.1", "M1", &["R03"]);
    followup["deps"] = json!(["P2"]);
    doc["plans"].as_array_mut().unwrap().push(followup);
    s.write(&format!("{RUN}/plan.json"), &doc.to_string());
    let text = checked(&s, "M1", &["R01", "R02", "R03"]);
    for line in [shared, inherited, selected] {
        assert!(text.contains(line));
    }
    assert!(!text.contains(other));
    let integration = text.split("=== INTEGRATION ===\n").nth(1).unwrap();
    assert!(integration.contains("P2.1 fixture status:"));
    assert!(!integration.contains("P2 fixture status:"));
}

#[test]
fn R13__unknown_and_unscoped_open_items_have_line_numbered_errors() {
    for line in [
        "- [P999, security] missing Plan ownership.",
        "- [M99, integration] missing milestone ownership.",
        "- R99 has no known requirement ownership.",
        "- There is no scope on this finding.",
        "- [P1] R01 also refers to unknown Plan P999.",
        "- [P2] unrelated scope still contains unknown R99.",
        "- [AP1, security] must not be mistaken for a Plan reference.",
    ] {
        let s = fixture(&format!("# Findings\n\n{line}\n"));
        let out = bundle(&s, "M1");
        assert!(!out.status.success(), "silently accepted {line}");
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(error.contains("findings.md:3"), "{error}");
        assert!(error.contains("cannot scope"), "{error}");
        assert!(!s.0.join("bundle.txt").exists());
    }
}

#[test]
fn R13__mixed_cross_scope_ids_stay_intact_and_are_rejected() {
    for line in [
        "- [P1] R01 depends on unresolved R03.",
        "- [P2] R03 depends on R01.",
        "- [P1] R03 conflicts with the selected Plan.",
    ] {
        let s = fixture(line);
        let out = bundle(&s, "M1");
        assert_eq!(out.status.code(), Some(1));
        let report = String::from_utf8_lossy(&out.stdout);
        assert!(
            report.contains("cited but not in REQUEST:   R03"),
            "{report}"
        );
        assert!(!s.0.join("bundle.txt").exists());
    }
}

#[test]
fn R13__resolved_and_empty_ledgers_keep_existing_behavior() {
    for findings in [
        "",
        "# Findings\n",
        "- [P1] R01 was fixed — resolved in P1.\n",
        "- [P2] R03 is fixed; resolved: verified.\n",
    ] {
        let s = fixture(findings);
        let text = checked(&s, "M1", &["R01", "R02"]);
        assert!(text.contains("no open items"));
    }
    let s = fixture("");
    std::fs::remove_file(s.0.join(format!("{RUN}/findings.md"))).unwrap();
    let args = [
        "review",
        "--run",
        "sample",
        "--scope",
        "milestone",
        "--milestone",
        "M1",
        "--out",
        "bundle.txt",
    ];
    let output = s.run(&args);
    record(&args, &output);
    assert!(output.status.success());
    assert!(s.read("bundle.txt").contains("no findings.md"));
}

#[test]
fn R13__missing_frozen_requirement_is_still_rejected() {
    let s = fixture("- [P1] R01 remains open.");
    s.write(
        &format!("{RUN}/request.md"),
        &REQUEST
            .lines()
            .filter(|l| !l.contains("**R02**"))
            .map(|l| format!("{l}\n"))
            .collect::<String>(),
    );
    let output = bundle(&s, "M1");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("three counts disagree"));
    assert!(!s.0.join("bundle.txt").exists());
}

#[test]
fn R13__T16_negative_false_and_unknown_resolution_forms_stay_open() {
    for tail in [
        "is not resolved: the failure still reproduces.",
        "is still open; resolved: false.",
        "is still open; resolved: FALSE",
        "is still open; resolved: no",
        "is still open; resolved: 0",
        "is still open; resolved: null",
        "is still open; resolved: pending",
        "is still open; resolved: unknown",
        "is still open; resolved: not verified.",
        "is still open; resolved:",
        "is still open; resolved: maybe later.",
        "is still open; resolved: fixed if the next run passes.",
        "is still open; resolved in progress.",
        "is still open; resolved: commit unknown.",
        "still reports file.resolved: verified.",
    ] {
        let line = format!("- [P1] R01 {tail}");
        let s = fixture(&line);
        assert!(checked(&s, "M1", &["R01", "R02"]).contains(&line));
    }
}

#[test]
fn R13__T16_quoted_diagnostic_resolution_tokens_stay_open() {
    for tail in [
        "diagnostic says \"resolved: false\".",
        "diagnostic says \"error; resolved: verified.\"",
        "diagnostic says 'error; resolved: verified.'",
        "diagnostic says `error; resolved: verified.`",
        "diagnostic says “error; resolved: verified.”",
        "diagnostic says ‘error; resolved: verified.’",
        "is still open; resolved: \"false\"",
        "is still open; resolved: `verified`",
        "diagnostic says ``error; resolved: commit ce585151``",
        "diagnostic says \\\"error; resolved: commit ce585151\\\"",
    ] {
        let line = format!("- [P1] R01 {tail}");
        let s = fixture(&line);
        assert!(checked(&s, "M1", &["R01", "R02"]).contains(&line));
    }
}

#[test]
fn R13__T16_affirmative_ledger_annotations_still_close_items() {
    for tail in [
        "was fixed — resolved in P1.",
        "was fixed; resolved: verified.",
        "was fixed; resolved: fixed in P1.",
        "was fixed; resolved: P1",
        "was fixed. resolved: commit b28842af on plan/P1; checked the recovery path.",
        "diagnostic's \"resolved: false\" output was fixed. resolved: commit ce585151.",
    ] {
        let line = format!("- [P1] R01 {tail}");
        let s = fixture(&line);
        assert!(!checked(&s, "M1", &["R01", "R02"]).contains(&line));
    }
}
