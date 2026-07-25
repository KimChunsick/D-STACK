# Codex adversarial review — Round 007

## Review scope

Re-review. Verifies Round 006's identity-bound companion and the round-selection retrieval
mechanism. Bundle 83,328 bytes; rounds 001-004 compacted, 005-006 full.

## GPT findings

[severity:medium][technical correctness] Companion validation still cannot detect missing, altered, or contradictory carried state, so it can silently replace the authoritative round.
Sites: primary: `assemble-review.sh` `carried_ok`; confirmed: `SKILL.md` Step 3 sealing workflow, task doc section 5 and E2E check 5; suspected: none.
Evidence: `carried_ok` checks snapshot gates, the round-number heading, and any sealed consensus ending, but never binds the decisions or consensus to the sealed round.
Verification: A two-line companion with the correct heading and `Consensus: agreed` passes even when its round contains decisions and ends `Consensus: disagreed`; once old, only the false companion is emitted.
Suggested direction: Generate both artifacts from one canonical carried-state input and require a reader-verifiable binding to the round's payload and consensus before compaction.

[severity:medium][UI & UX / DX] Leading-zero round IDs pass validation but are silently ignored, leaving an explicitly requested round compacted.
Sites: primary: `assemble-review.sh` `EXTRA_FULL` validation and emission test; confirmed: `SKILL.md` older-round retrieval procedure, task doc E2E check 7; suspected: none.
Evidence: IDs are range-checked numerically but later matched textually against canonical decimal `idx` values.
Verification: `REVIEW_FULL_ROUND_IDS=001` passes the digit and range checks for round 1, but `" 001 "` does not match `grep " 1 "`, so round 1 is not emitted in full.

[severity:low][UI & UX / DX] The generated bundle description and review prompt falsely state that all older rounds are compacted despite the documented full-round fallback.
Sites: primary: `SKILL.md` Step 2 prompt; confirmed: `assemble-review.sh` history preamble and `emit_round_compact`; suspected: none.
Evidence: Both descriptions categorically call older rounds compacted, while missing or invalid companions cause full snapshots.
Verification: Any legacy round without a companion produces a bundle whose summary says "older rounds are compacted" immediately before a full-snapshot entry.

[severity:low][software structure] A stale assembler comment instructs maintainers to reason about locating `## Carried decisions` inside the round, contradicting the companion-only invariant.
Sites: primary: `assemble-review.sh` comment above `emit_round_compact`; confirmed: its implementation, `SKILL.md` companion design, task doc section 5; suspected: none.
Evidence: The comment says fallback occurs when the round lacks a carried-decisions section, although the implementation never examines round Markdown.
Verification: `emit_round_compact` derives `carried-<NNN>.md`, validates that file, and otherwise emits the full round regardless of headings inside the round.

[severity:low][UI & UX / DX] The verification ledger has duplicate and out-of-order numbering and reports the wrong total.
Sites: primary: task doc `E2E verification`; confirmed: task doc `Gate status`; suspected: none.
Evidence: The entries are numbered 1, 2, 3, 4, 5, 7, 6, 7, 8, while the gate claims eight checks.
Verification: There are nine listed verification items and two distinct items numbered 7.

Omitted-detail: 0 low

GPT verdict: reject — companion integrity and requested-round retrieval still have concrete medium-severity failure paths that can suppress required review history.

## Maintainer response

All five accepted, none rebutted.

**M1 — companion not bound to its round's verdict. Agreed.** Identity binding in Round 006
covered *which* round a companion claims to be, not whether it agrees with it. `carried_ok`
now also compares the companion's last nonblank line to the round's own last nonblank line,
which is the one part of a round the assembler can read without ambiguity (`sealed_round_ok`
already guarantees it is the sealed consensus). The reviewer's counterexample — an `agreed`
companion standing in for a `disagreed` round — is exactly what this rejects. Binding the
decisions *payload* would require deriving that payload from the round, which is the thing
five rounds have now shown cannot be done safely; the honest scope is the two unambiguous ends
plus authoring both artifacts from one text, which Step 3 already requires.
Verification: rewriting `carried-001.md`'s final line to `Consensus: agreed` while its round
ends `disagreed` yields `carried-001.md is not a complete carried-state companion` and round
001 is emitted whole; restoring the file compacts again.

**M2 — leading zeros validated in one form, matched in another. Agreed.** IDs are now
canonicalised with `$((10#$r))` before both the range check and the emission test, so `001`
and `1` name the same round. Verification: `REVIEW_FULL_ROUND_IDS=001` and `=1` both emit
round 001 in full.

**L1 — the preamble and prompt overstated. Agreed.** Both now say older rounds are compacted
*where their companion allows* and sent whole otherwise, and that each entry states which.
Verification: the legacy task's bundle preamble carries the qualified wording directly above
its full-snapshot entries.

**L2 — stale comment. Agreed**, replaced; nothing in the assembler now describes locating a
section inside a round. The header comment and `SKILL.md` also said "three review rounds"
where it is now five.

**L3 — verification ledger numbering. Agreed.** Renumbered 1-9, the gate line corrected from
"8 checks" to 9, and item 3's stale "−56%" measurement replaced: that figure described the
derived design on the legacy task, which no longer compacts at all now that compaction
requires companions.

## Carried decisions

- Compaction reads a **companion file, never the round's Markdown body** — and the companion is
  **authored, not extracted**. Six rounds killed six successive derived rules. Do not
  reintroduce derivation at either the writing or the reading end.
- A companion is trusted only when: it passes the snapshot gates, its first line is
  `## Carried decisions — Round <NNN>` for the round it stands for, its last nonblank line is a
  sealed consensus, and that line equals the round's own last nonblank line. Anything else
  sends the round whole. Write it through a same-directory temp file and `mv`.
- Binding the decisions payload itself is out of scope by decision, not oversight: it would
  require the derivation this design exists to avoid. Authoring both artifacts from one text is
  the control.
- `REVIEW_FULL_ROUND_IDS` names rounds, never a count, canonicalises leading zeros, cannot
  shrink the two-most-recent floor, and treats malformed or out-of-range values as fatal.
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
- Accepted, unchanged: no public documentation of a `codex exec` stdin byte cap or its
  overflow semantics was found.

Consensus: disagreed
