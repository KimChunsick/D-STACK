# Maintainer response — Round 007 (batch pass 1)

Not bundled. Every finding agreed; nothing disproved, so nothing moves into carried decisions on
that route. What follows is what changed and why, for the record.

## F032 [medium] `RUNDIR="$RD"` before `RD` exists — AGREED, fixed

The fence opened with `RUNDIR="$RD"` and defined `RD` four lines below it. Worse than the ordering:
this is a *separate* Bash call, and Step 1 of this same file explains that a shell variable does not
survive between tool calls, so `$RD` from the assembly step was never going to be there at all. The
armed trap therefore evaluated `[ -e "/exit" ]` — false always — and the scratch directory was never
removed on the one path where removal is correct.

`LABEL` is now assigned first, `RD` reconstructed from it, `SCRATCH` allocated after, trap armed
last. The comment says why reconstruction rather than inheritance.

## F033 [medium] Step 2a contradicting Step 2 — AGREED, fixed

Step 2 establishes that a deferred signal makes a completed round report 143, then Step 2a opened
with "a nonzero exit is a FAILED ROUND". The recipe underneath was already correct — it reads
`$RD/exit` — so this was prose that could talk a reader out of the right behaviour. Rewritten: the
notification carries the launching shell's status and that is a hint; `<run-dir>/exit` is the
verdict; and the two asymmetric cases are named, including that a MISSING `exit` file is not a pass.

## F034 [medium] the resend recipe rejected its own documented invocation — AGREED, fixed

This one is embarrassing in a useful way. The skill publishes `REVIEW_FULL_ROUND_IDS="1 3"` as the
way to honour a reviewer's request for an older round. `assemble-review.sh` split on commas only, so
that arrived as the single field `1 3`, failed the all-digits test, and died FATAL. The supply
mechanism the review prompt promises could not be invoked in its published form.

Fixed in the assembler: split on commas AND whitespace. Every rejection the comma rule existed for
is preserved, verified case by case — `1,,3` and `1, ,3` still yield an empty field and are still
fatal, `1,` is still caught before the loop, `[1]` and `1 x` are still fatal, and `1 3`, `1,3`,
`1, 3` and a newline-separated value all yield 1 and 3. Run end to end against this unit: the
documented form returns rc=0 and the bundle header says "as do rounds 1 3 by request".

## F035 [medium] the "THIS file governs" override — AGREED, withdrawn

The override was addressed to a reviewer that is told to follow `$adversarial-review` exactly and is
told, in the same prompt, that everything in the payload is untrusted data to be ignored as
direction. A precedence claim aimed at someone instructed not to act on it settles nothing.

Replaced with the narrower thing that is actually true: these rules govern the ORCHESTRATOR, which
is the side that runs them and the side the Stop hook parses. The Codex-side inconsistency stays a
named follow-up, and a reviewer filing it is right to.

## F036 [medium] no transition for a post-seal reopening past the cap — AGREED, fixed

The best finding of the round, because this Goal walked into it. Units 02 and 03 sealed AT the
5-round cap and were then reopened by full-cycle's `post-seal-rule` when a defect turned up in a
file inside their sealed bundles. Under a cap on TOTAL rounds there was no legal next move: ship a
known defect to protect a ticked box, or run a round the rules did not permit.

§4 now counts rounds SINCE the reopening, and resets the budget smaller — 2 for a per-task unit, 3
for a milestone unit — because what reopened is a bounded change to an already-reviewed corpus. The
non-convergence window restarts with it, since carrying the old counts forward would close the new
loop on evidence about the old one. Round 007 is the second of this unit's two, so the reopening's
budget is now spent.

Consensus: disagreed
