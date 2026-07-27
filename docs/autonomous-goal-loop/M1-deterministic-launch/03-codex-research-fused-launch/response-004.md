# Maintainer response — Round 004

Not bundled into any review round. The measurements are also in `## Carried decisions` of
`codex-review-004.md`.

## F015 [medium][security] the quoted assignment was escapable — AGREED, fixed

Round 003 told me to single-quote the placeholders and I did, and it was not enough, which is the
useful kind of being wrong. `x'$(printf PWNED)'` closes my literal, the substitution runs, and the
validator is handed `xPWNED` — a perfectly good slug. The check never had a chance; the damage
happened one line above it.

The fix is to stop putting these values in shell source at all. A `<<'SLUG'` heredoc expands
nothing — not `$`, not backticks, not quotes — so the same input arrives as an inert string:

```
assignment form   bash ACCEPTED [xPWNED]                zsh ACCEPTED [xPWNED]
heredoc form      bash REFUSED  [x'$(printf PWNED)']    zsh REFUSED  [x'$(printf PWNED)']
benign slug       bash ACCEPTED [autonomous-goal-loop]  zsh ACCEPTED [autonomous-goal-loop]
```

The suggested direction was argv or environment. A recipe the orchestrator pastes into a Bash call
has no argv from outside it, so the heredoc is the form of "as data, not as source" that this
context actually supports. Step 1's invariant stays, and is now redundancy rather than the only
defence — which is what it should have been from the start.

## F016 [medium] the trap suppressed cancellation — AGREED, fixed

Also a round-003 fix that was wrong in a way I did not check. I added `INT TERM HUP` to the trap so
the cleanup would run under zsh, and stopped there. A handler with no `exit` returns control to the
shell, which carries on:

```
cleanup-only:  bash rc=0   [CLEANSURVIVEDCLEAN]      zsh rc=0   [CLEANSURVIVEDCLEAN]
corrected:     bash rc=143 [CLEAN]                   zsh rc=143 [CLEAN]
normal path:   bash rc=0   [DONECLEAN]               zsh rc=0   [DONECLEAN]
```

So the wrapper survived a TERM, cleaned up twice, and could report success — while `rm -rf
"$SCRATCH"` pulled the cwd out from under a still-running `codex exec`. Now each signal handler
disarms EXIT, cleans once, and exits with the signal's status.

I did not implement signal propagation to `dstack`. The recipe runs `dstack run` in the foreground,
so a signal to the process group reaches it already; a wrapper-only signal is the case propagation
would cover, and the honest answer there is the retry fence — check the capture for a live pid or
group before relaunching. Claiming propagation I had not built would have been the same class of
error as the two above. Stated as a limit instead.

## F017 [low] no post-fix end-to-end run — AGREED, fixed

Fair, and more clearly fair than when the same point was raised at round 003, because F015 and F016
changed the fence materially rather than additively. The corrected block was run in full and its
result is in `task.md`.

## F018 [low][security] disposition language in the residual — AGREED, and I concede the point I
held at round 003

At round 003 I held that a process rule addressed to the orchestrator is legitimate content in an
orchestrator's instruction file, and I still think that distinction is right in general. This round
made the cost concrete rather than theoretical: my residual said the launcher's gaps were accepted
and belonged to another review unit, and the cancellation defect the reviewer then found was sitting
just past where that sentence invited a reader to stop. A disposition is a claim that a question is
settled, and settled questions are exactly what an adversarial round exists to reopen. The residual
now states what is true of the tool; the follow-up bookkeeping lives in `findings.md`, which is
mine, not in the payload.

## Class-wide sweep (Step 0)

Class: *a fix that addressed the symptom the reviewer named and not the mechanism underneath*.
F015 and F016 are both round-003 repairs of exactly that shape — quote it (but not against
escaping), trap it (but not so it terminates). Swept every other round-003 repair for the same
pattern: the Step 1 invariant (prose, no mechanism to be wrong about, and now backed by the
heredoc), the traversal-depth correction (a fact, re-measured), and the pid-window caveat (a stated
limit rather than a fix, which is what it claims to be). Where a mechanism was in play I ran it
instead of reasoning about it, which is what produced both tables above.
