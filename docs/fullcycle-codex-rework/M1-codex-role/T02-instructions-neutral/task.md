# T02 — codex/instructions.md → stack-neutral

## Intent / Why
`codex/instructions.md` is Codex's **global** instruction file. Today it hard-mandates a
Next.js/React/Tailwind/Vitest stack, so a stack-neutral adversarial reviewer/researcher
would wrongly assume Next.js when reviewing a Python/Go/Rust project. Codex's Round-2 review
of T01 left exactly this open (#3). Trim the file to stack-neutral engineering defaults so
the live Codex identity is consistent — closing M1.

## How (plan, before coding)
1. **Red**: add guards asserting `instructions.md` no longer mandates a single framework
   (no "Framework: Next.js" stack block, no "Server Components by default", no Tailwind
   "sole styling method") and *does* declare stack-neutrality + keeps "Research first".
2. **Green**: rewrite `instructions.md` as condensed stack-neutral engineering defaults;
   move "stack-specific rules belong per-project" up front.
3. **Refactor**: keep only genuinely universal principles; ensure public-safe.

## What was done (what / why)
- Rewrote `codex/instructions.md` from a 319-line Next.js/React/Tailwind/Vitest mandate into
  a ~30-line **stack-neutral engineering-defaults** file. Kept the genuinely universal
  principles (research-first, simplicity/surgical change, derive-don't-duplicate, DAG data
  flow, handle-every-state, strict types, a11y, behavior-not-implementation testing) and
  moved "stack-specific rules belong per-project; do not assume a default framework" to the
  top. Closes T01 Codex review open item #3 → live Codex is now stack-neutral.

## Files changed (where / why)
- `codex/instructions.md` — rewritten stack-neutral (the deliverable).
- `tests/test_codex_artifacts.sh` — added guards: no mandated framework / Server-Components
  default / Tailwind "sole styling"; must declare stack-neutrality + keep research ethos.

## E2E verification
- `tests/run.sh` → ALL TESTS PASSED. `grep -ci next.js codex/instructions.md` → 0.
- (Codex review consensus: see codex-review.md)

## Files changed (review round 2 also touched)
- `codex/AGENTS.md` — reconciled the instructions/AGENTS boundary (review #1).
- `codex/instructions.md` — qualified DAG/state/types, added security baseline + reduced-motion.
- `tests/test_codex_artifacts.sh` — added stack-token regression guards.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus (Round 2: approve / agreed)
- [x] E2E capture verified
