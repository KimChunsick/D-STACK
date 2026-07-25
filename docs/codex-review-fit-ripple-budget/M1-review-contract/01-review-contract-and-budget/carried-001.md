## Carried decisions — Round 001

- Compaction requires exactly one **unfenced** `## Carried decisions` heading; anything else
  emits the full round. Fail-open on content is deliberate — a compacted fragment that drops
  real carried state is the one loss this mechanism must never cause.
- The output budget bounds elaboration only. Every low actually found is reported, in full or
  as a one-line title. Any future budget change must preserve that.
- `MAX_BUNDLE` is derived from the smallest documented context window (272000 for
  `gpt-5.6-sol`), not chosen for caution. Changing it requires citing a window.
- Accepted, unchanged from the task doc: no public documentation of a `codex exec` stdin byte
  cap or its overflow semantics was found; the context figures come from local CLI metadata
  and the public model spec, and the budget is a runaway detector rather than a proven ceiling.

Consensus: disagreed
