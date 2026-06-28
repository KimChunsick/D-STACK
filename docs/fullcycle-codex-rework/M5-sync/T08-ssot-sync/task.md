# T08 — SSOT sync: inject hook + CLAUDE.md + root AGENTS.md → new pipeline

## Intent / Why
The global entry points still describe the OLD workflow (conditional deep-research, single
task-doc, in-document review) — Codex flagged this (M4 #8). They must match the new pipeline
(per-Goal Codex research, GOAL.md + task folders, separate `codex-review.md`, milestone + Goal
E2E) or agents route off stale instructions. Then deploy via `install.sh` and confirm green.

## How (plan)
1. **Red**: guards asserting `fullcycle-inject.sh` + `claude/CLAUDE.md` describe the new
   pipeline (Codex research / GOAL.md / codex-review.md / milestone+Goal E2E) and not the old
   conditional-deep-research / in-document-review wording.
2. **Green**: update the inject hook ctx, `claude/CLAUDE.md` §0, and the root `AGENTS.md`
   codex note.
3. **Refactor**: keep public-safe; `bash tests/run.sh` green; `./install.sh --dry-run` clean.

## What was done (what / why)
- Synced the global entry points to the new pipeline: `fullcycle-inject.sh` ctx + `claude/CLAUDE.md`
  §0 now describe per-Goal Codex research, GOAL.md + task folders, separate `codex-review.md`,
  and per-task/milestone/Goal E2E (resolving M4 #8). Aligned stale `codex-review` vocabulary
  (DX in frontmatter; `docs/<goal>/…`).

## Files changed (where / why)
- `claude/hooks/fullcycle-inject.sh` — new-pipeline injected directive.
- `claude/CLAUDE.md` — §0 pipeline + gate description (tripwire framing).
- `claude/skills/codex-review/SKILL.md` — frontmatter DX + `docs/<goal>` vocabulary.
- `tests/test_fullcycle_skill.sh` — guards entry points match the new pipeline.

## E2E verification
- `bash tests/run.sh` → ALL PASSED; `./install.sh --dry-run` → clean.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus (2 rounds → agreed)
- [x] E2E capture verified
