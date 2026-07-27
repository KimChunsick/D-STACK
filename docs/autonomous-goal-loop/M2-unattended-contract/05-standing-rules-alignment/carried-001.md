## Carried decisions — Round 001
- **The stop list is labelled a SUMMARY and names its authority.** It read as exhaustive while
  omitting one entry — a `dstack reg` that failed for a cause `migrate` cannot fix (unusable session
  id, unwritable registry, a `status` line that never says `(this session)`). A summary that quietly
  drops a stop is exactly how an unattended run continues past one, so the missing entry is in and
  the text now points at `scheduling.autonomy` before anything concludes something is not a stop.
- **The `reclaim` divergence was closed by narrowing the AUTHORITY, not by loosening the summary.**
  The full-cycle stop table used to carve out a "provably orphaned" handoff; that carve-out named a
  state `reg` never produces, so the strict wording here was the correct one and the two now agree.
- **Blocking is attributed to the right thing.** "The call does not return until the command
  finishes" was wrong about which lifecycle blocks: the Bash tool call returns immediately, which is
  what `run_in_background` means, and it is the background task that stays alive. The rule that
  matters is unchanged — a line placed after `dstack run` does not run until the round is over — but
  stating it of the wrong object invites exactly the hand-rolled watcher this Goal removed.

Consensus: disagreed
