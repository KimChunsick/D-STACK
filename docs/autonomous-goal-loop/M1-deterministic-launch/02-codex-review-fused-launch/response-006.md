# Maintainer response — Round 006 (reopened unit)

Not bundled. Measurements are in `## Carried decisions` of `codex-review-006.md`.

## F027 [medium] my §3 repair left a broken sentence — AGREED, fixed

Round 005 told me to delete an exemption. I deleted the middle of the sentence containing it and
left "a restatement, a variant of an already-recorded class in / or an objection", which reads as
the old exemption with a hole in it, immediately above the paragraph saying the opposite. §3 is now
stated POSITIVELY — exactly two things do not reopen — so there is nothing left to half-delete.

## F028 [medium][security] unquoted allowlist entries and label — AGREED, fixed

Reproduced: `ALLOW=( */task.md )` expands into two entries under bash and zsh both. Every literal
in the array is quoted now, and `LABEL="<label>"` is assigned once and passed as `"$LABEL"` — an
unquoted placeholder is parsed as shell syntax before `dstack` ever sees it, so its label validator
cannot help.

## F029 [medium] the `adversarial-review` contract override — RAISED A FOURTH TIME, unchanged

Fourth round, same two clauses, same disposition. This file governs the pipeline's closure
semantics; that file needs the same two edits; it is outside this declaration. It is a REAL
outstanding inconsistency and is named as one in `findings.md`, not closed.

## F030 [low] cleanup leaks on pre-launch failure and after a deferred signal — AGREED in part

The scratch trap is now gated on `<run-dir>/exit`, so it leaks whenever quiescence is unknown —
which is deliberate and is the safer direction. A leaked `mktemp` dir under the OS temp root costs
nothing; deleting a live child's cwd is a real failure. Recorded as accepted rather than fixed.

## F031 [low] `SIGPROF` is not untrappable — AGREED, fixed, and I was plainly wrong

Measured: an explicit `trap … PROF` handler runs in bash and zsh both, rc=155. `SIGPROF` is
catchable; it is simply absent from `RUN_SIGNALS` and outside bash's implicit EXIT-trap firing.
"Untrappable" applies to `SIGKILL` and nothing else. Corrected in this file and in `codex-research`,
where I had copied the same error.
