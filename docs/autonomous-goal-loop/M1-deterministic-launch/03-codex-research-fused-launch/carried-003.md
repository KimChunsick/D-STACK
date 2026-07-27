## Carried decisions — Round 003
- The slug invariant is established in **Step 1**, not Step 2. Step 2's `case` is a backstop, and a
  backstop is all it can be: Step 1 already builds a path from both values, and a placeholder
  substituted into double quotes runs its `$(…)` at assignment time, before any later check exists.
- Placeholders that get substituted TEXTUALLY are single-quoted. Measured: `GOAL="$(printf PWNED)"`
  yields `PWNED` in both bash and zsh; `GOAL='$(printf PWNED)'` yields the literal, which the `case`
  then refuses.
- Traversal depth, corrected: from `docs/<goal>/research`, `../../AGENTS.md` is `docs/AGENTS.md` and
  the tracked root file needs `../../../AGENTS.md`. The reviewer was right and the comment was
  wrong.
- The recovery path has two real caveats and they are stated rather than smoothed over: the launched
  pid is recorded just after the fork, so a kill inside that window leaves a live group with no
  record (`rm-run` treats a missing record as live, which is the mitigation), and the fence's trap
  removes `$SCRATCH` on exit, so a surviving orphan can lose its cwd.
- Signal coverage is the measured table, not the trap list: bash 3.2.57 fires the EXIT trap on
  fatal signals, so the gaps are `SIGKILL` and `SIGPROF` only. zsh 5.9 never fires it, which is why
  the fence names its signals.

Consensus: disagreed
