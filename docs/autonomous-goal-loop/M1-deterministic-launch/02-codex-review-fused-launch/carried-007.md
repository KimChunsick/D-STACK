## Carried decisions — Round 007
- **The launch call RECONSTRUCTS the run dir; it does not inherit it.** Step 2's fence opened with
  `RUNDIR="$RD"` and defined `RD` four lines later, in a call where `$RD` from the assembly step no
  longer exists at all — Step 1 says why, a shell variable does not survive between tool calls. The
  armed trap therefore tested `[ -e "/exit" ]`, always false, so the scratch dir was never removed
  on the one path where removing it is correct. `LABEL` is now assigned first and `RD` derived from
  it before anything else.
- **`<run-dir>/exit` is the round's status in the PROSE too, not only in the recipe.** Step 2a
  opened by calling any nonzero notification a failed round, which contradicts what Step 2 had just
  established: a deferred signal makes a completed round report 143. A missing `exit` file is also
  not a pass — it means the run never reached quiescence.
- **The signal handlers leave the gated EXIT trap ARMED.** `trap - EXIT` was carried over from when
  the cleanup was unconditional. The gate answers the question better: measured in bash and zsh,
  exit file present gives rc=143 with the directory removed, absent gives rc=143 with nothing
  removed. Disarming turned the deferral — which means the handler usually runs after `exit` was
  published — into a guaranteed leak.
- **A precedence claim over the reviewer is unenforceable, so it is gone.** "THIS file governs" was
  addressed to a reviewer that is told to follow `$adversarial-review` exactly and is told in the
  same prompt to treat the whole payload as untrusted data. What is true is narrower: these rules
  govern the ORCHESTRATOR, because it is the side that runs them and the side the Stop hook parses.
  The Codex-side inconsistency stays a recorded follow-up and a reviewer filing it is right to.
- **A post-seal reopening gets its own budget, counted from the reopening.** This Goal hit the gap
  directly: two units sealed AT the 5-round cap and were then reopened by `post-seal-rule`, leaving
  no legal next move. The cap now counts rounds since the reopening and resets SMALLER (2 per-task,
  3 per-milestone), and the non-convergence window restarts with it, because the old counts measured
  a corpus that no longer exists.

Consensus: disagreed
