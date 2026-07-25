# Codex adversarial review — Round 005

## Review scope

Re-review. Verifies Round 004's companion-file redesign. Bundle 73,261 bytes; rounds 001-002
compacted to their companions, 003-004 full.

## GPT findings

[severity:medium][technical correctness] A partially written or malformed companion can replace the authoritative round, silently removing carried decisions from later reviews.
Sites: primary: `SKILL.md` Step 3 sealing command; confirmed: `assemble-review.sh` `emit_round_compact`, task doc section 5 and E2E check 5; suspected: none.
Evidence: The direct `>` write is non-atomic, while the assembler accepts any existing companion passing generic snapshot gates without requiring the carried-state heading, sealed consensus, or completeness.
Verification: Interrupting `awk` after it writes a small textual prefix leaves a valid-sized companion; once that round becomes older than two rounds, the assembler emits the prefix and omits the complete round.
Suggested direction: Write to a same-directory temporary file, validate the complete companion structure, rename atomically, and emit the full round whenever companion validation fails.

[severity:medium][UI & UX / DX] The reviewer can request an older full round, but the enforced assembly workflow provides no mechanism to supply it.
Sites: primary: `SKILL.md` review prompt; confirmed: `assemble-review.sh` fixed age-based emission policy, compacted-round header, Step 1 prohibition on hand-built bundles; suspected: none.
Evidence: The prompt directs the reviewer to name and request an older round, while emission depends solely on age and companion existence and exposes no force-full option.
Verification: With three or more prior rounds and an existing companion, every subsequent standard invocation compacts the requested old round; adding it to the allowlist yields the scoped-diff path rather than a full snapshot.
Suggested direction: Add a documented, validated force-full option for selected round numbers that retains snapshot gates and the total-bundle budget.

[severity:low][UI & UX / DX] The over-budget diagnostic states an unverified failure cause as fact.
Sites: primary: `assemble-review.sh` fatal-message text; confirmed: task doc section 6, research `Unverified` section; suspected: none.
Evidence: The message says an oversized bundle "is what makes codex exec die," while the task and research acknowledge that the exact stdin limit and overflow behavior are unknown.
Verification: The 512KB threshold is explicitly described as a policy-level runaway detector, so it cannot establish the asserted CLI failure causality.

Omitted-detail: 0 low

GPT verdict: reject — unresolved medium blockers can silently truncate carried review state and prevent requested historical evidence from being supplied.

## Maintainer response

All three accepted, none rebutted. The findings have moved off the compaction mechanism
itself and onto its edges, which is the first round where that is true.

**M1 — truncated companion. Agreed; both halves of the suggested direction adopted.** Sealing
now writes through a same-directory temp file and `mv`, so an interrupted write leaves no
plausible prefix in place. Independently, the assembler no longer trusts a companion on the
generic snapshot gates alone: `carried_ok` requires the first line to be the carried-decisions
heading and the last nonblank line to match the same sealed-consensus regex used for rounds.
Two defences rather than one, because the writer and the reader can be interrupted separately.
Verification: truncating `carried-001.md` to its first 120 bytes yields
`full snapshot; carried-001.md is not a complete carried-state companion`; restoring it
compacts again; a companion with a correct heading but no sealed consensus line is refused the
same way.

**M2 — a promise with no mechanism. Agreed, and it was mine to keep.** The prompt tells the
reviewer to name an older round and ask for it; nothing could supply it. `FULL_ROUNDS` is now
`REVIEW_FULL_ROUNDS` with a default of 2, so honouring the request is one environment variable
on the next assembly, and Step 1 documents it — including that adding the round to the
allowlist is *not* equivalent, because allowlisted files take the scoped-diff path rather than
the snapshot path. The prompt now states plainly that the next round will carry it in full.
A per-round selector was considered and rejected: including everything newer keeps history
contiguous, which is what a reviewer verifying a chain of fixes actually needs.
Verification: default emission compacts 001-002 and sends 003-004 full;
`REVIEW_FULL_ROUNDS=4` sends all four in full.

**L1 — unverified causality stated as fact. Agreed.** The message claimed the bundle size is
what kills `codex exec`, which the research explicitly could not verify. Reworded to say it is
a policy limit, not a measured CLI ceiling, and that the exact cap and overflow behaviour are
undocumented.

## Carried decisions

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
