## Carried decisions — Round 001
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
