## Carried decisions — Round 002
- **`reclaim` is a HUMAN STOP, not an internal recovery.** `cmd_reclaim` has no liveness signal, so
  it cannot distinguish a crashed session from a working one — it replaces the owner outright, and
  the Stop hook then SKIPS records owned by another session. An autonomous reclaim therefore
  silently un-gates a live session's work while both keep running. Automatic only when the handoff
  is provably orphaned; otherwise ask.
- **The P6 fence is a RUNNABLE BLOCK, and the other subcommands are a reference list.** Putting
  `set -e` above a list that ends in `unreg` made the success path deregister the document it had
  just registered. Split, and the runnable block now ASSERTS its promised end state with `status`
  rather than trusting `reg`'s exit code. Verified: normal path rc=0 "P6 registration confirmed";
  a missing record rc=1 "P6 BLOCKED".
- **A `stops` entry beats an `internal-recoveries` entry.** An unavailable pinned review model is
  both a nonzero run (retry) and a missing required dependency (stop); without precedence the
  orchestrator had no unique transition and would retry a missing model forever. Stops win, and
  automatic retry is restricted to a DIAGNOSED transient failure.
- The launch invariant and the signal-handler form must be identical in `full-cycle` and in the
  skills it invokes. Fixing the contract and leaving the invoked recipe contradicting it hands the
  orchestrator both rules.

Consensus: disagreed
