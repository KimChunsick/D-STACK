## Carried decisions — Round 003

- Compaction is decided by **three total checks — last occurrence, final section, balanced
  delimiters before it** — not by tracking Markdown block state. Three successive stateful
  versions lost to a construct their author had not thought of. Do not reintroduce a parser:
  if a new construct appears, add a balance count for it.
- The check fails toward emitting the whole round. Any future change must keep that direction:
  sending too much is a cost, dropping real carried state is a defect.
- The budget bounds elaboration only. Every low actually found is reported, in full or as a
  one-line title.
- `MAX_BUNDLE` = 524288, derived from the smallest documented window (`context_window` 272000
  for `gpt-5.6-sol`). Changing it requires citing a window.
- Changing the contract means changing every surface in the same edit: `codex/AGENTS.md`,
  `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`,
  `claude/hooks/fullcycle-inject.sh`, and the task doc.
- Accepted, unchanged: no public documentation of a `codex exec` stdin byte cap or its
  overflow semantics was found; the context figures come from local CLI metadata and the
  public model spec, and the budget is a runaway detector rather than a proven ceiling.

Consensus: disagreed
