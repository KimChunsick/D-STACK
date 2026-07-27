## Carried decisions — Round 006
- **P6's registration loop READS the depth, it does not write it.** The fence iterated a literal
  `<Mn>/<NN-task>/task.md`, so it registered task-depth documents even for a Goal that declared
  milestone granularity — the exact misregistration the granularity table above it warns about,
  baked into the recipe that implements it. It now calls `check-registration.sh --depth`, which
  returns 3 or 2 from the same GOAL.md parse the check itself uses, so the two cannot disagree.
- **The checker parses GOAL.md exactly as `check-parallel.sh` does.** Fences tracked globally from
  line one, task rows at column zero with the `-` marker, a repeated section heading keeping the
  section open. Two parsers that disagree about what a declaration is are worse than one that is
  wrong, because the disagreement is invisible — measured on a fixture with a fenced example above
  the section, the old parser read the FAKE `per milestone` granularity and the FAKE task rows and
  none of the real ones.
- **Cap closure has ONE authority, and it is `codex-review` §4.** P9 said to escalate to the user
  when the budget is reached with `blockers` still open; `blockers` means high AND medium, so it
  demanded a human for exactly the case §4 and `autonomy.stops` both close without one. Two
  transitions out of one state is how an unattended run stalls on a medium nobody needs to see.
- **`reclaim` has no autonomous carve-out, because the case it named does not exist.** The stop
  entry allowed it for a "provably orphaned" handoff whose owner is this session — but `reg` returns
  0 for a document this session already owns, so that state is never reached, and every other case
  is unprovable without a liveness signal. The other half, "or the user says so", is not autonomy.
- **The `RUNDIR` ordering defect is real and was fixed where it lives**, in `codex-review`'s Step 2
  fence rather than here; this file's `waits.external` describes the contract, not the recipe.

Consensus: disagreed
