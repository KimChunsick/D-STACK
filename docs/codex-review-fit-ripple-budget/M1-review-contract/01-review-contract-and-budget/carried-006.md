## Carried decisions — Round 006

- Compaction reads a **companion file, never the round's Markdown** — and the companion is
  **authored, not extracted**. Five rounds killed five successive derived rules, the last of
  them at the writing step rather than the reading step. Do not reintroduce derivation at
  either end.
- A companion is trusted only when its first line is `## Carried decisions — Round <NNN>` for
  the round it stands for and its last nonblank line is a sealed consensus. Write it through a
  same-directory temp file and `mv`.
- `REVIEW_FULL_ROUND_IDS` names rounds, never a count, and cannot shrink the two-most-recent
  floor. Malformed or out-of-range values are fatal, not ignored.
- Sealing a round means writing two files: `codex-review-<NNN>.md` and `carried-<NNN>.md`.
  Restate the complete live decision set in each round rather than only the delta.
- The companion name must stay outside the `codex-review*.md` namespace the assembler
  validates for contiguity.
- Every check fails toward emitting the whole round: sending too much is a cost, dropping real
  carried state is a defect.
- The budget bounds elaboration only. Every low actually found is reported, in full or as a
  one-line title.
- `MAX_BUNDLE` = 524288, derived from the smallest documented window (`context_window` 272000
  for `gpt-5.6-sol`), and described as a policy limit rather than a measured ceiling.
- Changing the contract means changing every surface in the same edit: `codex/AGENTS.md`,
  `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`,
  `claude/hooks/fullcycle-inject.sh`, and the task doc.
- Accepted, unchanged: no public documentation of a `codex exec` stdin byte cap or its
  overflow semantics was found.

Consensus: disagreed
