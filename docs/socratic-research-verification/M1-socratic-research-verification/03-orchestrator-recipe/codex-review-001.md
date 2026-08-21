# Codex adversarial review — Round 001

## Review scope
Initial review | serial | bundle 33796 bytes | label socratic-research-verification-t03-r001

## GPT findings
[severity:high][security] Terminal artifact paths are unprotected: a pre-existing audit-input symlink is truncated, while symlinked research or data-check inputs can be copied into a regular file sent to Codex.
Sites: Primary: SKILL.md:326; confirmed: SKILL.md:305, SKILL.md:320.
Evidence: Only ancestor directories are checked; the leaf inputs use `-s`/`cat`, and the predictable audit-input path is opened with `>`.
Verification: `-s`, `cat`, and shell redirection follow terminal symlinks; `.audit-input.txt -> ../../../AGENTS.md` therefore truncates the tracked root file before `dstack` runs.
Suggested direction: Reject symlink/non-regular leaves and assemble stdin under the unique scratch directory with every write checked.

[severity:high][technical correctness] The research-fallback path can finish Phase 3 without contract blocks, a data-check record, or any audit, despite the new unconditional-audit requirement.
Sites: Primary: SKILL.md:403; confirmed: SKILL.md:267, SKILL.md:406.
Evidence: Step 2a applies only "On success"; the research fallback merely performs alternate research and records it in `GOAL.md`, without resuming Steps 2a–2b.
Verification: Two failed research attempts enter this branch, and neither direct web research nor an unspecified host skill is required to emit the three new blocks or `<topic>.data-checks.md`.
Suggested direction: Give fallback research the same artifact contract, validate it, and explicitly resume at Step 2a.

[severity:medium][technical correctness] Audit acceptance checks only for `## Verdict summary`, so an exit-zero artifact missing target examination, data-check reconciliation, or the other required sections proceeds to P5.
Evidence: SKILL.md:340 requests seven sections, but SKILL.md:346 and SKILL.md:406 reject only a missing summary.
Verification: An exit-zero file containing only that heading and a table bypasses retry/fallback and is consumed by Step 3.
Suggested direction: Require all seven sections and reconcile every H-item, F-item, and ledger row before accepting an audit.

[severity:medium][technical correctness] New check results are re-audited only when decision-critical, allowing noncritical claims to retain stale verdicts that contradict appended evidence.
Sites: Primary: SKILL.md:351; confirmed: SKILL.md:366, socratic-audit/SKILL.md:74.
Evidence: Step 2c skips delta audit unless a decision-critical verdict would change, while Step 3 reports the existing verdict counts.
Verification: If a new check refutes a noncritical `unverifiable` H-item, the result is appended but the audit and reported counts remain `unverifiable`.
Suggested direction: Reconcile every completed check into an updated verdict; use decision-criticality only to decide whether Phase 4 is required.

[severity:low][real Why] The introduction calls this exact design "evidence-backed," although the supporting research could not verify separate-Codex-context gains or pipeline-specific cost and latency.
Evidence: research:38 records both limitations, while SKILL.md:8 makes the stronger characterization.
Verification: The cited artifact supports analogous verification patterns but explicitly leaves this mechanism, model, and deployment unverified.
Suggested direction: Describe the design as an evidence-informed hypothesis and record mechanism-specific evidence during the planned E2E round.

Omitted-detail: 0 low

GPT verdict: reject — unresolved high-severity leaf-path and fallback defects can clobber or expose files and silently skip the mandated verification layer.

## Carried decisions
- F1 (high, unprotected leaf paths in Step 2b): ACCEPTED — verified that `>` /`cat`/`-s`
  follow terminal symlinks and only ancestors were checked. Fix: leaf guards on both
  inputs (regular, non-symlink, non-empty) and on the `-o` audit target; the stdin
  concatenation moves into `$SCRATCH` so no predictable path is opened for writing.
  Class-wide sweep: the same leaf guard added to Step 2's fence (brief + `-o` artifact).
- F2 (high, fallback bypasses the verification layer): ACCEPTED — fix: the fallback
  replaces only the researcher; the artifact contract (nine sections), the data-checks
  record, and Steps 2a–2c apply unchanged to a fallback-produced `<topic>.md`.
- F3 (medium, single-section audit acceptance): ACCEPTED — structural acceptance now
  requires all seven pinned sections AND a verdict-summary row for every H-item the
  artifact enumerates; mirrored in the fallback trigger.
- F4 (medium, stale verdicts for noncritical claims): ACCEPTED — every completed check is
  reconciled: a contradicting outcome gets a `superseded` line in `<topic>.data-checks.md`
  (audit artifact untouched); decision-criticality now gates only the delta-audit
  escalation; Step 3 reports reconciled verdicts.
- F5 (low, "evidence-backed" overclaim): ACCEPTED — intro reworded to evidence-informed,
  with the mechanism-specific residue owned by the E2E round.
- Standing context: no-new-tests repo policy (direct-run evidence in task.md); install.sh
  is untouched by this unit.

Consensus: disagreed
