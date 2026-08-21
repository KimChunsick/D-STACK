## Carried decisions — Round 002
- F1–F5 (round 001): fixes verified by round 002 except where sharpened: F1's repair is
  extended by F6, F3's by F8, F4's rule is replaced by F9's.
- F6 (high, hard links pass `-L`; orchestrator writes unguarded): ACCEPTED — guards
  refuse any aliased leaf (regular, non-symlink, link count 1 via `find -prune -links 1`)
  for inputs and output targets in both fences; prose rule covers every
  orchestrator-written artifact (Step 1 brief, Step 2a record, fallback artifacts).
- F7 (medium, fail-open assembly): ACCEPTED — `-r` on inputs, `&&`-chained
  concatenation, nonzero group status refuses the launch.
- F8 (medium, producer-declared coverage): ACCEPTED with a boundary — structural test
  extended to ledger/deferred reconciliation coverage and empty-F-over-claims breakage;
  research triggers treat all-`none` blocks over measurable claims as missing sections.
  Claim-level semantic coverage remains the auditor's contract; the orchestrator's checks
  are a structural backstop.
- F9 (medium, unaudited verdict changes): ACCEPTED — any verdict-changing check outcome
  re-enters the auditor (Step 2b under the next label, appended results on stdin); the
  `superseded:` line records the delta audit's verdict; decision-criticality decides only
  Phase 4 re-entry. GOAL.md interview assumption amended accordingly.
- Standing context: no-new-tests repo policy; install.sh untouched by this unit.

Consensus: disagreed
