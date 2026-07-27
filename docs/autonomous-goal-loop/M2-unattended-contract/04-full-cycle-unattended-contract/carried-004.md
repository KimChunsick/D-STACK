## Carried decisions — Round 004
- **A hand-listed expected set is its own proof and proves nothing.** `DOCS=(GOAL u1 u1)` printed
  "3 documents" with a required unit absent, because same-session `reg` is idempotent and every
  element was checked independently. The set is now DERIVED — `find` at the depth the granularity
  fixes, since P6 scaffolds exactly one `task.md` per review unit and the filesystem cannot omit or
  duplicate. Verified across six scenarios.
- **`find` proves what was SCAFFOLDED, not what was DECOMPOSED**, so the count is cross-checked
  against GOAL.md's task rows — the same section the parallelism checker parses. Without it a unit
  whose folder was never created is invisible and the fence confirms the smaller set. Verified in
  both directions: 2 scaffolded vs 3 rows BLOCKS, 4 scaffolded vs 3 rows BLOCKS.
- **P6 names no failure outcomes of its own; `scheduling.autonomy` decides.** Prose here that also
  routed failures is what kept `reclaim` alive as a "recovery" for foreign ownership after the stop
  table already forbade it. Third repair of the same defect, and the fix is to remove the second
  authority rather than to reword it.
- **Scratch cleanup is CONDITIONAL on `<run-dir>/exit` existing.** `dstack run` publishes that file
  only after confirming its child's process group is gone, so it is the quiescence proof. An
  unconditional EXIT trap deletes a live `codex exec`'s cwd whenever `dstack` died to something it
  cannot trap — reproduced with `SIGPROF`, which kills the supervisor at rc=155 while the child
  lives.
- **The precedence table overrides any per-skill retry text.** `codex-review` Step 2a says to re-run
  every nonzero result; a missing dependency or a rejected model pin is a stop, and retrying it
  burns rounds and changes nothing.

Consensus: disagreed
