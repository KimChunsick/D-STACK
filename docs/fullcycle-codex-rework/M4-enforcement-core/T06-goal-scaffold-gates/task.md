# T06 (incl. T07) — enforcement core: GOAL.md scaffold, task folders, DX, milestone+Goal E2E, gate schema, Stop hook

## Intent / Why
The structural heart that closes G1/G3/G4 and makes the 16 steps mechanically enforced:
- **One Goal**, with `docs/<goal>/GOAL.md` (intent + interview + research summary + checklist
  + **Goal gate**) and **task folders** (`<milestone>/<NN-task>/task.md` + `codex-review.md`).
- **DX** added to the UX axis (step 2).
- **Milestone-level E2E** (step 15) and **final Goal-level E2E** (step 16 termination).
- A **gate schema** the Stop hook scans, so milestone/Goal E2E are mechanically enforced.

> T06 and T07 are executed together: they rewrite the same `full-cycle/SKILL.md` + Stop hook
> and must stay internally consistent (GOAL.md template ↔ gate schema ↔ hook). Splitting them
> would create an inconsistent intermediate state.

## How (plan)
1. **Red**: extend `test_fullcycle_skill.sh` to assert the M4 contract (DX, one-Goal,
   GOAL.md scaffold + Goal gate, task folders, milestone E2E, Goal E2E).
2. **Green**: rewrite `full-cycle/SKILL.md` (Phases 2/5/6/9/10/11/12 + GOAL.md & task
   templates + gate schema); update `fullcycle-gate.sh` to be Goal-gate aware.
3. **Refactor**: keep phase numbering + templates consistent; tests green.

## What was done (what / why)
- Rewrote `full-cycle/SKILL.md` to the enforcement core: exactly **one Goal** + `GOAL.md`
  scaffold with a **Goal gate**, **task folders** (`task.md` + `codex-review.md`), **DX** on the
  UX axis, and **per-task / per-milestone / final-Goal E2E** phases (G1/G3/G4).
- Rewrote `fullcycle-gate.sh` into a real enforcement tripwire: **section-scoped** parsing
  (no checklist deadlock), **milestone↔gate tie**, **one-Goal** + **task-requires-Goal**,
  **schema-required** (a real `- [ ] GOAL E2E` checkbox row), **Codex-artifact** gating, and
  `docs/`-only path safety — all after a 4-round adversarial loop.

## Files changed (where / why)
- `claude/skills/full-cycle/SKILL.md` — full enforcement-core rewrite + GOAL.md/task templates.
- `claude/hooks/fullcycle-gate.sh` — Goal-aware, section-scoped, milestone-tied, schema-required.
- `claude/skills/codex-review/SKILL.md` — review axis now "UI/UX & DX".
- `tests/test_fullcycle_skill.sh` — M4 contract guards.
- `tests/test_fullcycle_gate_hook.sh` (new) — 11-case behavioral enforcement test.

## E2E verification
- `tests/test_fullcycle_gate_hook.sh` → 11 cases PASS: unchecked Goal/milestone/task gates block;
  ticked pass; checklist items don't deadlock; missing/prose schema blocks; one-Goal enforced;
  Codex gate needs a consensus artifact; escape hatch works. `bash tests/run.sh` → ALL PASSED.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus (4 rounds → agreed; hardened the core)
- [x] E2E capture verified
