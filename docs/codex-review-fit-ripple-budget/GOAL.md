# GOAL — Make the Codex adversarial review right-sized, ripple-aware, and size-bounded

## Goal (the one Why)

The adversarial review loop is losing value in three specific ways, and this Goal fixes
exactly those three without weakening the loop's adversarial function.

1. **Findings are not right-sized.** The reviewer sometimes demands machinery the project
   will never need (the maintainer's example: database replication concerns raised against
   a tool that only ever runs locally on one machine). That is noise the maintainer must
   read, rebut, and record — it costs rounds and buys nothing.
2. **Findings stop at the primary site.** A finding says "fix file A" when the same
   invariant is violated at B and C, so the next round rediscovers the sibling instance.
   Each rediscovery is a full round of wall-clock.
3. **The review bundle grows without bound.** Round N re-feeds every sealed round 1..N-1 in
   full, so history grows quadratically in round count, and the per-file 64 KB cap in the
   assembler has no total-bundle counterpart. Measured in this repo: the 10-round task
   `docs/fullcycle-review-speedup/M1-slim-tests/01-secret-guard-only` carries 60 KB of
   prior rounds plus a 23 KB task doc — 83 KB of pure history before a single diff is
   added. The maintainer has seen `codex exec` die on an over-limit error.

The Why underneath all three: a review round must stay cheap enough to run and honest
enough to trust. Every change here is bounded by one invariant — **none of it may make the
reviewer less able to report a real defect.**

## Interview record (Phase 4)

**Q1 — What does "it blows up" concretely mean?** → `codex exec` dies with an error
(only this symptom; not quality degradation, not unreadability, not cost). Follow-up: the
maintainer recalls a message about exceeding a maximum KB, exact wording not remembered.

*Investigation, recorded because it changes the design:* 83 KB of history is roughly 21K
tokens, and the locally bundled model catalog reports `gpt-5.6-sol` with
`"context_window": 272000`. So accumulated rounds alone do not explain a hard limit error.
Strings in `codex-cli 0.145.0` confirm a real runtime error
`"Codex ran out of room in the model's context window. Start a new thread or clear earlier
history before retrying."`, but the exact message the maintainer saw could not be verified.
The unbounded term in the bundle is **the allowlisted diffs**: the assembler caps each file
at 64 KB but never caps the file count or the total. A task touching many files can produce
a bundle an order of magnitude past the history. Design consequence: compaction alone is
not enough — the assembler must also enforce a **total-bundle budget that fails loudly with
the measured size**, so a future occurrence is diagnosable instead of a mystery.

**Q2 — What to do about bundle accumulation?** → Keep the most recent rounds in full and
reduce older rounds to their `## Carried decisions` + `Consensus:` line. Sealed files stay
byte-identical on disk; only what is re-fed to the model changes.

**Q3 — How many recent rounds stay full?** → **2.** Enough to verify the previous round's
findings and the round before it that produced the claimed fix.

**Q4 — How far may a code sketch go?** → Shape only, about 3–6 lines, pseudocode or
structure, no imports, no complete function body, nothing copy-pastable, and only when a
one-sentence `Suggested direction:` cannot name the invariant. This overrides the previous
absolute ban on code examples. The conflict is recorded, not averaged: the older rule was
"forbid large fix recipes"; the new rule keeps that and carves out a bounded exception.

**Q5 — How does the reviewer learn the project's scale?** → A declared field in `task.md`,
read by the reviewer as data. This is also the only technically viable place: the assembler
sends `task.md`, prior rounds, allowlisted files, and research artifacts — `GOAL.md` is
never in the bundle.

**Q6 — Should output length be capped?** → Yes, severity-differentiated: no cap on
high/medium, an explicit cap on low with the omitted count disclosed.

## Research summary (Phase 3)

Full artifact: `docs/codex-review-fit-ripple-budget/research/review-fit-ripple-budget.md`
(24 cited sources). Prior in-repo art: `docs/fullcycle-review-speedup/research/review-loop-speedup.md`.

**Key findings.** Right-sizing is already mainstream review doctrine: Google's guide names
over-engineering (excess generality, speculative future functionality) as reviewable and
asks whether a design is appropriate *for your system*; Azure Well-Architected warns against
both over- and under-engineering. Blast-radius reporting has guideline support (Google:
look beyond the diff hunks; Metabase: point out implications for code the PR does not
touch) but **no measured evidence that it reduces LLM review iteration count** — the benefit
is inferred. The absolute ban on code examples is stricter than any major guideline; Google
explicitly allows that "sometimes code is useful". OpenAI's compaction guidance supports
carrying a compact state item instead of a full transcript for long-running loops.

**Strongest opposing points, and how each is answered in the design:**
- *Telling a reviewer "small / local-only" biases it toward dismissing real issues.*
  Prompt-framing studies show framing shifts error profiles. → Answered by an explicit
  **"context is not a waiver"** clause enumerating what declared scale can never suppress.
- *An over-engineering axis will produce false "too much" findings against this repo's
  deliberate hardening.* → Answered by requiring a **counterfactual**: name the concrete
  current requirement the complexity makes harder. No demonstrated cost → low or omitted.
- *Blast-radius reporting invites scope creep and hallucinated call sites* (CR-Bench:
  pushing agents to find more issues raises spurious findings). → Answered by splitting
  sibling claims into **confirmed** (verified in supplied material, blocks) vs **suspected**
  (non-blocking follow-up), and restricting the search to supplied material.
- *"Commonize the duplicates" is how premature abstraction happens* (Metz: the wrong
  abstraction is worse than duplication). → Answered by allowing an extraction
  recommendation only at ≥2 confirmed sites with an already-present or obviously local
  boundary; otherwise the finding says "apply the same invariant at B and C".
- *Sketches anchor the implementer on the reviewer's possibly-wrong design.* → Answered by
  the shape-only bound plus keeping the standing rule that the builder owns the fix and
  must independently verify.
- *Compaction is lossy and can silently reopen an accepted risk or drop an unverified
  claimed fix.* → Answered by keeping 2 rounds full, compacting only to the section that
  exists to carry decisions forward, **falling back to the full round whenever that section
  is absent**, and telling the reviewer the sealed files remain available on request.
- *Output caps can hide high findings if the reviewer spends slots on lows.* → Answered by
  capping only low severity.

**Unverified, carried as accepted risk:** no study measures whether declaring deployment
context to an LLM reviewer suppresses legitimate findings; no study measures whether
blast-radius reporting reduces iteration count; no public documentation of a `codex exec`
stdin byte cap or its overflow semantics. The model-catalog context numbers were read from
local CLI metadata, not a published contract.

## Milestones & tasks (Phase 5)

*Revised during execution.* The original split (M1 contract / M2 assembler) put
`claude/skills/codex-review/SKILL.md` in both tasks. Reviewing them separately would have
deadlocked on the freeze rule — every file inside an open review bundle is immutable until
that round seals, so T02 could not have edited a file frozen by T01's round. They are one
coherent change to one skill and are now one task. The user also removed Red-Green-Refactor
from this repository mid-task, and later moved the contract itself out of the CLI prompt into
`~/.codex/AGENTS.md`; both are recorded in the task doc.

### M1 — Review contract and bundle budget
- [x] **T01** review-contract-and-budget — the right-sized-technology axis with its "context is not a waiver" and counterfactual guardrails; the confirmed/suspected sibling-site format with the bounded commonization rule; the shape-only `Sketch:` allowance replacing the absolute ban; the severity-differentiated output budget; the `## Deployment context` field in the task.md template; two-most-recent-rounds-in-full structural bundle compaction with a full-round fallback; a fail-loud 512KB total-bundle budget; and the contract relocated to `~/.codex/AGENTS.md` with the CLI prompt reduced to what is call-specific. deps: []; files: [claude/skills/codex-review/SKILL.md, claude/skills/codex-review/assemble-review.sh, codex/AGENTS.md, claude/skills/full-cycle/SKILL.md, claude/hooks/fullcycle-inject.sh]

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: ten direct checks recorded in the task doc — the prompt parses as one 1090-byte argument (was 4118); standing AGENTS.md instructions verified to load under the exact review flags; companion-based compaction with forged, truncated, misfiled and contradictory companions each refused and the round sent whole; requested-round retrieval honoured with fatal validation on malformed input; the budget guard firing at 512KB with its measured size; hook JSON valid; secret guard green
- [x] GOAL E2E: eight real `codex exec` review rounds ran end to end through the new assembler and prompt, ending `Consensus: agreed`. The reviewer used the new contract throughout — the right-sized-technology axis, `Sites:` with confirmed/suspected, and `Omitted-detail: N low` — which is the working proof that the contract loads from `~/.codex/AGENTS.md` rather than the CLI prompt. Final round trip: round history 52,506 bytes all-full versus 28,288 as sent (46%), and the loop closed under the wind-down rule with both remaining lows fixed rather than deferred
