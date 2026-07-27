# Maintainer response — Round 003

Not bundled into any review round.

## F007 [high] leader death is not group death — AGREED, fixed as directed

Correct, and it invalidates the previous round's evidence as much as the previous round's code. My
zero-stray probe used `/bin/sleep`, which honours TERM, so it could only ever demonstrate teardown
for well-behaved children. The reviewer's reproduction (`direct_status=143 group_survived=yes`) is
the case that mattered.

Implemented as suggested: `$child` is treated as both pid and pgid. `run_group_gone` asks
`kill -0 -<pgid>` — whether the group still has any member we may signal — and `run_group_settle`
escalates on a bound: quiesce 5s, TERM, 5s, KILL, 5s, then warn if the group somehow persists.
Both paths use it, not just the abort path: on normal completion the leader exited on its own, so a
surviving descendant is a leak, and publishing a terminal record while it writes into the capture
is exactly the defect. `rm-run` now refuses while the pid **or** the group has a live member.

Verified with a leader that exits immediately leaving a TERM-ignoring busy loop in its group:

```
start 00:49:19
DONE f7 exit=7 …                    dstack status=6
end   00:49:30                      ← 11s: 5s quiesce, TERM ignored, 5s, KILL
exit file=[7]                       ← the leader's real status, not the signal's
stray TERM-ignoring busy loops: 0
```

## F008 [medium] cleanup does not own every catchable exit — AGREED, fixed as directed

Both halves confirmed. The trap went up after the claim and the reserved/capture/cmd writes, so a
TERM in that interval stranded the claim; and only INT/TERM/HUP were covered, so an untrapped
terminating signal orphaned the child. The reviewer demonstrated the second with USR1
(`supervisor_alive=no child_alive=yes`).

Cleanup ownership now starts one statement after the claim succeeds and is armed on `EXIT` as well
as on `INT TERM HUP QUIT PIPE ALRM USR1 USR2`. Because EXIT is covered, every `die` past the claim
leaves through the same owner, which is what let `claim_release_and_die` be deleted outright rather
than kept as a parallel path — one owner, not two. It is disarmed only after `run_published=1`.

```
post-claim die (adopted dir already holds `exit`)
  → dstack: '…/g-pre/exit' already exists — …
  → dstack: run g-pre aborted before anything was launched — claim released, label is free to retry
  → .launch left: no

USR1 to a running supervisor → child gone, exit file [143], stray 0
TERM straddling the fork, 20 samples → stray 0, published 20, STUCK 0
```

## F009 [low] the recycled-pid rationale was wrong — AGREED, loop removed

The reviewer is right about the deployed behaviour and I had the mechanism backwards: bash caches a
reaped job's status, so the second `wait` returns that status rather than 127, while `kill -0`
asks about whatever now occupies the pid — so a recycled pid could spin the loop until an unrelated
process exited. The loop was also unnecessary, for the reason given: the signal handler exits, so
control never returns to it. Replaced with a single `wait`, and the wrong comment is gone rather
than corrected in place.

Not treated as non-blocking despite its severity: it was a wrong belief written into the code as
justification, which is the same class as the false residual round 001 caught.

## Class-wide sweep (Step 0)

Class: *a check that names the wrong entity* — pid where the group was meant, one signal set where
"terminating signals" was meant, a leader's status where the group's quiescence was meant. Swept
every liveness or identity test in the change: `rm-run`'s supervisor check (pid is right there — it
is dstack itself, not a group), `rm-run`'s launched check (widened to the group), `run_cleanup`'s
teardown (widened), the post-wait probe (deleted), and `run_group_gone`'s own fallback when job
control did not take effect (documented: `-<pid>` then fails, and the bare-pid TERM is the
fallback).

## Carried decisions

- `$child` is both pid and pgid. Every liveness question about the launched work asks the GROUP;
  only questions about the supervisor itself ask a bare pid.
- Terminal publication is gated on group quiescence with bounded TERM→KILL escalation, on the
  normal path as well as the abort path.
- One cleanup owner (`run_cleanup` on EXIT + `$RUN_SIGNALS`), armed one statement after the claim,
  disarmed only after publication. No second release path.
- Evidence must exercise a child that does NOT honour the signal being tested; a `sleep`-based
  probe cannot speak to teardown completeness.
- Round 001's and 002's carried decisions still stand.
