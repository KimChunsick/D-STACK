## Carried decisions — Round 002

- Cleanup ownership in `run` spans fork→publication: the trap goes up before `set -m` and comes
  down only after `exit` has landed. Both ends were found open once already (round 002); do not
  narrow either again.
- `$!` is the tiebreaker for the instruction-level window between `&` returning and `child=$!`.
  The shell sets it as part of executing the background command, so it distinguishes "nothing was
  launched" from "a child exists but is not yet recorded". The claim is released in the first case
  and never in the second. Reading it is bracketed by `set +u`/`set -u`, because an unset `$!` is
  fatal under `set -u` and a crash inside a signal handler is the opposite of a report.
- Every failure BEFORE the fork releases the launch claim; every path after it keeps the claim.
  That asymmetry is what lets `rm-run` be fail-closed on unknown state without stranding ordinary
  setup failures until the retention sweep.
- Not detaching is settled (design consult, then round 001's repair). The residual is SIGKILL-only
  and is covered by `rm-run` refusing to delete a capture while either recorded pid is alive — not
  by asserting the orphan cannot happen. Round 001 killed one such assertion; do not write another.
- Completion re-invocation when a harness-tracked background command exits is a MEASURED local
  behaviour of Claude Code client 2.1.220, not a documented platform guarantee. It is this
  change's load-bearing external dependency and is named as such in the code.
- Round 001 is itself the long, output-silent end-to-end run: 10.3 minutes through one background
  `dstack run`, zero bytes of command output visible to the harness, session re-invoked with no
  human input.
- A test that signals only after the child pid is observable cannot speak to the pre-record window.
  Round 002 caught exactly that overclaim; evidence must state which interval it actually covers.
- `prune` deliberately does not consult launch state; its eight-complete-day threshold against runs
  of 3–25 minutes is the argument. Non-blocking follow-up.
- `--stdin` retains a TOCTOU window because bash cannot express `O_NOFOLLOW`. Accepted: the callers
  are recipes in this repository naming a file the same session just wrote, in a mode-700 directory
  it created.

Consensus: disagreed
