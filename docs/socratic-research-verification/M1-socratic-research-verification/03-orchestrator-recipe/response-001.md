# Response — Round 001 (never bundled)

All five findings accepted; all five fixed this round in
`claude/skills/codex-research/SKILL.md` (task.md updated to match).

- F1 (high, leaf paths): verified the reviewer's mechanism — `-s`, `cat`, and `>` follow
  terminal symlinks, and only ancestor directories were checked. Fixed: Step 2b now
  refuses a non-regular / symlinked / empty input leaf and a symlinked `-o` target, and
  assembles the stdin concatenation under `$SCRATCH` (fresh `mktemp -d`), so the fence no
  longer opens any predictable repo path for writing. Class-wide sweep: Step 2's fence
  gained the same leaf guard for `<topic>.brief.txt` and the `-o` artifact. Residual
  stated in task.md: Step 1's brief write is the orchestrator's own file tool, and dstack
  independently refuses a symlinked stdin file at launch.
- F2 (high, fallback bypass): fixed with an explicit paragraph — the fallback replaces
  the researcher, never the contract or the audit; the fallback artifact carries the same
  nine sections (explicit `none`), then Steps 2a–2c run unchanged; Phase 3 without a
  data-checks record and an audit verdict summary "did not finish, whichever path
  produced the research".
- F3 (medium, summary-only acceptance): fixed — Step 2c's structural test is now "any of
  the seven pinned sections missing, or the verdict summary lacks a row for an enumerated
  H-item"; the audit fallback trigger references the same test instead of naming only the
  summary section.
- F4 (medium, stale noncritical verdicts): fixed — every completed check reconciles into
  its claim's verdict via a `superseded:` line in `<topic>.data-checks.md` (the audit
  artifact stays untouched; the supersession record is the orchestrator's);
  decision-criticality now gates only the delta-audit escalation; Step 3 reports verdict
  counts AS RECONCILED with supersessions noted.
- F5 (low, overclaim): fixed — the intro now reads "evidence-informed, not proven for
  this exact deployment", cites the research's own Unverified limitation, and assigns
  mechanism-specific evidence to the Goal's E2E rounds.

Verification after fixes: all three extracted bash fences pass `bash -n`; section
sequence unchanged; `bash tests/secret-guard.sh` green. Round 002 requested on the same
allowlist.
