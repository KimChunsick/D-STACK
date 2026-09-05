// Historical shell comparisons are opt-in; ordinary checks need no archived tag.

use std::path::PathBuf;
use std::process::Command;

fn harness() -> Command {
    let mut command = Command::new("bash");
    command.arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("parity/run.sh"));
    command
}

#[test]
fn r04_default_parity_is_skipped_without_creating_output_or_building() {
    let out_dir =
        std::env::temp_dir().join(format!("dstack-optional-parity-{}", std::process::id()));
    assert!(!out_dir.exists(), "the output path must start absent");
    let result = harness()
        .args(["--rust", "/nonexistent/dstack-test-binary", "--out"])
        .arg(&out_dir)
        .output()
        .expect("run the parity harness");
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "skipped: historical shell comparison is opt-in; use --shell-ref <ref> or --shell <dispatcher>\n"
    );
    assert!(result.stderr.is_empty());
    assert!(
        !out_dir.exists(),
        "skipping must not create an output directory"
    );
}

#[test]
fn r04_explicit_shell_comparison_still_reports_a_missing_reference() {
    let result = harness()
        .args(["--shell-ref", "refs/tags/dstack-test-missing-reference"])
        .args(["--rust", env!("CARGO_BIN_EXE_dstack")])
        .output()
        .expect("run the explicit comparison");
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr)
        .contains("cannot extract the shell reference of refs/tags/dstack-test-missing-reference"));
    assert!(!String::from_utf8_lossy(&result.stdout).contains("skipped:"));
}

#[test]
fn r04_default_parity_still_rejects_unknown_options() {
    let result = harness().arg("--bogus").output().expect("run the harness");
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown option: --bogus"));
}
