## Carried decisions — Round 001
All Round-1 findings were accepted and fixed; none were rebutted. Standing decisions:

- **Discovery time never changes a finding's blocking status.** A concrete high or medium blocks
  whichever round surfaces it. Only non-concrete items may be aged out, and only from Round 4.
  The six-round budget escalates to the user; it never downgrades a defect.
- Review triage must match the contract's own `[severity:…][axis]` line format, and the
  high/medium query must never carry a fixed result cap.
- `<review-unit>` is a single abstraction: one folder holds the registered, gated, reviewed
  `task.md` and its review series. Subordinate task documents are records only.
- Migration filenames: timestamps reduce collisions, they do not remove them. Pin precision,
  refuse to overwrite an existing path, and keep ordering as a declared dependency.
- Open and deliberately deferred: the background-handoff E2E belongs to P11 (milestone E2E),
  after review by phase order. It is not evidence this round claims.
- Repo policy: no tests, no Red-Green-Refactor; gates rest on recorded direct-run evidence.

Consensus: disagreed
