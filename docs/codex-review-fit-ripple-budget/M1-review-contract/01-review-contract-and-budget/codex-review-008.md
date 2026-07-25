# Codex adversarial review — Round 008

## Review scope

Re-review, run with the wind-down rule stated in the prompt. Verifies Round 007's
consensus-binding and ID canonicalisation. Bundle 95,562 bytes; rounds 001-005 compacted to
their companions, 006-007 full.

## GPT findings

[severity:low][technical correctness] Requested-round parsing accepts malformed separators and allows integer overflow to alias an invalid ID to an existing round instead of failing.
Sites: primary: `assemble-review.sh` `EXTRA_FULL` parsing; confirmed: `SKILL.md` older-round retrieval contract, task-doc E2E check 6; suspected: none.
Evidence: Word splitting discards empty comma fields, while `$((10#$r))` performs fixed-width arithmetic before the range check.
Verification: `1,,3,` parses as `1 3`; on the tested Bash, `18446744073709551617` canonicalizes to `1`, silently selecting round 1.

[severity:low][software structure] The documentation remains duplicated and internally inconsistent, needlessly enlarging every bundle and weakening the audit narrative.
Sites: primary: task doc after its first `Files changed` section; confirmed: repeated task-doc `What was done`/`Files changed` sections, task-doc "Three review rounds" claim, `SKILL.md` Step 3 "four review rounds" claim; suspected: none.
Evidence: The task snapshot restarts through a malformed `## Gate status` fragment and repeats sections 1-7, while nearby surfaces describe the abandoned derivation attempts as three, four, or five rounds.
Verification: Direct comparison of the supplied task snapshot and scoped `SKILL.md` diff confirms both the duplicated body and conflicting counts.

Omitted-detail: 0 low

GPT verdict: approve-with-fixes — Round 007's blockers are resolved, with only non-blocking parser hardening and documentation cleanup remaining.

## Maintainer response

Consensus reached. No high and no medium findings remain, so this round closes under the
closure rule. Both lows were nonetheless fixed and verified here rather than deferred, because
each is a few lines and one of them was a real defect in the artifact itself.

**L1 — parser hardening. Agreed and fixed.** Empty comma fields are now a fatal error rather
than being silently dropped by word splitting, and every id is width-bounded to six digits
*before* `$((10#…))` runs, so a value that would wrap into the valid range is rejected instead
of aliasing to a real round. Verification: `1,,3`, `,1`, and `1,` each exit 1 with
`has an empty field`; `18446744073709551617` and `1234567` each exit 1 with `value out of
range`; `1,3` still succeeds.

**L2 — duplicated task doc. Agreed, and this one mattered more than its severity.** The task
document had a genuinely corrupted body: a scripted edit had spliced in a second copy of the
Method note fragment, `What was done`, and `Files changed`, so the file carried 108 duplicated
lines and a broken `## Gate status` fragment as a heading. Removed; the document now runs
Intent → Deployment context → Design consult → Method note → What was done → Files changed →
E2E verification → Gate status exactly once, 191 lines. Two `Files changed` entries that still
described the superseded derived-compaction design were corrected in the same pass. The
"three / four / five review rounds" counts across `SKILL.md`, `assemble-review.sh`, and the
task doc are now uniformly six, matching the actual number of derivation attempts these rounds
killed.

**On the loop itself.** Eight rounds is long, and the reason is worth recording: rounds 001-007
each defeated a successive attempt to *derive* carried state from a document that quotes other
documents. The escape was not a better parser but a different artifact. The pre-review
defect-class checklist for this repo should carry that class — "deriving structure from a
document that can quote itself" — so the next task reaches for a companion artifact instead of
spending seven rounds rediscovering that parsing loses.

## Carried decisions

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
