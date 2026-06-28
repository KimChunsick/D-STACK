# T04 — full-cycle Phase 3 → mandatory Codex research

## Intent / Why
Close the self-judgment loophole: Phase 3 must change from "conditional, Claude decides,
invoke deep-research if uncertain" to "**every Goal, run `codex-research`**." This wires the
T03 skill into the pipeline so the user's decision (research every Goal, by Codex) is the
default path, with deep-research only as fallback.

## How (plan)
1. **Red**: guard asserting full-cycle Phase 3 invokes `codex-research`, runs every Goal /
   unconditionally, keeps both-sides, and lists deep-research only as fallback; and that the
   old "conditional / skip when nothing uncertain" wording is gone.
2. **Green**: rewrite Phase 3 in `claude/skills/full-cycle/SKILL.md`.
3. **Refactor**: keep the phase list/desc consistent.

## What was done (what / why)
- Rewrote full-cycle **Phase 3** from "conditional Claude deep-research" to "**every-Goal
  Codex research**" via the `codex-research` skill; updated the phase checklist + frontmatter.
- After review: added ordering/`mkdir -p`, proportionality (depth scales, no skip), Goal-drift
  re-run rule, secret/untrusted restatement, and aligned fallback triggers.

## Files changed (where / why)
- `claude/skills/full-cycle/SKILL.md` — Phase 3 rewrite + checklist line + frontmatter.
- `claude/skills/codex-research/SKILL.md` — added `mkdir -p "$GOAL_DIR"` (ordering robustness).
- `tests/test_fullcycle_skill.sh` (new) — guards the Phase-3 contract (Codex/every-Goal/
  unconditional/both-sides/sources/fallback + no stale frontmatter).

## E2E verification
- `bash tests/run.sh` → ALL TESTS PASSED (incl. the new full-cycle contract test).
- Mechanical GOAL-level gate is M4's job; T04 is the textual pipeline change (Codex agreed).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus (2 rounds → agreed)
- [x] E2E capture verified
