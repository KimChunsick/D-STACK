# T05 — codex-review skill: separate review file + untracked-aware + attack research

## Intent / Why
Three fixes the prior audit (G2) and live reviews surfaced:
1. Codex's verdict + Claude's rebuttals must live in a **separate `codex-review.md` in the
   task folder**, not inline in the task doc (the user's explicit step 11/12).
2. The review material must include **untracked new files** — `git diff` omits them, so a
   brand-new deliverable was invisible to the reviewer (caught in T01 review).
3. The reviewer must also **attack the research's own assumptions** (dual-role mitigation).

This task codifies the convention this whole Goal has been dogfooding.

## How (plan)
1. **Red**: guard asserting the codex-review skill writes `codex-review.md` in the task
   folder, includes untracked files in review material, and tells the reviewer to challenge
   research assumptions.
2. **Green**: edit `claude/skills/codex-review/SKILL.md` Steps 1 & 3 + reviewer prompt.
3. **Refactor**: keep consensus loop intact.

## What was done (what / why)
- codex-review skill now writes the verdict + rebuttals to a **separate `codex-review.md` in
  the task folder** (G2), the reviewer is told to **attack the research's assumptions**, and —
  after a 4-round adversarial loop that caught a secret-exfil regression — review material is
  assembled by a **fail-closed allowlist helper** instead of auto-collecting untracked files.

## Files changed (where / why)
- `claude/skills/codex-review/SKILL.md` — frontmatter + Step 1 (allowlist helper) + Step 3
  (separate file) + Step 4 + reviewer prompt (attack research).
- `claude/skills/codex-review/assemble-review.sh` (new) — fail-closed allowlist assembler
  (symlink/secret-deny/size/binary gates; scoped per-file diff; all reads gated).
- `tests/test_codex_review_assembler.sh` (new) — behavioral fixture test (planted secret/
  symlink/binary/oversize/unnamed-secret).
- `tests/test_claude_artifacts.sh` — guards for the new convention + helper mechanisms.

## E2E verification
- Behavioral fixture test → planted secret/symlink/binary/oversize all skipped; unnamed secret
  absent; normal file + tracked scoped-diff included. `bash tests/run.sh` → ALL PASSED.
- A symlinked `task.md`→secret is skipped (no leak). Every task folder in this Goal already
  carries a separate `codex-review.md` (the convention, dogfooded).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus (4 rounds → agreed; caught a real exfil bug)
- [x] E2E capture verified
