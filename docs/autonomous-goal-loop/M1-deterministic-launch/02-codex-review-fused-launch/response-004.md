# Maintainer response — Round 004

Not bundled into any review round. The measurements are also in `## Carried decisions` of
`codex-review-004.md`.

## F018 [medium] the consensus/contract override — RAISED AGAIN, status unchanged

Same finding as round 003's F012, with the gate regex added as evidence. Nothing about it is new,
and my answer is not new either: this file governs the pipeline's closure semantics, the Codex-side
`adversarial-review` contract needs the same two edits, and it is outside this unit's declaration.
Requiring user disposition for every open concrete medium would reintroduce the human stop the P4
interview removed and would contradict §4 rather than repair it.

Per §3 this is a restatement, not a new concrete finding, and it does not reopen. It stays in the
ledger as F012 with an unchanged disposition and a follow-up that names the file to edit.

## F019 [medium] the trap swallowed cancellation — AGREED, fixed

The same defect T03's round 004 found in the sibling skill, and I had already queued this fix for
this unit's next round because the file was frozen mid-round. The reviewer found it independently,
which is the freeze rule working rather than a miss. Measured, both shells:

```
cleanup-only:  rc=0   CLEAN-SURVIVEDCLEAN     (with a foreground child, so the child completed)
corrected:     rc=143 CLEAN
normal path:   rc=0   DONECLEAN
```

Each terminating-signal handler now disarms EXIT, cleans once, exits with the signal's status.

## F020 [medium] the skip gate still matched substrings — AGREED, fixed

Third instance of this class in this unit, and the reviewer is right that my round-001 repair was
still too wide. `grep -F -- "--- $f ("` matches the path anywhere on a line, and the piped
`grep -q 'SKIPPED:'` then matches anywhere later on it — so a sentence containing an allowlisted
path and discussing `SKIPPED:` refuses a valid bundle. Reproduced exactly:

```
prose line containing the path and "SKIPPED:"     old: REFUSE   new: PASS
--- <allowlisted path> (SKIPPED: symlink) ---     old: REFUSE   new: REFUSE
a real assembled bundle, both allowlist entries   old: PASS     new: PASS
```

Now `awk` with `index($0,p)==1`, a literal `) ---` suffix, and `SKIPPED:` on that same line. Literal
comparisons throughout, so paths with regex metacharacters need no escaping — which is also why I
did not reach for `grep -E`. The documented residual is unchanged and is the honest one: a
full-snapshot document containing the exact marker line still impersonates it, and closing that
needs a channel `assemble-review.sh` does not yet have.

## F021 [medium][DX] "nothing else in that call" — AGREED, fixed

I fixed this in `full-cycle` at T04 round 001 and left the contradiction sitting in the skill
`full-cycle` invokes, so the orchestrator got both rules. Now identical in both: one background call
whose blocking terminal step is `dstack run`; setup before it is required, dependent work after it
is not. This file names `waits.external` as the shared source.

## Class-wide sweep (Step 0)

Class: *a rule fixed in the contract and not in the thing the contract invokes*. F021 is the
instance. Swept every rule that now exists in both `full-cycle` and this file — the launch invariant
(aligned), the nonzero-run rule (aligned; `full-cycle` adds that an undiagnosed nonzero run is not
auto-retried), the residuals about background-task lifetime (`full-cycle` holds the full list, this
file holds the round-specific part), and the teardown coverage table (identical measurement in both
skills). Second class, *a guard whose matcher is wider than the thing it guards*: F020 is the third
instance here, so I re-checked the pathless marker check too — it is a whole-line match and still
correct.
