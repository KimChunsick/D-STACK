# Maintainer response — Round 003

Not bundled into any review round. The measurements are also in `## Carried decisions` of
`codex-review-003.md`, because a disproof the next round cannot see gets re-raised.

## F011 [medium][security] validation ran too late — AGREED, fixed

Two separate reasons it was too late, and I had only thought about neither:

1. **Step 1 already builds a path from both values** — `docs/<goal>/research/<topic>.brief.txt` —
   so by the time Step 2's `case` runs, a traversal has already been used once.
2. **Substitution into double quotes executes.** These are placeholders replaced textually, so
   `GOAL="<goal>"` with a `$(…)` in it runs at assignment, before any check.

Both fixed. The slug invariant now lives in Step 1 and is stated as the rule; Step 2's check is
explicitly a backstop. The assignments are single-quoted:

```
bash  double: GOAL=[PWNED]              single: GOAL=[$(printf PWNED)]
zsh   double: GOAL=[PWNED]              single: GOAL=[$(printf PWNED)]
then: case "$GOAL" in … *[!A-Za-z0-9_-]*) → "refusing: not a plain slug"
```

## F012 [low] the recovery guarantee overreaches in the fork window — AGREED, fixed

Right on both halves. `run` forks and then writes `.launch/child`, so a kill inside that window
leaves a live group with no recorded pid, and my sentence promised the pid would be there. And the
fence's own trap removes `$SCRATCH` when the launching shell exits, which can pull the cwd out from
under a surviving orphan. Both are now stated. `rm-run` already treats a missing child record as
unknown-and-live, which is the mitigation for the first — that is a real mitigation and I say so
rather than presenting it as a fix. Narrowing the window is a `dstack` change and is a follow-up.

## F013 [low] the verification claim and the wrong traversal depth — AGREED on the depth, PARTLY on
the claim

The depth is simply wrong and I introduced the error while rewriting the comment. Measured:

```
from docs/<goal>/research:  ../../AGENTS.md    -> <repo>/docs/AGENTS.md
                            ../../../AGENTS.md -> <repo>/AGENTS.md   (the tracked one)
```

Fixed to `../../../AGENTS`.

On "no post-fix exact-block run is recorded": true, and I am not running another 6-minute research
round to re-verify it. The change since the recorded run is additive — two assignments re-quoted
and a validation loop added ahead of the launch — and it does not alter the command `dstack run`
executes, which is what the capture attests. The validation loop itself was probed directly across
nine inputs including the reviewer's counterexample. That is the evidence, stated for what it is: a
verified recorded run of the launch, plus a verified direct run of the guard added in front of it,
not a single end-to-end re-run of both together.

## F014 [low][security] evaluator-scope directives — PARTLY agreed

Agreed on `task.md`. "Those belong in the pipeline rule, not in this file; carried to T04" is me
telling a reader where a finding should be filed, inside the reviewed payload. Reworded to state
what was recorded and where, without the disposition.

Held on the skill's "an allowlist does not grow to absorb a finding". That is a rule the
orchestrator follows when assembling the NEXT round — it is the operating instruction this file
exists to give, and the file is an instruction document; every sentence in it directs the
orchestrator. Removing it does not make the reviewed material neutral, it removes a process rule
from the only place it is written. The distinction I am drawing: a rule addressed to the
orchestrator about how to run the pipeline is legitimate content; a sentence addressed to the
EVALUATOR about how to read the payload is not. The reviewer correctly ignored it and reported it,
which is exactly the behaviour the contract asks for, and this response is the disagreement being
recorded rather than either side giving way.

## Class-wide sweep (Step 0)

Class: *a guard placed after the thing it guards*. Swept the whole file for values used before they
are checked — `GOAL`/`TOPIC` (fixed, moved to Step 1), `LABEL` (derived from `GOAL`, so it inherits
the check, and `dstack` validates it again), the brief path (now downstream of the Step 1
invariant), and `$SCRATCH` (created by `mktemp -d`, never interpolated from input). Second class,
*a claim adopted without measuring it*: re-measured the signal coverage table and the single-quote
behaviour rather than reasoning about them, and corrected the traversal depth the same way.
