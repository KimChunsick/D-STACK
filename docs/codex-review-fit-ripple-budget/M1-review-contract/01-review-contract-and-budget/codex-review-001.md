# Codex adversarial review — Round 001

## Review scope

Adversarial review. First round under the new contract, and the first run of the new
assembler — the round reviews the change that produced it. Bundle 53,980 bytes across 12
emitted items (task doc + 5 changed files + research artifact); no prior rounds existed.

## GPT findings

[severity:medium][technical correctness] Compaction trusts any matching line as canonical carried state, allowing a sealed round's substantive history to be silently discarded.
Sites: primary: `assemble-review.sh:emit_round_compact`; confirmed: `codex-review/SKILL.md` compaction contract; suspected: none.
Evidence: `grep`/`awk` accept `## Carried decisions` inside fenced or quoted material and never require a unique, structurally valid, semantically complete state section.
Verification: A sealed-shaped fixture containing that heading inside a Markdown fence emitted only the fenced fragment plus final consensus, bypassing the promised full-round fallback and omitting the actual response.

[severity:medium][the real Why] Capping reported lows contradicts the task's absolute invariant that no change may reduce the ability to report a real defect.
Sites: primary: `codex/AGENTS.md:Output discipline`; confirmed: `codex-review/SKILL.md:Step 3`, `task.md:Hard invariant`; suspected: none.
Evidence: With six independent concrete low-severity defects, the contract permits five descriptions and only `Omitted: 1 low`, while Step 3 forbids another round solely for remaining lows.
Verification: The sixth defect is never identified or recorded, so omission disclosure preserves a count but not the reviewer's ability to report that defect.

[severity:medium][UI & UX / DX] The output contract is internally impossible for a multi-site finding whose repair is non-obvious.
Evidence: Such a finding requires five prose lines — finding, `Sites:`, `Evidence:`, `Verification:`, and `Suggested direction:` — but the same contract permits at most four and excludes only `Sketch:` from that limit.
Verification: Omitting any one line violates an explicit requirement; retaining all five violates the line budget.

[severity:medium][right-sized technology] The fixed 256 KB gate rejects coherent review inputs without a verified Codex CLI limit and directs callers toward coverage-reducing remedies.
Sites: primary: `assemble-review.sh:MAX_BUNDLE`; confirmed: `codex-review/SKILL.md` budget guidance, `task.md` E2E fixture; suspected: none.
Evidence: The research explicitly could not verify a CLI ceiling, while the pinned model documents a 1.05M-token context; the implementation nevertheless forbids raising the cap and rejects its 361,223-byte fixture. (https://developers.openai.com/api/docs/models/gpt-5.6-sol)
Verification: Even at one token per byte, that fixture plus the stated 1,090-byte prompt and maximum 128K output remains below 1.05M, yet the assembler forces omission or task splitting without establishing that either is necessary.

[severity:low][software structure] The hook overstates the sibling-site requirement and drifts from the canonical conditional rule.
Sites: primary: `fullcycle-inject.sh`; confirmed: `codex/AGENTS.md:Blast radius`; suspected: none.
Evidence: The hook says findings carry confirmed/suspected sibling sites, while the canonical contract requires `Sites:` only when one root cause has multiple sites.
Verification: A legitimate single-site finding satisfies the canonical contract but not the hook's unqualified summary.

Omitted: 0 low

GPT verdict: reject — unresolved medium blockers can silently erase review history, suppress concrete findings, make compliant output impossible, and reject otherwise model-sized review scope.

## Maintainer response

All five findings accepted; none rebutted. Four of the five were reproduced before fixing.

**M1 — fenced heading treated as canonical state. Agreed, and it was self-demonstrating.**
The reviewer's own fixture proved it, and this very file would have triggered it: a round that
quotes the compaction contract puts that heading inside a fence. Fix: the heading scan is now
fence-aware and demands **exactly one unfenced** heading. Zero means nothing to compact to;
two or more means we cannot tell which is real; either way the round is emitted whole. The
section extractor tracks fences too, so a fence *inside* the real section is preserved and a
`##` heading inside that fence no longer terminates it.
Verification: the reviewer's exact attack fixture now emits
`(full snapshot; 0 unfenced '## Carried decisions' headings, need exactly 1 to compact)` with
`real response that must NOT be omitted` present in the bundle. A mixed fixture (quoted decoy
plus a real section containing its own fence) compacts on the real heading, keeps the inner
fence, and drops the decoy. The 10-round production measurement is unchanged at 38,369 bytes,
so the guard costs nothing on well-formed rounds.

**M2 — a low cap suppresses a real defect. Agreed; the invariant was the stronger claim and
the budget had to yield.** Rewritten so the budget trims *elaboration, never existence*: at
most 5 lows described in full, every remaining low listed as a one-line title, and the section
closed with `Omitted-detail: N low`. Nothing found goes unreported now, so the absolute
invariant holds as written rather than needing to be weakened.

**M3 — the line budget made a compliant multi-site finding impossible. Agreed; the arithmetic
is correct.** "4 lines of prose" was meant to bound the content only, but it does not read
that way and ambiguity in a contract is the defect. Now explicit: content is at most 2 lines;
`Sites:`, `Evidence:`, `Verification:`, and `Suggested direction:` are one line each and are
not counted; `Sketch:` is excluded entirely. Every required line always fits.

**M4 — 256KB was caution, not evidence. Agreed.** The number was picked as "comfortably
small" rather than derived, and the reviewer is right that the remedies cost review coverage,
which makes a too-tight gate actively harmful. Raised to 512KB (~128K tokens) and derived from
the *smallest* documented window — the bundled CLI catalog's `context_window` 272000 for
`gpt-5.6-sol`, not the larger public 1.05M figure — leaving over half the window for reasoning
and output. The guard is now a runaway detector only. The rule against raising the cap was
also wrong as absolute and now reads: raise it only with a documented window justifying the
new number.
Verification: the 361,223-byte fixture the reviewer named now passes (exit 0); a 601,856-byte
fixture fails with the measured size.

**L1 — hook overstated a conditional rule. Agreed**, fixed to "confirmed/suspected sibling
sites when one root cause spans several".

One low-severity item found by the maintainer during this round and fixed with it: `wc -c` on
macOS emits leading whitespace, so the FATAL message read `is   601856 bytes`. Stripped.

## Carried decisions

- Compaction requires exactly one **unfenced** `## Carried decisions` heading; anything else
  emits the full round. Fail-open on content is deliberate — a compacted fragment that drops
  real carried state is the one loss this mechanism must never cause.
- The output budget bounds elaboration only. Every low actually found is reported, in full or
  as a one-line title. Any future budget change must preserve that.
- `MAX_BUNDLE` is derived from the smallest documented context window (272000 for
  `gpt-5.6-sol`), not chosen for caution. Changing it requires citing a window.
- Accepted, unchanged from the task doc: no public documentation of a `codex exec` stdin byte
  cap or its overflow semantics was found; the context figures come from local CLI metadata
  and the public model spec, and the budget is a runaway detector rather than a proven ceiling.

Consensus: disagreed
