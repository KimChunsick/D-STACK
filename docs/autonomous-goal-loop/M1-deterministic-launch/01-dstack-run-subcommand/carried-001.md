## Carried decisions — Round 001

- Not detaching the launched command is settled, by the pre-implementation design consult and by
  Round 001's repair of F001. The residual is SIGKILL-only: `run` cannot trap it, so a hard-killed
  supervisor can leave an orphan. That is covered by `rm-run` refusing to delete a capture while
  either recorded pid is alive — not by asserting the orphan cannot happen. Round 001 killed an
  earlier version of exactly that assertion, so do not reintroduce one.
- Completion re-invocation when a harness-tracked background command exits is a MEASURED local
  behaviour of Claude Code client 2.1.220, observed repeatedly in this session, and NOT a
  documented platform guarantee. It is this change's load-bearing external dependency and is named
  as such in the code.
- Round 001 is itself the long, output-silent end-to-end run: 10.3 minutes through one background
  `dstack run`, zero bytes of command output visible to the harness, session re-invoked with no
  human input. F004 was raised before that evidence existed.
- `prune` deliberately does not consult launch state; its eight-complete-day threshold against
  runs of 3–25 minutes is the argument. Recorded as a non-blocking follow-up, not a defect to fix.
- `--stdin` retains a TOCTOU window because bash cannot express `O_NOFOLLOW`. Accepted: the
  callers are recipes in this repository naming a file the same session just wrote, inside a
  directory it created at mode 700.

Consensus: disagreed
