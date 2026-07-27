## Carried decisions — Round 004
- **A quoted assignment is not a boundary; a QUOTED HEREDOC is.** `GOAL='<goal>'` with `<goal>`
  replaced by `x'$(printf PWNED)'` closes the literal, runs the command, and hands the validator the
  valid slug `xPWNED`. `<<'SLUG'` expands nothing at all, so the same input arrives as a literal and
  is refused. Measured, both shells:
  `assignment → ACCEPTED [xPWNED]` / `heredoc → REFUSED [x'$(printf PWNED)']`, benign slug still
  accepted.
- **A signal handler that only cleans up lets the shell CONTINUE.** Measured, both shells: the
  cleanup-only form ran the handler twice and returned 0 (`CLEANSURVIVEDCLEAN`). The handler must
  disarm EXIT, clean once, and exit with the signal's status — that form returns 143 and cleans
  once, and the normal path still cleans exactly once.
- The wrapper exiting nonzero is not the same as cancelling the round. If a signal reached only the
  wrapper and not the process group, `dstack run` can still be alive; the retry fence exists for
  exactly that and must be run before relaunching a capture with no terminal record.
- A residual paragraph states what is true of the tool. Attaching a disposition to it — "accepted",
  "belongs to another unit" — is disposition language inside the reviewed payload, and this round
  showed the cost: it invited the reader to stop where I had stopped, and the cancellation defect
  was past that point.

Consensus: disagreed
