## Carried decisions — Round 008

- Compaction reads a **companion file, never the round's Markdown body**, and the companion is
  **authored, not extracted**. Six rounds killed six successive derived rules. Do not
  reintroduce derivation at either the writing or the reading end.
- A companion is trusted only when: it passes the snapshot gates, its first line is
  `## Carried decisions — Round <NNN>` for the round it stands for, its last nonblank line is a
  sealed consensus, and that line equals the round's own last nonblank line. Anything else
  sends the round whole. Write it through a same-directory temp file and `mv`.
- Binding the decisions payload itself is out of scope by decision, not oversight: it would
  require the derivation this design exists to avoid. Authoring both artifacts from one text is
  the control.
- `REVIEW_FULL_ROUND_IDS` names rounds, never a count; rejects empty fields and ids wider than
  six digits before any arithmetic; canonicalises leading zeros; cannot shrink the
  two-most-recent floor; and treats malformed or out-of-range values as fatal.
- Sealing a round means writing two files: `codex-review-<NNN>.md` and `carried-<NNN>.md`.
  Restate the complete live decision set in each round rather than only the delta.
- The companion name must stay outside the `codex-review*.md` namespace the assembler
  validates for contiguity.
- Every check fails toward emitting the whole round: sending too much is a cost, dropping real
  carried state is a defect. Descriptions of the bundle must say so rather than claiming all
  older rounds are compacted.
- The budget bounds elaboration only. Every low actually found is reported, in full or as a
  one-line title.
- `MAX_BUNDLE` = 524288, derived from the smallest documented window (`context_window` 272000
  for `gpt-5.6-sol`), and described as a policy limit rather than a measured ceiling.
- Changing the contract means changing every surface in the same edit: `codex/AGENTS.md`,
  `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`,
  `claude/hooks/fullcycle-inject.sh`, and the task doc.
- New defect class for this repo's checklist: deriving structure from a document that can quote
  itself. Prefer a separate artifact over a smarter parser.
- Accepted, unchanged: no public documentation of a `codex exec` stdin byte cap or its overflow
  semantics was found; the context figures come from local CLI metadata and the public model
  spec.

Consensus: agreed
