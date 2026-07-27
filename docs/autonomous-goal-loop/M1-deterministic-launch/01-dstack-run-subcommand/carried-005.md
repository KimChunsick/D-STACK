## Carried decisions — Round 005

- Reentrancy protection and finalisation are SEPARATE facts. `run_cleanup` disarms its own traps on
  entry — that is what prevents re-entry — and `run_done` is set only once the run has been
  published or publication explicitly refused. Round 005 was caused by conflating them: setting
  `run_done` before settlement let a signal during that 15-second window skip teardown entirely.
- The `die`-loses-locals mechanism (round 004 F010, round 005 F014) is DISPROVED on the deployed
  /bin/bash 3.2.57 by two instrumented runs of the real handler, which printed populated `d` and
  `label` on both the pre-fork and post-fork paths and completed claim release and publication.
  Do not add machinery premised on it. The defensive defaults introduced in round 004 stay, since
  they cost nothing and caught a real `/.launch` bug.
- Publication is gated on confirmed group quiescence at both call sites; a group surviving SIGKILL
  yields no terminal record, which is what keeps `rm-run` guarding the capture.
- `$child` is both pid and pgid; every liveness question about launched work asks the GROUP.
- One cleanup owner on EXIT + `INT TERM HUP QUIT PIPE ALRM USR1 USR2`, armed one statement after
  the claim. Failures before the fork release the claim; every path after it keeps it. `$!` is the
  tiebreaker for the window between `&` and `child=$!`.
- pgid recycling between probe and signal is an accepted residual, recorded in the code.
- Evidence must exercise a child that does NOT honour the signal under test, must run under the
  interpreter the shebang names, and must instrument the real code rather than a synthetic
  look-alike. Rounds 003, 004 and 005 each caught a version of the opposite mistake.
- Completion re-invocation on background-command exit is a MEASURED local behaviour of client
  2.1.220, not a documented guarantee — the change's load-bearing external dependency.
- `prune` does not consult launch state; `--stdin` retains a TOCTOU window. Both non-blocking.
- CLOSED AT THE ROUND CAP (5 for a per-task unit). Nothing concrete left open. Residual for the
  final report: F013's fix is verified by direct run but was not itself adversarially reviewed, and
  rounds 001–005 each found something, so the loop stopped on its cap rather than on exhaustion.

Consensus: resolved
