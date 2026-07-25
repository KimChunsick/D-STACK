## Carried decisions — Round 004

- Compaction reads a **companion file, never the round's Markdown**. Four rounds killed four
  successive derived rules (fenced heading, HTML-commented heading, `~~~`-inside-``` plus a
  line that closes and opens a comment, and a ``` line inside an open ```text fence). Do not
  reintroduce derivation in any form, including "just one more delimiter check".
- Sealing a round means writing two files: `codex-review-<NNN>.md` and `carried-<NNN>.md`.
  A missing companion costs bundle size only — the round is sent whole. A *wrong* companion
  misleads every later round, so restate the complete live decision set in each round rather
  than only the delta.
- The companion name must stay outside the `codex-review*.md` namespace the assembler
  validates for contiguity.
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
