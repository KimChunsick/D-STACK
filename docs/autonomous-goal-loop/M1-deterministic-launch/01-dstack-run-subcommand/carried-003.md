## Carried decisions — Round 003

- `$child` is both pid and pgid. Every liveness question about the launched work asks the GROUP
  (`kill -0 -<pgid>`); only questions about the supervisor itself ask a bare pid. Round 003
  reproduced a dead leader with a live descendant, so leader status is never evidence of teardown.
- Terminal publication is gated on group quiescence with bounded TERM→KILL escalation (5s per
  stage), on the NORMAL path as well as the abort path: on normal completion the leader exited on
  its own, so a survivor is a leak and publishing while it writes is the defect.
- One cleanup owner — `run_cleanup`, armed on EXIT plus `INT TERM HUP QUIT PIPE ALRM USR1 USR2`,
  one statement after the claim succeeds, disarmed only after `run_published=1`. Because EXIT is
  covered, every `die` past the claim leaves through it; there is no second release path.
- `$!` remains the tiebreaker for the instruction-level window between `&` and `child=$!`.
- Every failure BEFORE the fork releases the launch claim; every path after it keeps it.
- Evidence must exercise a child that does NOT honour the signal under test. A `sleep`-based probe
  demonstrates nothing about teardown completeness — that mistake is what round 003 caught.
- bash CACHES a reaped job's status, so a second `wait` returns it rather than 127; `kill -0` asks
  about the pid's current occupant. Do not reintroduce a post-wait liveness loop.
- Not detaching is settled. The residual is SIGKILL-only, covered by `rm-run` refusing to delete a
  capture while either recorded pid, or the launched group, is alive.
- Completion re-invocation on background-command exit is a MEASURED local behaviour of client
  2.1.220, not a documented guarantee. It is the change's load-bearing external dependency.
- Round 001 is itself the long, output-silent end-to-end run (10.3 min, zero harness-visible
  output, session re-invoked with no human input). Round 003's harness-initiated kill is the
  real-environment teardown evidence.
- `prune` deliberately does not consult launch state (eight-day threshold vs 3–25 minute runs);
  `--stdin` retains a TOCTOU window because bash cannot express `O_NOFOLLOW`. Both non-blocking.
- The bundle ratchet cannot be held by this unit while its sealed rounds cannot compact: the
  assembler requires a `## Carried decisions` section inside the round file, and the termination
  rules say the round file carries nothing but findings, size and consensus. Defect recorded
  against the task that owns the `codex-review` skill; sealed rounds are not rewritten to dodge it.

Consensus: disagreed
