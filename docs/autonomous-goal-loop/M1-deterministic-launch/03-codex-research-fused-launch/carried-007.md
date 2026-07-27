## Carried decisions — Round 007
- **Every path anchors to the repository root, not to the cwd.** `docs/<goal>/research` is a promise
  about where the pipeline's artifacts live, and a bare relative path keeps it only when the shell
  happens to start at the root. Anywhere else it silently builds `<subdir>/docs/<goal>/…` — a second
  docs tree the gate, the assembler and the next round all fail to find. `ROOT` is resolved once and
  both the artifact path and the run dir come from it.
- **A reused label is refused BEFORE anything is allocated.** `dstack run` does refuse one, but the
  refusal is easy to mistake for a result: nothing launches, so the previous attempt's `exit=0` and
  its `-o` artifact are still sitting there, and Step 2a's own rule then reads a stale zero and
  calls a rejected invocation a success. Checking for the run dir first is what makes that rule
  sound.
- **An EMPTY `CLAUDE_CODE_SESSION_ID` is checked explicitly.** `set -u` catches an unset variable
  and not an empty one, and an empty one builds `runs//<label>` — a path `dstack` never publishes
  `exit` into, so the cleanup gate can never fire. Measured: bash exited 127 and zsh exited 1, both
  after `mktemp`, neither cleaning up. Both checks now run before `mktemp`, so a refusal leaks
  nothing.
- **The signal handlers leave the gated EXIT trap ARMED**, for the same measured reason as the
  review skill: the deferral means the handler usually runs after `exit` was published, which is
  exactly when removal is correct, and `trap - EXIT` made that a guaranteed leak.
- **The pinned source counter is a runnable fence, not prose with an ellipsis in it.** Run against
  this Goal's four research artifacts it returns 22, 12, 7 and 5 — every one nonzero, so no false
  fallback trigger.
- **The "verified runnable" bullet now says what the current block is NOT covered by.** Root
  anchoring, the two pre-checks and the armed trap all landed after the recorded `codex exec` run,
  and what backs them is direct measurement of the constructs themselves.

Consensus: disagreed
