# Maintainer response — Round 001 (batch pass 1)

Not bundled. Every finding agreed and fixed.

## F001 / F002 [medium] the summary is not equivalent to its authority — AGREED, fixed in both files

One defect seen from two sides. `CLAUDE.md` §0 lists what stops an unattended run; `full-cycle`'s
`scheduling.autonomy.stops` is the authority. They disagreed in both directions at once, which is
the worst arrangement — whichever one you read, you are wrong about something.

- **The summary was missing a stop.** A `dstack reg` that fails for a cause `migrate` cannot fix —
  an unusable session id, an unwritable registry, a `status` line that never says `(this session)` —
  is a stop in the authority and absent from the summary. Under the unattended rule that reads as
  permission to continue ungated. Added, and the list now says it is a summary and names the
  authority, because a summary that quietly drops an entry is exactly how a run passes a stop.
- **The summary was STRICTER about `reclaim` than the authority**, and here the summary was right.
  The authority carved out a "provably orphaned" handoff whose owner is this session. That state
  does not exist: `reg` returns 0 for a document this session already owns, so it never refuses in
  that case, and every remaining case is unprovable without a liveness signal. The other half of the
  carve-out, "or the user says so", is the user answering the question rather than autonomy. So the
  fix went into the AUTHORITY — the carve-out is gone — and the two now agree without loosening
  anything.

Fixing only the summary would have left two documents that can drift again on the next edit. This is
the same failure mode as the reviewer's own point, one level up.

## F003 [low] blocking attributed to the wrong lifecycle — AGREED, fixed

"The call does not return until the command finishes" is false of the Bash tool call — with
`run_in_background` it returns immediately, and it is the background task that stays alive. The rule
it was defending is unchanged and still correct: a line placed after `dstack run` does not execute
until the round is over, so nothing whose result you need may sit there. But stating it of the wrong
object is not harmless. Someone reading "the call blocks" concludes the harness is stuck and reaches
for exactly the hand-rolled watcher this Goal removed.

## F004 [low] a gate row claiming more than the evidence — AGREED, fixed

The E2E section said behaviour was not verified and the gate row said behaviour was confirmed by
direct run. Under this repo's no-TDD policy the row's wording is free, so there was no reason for it
to overstate; it now says what the recorded commands actually establish.

Consensus: disagreed
