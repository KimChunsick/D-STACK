# Finding ledger — M1 (state store and the gate that reads it)

The termination signal for this unit's review loop. The `GPT verdict` line is advisory; what
closes the loop is this ledger going quiet, or the non-convergence rule firing.

Blocking findings per round: **R1 8 · R2 9 · R3 6 · R4 7 · R5 6 · R6 6 · R7 4 · R8 3 · R9 6**

R7-R9 is 4, 3, 6 — not strictly decreasing across three consecutive rounds. Non-convergent by
measurement, so the loop closed at R9. The rise from 3 to 6 is the sharpest evidence in this Goal
that "loop until the reviewer approves" has no fixpoint: every R8 finding that was actually fixed
was fixed correctly, and the count still went up, because each repair opened new surface.

Bundle bytes: R7 198591 · R8 214387 · R9 193178 — up, like M2's.

## Open (carried out of the closed loop)

| # | Sev | Class | Finding | Raised |
|---|---|---|---|---|
| F-01 | low | security | invalid-record diagnostics print record basenames and doc fields unescaped, so terminal-control bytes reach the terminal | R8, R9 |
| F-02 | low | correctness | timestamp validation is inconsistent: writers accept any nonempty `ts` without checking `date`'s status, readers accept an empty one | R9 |
| F-03 | low | DX | `status` reports invalid records but still exits 0, so automation cannot tell a healthy registry from one the gate refuses to trust | R9 |
| F-04 | low | DX | `migrate` refuses duplicate legacy lines with the same owner and document, although collapsing them is lossless | R9 |
| F-05 | low | correctness | successful registration ignores a failure to remove its published temp name | R9 |
| F-06 | low | correctness | the hook reports a record removed between its existence check and `cat` as corruption, where `read_record` treats the same race as a deregistration | R8 |
| F-07 | low | correctness | legacy-lock cleanup stays silent on `die` paths — only the success paths call `release_legacy_lock` and warn | R8 |

Two further lows were summarised by the reviewer as `Omitted-detail: 2 low` and never itemised;
they are named here as unenumerated rather than pretended away.

## Closed in the final round (R9)

- [high][correctness] a lock-release trap could delete a successor's lock — disarm before rmdir,
  residual stated in the code
- [medium][correctness] `ls -1` hid dot-prefixed documents and an empty spelling match was
  silently ignored — `-a` in both listings, hard failure plus ambiguity check
- [medium][correctness] the gate heading matched any suffix and terminated on too little —
  exact-or-parenthetical, terminating on any ATX heading
- [medium][DX] recovery commands rendered as one single-quoted word — quoted executable, bare args
- [medium][DX] a deleted or renamed document's record could not be released — `stale_record_ok`
  plus key-addressed `unreg`
- [medium][correctness] fatal git discovery opened the gate for env-defined worktrees — explicit
  git environment and an ancestor `.dstack` now count as evidence
- [medium][security] (carried from R8) an unreadable `active/` read as an empty registry —
  traversal proved before an empty scan is believed
- [medium][correctness] (carried from R8) the milestone sweep saw only `sort`'s status — every
  producer checked
- [low][correctness] a FIFO at the legacy registry path read as absent
- [low][correctness] the seven-day pruning claim in `02-dstack-cli/task.md` (it is eight)
