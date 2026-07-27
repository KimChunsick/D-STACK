# Maintainer response — Round 007 (batch pass 2, §4 cap closure)

Not bundled. One high, two mediums, all fixed, plus two deferred from unit 05 that landed once their
freeze lifted. This is the closure round.

## F028 [high] checker failures with no autonomy transition — AGREED, fixed

Correct and important for an unattended run: `check-registration.sh` refuses a wrong-depth or closed
registration, `set -e` halts P6, and nothing in `internal-recoveries` or `stops` says what to do.
That is a dead end, and a dead end with nobody watching is a silent stall.

Three transitions added. A document THIS SESSION registered that must not be gets `unreg`-ed and the
check re-run — a genuine recovery rather than a disguised `reclaim`, because the record is our own,
so releasing it takes nothing from anyone and the gate it was holding was over a document no phase
governs. A STRUCTURAL mismatch goes back to P6 or P5 instead of being registered around. Exit 2 is
deliberately excluded: a check that could not run must never be treated as one that found nothing.

## F029 [medium] `find -exec` masks the failure of what it runs — AGREED, fixed

Measured: `find . -exec false {} \;` exits 0. So a failed `reg` was invisible and every later
document kept being claimed, turning one ownership conflict into several. This was my own round-006
fix for the hard-coded-glob bug, and it traded a wrong level for a swallowed status.

## F030 [medium, security] `<goal>` interpolated into shell source — AGREED, fixed

Verified: the substituted value `safe; printf INJECTED` executed the second command under both
shells. Now refused by the same plain-slug `case` `codex-research` uses, with the same honest
framing — the orchestrator writes the whole command, so this is defence in depth against a mistake
and not a boundary against an adversary; if the value ever comes from a user string, a file, or a
tool result, the recipe is the wrong shape and no quoting fixes it.

## F031 register-before-classify — found while fixing F029, fixed

The reviewer named it too: a depth-wide loop claims undeclared folders and already-closed units
before anything decides whether they belong, which is also why "safe to re-run" was false. The fence
now reads `--list`, which emits GOAL.md plus every declared, scaffolded, still-open unit, so it
cannot create the state the checker is about to refuse.

## F032 / F033 — deferred from unit 05, completed here

Unit 05's round 002 raised two findings against this file and deferred them under the freeze-rule
while this bundle was open. Both landed once it closed:

- The milestone-boundary handoff prescribed `/clear` then `reclaim`, while `autonomy.stops` forbids
  autonomous `reclaim`. One state, two rules, inside a single file. Resolved by presence rather than
  by ranking: after `/clear` a person is at the keyboard by construction, so the handoff lists what
  it intends to reclaim and asks. Nothing can prove a record is orphaned without a liveness signal,
  and reclaiming a live session's document un-gates its work while both keep running.
- `waits.external` still said "the call does not return" of a `run_in_background` Bash call. It names
  the STEP now, and says the tool call returns immediately, because "the call blocks" reads as a
  stuck harness and invites the hand-rolled watcher this contract replaced.

Consensus: resolved
