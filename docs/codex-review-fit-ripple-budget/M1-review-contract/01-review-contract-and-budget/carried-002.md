## Carried decisions — Round 002

- Compaction is **structural, not textual**: `## Carried decisions` must be the round's final
  real section, with fenced blocks and HTML comments excluded when locating it. Do not
  reintroduce a text-match rule; two review rounds defeated two successive textual versions.
- The check fails toward emitting the whole round. Any future change must keep that direction:
  sending too much is a cost, dropping real carried state is a defect.
- The budget bounds elaboration only. Every low actually found is reported, in full or as a
  one-line title.
- `MAX_BUNDLE` = 524288, derived from the smallest documented window (`context_window` 272000
  for `gpt-5.6-sol`). Changing it requires citing a window.
- Changing the contract means changing every surface in the same edit: `codex/AGENTS.md`,
  `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`,
  `claude/hooks/fullcycle-inject.sh`, and the task doc. Round 002 exists because that was
  missed once.
- Accepted, unchanged: no public documentation of a `codex exec` stdin byte cap or its
  overflow semantics was found; the context figures come from local CLI metadata and the
  public model spec, and the budget is a runaway detector rather than a proven ceiling.

Consensus: disagreed
