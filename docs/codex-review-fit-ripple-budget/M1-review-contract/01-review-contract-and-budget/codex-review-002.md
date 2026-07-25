# Codex adversarial review — Round 002

## Review scope

Re-review. Verifies Round 001's five claimed fixes and hunts regressions in the compaction
and bundle-budget paths. Bundle 64,222 bytes across 13 emitted items; Round 001 emitted in
full (only one prior round exists, so no compaction applied yet).

## GPT findings

[severity:medium][technical correctness] Round 001's compaction fix still mistakes non-section Markdown content for canonical carried state.
Sites: primary: `emit_round_compact` heading counter; confirmed: `emit_round_compact` section extractor; suspected: none.
Evidence: Both AWK passes exclude simple fences only; a column-zero `## Carried decisions` inside an HTML comment is counted and extracted as a real section.
Verification: A sealed round containing an HTML comment open, the false heading, another heading, the comment close, real response text, and final consensus yields one hit and emits only the commented fragment plus consensus, silently discarding the real response.
Suggested direction: Require carried decisions to be the final top-level H2 before sealed consensus, using parsing that excludes Markdown block constructs such as comments and correctly matched fences.

[severity:medium][the real Why] The contract surfaces remain inconsistent after the accepted Round 001 fixes, violating the task's explicit synchronization invariant.
Sites: primary: `task.md` sections 4 and 6 plus E2E check 6; confirmed: `codex/AGENTS.md`, `codex-review/SKILL.md`, `assemble-review.sh`, `fullcycle-inject.sh`; suspected: none.
Evidence: The task still specifies `Omitted: N low`, four-line findings, and 256KB, while the implementation requires titled remaining lows with `Omitted-detail`, two content lines with uncounted labels, and 524288 bytes; the hook also omits the sketch's high/medium-only restriction.
Verification: The documented 361223-byte case must fail under `task.md` but passes under current code, and a low-severity sketch satisfies the hook summary while violating the canonical contract.

Omitted-detail: 0 low

GPT verdict: reject — compaction can still erase substantive review history, and multiple contract surfaces remain observably inconsistent.

## Maintainer response

Both accepted, both reproduced, neither rebutted.

**M1 — HTML-comment bypass. Agreed, and the suggested direction was better than patching
another construct.** Round 001's fix chased a *textual* problem — teach the matcher about
fences — which loses to the next construct, and the reviewer found it immediately. Adopted the
structural rule instead: compaction fires only when `## Carried decisions` is the round's
**final real section**, tracking both fenced blocks and HTML comments while finding that last
heading. A quoted or commented copy cannot structurally impersonate the last section, and
because carried decisions is last, emitting from it to EOF picks up the sealed consensus line
with no second pass — the whole `section` / `consensus` / `section_last` reconciliation from
Round 001 is deleted rather than extended.
Verification, four fixtures, each asserting the real maintainer response survives:
fenced decoy → `(full snapshot; '## Carried decisions' is not this round's final section)`
with `real response that must NOT be omitted` present; HTML-comment decoy (the reviewer's
exact attack) → same; real final section with a fenced decoy above it → compacted on the real
heading, decoy dropped; real section followed by `## Notes` → full snapshot. The production
10-round case is byte-identical to before at 38,369 bytes with rounds 001–008 compacted,
009–010 full, and exactly one `Consensus:` line per emitted file, so the stricter rule costs
nothing on well-formed rounds.

**M2 — contract surfaces drifted. Agreed, and this one is on the task's own stated
invariant.** Round 001 changed the implementation and left `task.md` describing the previous
contract, which is precisely the drift this task exists to prevent. Fixed: `task.md`
sections 4, 5 and 6, its hard-invariant sentence, its E2E checks 5 and 6, its files-changed
note, and the GOAL.md task row now all state the current contract. The hook gained the
`high/medium finding only` restriction on `Sketch:`.
Verification: a scan for every superseded token — `256KB`, `262144`, `Omitted: N low`,
`4 lines of prose` — across `docs/`, `claude/` and `codex/` returns nothing outside sealed
round 001, which is immutable history and correctly still shows what was true then. The hook
re-parsed as valid JSON with `high/medium finding only` present; secret guard green.

## Carried decisions

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
