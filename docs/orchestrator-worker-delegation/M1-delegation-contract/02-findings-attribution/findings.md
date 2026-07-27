# Finding ledger — T02 (findings attribution)

Blocking findings per round: **R1 4 · R2 5 · R3 4**
Bundle bytes: R1 34,282 · R2 47,052 · R3 58,114 (ratchet VIOLATED at R2 and R3)

Closed at R3 by the non-convergence rule — 4, 5, 4 is not strictly decreasing. No finding in three
rounds was a false positive; the count held because each repair opened new surface.

## Open

None yet.

## Closed in round 3

- [medium][correctness] worktree binding contradicted itself (hook-only vs `git worktree add`) and
  had no pre-agreed record key — orchestrator-chosen name, hook-only creation for workers
- [medium][structure] a globally configured fail-closed hook would break ordinary `--worktree`
  usage — a missing record now falls through to normal creation
- [medium][correctness] "the reviewed commit" does not exist in SERIAL mode — the predicate fails
  closed to the orchestrator there
- [medium][correctness] the state machine dead-ended at `recalled` — `recalled` → `verified` →
  `closed`
- [low][security] the record retained the superseded "reads anywhere" summary
- [low][DX] reviewer-directed language survived a third round
- [low][correctness] the `/clear` rationale assumed foreground workers

## Closed in round 2

- [medium][correctness] the state notation made expansion mandatory, leaving a clean in-scope fix
  with no path to `verified`
- [medium][security] "a worker may read anywhere" was an unqualified secret-read capability — the
  secret deny list now outranks the read permission unconditionally
- [medium][correctness] taint recovery conflated repository state with external side effects —
  separate dispositions, and an uncleaned external effect blocks sealing
- [medium][structure] resource isolation still gated DELEGATION on a contention property — moved
  to `parallel-when`
- [medium][correctness] `WorktreeCreate` had no base/branch/fixture handoff — a durable `.dstack/`
  intent record keyed by the hook's name, verified hook-side, emitting no path on mismatch
- [low][security] the record embedded reviewer-facing imperatives and a review-disposition claim

## Closed in round 1

- [medium][structure] the record claimed untouched sections the diff rewrote — cause is serial
  review against an uncommitted Goal; stated outright, with what this task authored listed
- [medium][correctness] declaration containment did not prove the original worker owns the current
  code — routing now also requires the branch head to equal the reviewed commit
- [medium][correctness] the taint guarantee contradicted `honest-scope` — only commit-reaching
  writes are enforced; the rest is self-reported policy with a stated recovery
- [medium][structure] `WorktreeRemove` was described as owning retention; it cannot block — the
  orchestrator removes worktrees explicitly after closure
- [low][correctness] the `/clear` rationale said "every warm worker" where the evidence says
  backgrounded tasks survive
- [low][DX] the state machine omitted `tainted`, `resume-failed`, `verification-failed`
