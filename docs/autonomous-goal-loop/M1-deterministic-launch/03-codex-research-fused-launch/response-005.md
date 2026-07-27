# Maintainer response — Round 005

Not bundled into any review round. This is the round cap for a per-task unit, and the loop closes
here. Every concrete finding below is fixed or resolved; nothing is deferred as an open defect.

## F019 [medium][security] the heredoc delimiter is injectable — AGREED, and the CLAIM is withdrawn

Three rounds, three quoting forms, three escapes. This round's is the cleanest: a payload whose
body contains a line equal to the delimiter closes the heredoc, and everything after it is shell
source. Reproduced in both shells — `GOAL=[valid] ACCEPTED PWNED=1`. And the form had a second,
duller failure I had not noticed: `SLUG` is itself a valid slug under the grammar, so a legitimate
topic named `SLUG` would break the fence.

No delimiter fixes it. A payload can read the delimiter out of the recipe, and I confirmed that
`SLUG.END` and `__SLUG__` fail identically.

So I stopped patching and looked at what I had actually been claiming. `<goal>` and `<topic>` are
slugs the orchestrator picks from the Goal's own name. Nothing outside the session supplies them.
There is no adversary in the threat model, and there cannot be a boundary here anyway, because the
orchestrator writes the entire command — no construct inside a template defends against the thing
that writes the template. What the check genuinely does is catch a MISTAKE, and the mistake is
real: a `..` component that puts `-o` on a tracked file.

The recipe now says exactly that, keeps the simplest form (a single-quoted assignment, since the
heredoc bought nothing and added a delimiter hazard), and states the condition that would make it
the wrong shape: if these values ever come from a user string, a file, or a tool result, they must
reach the process as argv or environment data set by the caller. That is the reviewer's suggested
direction, recorded as the rule for when it applies rather than pretended to be already in place.

I would rather ship an accurate statement of a narrow guarantee than a fourth quoting form that
survives until someone tries the next trick.

## F020 [medium] a wrapper-only signal does not cancel the child — AGREED, fixed, twice over

The sharpest finding of the round, and it invalidates the fix I made at round 004. Both shells defer
a pending trap while a foreground command runs, so:

```
bash 3.2.57   rc=143   CHILD_STARTED CLEAN CHILD_FINISHED
zsh 5.9       rc=143   CHILD_STARTED CLEAN CHILD_FINISHED
```

Two separate defects in one line. The round finishes and the wrapper reports 143, so the documented
"nonzero is a failed round" rule throws away completed work and pays for another. And `CLEAN` prints
BEFORE `CHILD_FINISHED` — my round-004 handler was deleting the scratch directory out from under a
live `codex exec`, which is worse than the leak it was fixing.

Both fixed. `<run-dir>/exit` is now stated as the round's status and the notification as a hint,
with the measurement in the file. And the signal handlers no longer clean up at all: they terminate
with the signal's status and leave the directory, because a leaked temp dir is free and deleting a
running process's cwd is not. Only normal completion removes it.

I did not implement propagation to `dstack`. A wrapper-only signal is the case it would cover, and
the capture file plus the retry fence already answer it correctly. Claiming propagation I had not
built is the mistake rounds 003 and 004 both made.

## F021 [low] the printed measurement command is wrong — AGREED, fixed

`/bin/bash -c "… kill -<sig> $$ …"` has its `$$` expanded by the invoking shell, so it signals that
shell rather than the bash under test. The measurements themselves were taken with escaping and
stand, but the command as printed does not reproduce them. Now single-quoted with the signal name
passed as an argument.

I proved the finding by accident while re-measuring for this round: my probe signalled its own shell
and the tool call died with 144. Cheapest possible confirmation.

## F022 [low] the source-count command — AGREED, fixed

`[^ )]*` accepts a bare `https://` as a source and counts one URL twice if a comma follows.
Now `grep -oE 'https?://[A-Za-z0-9._-]+[^ )]*' | sed 's/[.,;]*$//'`. Verified: 22 / 12 / 5 on the
three real artifacts, and a Sources section containing only `https://` counts 0, so the fallback
still triggers.

## F018 [low][security] disposition language — AGREED, fixed

"Residual, accepted" → "Residual". The bookkeeping about which file owns a follow-up lives in
`findings.md`, which is mine, not in the reviewed payload.

## Closure (§4 round cap)

Five rounds is the cap for a per-task unit. Open concrete findings at the end of this round: **0**.
Raised per round ran 3, 2, 1, 2, 2 — flat rather than decaying, and the reason is visible in the
ledger: rounds 3, 4 and 5 each found a defect in the PREVIOUS round's fix, three times in the same
two lines of shell. That is the honest signal, and it is why F019's resolution is a withdrawn claim
rather than a fourth attempt.

Follow-ups recorded against other files, none of them open defects in this unit: `SIGPROF` coverage
and the fork-to-pid-record window in `claude/bin/dstack`.

Sealed `Consensus: resolved`.

## Class-wide sweep (Step 0)

Class: *a fix that is wrong in the same place as the fix before it*. Rounds 3, 4 and 5 all landed on
the placeholder handling and the trap, and each repair introduced the next finding. The sweep this
time was not "where else does this pattern appear" but "what am I assuming about these two lines" —
which produced the withdrawal in F019 and the second defect in F020 that the reviewer had not named.
Also swept every measurement printed in the file for the F021 class, since a command that does not
reproduce is a claim without evidence.
