## Carried decisions — Round 004

- Publication is GATED on confirmed group quiescence at both call sites, not merely warned about.
  A group that survives SIGKILL yields no terminal record on purpose: nonterminal is what keeps
  `rm-run` guarding the capture. `run_done` is the single finalisation state (published, or
  deliberately not published) and the EXIT handler cannot re-enter to undo a refusal.
- The deployed interpreter is /bin/bash 3.2.57 — both the shebang target and the shell on PATH —
  and it runs an EXIT trap fired by `exit`-from-inside-a-function with that function's locals still
  readable, including from a nested `die`. Verified directly, and by fault-injecting the post-fork
  child-record failure into the real script (group torn down, exit 143 published, no stray).
  `run_cleanup` still defaults every read, and the claim release is guarded on `$d` being non-empty
  so it can never resolve to `/.launch`.
- pgid recycling between the liveness probe and the signal is an ACCEPTED RESIDUAL, recorded where
  the signalling happens. The window needs the group fully gone first, so it is the probe-to-signal
  instant only.
- `$child` is both pid and pgid; every liveness question about launched work asks the GROUP.
- One cleanup owner (`run_cleanup` on EXIT + `INT TERM HUP QUIT PIPE ALRM USR1 USR2`), armed one
  statement after the claim, disarmed only after publication. Every failure before the fork
  releases the claim; every path after it keeps it.
- `$!` is the tiebreaker for the window between `&` and `child=$!`.
- Evidence must exercise a child that does NOT honour the signal under test, and must run under the
  interpreter the shebang names. Rounds 003 and 004 each caught a version of the opposite mistake.
- Not detaching is settled; the residual is SIGKILL-only and is covered by `rm-run` refusing to
  delete a capture while either recorded pid, or the launched group, is alive.
- Completion re-invocation on background-command exit is a MEASURED local behaviour of client
  2.1.220, not a documented guarantee — the change's load-bearing external dependency.
- `prune` does not consult launch state (eight-day threshold vs 3–25 minute runs); `--stdin`
  retains a TOCTOU window (bash has no `O_NOFOLLOW`). Both non-blocking follow-ups.
- The bundle ratchet cannot be held while this unit's sealed rounds cannot compact (the assembler
  wants a `## Carried decisions` section inside the round file; the termination rules say the round
  file holds nothing but findings, size and consensus). Defect recorded against the task that owns
  the `codex-review` skill; sealed rounds are not rewritten to dodge it.

Consensus: disagreed
