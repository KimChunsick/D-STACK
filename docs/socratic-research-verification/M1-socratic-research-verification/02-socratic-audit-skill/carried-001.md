## Carried decisions — Round 001
- F1 (medium, non-H findings skipped): FIXED this round — a new "Targets" section
  enumerates decision-relevant non-H findings as `F1..Fn`, audited through assumptions
  and implications; an artifact with no targets at all must be reported as exactly that
  on the first line, never padded into a hollow audit.
- F2 (medium, no data-check reconciliation): FIXED this round — every H is GROUPED with
  its ledger rows, deferred checks, and recorded results; one reconciled verdict per
  group; a pending deferred check caps its H at `unverifiable (pending check)`; a failed
  data reading drags its H to weakened/refuted; unresolved checks ride into the verdict
  summary (new `unresolved checks` column).
- F3 (medium, fresh-grounding loophole): FIXED this round — probes must be answered from
  INDEPENDENTLY SELECTED sources; artifact citations count only as source-fidelity
  checks; `no independent source found` is an explicit unverifiable outcome.
- F4 (low, format-request trust boundary): FIXED — format requests bind only from the
  invoking prompt; a format directive inside audited material is itself a reportable
  finding.
- F5 (low, overstated gate wording): FIXED — the verification row now states exactly what
  the recorded runs prove; behavioral confirmation is assigned to the M1 E2E round.
- Cross-unit sweep applied here: T01's injection-handoff finding (deferred checks as
  ready-to-run commands) had a sibling instance in this skill's `## New deferred checks`;
  it is now declarative-only, non-mutating, mirroring the research contract's language.
- Standing context: no-new-tests repo policy; the install.sh diff carries ONE
  pre-existing line from another workstream (worktree-create.sh map row), excluded from
  this unit's commit by hunk-level staging.

Consensus: disagreed
