// selftest/writers.rs
// The request, plan and artifact writers every checker shares (the _selftest_* helpers).

use std::fs;
use std::path::{Path, PathBuf};

use time::macros::format_description;
use time::{Duration, OffsetDateTime};

use crate::core::error::{Error, Result};
use crate::core::fsx::{sha256_file, utc_now};
use crate::selftest::sandbox::Sandbox;

/// A plain cli request with R01 and R02 (_selftest_write_request).
const REQUEST: &str = "---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: skip
review: off
codex_effort: high
e2e: cli
unit_tests: off
visual: none
korean_polish: off
---
# selftest request

- [ ] **R01** the command prints what it counted — accept: stdout carries \"checked N\"
- [ ] **R02** the command refuses bad input — accept: exit code 1 with a reason
";

impl Sandbox {
    /// The approval stamp request approve writes (two spaces between the fields).
    pub fn approve(&self, run_dir: &Path) -> Result<()> {
        let hash = sha256_file(&run_dir.join("request.md"))
            .map_err(|e| Error::cannot_decide(format!("sandbox: no request.md: {e}")))?;
        self.write(
            &run_dir.join("request.approved"),
            &format!("sha256 {}  approved_at {}\n", hash, utc_now()),
        )
    }

    /// request approve, plan add and task add belong to other modules; a fixture run must not
    /// wait on them, so these writers produce the formats design.md fixes. If a format changes,
    /// they break loudly — which is why they sit next to the checkers that read them.
    pub fn write_request(&self, run_dir: &Path) -> Result<()> {
        self.write(&run_dir.join("request.md"), REQUEST)?;
        self.approve(run_dir)
    }

    pub fn write_plan(&self, run_dir: &Path, covers: &[&str]) -> Result<()> {
        let covers: Vec<String> = covers.iter().map(|r| format!("\"{r}\"")).collect();
        let plan = format!(
            "{{ \"v\": 2,
  \"milestones\": [ {{\"id\":\"M1\",\"slug\":\"selftest\",\"order\":1}} ],
  \"plans\": [ {{\"id\":\"P1\",\"milestone\":\"M1\",\"slug\":\"selftest\",\"files\":[\"artifacts\"],\"deps\":[],
              \"status\":\"in-progress\",\"worktree\":\"\",\"started_at\":\"\",\"done_at\":\"\",
              \"tasks\":[ {{\"id\":\"T1\",\"slug\":\"selftest\",\"covers\":[{}],\"files\":[\"artifacts\"],
                         \"deps\":[],\"commit\":\"\",\"done_at\":\"\"}} ] }} ] }}
",
            covers.join(",")
        );
        self.write(&run_dir.join("plan.json"), &plan)
    }

    /// _selftest_artifact: a file under the sandbox's artifacts directory.
    pub fn artifact(&self, name: &str, text: &str) -> Result<PathBuf> {
        let dir = self.dir.join("artifacts");
        fs::create_dir_all(&dir).map_err(|e| Error::cannot_decide(format!("sandbox: {e}")))?;
        let path = dir.join(name);
        self.write(&path, &format!("{text}\n"))?;
        Ok(path)
    }

    /// A fixture carries its own data as an HTML comment (`<!-- selftest-evidence: R01 -->`), so
    /// one driver serves every fixture of a checker.
    pub fn directive(fixture: &Path, key: &str) -> Option<String> {
        let text = fs::read_to_string(fixture).ok()?;
        let opening = format!("<!-- selftest-{key}:");
        for line in text.lines() {
            let value = match line.strip_prefix(&opening) {
                Some(value) => value.trim_start_matches(' '),
                None => continue,
            };
            let value = match value.find("-->") {
                Some(at) => value[..at].trim_end_matches(' '),
                None => value,
            };
            return Some(value.to_string());
        }
        None
    }

    /// _selftest_yesterday: the stamp a file needs to look a day old.
    pub fn yesterday() -> String {
        let stamp = format_description!("[year][month][day][hour][minute]");
        (OffsetDateTime::now_utc() - Duration::days(1))
            .format(&stamp)
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r05__the_writers_produce_the_shell_formats() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let run_dir = sandbox.dir.join("run");
        std::fs::create_dir_all(&run_dir).expect("run dir");
        sandbox.write_request(&run_dir).expect("request");
        let request = std::fs::read_to_string(run_dir.join("request.md")).expect("request");
        assert!(request.starts_with("---\nwork_type: cli\n"));
        assert!(request.contains("- [ ] **R01** the command prints what it counted"));
        let approved = std::fs::read_to_string(run_dir.join("request.approved")).expect("approval");
        let fields: Vec<&str> = approved.trim_end().split("  ").collect();
        assert_eq!(
            fields.len(),
            2,
            "the two fields are separated by two spaces"
        );
        assert_eq!(fields[0].len(), "sha256 ".len() + 64);
        assert!(fields[1].starts_with("approved_at 20"));
    }

    #[test]
    fn r05__write_plan_covers_the_given_rows() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        sandbox
            .write_plan(&sandbox.dir, &["R01", "R02"])
            .expect("plan");
        let plan = std::fs::read_to_string(sandbox.dir.join("plan.json")).expect("plan");
        assert!(plan.contains(r#""covers":["R01","R02"]"#));
        assert!(plan.contains(r#""status":"in-progress""#));
    }

    #[test]
    fn r05__directives_carry_the_fixture_data() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let fixture = sandbox.dir.join("bad-one.md");
        std::fs::write(
            &fixture,
            "<!-- selftest-evidence: R01   -->\n<!-- selftest-kind: cli -->\nbody\n",
        )
        .expect("fixture");
        assert_eq!(
            Sandbox::directive(&fixture, "evidence").as_deref(),
            Some("R01")
        );
        assert_eq!(Sandbox::directive(&fixture, "kind").as_deref(), Some("cli"));
        assert_eq!(Sandbox::directive(&fixture, "missing"), None);
    }

    #[test]
    fn r05__artifact_and_yesterday() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let path = sandbox
            .artifact("cli.txt", "checked 3, missing 0")
            .expect("artifact");
        assert_eq!(
            std::fs::read_to_string(&path).expect("artifact"),
            "checked 3, missing 0\n"
        );
        assert_eq!(Sandbox::yesterday().len(), 12);
    }
}
