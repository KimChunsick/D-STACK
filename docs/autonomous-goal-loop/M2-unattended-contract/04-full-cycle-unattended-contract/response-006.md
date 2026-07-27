# Maintainer response — Round 006 (batch pass 1)

Not bundled. Every finding agreed and fixed.

## F023 [high] the P6 fence registers task-depth paths whatever the granularity — AGREED, fixed

The granularity table sits directly above the fence and warns that registering the wrong level
silently un-gates the milestone's own doc. The fence then iterated a literal
`<Mn>/<NN-task>/task.md`. The warning and the recipe that violates it were four lines apart.

Deriving the depth is a deterministic transform, so code does it: `check-registration.sh --depth`
returns 3 or 2 from the same GOAL.md parse the check itself uses, and the fence reads it. Two
consequences worth stating. The fence can no longer disagree with the check about what a review unit
is, and a Goal with a missing or ambiguous granularity now fails at the FIRST line instead of after
the wrong documents are already registered. `reg` is idempotent for a document this session owns, so
re-running the fence is safe.

## F024 [high] the checker does not parse the same declaration source as the scheduler — AGREED, fixed

Two parsers with different ideas of what counts as a declaration is worse than one parser that is
wrong, because the disagreement is invisible: each is self-consistent and they only diverge on the
files where it matters. The fixture makes it concrete rather than theoretical. Given a GOAL.md with
a fenced decomposition example in an earlier section, the old parser read that block's
`Review granularity: **per milestone**` and its two fake `T91`/`T92` rows — and read NONE of the
three real rows, because the fence's closing backticks toggled it on just as the real section
started. The rewritten parser reads 3 rows and `task`.

It now mirrors `check-parallel.sh` exactly: fences tracked globally from line one, task rows only at
column zero with the `-` marker, a repeated section heading keeping the section open.

## F025 [medium] `RUNDIR="$RD"` before `RD` is defined — AGREED, fixed where it lives

Real, and fixed in `codex-review`'s Step 2 fence rather than here — this file states the contract,
that file holds the recipe. Recorded in both ledgers so neither can lose it.

## F026 [medium] concrete-MEDIUM cap closure has two transitions — AGREED, fixed

P9 said to escalate to the user when the round budget is reached with "blockers" still open.
`blockers` means high AND medium — Step 2a's own grep is `high|medium` — so P9 demanded a human for
exactly the case §4 and `autonomy.stops` both close without one. Two transitions out of one state is
how an unattended run stalls on a medium nobody needs to see, which is the failure this Goal exists
to remove.

P9 now defers cap closure entirely to §4 and states no rule of its own, and it says explicitly what
the single rule is: record every open finding with severity and evidence, seal `Consensus: resolved`,
name them in the final report, escalate only a concrete HIGH.

## F027 [medium] the exit-2 guarantee is false when identity checks fail — AGREED, fixed

An erased delta is indistinguishable from no delta, so a failed comparator reads as a pass. Fixed in
`check-registration.sh`: every `find`, `sort`, `comm` and id-extraction status is checked, and a
count-in/count-out guard dies when the extractor reads fewer ids than the parser found rows — the
specific shape where a silently shrinking `want` set makes every comparison empty and the check
passes by producing nothing.

## Also fixed this round, not from a finding

`autonomy.stops` carried a carve-out letting `reclaim` run autonomously for a "provably orphaned"
handoff whose owner is this session. Checking it against the tool: `reg` returns 0 for a document
this session already owns, so that state is never reached, and every other case is unprovable
without a liveness signal. The other half of the carve-out — "or the user says so" — is the user
answering, not autonomy. Removed, which also makes `CLAUDE.md`'s stricter summary correct rather
than merely stricter (unit 05, F002).

Consensus: disagreed
