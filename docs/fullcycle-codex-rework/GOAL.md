# GOAL — Codex as dedicated adversarial researcher + reviewer; full-cycle hardened to spec

> One Goal only. This document is the single source of truth for this work: the goal,
> the interview record, the research summary, and the milestone/task checklist + gates.

## Goal (the one Why)
Re-architect the full-cycle workflow SSOT so that:
1. **Codex (GPT-5.5, xhigh) becomes the maintainer's dedicated adversarial _researcher_
   AND _reviewer_.** Claude builds; Codex gathers balanced evidence (needed info,
   opposing views, evidence *for* and *against* the stated goal) and adversarially
   attacks the result.
2. **The 16-step workflow is mechanically enforced to spec** — closing the gaps found in
   the prior audit (G1 single-Goal + GOAL.md, G2 Codex output as a separate file in the
   task folder, G3 milestone-level E2E, G4 Goal-level loop with an explicit termination).

The underlying purpose: remove the *honor-system* weaknesses (research execution and gate
ticking were both Claude's self-judgment) by outsourcing the skeptical/evidence role to a
different model — turning "Claude grades its own homework" into genuine two-model
adversarial tension.

## Interview record (Phase 4 decisions)
| # | Question | Decision |
|---|---|---|
| 1 | When does Codex research run? | **Every Goal, once, unconditionally** (after tri-axis). Removes the self-judgment skip loophole. |
| 2 | How hard to enforce the new gates (milestone-E2E, Goal-loop)? | **Mechanically, via the Stop hook** scanning GOAL.md gates. |
| 3 | What terminates the "infinite loop" (step 16)? | **All milestone E2E pass + a final Goal-level E2E passes.** |
| 4 | What about `codex/instructions.md` (Next.js-global rules)? | **Trim to stack-neutral** (a global reviewer/researcher must not assume Next.js). |

## Research summary (Phase 3)
Single external unknown — *can `codex exec` actually do live web research?* — probed
empirically and **confirmed: Codex exec has a live `web.run` tool (WEB: yes)**, so the
"Codex is the researcher" premise holds. Full evidence + opposing view:
[research/codex-capability.md](research/codex-capability.md).

## Tri-axis notes (Phase 2)
- **Security**: high blast radius (this edits the *global* full-cycle skill + Stop hook =
  every future task in every project). `web.run` prompt-injection → treat Codex output as
  untrusted data. Codex review/research stays `sandbox=read-only`. Never pipe secret files.
- **UX/DX**: more Codex round-trips = slower turns; must degrade gracefully if Codex is
  unavailable (fallback to Claude `deep-research`). Gate schema must be unambiguous so the
  hook parses it and Claude does not mis-tick.
- **Technical**: keep hooks pure-bash (no deps). Bound the consensus loop + Goal loop to
  avoid deadlock. Dual-role conflict mitigated by making the reviewer attack its own
  research assumptions.

## Milestones & tasks (Phase 5)

### M1 — Codex role redefinition (identity + config) ✅
- [x] **T01** `codex/AGENTS.md` (new): dual-role identity (researcher + reviewer), stack-neutral. Wire `.gitignore` allow + `install.sh` map + `tests/test_codex_artifacts.sh` guard. (Codex review: resolved)
- [x] **T02** `codex/instructions.md`: trim Next.js-global assumptions → stack-neutral. (Codex review: agreed)

### M2 — Codex researcher pipeline ✅
- [x] **T03** New `claude/skills/codex-research/SKILL.md`: `codex exec` + `web.run`, gathers needed info + opposing views + goal-for/against, with sources & recency, fallback. Registered. (Codex review: agreed, 3 rounds — found a real copy-paste bug)
- [x] **T04** full-cycle Phase 3 rewrite: research run **every Goal** by Codex; artifact + GOAL.md summary; proportional depth; fallback. (Codex review: agreed)

### M3 — Codex reviewer output separation (G2) ✅
- [x] **T05** `codex-review` skill: verdict saved as a **separate `codex-review.md`** in the task folder; reviewer attacks research assumptions; review material via a **fail-closed allowlist helper** (Codex caught a secret-exfil regression — 4 rounds → agreed).

### M4 — Enforcement core (G1, G3, G4) ✅  (T06+T07 executed together)
- [x] **T06+T07** full-cycle skill: one Goal + GOAL.md scaffold + Goal gate, task **folders**, **DX** axis, per-task/per-milestone/final-Goal E2E. Stop hook rewritten: section-scoped, milestone-tied, one-Goal, schema-required (checkbox row), Codex-artifact-gated, docs/-only. 11-case behavioral test. (Codex review: agreed, 4 rounds — substantially hardened the core; #4/#6 accepted residuals, #8→M5.)

### M5 — SSOT sync & green ✅
- [x] **T08** Synced `claude/CLAUDE.md` §0 + inject hook to the new pipeline; aligned codex-review vocabulary (DX, `docs/<goal>`). `bash tests/run.sh` green; `./install.sh --dry-run` clean. (Codex review: agreed)

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: Codex role config applied & verified — live `~/.codex/AGENTS.md` linked; `codex exec` replied `MODES: Research; Adversarial review` + `DEFAULT_FRAMEWORK: no; inspect the project first`; tests green.
- [x] M2 E2E: Codex research pipeline runs end-to-end — documented Phase-3 command (mkdir + `--ephemeral` + `-o`) executed verbatim → exit 0, artifact with all 6 sections + cited dated sources landed in `docs/<goal>/research/`.
- [x] M3 E2E: Codex review produces a separate task-folder file — every task folder in this Goal carries its own `codex-review.md`; fail-closed assembler behaviorally tested (planted secret/symlink/binary skipped, unnamed secret absent).
- [x] M4 E2E: enforcement core works — 11-case Stop-hook behavioral test passes (unchecked Goal/milestone/task gates block; checklist doesn't deadlock; missing/prose schema blocks; one-Goal + Codex-artifact enforced; escape hatch sound).
- [x] M5 E2E: full `tests/run.sh` green (8 test files) + `./install.sh --dry-run` clean.
- [x] GOAL E2E: the whole Goal was executed THROUGH the new pipeline (dogfooded — 8 tasks, real Codex adversarial rejects + consensus loops), then **deployed live** via `install.sh` (all skills/hooks/AGENTS.md symlinked to this SSOT) and the **live Stop hook** was confirmed to gate this very GOAL.md (blocked on the unchecked GOAL E2E box, with the milestone tie M1–M5 satisfied) — i.e. one full end-to-end pass, mechanically enforced.
