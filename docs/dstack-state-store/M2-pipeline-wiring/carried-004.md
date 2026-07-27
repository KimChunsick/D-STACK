## Carried decisions — Round 004
Rounds 1-3 decisions stand. Added in Round 4:

- **A backgrounded long run does not outlive its turn — detach it.** Sentinel file plus an
  explicit VANISHED branch; never treat silence as "still running".
- **One runnable fence per procedure.** If a later fence consumes a variable an earlier one set,
  it is one procedure that was mis-split, not two steps.
- `review-unit` is a schema PARAMETER; hard-coding `task` makes the pipeline unsatisfiable for a
  milestone-granularity Goal.
- The review bundle carries every document the unit doc tells the reviewer to read; an omitted
  subordinate record hides contradictions the reviewer cannot report.
- A guard must match the SHAPE of what it guards, not a substring of it — a bundle contains the
  skill's own prose about the guard.
- A check that can be satisfied by unrelated text pins nothing; bind the assertion to the call.
- Accepted residuals unchanged: cache-read economics limit the injection saving, `gitignored` is
  not confidential, a ticked box is self-attested.

Consensus: disagreed
