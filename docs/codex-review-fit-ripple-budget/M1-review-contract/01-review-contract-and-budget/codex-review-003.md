# Codex adversarial review — Round 003

## Review scope

Re-review. Verifies Round 002's structural-compaction rewrite and the contract-surface
resynchronization. Bundle 71,393 bytes; Rounds 001–002 both emitted in full (only two prior
rounds exist, so compaction has still not been exercised on this task's own history).

## GPT findings

[severity:medium][technical correctness] Round 002's "structural" compaction still mistakes headings inside live Markdown fences or comments for canonical carried state, allowing silent history loss.
Evidence: `emit_round_compact` toggles one fence boolean for either ``` or `~~~`, ignoring opener type and length; its comment state also misses a second `<!--` after an earlier comment closes on the same line.
Verification: The supplied AWK selected the false heading in both sealed-shaped counterexamples — line 8 for ``` / `~~~` / `## Carried decisions` / ``` and line 7 for `<!-- first --> <!--` / `## Carried decisions` / `-->` — which would omit the preceding maintainer response while retaining the valid final consensus.
Suggested direction: In `emit_round_compact`, track compatible fence character/length and process every ordered HTML-comment delimiter before recognizing headings.

Omitted-detail: 0 low

GPT verdict: reject — compaction still has reproducible paths that discard substantive sealed-round history.

## Maintainer response

Accepted. Both counterexamples reproduce exactly as described. The diagnosis is right and the
suggested direction is where I stopped agreeing.

**The finding: agreed, no rebuttal.** One boolean cannot represent fence state when two fence
characters exist, and one boolean cannot represent comment nesting when a single line both
closes and opens. Round 002 was the second textual patch to lose to the next construct that
Round 001's version had not anticipated.

**The suggested direction: declined, with a stronger fix instead.** "Track compatible fence
character/length and process every ordered HTML-comment delimiter" means hand-writing a
Markdown block parser inside the assembler — tilde-vs-backtick, opener length, indentation,
comments inside fences, fences inside comments — and every round of this loop so far has been
me getting one more construct wrong. That is the over-engineering the right-sized-technology
axis exists to catch: these are local Markdown files the maintainer's own pipeline writes, and
the requirement is not "parse Markdown correctly", it is "never compact when the heading might
not be real".

Replaced the state machine with three total checks, no state at all:
1. `## Carried decisions` must be the file's last occurrence of that heading;
2. no `^## ` heading may follow it — it is the final section;
3. every fence and comment delimiter opened before it must also close before it — counts of
   leading ```` ``` ````, leading `~~~`, and `<!--` versus `-->` must all balance.

Counting cannot be fooled the way a boolean can: any construct that hides the heading
necessarily leaves an unbalanced delimiter ahead of it, whatever its type, length, or nesting.
It is also shorter than what it replaced, and it fails toward emitting the whole round.

Verification, five fixtures, each asserting the real maintainer response survives:
- ``` / `~~~` / heading / ``` (this round's counterexample 1) → full snapshot, `MUST-SURVIVE` present
- `<!-- first --> <!--` / heading / `-->` (counterexample 2) → full snapshot, `MUST-SURVIVE` present
- a fenced decoy placed *after* the real content → full snapshot, `MUST-SURVIVE` present
- a legitimate final carried-decisions section → compacted
- a legitimate final section with balanced fences before it and its own fence inside it → compacted, inner fence preserved

Regression: the production 10-round bundle is byte-identical at 38,369 with 8 rounds compacted
and exactly one `Consensus:` line per emitted file; Round 001's and 002's four earlier
fixtures are unchanged; the budget guard still exits 1 at 601,856 bytes against the
524,288-byte budget; secret guard green.

## Carried decisions

- Compaction is decided by **three total checks — last occurrence, final section, balanced
  delimiters before it** — not by tracking Markdown block state. Three successive stateful
  versions lost to a construct their author had not thought of. Do not reintroduce a parser:
  if a new construct appears, add a balance count for it.
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
