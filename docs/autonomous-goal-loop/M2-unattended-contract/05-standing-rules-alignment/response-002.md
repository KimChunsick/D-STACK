# Maintainer response — Round 002 (batch pass 2)

Not bundled. All four agreed. Two fixed here; two are against `full-cycle/SKILL.md`, which is inside
an OPEN review bundle right now, so they are deferred under the freeze-rule rather than edited
mid-round.

## F005 [medium] `reclaim` still has two transitions — AGREED, DEFERRED (frozen file)

Confirmed by reading both sites. `autonomy.stops` now says there is no autonomous path for a
foreign-owned document; the milestone-boundary handoff prose, two hundred lines later in the same
file, says to `/clear` and then run `reclaim` on the records the session-id rotation orphaned. One
state, two rules — which is the same defect round 001 raised between the summary and the authority,
now found *inside* the authority.

**Resolution, for unit 04's next round:** the reviewer's second option. After `/clear` the operator
is present by construction — they just typed it — so the handoff reclaim is an explicit user-input
confirmation, not something the unattended pipeline does. The first option, recording verifiable
handoff provenance before `/clear`, would mean inventing a liveness signal `dstack` deliberately
does not have; without one, "these are my own orphaned records" is unprovable and confirmation is
the honest form. Cost is one confirmation at a milestone boundary where a human is already there.

**Why not now:** `claude/skills/full-cycle/SKILL.md` is in unit 04's round-007 bundle, open as this
is written. The freeze-rule says work touching a frozen file is deferred whatever unit it belongs
to, and that is exactly what keeps concurrent rounds honest — editing it would change the file under
a review already in flight. Carried into unit 04's ledger.

## F006 [low] the lifecycle correction landed in the summary and not the authority — AGREED, DEFERRED

Round 001 fixed "the call does not return until the command finishes" in `CLAUDE.md`;
`waits.external` in the skill still carries the original wording. Fair, and slightly pointed: fixing
the copy and not the original is the shape of the very defect round 001 was about. Same freeze, same
deferral, same unit.

## F007 [low] the standing file's honest limits were incomplete — AGREED, fixed

This is the one that mattered operationally. `CLAUDE.md` listed three residuals and omitted two, and
the two it omitted are the ones you need *when the skill is not loaded*: a background shell may be
reaped under OS memory pressure after 30 idle minutes, and `SIGKILL`/`SIGPROF` can orphan `dstack
run`'s child. Without the second, a capture with no terminal record reads as a plain failure and the
documented move is to relaunch — over a live `codex exec`, spending credits twice and letting two
runs write one label.

Both are in now. Verified with newlines folded, because a naive phrase grep reported two false
absences purely from line wrapping, and wrapping is not semantic.

## F008 [low, security] an evaluator directive in the task artifact — AGREED, fixed

Fourth instance of this class in the Goal, and the first where the repair *was* the next instance:
round 001's F004 asked the gate row to stop overstating, and the row I wrote to fix it said
"behaviour itself is out of scope here". That is an instruction to the reviewer sitting inside data
the reviewer is told to distrust.

The reviewer's demonstration is what makes this a real finding rather than a style note: treating
the disclaimer as untrusted is precisely what surfaced F007. Accepting it would have hidden a
defect. The section now says what the commands establish and what they do not, and issues no
instruction.

Consensus: disagreed
