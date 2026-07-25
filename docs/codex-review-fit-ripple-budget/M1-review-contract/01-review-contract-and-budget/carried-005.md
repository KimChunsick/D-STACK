## Carried decisions — Round 005

- Compaction reads a **companion file, never the round's Markdown**. Four earlier rounds killed
  four successive derived rules. Do not reintroduce derivation in any form.
- A companion is trusted only when it opens with the carried-decisions heading and closes with
  a sealed consensus line; anything else sends the round whole. Seal writes go through a
  same-directory temp file and `mv`.
- Sealing a round means writing two files: `codex-review-<NNN>.md` and `carried-<NNN>.md`.
  Restate the complete live decision set in each round rather than only the delta.
- `REVIEW_FULL_ROUNDS` (default 2) is the mechanism that honours a reviewer's request for an
  older round in full. The prompt promises it, so it must keep working.
- The companion name must stay outside the `codex-review*.md` namespace the assembler
  validates for contiguity.
- Every check fails toward emitting the whole round: sending too much is a cost, dropping real
  carried state is a defect.
- The budget bounds elaboration only. Every low actually found is reported, in full or as a
  one-line title.
- `MAX_BUNDLE` = 524288, derived from the smallest documented window (`context_window` 272000
  for `gpt-5.6-sol`), and described as a policy limit rather than a measured ceiling. Changing
  it requires citing a window.
- Changing the contract means changing every surface in the same edit: `codex/AGENTS.md`,
  `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`,
  `claude/hooks/fullcycle-inject.sh`, and the task doc.
- Accepted, unchanged: no public documentation of a `codex exec` stdin byte cap or its
  overflow semantics was found; the context figures come from local CLI metadata and the
  public model spec.

Consensus: disagreed
