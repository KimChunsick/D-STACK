# Maintainer response — Round 002

Not bundled into any review round. The measurements are also in `## Carried decisions` of
`codex-review-002.md`.

## F006 [high] `reclaim` in `internal-recoveries` was ownership theft — AGREED, fixed

This is the same class as round 001's high and I walked straight into it while fixing that one.
Round 001 said a failed `reg` must not be a warning; I made it fail-closed and then, one line
later, listed `reclaim` as the automatic answer to foreign ownership. `cmd_reclaim` has no liveness
signal — it says so in its own comments — so it cannot tell a crashed session from a working one.
It replaces the owner, and `fullcycle-gate.sh` skips records owned by another session. So an
autonomous reclaim takes a live session's document out of its own gate while both sessions keep
working, and neither notices.

`reclaim` is now a `stops` entry with that reasoning attached. Automatic only when the handoff is
provably orphaned. `migrate` stays an internal recovery, because it refuses anything it cannot
represent losslessly and so cannot silently lose a record.

## F007 [high] the P6 block could not reach the state it promised — AGREED, fixed

Correct and embarrassing in a useful way. That fence was a REFERENCE LIST of `dstack` subcommands.
Round 001 told me to make registration fail-closed, I put `set -e` at the top, and in doing so
turned the reference lines into executable steps — so the success path ran `reg`, `reg`, `status`,
then `unreg`, deregistering the document it had just registered, then `reclaim` and `migrate` on
top. The failure path exited before reaching any of the recovery commands it was supposedly
offering. Neither end of it worked.

Split into a runnable block and a reference list, and the block now asserts its end state instead
of trusting an exit code:

```
normal path                      rc=0   "P6 registration confirmed"
a record missing from status     rc=1   "P6 BLOCKED: … is not registered"
```

The reference list carries the reason `reclaim` is not in it.

## F008 [medium] the transition table had no unique answer — AGREED, fixed

A missing pinned review model is simultaneously a nonzero run (retry under the next label) and a
missing required dependency (`codex-review` says surface it and stop). Two rules, one state, no
precedence — so the orchestrator could retry a model that will never appear. Now: `stops` wins over
`internal-recoveries`, missing required dependencies are an explicit stop, and automatic retry is
restricted to a DIAGNOSED transient failure. An undiagnosed nonzero run means read `err.txt` first.

## F009 [medium] and F010 [medium] the contradiction survived in the invoked skill — AGREED, fixed

Both are the same shape and both are mine: I fixed the launch invariant and the signal handlers in
`full-cycle` and left `codex-review` stating the opposite, so the orchestrator got two rules for one
decision. Both files now carry the identical launch invariant (one background call whose blocking
terminal step is `dstack run`; setup before it is required, dependent work after it is not) and the
identical handler form (disarm EXIT, clean once, exit with the signal's status). Measured for the
handler in bash and zsh: cleanup-only returns 0 having run twice; the corrected form returns 143
having run once.

## Verification

```
YAML blocks 0/1/2                                       all parse
  scheduling.autonomy = [rule, internal-recoveries, stops, bounds, notify]
bash tests/secret-guard.sh                              ✓ PASS
claude/skills/full-cycle/tests/check-parallel.test.sh     PASS
claude/skills/full-cycle/tests/skill-schema.test.sh       PASS
P6 block probed against the real registry                rc=0 confirmed / rc=1 on a missing record
```

The YAML broke once more during this round, again on a bare `: ` inside a plain multi-line scalar
(`not automatically retried: read …`). Third time; the pinned schema test catches it every time,
which is the argument for running it after every edit rather than at the end.

## Class-wide sweep (Step 0)

Class: *a fix that created the next defect*. Both highs are that — round 001's fail-closed
registration produced the `reclaim` hole and the `unreg` block. Swept every round-001 change for the
same shape: the `internal-recoveries`/`stops` split (fixed here), the `notify` mechanism (names
`PushNotification`, best-effort, no new failure path), and the launch-invariant rewording (fixed in
the invoked skill this round). Second class, *a rule stated in the contract and not in what the
contract invokes*: F009 and F010, so I diffed every shared rule between `full-cycle` and both codex
skills rather than checking the two the reviewer named.
