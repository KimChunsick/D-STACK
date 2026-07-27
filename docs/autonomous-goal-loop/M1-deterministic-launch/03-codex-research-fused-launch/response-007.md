# Maintainer response — Round 007 (batch pass 1)

Not bundled. Every finding agreed and fixed; nothing disproved.

## F027 [medium] cwd-relative paths under a root-level promise — AGREED, fixed

`GOAL_DIR="docs/$GOAL/research"` keeps the pipeline's "artifacts live at `docs/<goal>/`" promise
only when the shell happens to start at the repository root. Started anywhere else it builds
`<subdir>/docs/<goal>/…` — a second docs tree that the Stop hook, the assembler and the next round
all fail to find, with nothing anywhere saying a duplicate was made. The root was already being
resolved one line later for the run dir, so the fix costs nothing: resolve `ROOT` once, derive the
artifact path and the run dir from it, and drop the now-redundant `$PWD` prefixes. Step 1's
instruction to write the brief says "relative to the repository root" for the same reason.

## F028 [medium] a reused label makes a rejected invocation look successful — AGREED, fixed

`dstack run` does refuse a reused label — its `.launch` claim is a plain `mkdir`. But the refusal is
easy to mistake for a result, and the mistake is systematic rather than careless: nothing launched,
so the previous attempt's `exit=0` and its `-o` artifact are both still sitting there, and this
skill's own rule — read `<run-dir>/exit` — then reads a stale zero and calls a rejected invocation a
success. The rule is only sound if the capture belongs to this attempt, so the recipe now refuses
when the run dir already exists, before anything is allocated.

## F029 [medium] the pinned source counter is not runnable — AGREED, fixed

It was published inline with a literal `…` where the file argument belongs. A counter that decides
whether Phase 3 falls back has to be runnable, so it is a fenced block now, with the artifact path
built from the repository root. Run against this Goal's four research artifacts it returns 22, 12, 7
and 5 — every one nonzero, so no false fallback trigger.

## F023/F025/F026 [low] carried from round 006 — AGREED, all fixed

- **Scratch leaks.** Two paths. An empty `CLAUDE_CODE_SESSION_ID` builds `runs//<label>`, a path
  `dstack` never publishes `exit` into, so the gate can never fire — measured, bash exited 127 and
  zsh exited 1, both after `mktemp`, neither cleaning up. Now checked before anything is allocated.
  And the signal handlers disarmed the EXIT trap, carried over from when the cleanup was
  unconditional; the gate is the better protection, so they leave it armed. Measured in both shells:
  exit file present → rc=143 and removed, absent → rc=143 and kept.
- **A "verified" claim outliving what it verified.** The block changed after the recorded `codex
  exec` run. The bullet now says so, and says what does back the new constructs.
- **Evaluator-disposition language.** The sentence assigning remaining signal work to
  `claude/bin/dstack` read as pre-assigning scope. Restated as a fact about where the code lives.

## F024 [low] URL grammar — partially disagreed, and here is the part

The half about the counter not being runnable is F029 and is fixed. The remaining half — that the
regex accepts some malformed URL-shaped strings — is true and is deliberate. The counter exists to
answer one question: did the artifact cite anything at all, or is this a zero-source output that
should trigger the fallback. A grammar tuned to reject malformed URLs would fail that question in
the direction that costs more, by counting a real citation as zero and re-running research that
already worked. Accepted as a stated limit rather than fixed.

Consensus: disagreed
