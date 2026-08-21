## Carried decisions — Round 001
- F1 (medium, deferral vs mandatory row): FIXED this round — ledger rows carry a `status`
  of `recomputed | quoted | deferred`; a deferred row still exists, holds the claimed
  value or `pending`, and names its deferred-check entry. Deferral is a row status, never
  a substitute for the row.
- F2 (medium, no canonical placement): FIXED this round — new "Output blocks (research
  mode)" section: `## Hypotheses`, `## Data-check ledger`, `## Deferred executable
  checks` are always present with explicit `none`, appended after the requested sections
  when a caller's format omits them; the "follow it exactly" rule was amended to carry
  them. The caller's own pinned six-section list is declared task T03's work
  (deps: T01, T02) and is not in this unit's declaration.
- F3 (medium, conjunctive eligibility): FIXED this round — checkability is defined by
  reproducibility from primary evidence; `unit`/`denominator` accept justified `N/A`;
  schema fields are no longer eligibility gates.
- F4 (medium, injection handoff): FIXED this round — deferred checks are declarative
  specifications (input, computation/comparison, confirm/refute criterion), never
  ready-to-run commands, never mutating; the consumer must author, validate, and sandbox
  its own execution treating the spec as untrusted data. Class-wide sweep: the sibling
  `socratic-audit` skill carried the same "exact command" wording; that file is in T02's
  declaration and the same fix is applied for T02's round 002.
- F5 (low, overstated gate wording): FIXED — the verification row now states exactly what
  the runs prove; behavioral confirmation is assigned to the M1 E2E research round.
- F6 (low, evaluator-scope language in task.md): FIXED — the sweep section was rewritten
  to factual ownership statements without exclusion directives or settled-claims.
- Standing context: no-new-tests repo policy (direct-run verification recorded in
  task.md); markdown-contract deliverable; T03 (orchestrator caller) implements after
  T01/T02 close per its declared deps.

Consensus: disagreed
