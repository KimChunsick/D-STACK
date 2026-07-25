# Codex adversarial review — Round 006

## Review scope

Re-review. Verifies Round 005's companion validation and the older-round retrieval mechanism.
Bundle 78,103 bytes; rounds 001-003 compacted, 004-005 full.

## GPT findings

[severity:medium][technical correctness] Companion state is still derived from untrusted round Markdown and is not bound to its source round, allowing incorrect history to replace the authoritative round.
Sites: primary: `SKILL.md` Step 3 sealing command; confirmed: `assemble-review.sh` `carried_ok`, Round 004's carried decision forbidding round-text derivation; suspected: none.
Evidence: `awk` starts at the first prefix-matching `## Carried decisions`, while `carried_ok` checks only the opening heading and final consensus, not source identity or exact payload.
Verification: A fenced decoy heading makes the writer include stale/intervening content and still pass validation; substituting `carried-001.md` for `carried-002.md` also passes and silently omits Round 002's state.
Suggested direction: Generate both artifacts from one explicit carried-state input and validate the companion's round identity before permitting compaction.

[severity:medium][UI & UX / DX] The promised older-round retrieval becomes impossible when its forced contiguous suffix exceeds the total bundle budget.
Sites: primary: `assemble-review.sh` `FULL_ROUNDS` emission policy; confirmed: `SKILL.md` older-round procedure, fixed `MAX_BUNDLE`, prohibition on hand-built bundles; suspected: none.
Evidence: Retrieving one old round requires emitting every newer round in full, while neither narrowing the allowlist nor splitting task changes that history suffix.
Verification: Eight valid 65,536-byte rounds already equal the 524,288-byte budget; task content and framing guarantee failure, so `REVIEW_FULL_ROUNDS=8` emits no bundle.
Suggested direction: Support validated selection of requested old rounds while retaining the default two-round full suffix.

[severity:medium][technical correctness] `REVIEW_FULL_ROUNDS` is unvalidated, so zero, negative, or nonnumeric values can compact the newest rounds while the assembler succeeds.
Sites: primary: `assemble-review.sh` `FULL_ROUNDS` assignment and comparison; confirmed: static two-full-round prompt and `SKILL.md` invocation contract; suspected: none.
Evidence: The value is used directly by `[ ... -lt "$FULL_ROUNDS" ]` without enforcing an integer minimum of two.
Verification: `REVIEW_FULL_ROUNDS=0` makes every comparison false; a nonnumeric value reports "integer expression expected" inside the `if` but execution continues and likewise compacts every round.

Omitted-detail: 0 low

GPT verdict: reject — three concrete medium blockers can silently misrepresent review history or prevent required historical evidence from reaching the reviewer.

## Maintainer response

All three accepted. Two of them share a root cause I introduced in Round 005, and fixing that
root cause removes both.

**M1 — the sealing step was still deriving from round text. Agreed, and it violated Round
004's own carried decision.** I moved derivation out of the assembler and then left it in the
writer, which is the same ambiguity one step earlier. Step 3 now says to author the companion
from the text composed for the round rather than scraping it back out, and shows the shape
instead of an extraction command. On the reading side, `carried_ok` gained identity binding:
the companion's first line must be `## Carried decisions — Round <NNN>` matching the round it
stands for. Verification: copying `carried-001.md` into `carried-002.md`'s slot is now refused
with `carried-002.md is not a complete carried-state companion` and round 002 is emitted whole.

**M2 and M3 — one wrong lever, replaced rather than patched.** Raising a *count* was the wrong
mechanism: it drags in every newer round, which is what makes the promise unkeepable on long
history (the reviewer's arithmetic is right), and a count is exactly the kind of value that
can be set to 0 or a string and quietly compact rounds the contract guarantees in full.
`REVIEW_FULL_ROUNDS` is gone. `REVIEW_FULL_ROUND_IDS` names the rounds asked for, sends only
those, and cannot shrink the two-most-recent floor because that floor is once again a constant.
Malformed or out-of-range values are fatal, because a silently dropped request would hand the
reviewer a compacted round with no sign its ask was ignored.
Verification: `REVIEW_FULL_ROUND_IDS=1` sends round 001 full while 002-003 stay compacted;
`1,3` sends both; `0`, `9`, `abc`, and `1 x` each exit 1 with a message naming the bad value
and the valid range.

## Carried decisions

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
