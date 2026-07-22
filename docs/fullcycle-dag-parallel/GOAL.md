# GOAL — DAG-parallel full-cycle pipeline + structured skill schema

## Goal (the one Why)
Cut full-cycle wall-clock time without weakening its gates: schedule each milestone's tasks
as a dependency DAG (conditional parallelism guarded by a deterministic disjointness check),
always overlap adversarial-review rounds across disjoint tasks, delegate implementation of
verified-disjoint tasks to worker subagents in isolated worktrees, and rewrite the prose
skill document as a YAML phase/gate schema with short prose annotations so the orchestrator
interprets it deterministically instead of re-parsing ~250 lines of prose each session.

## Interview record (Phase 4)
- Implementation ownership: MIXED conditional parallelism — the main loop stays the default
  serial implementer; cross-task review rounds always overlap; full worker fan-out
  (subagent + worktree) only for tasks that pass the deterministic disjointness check.
- Dependency declaration: GOAL.md is the single source — each task row declares `deps` and
  `files` (planned ownership). task.md does not duplicate the declaration.
- False-independence guard: a deterministic bash checker shipped with the skill parses the
  GOAL.md declarations (DAG acyclicity + file-set overlap) and gates fan-out; any failure
  falls back to serial execution (fail-closed).
- Skill-document format: YAML schema blocks (phases / gates / scheduling) + short prose
  annotations, kept inside the single SKILL.md file.
- Fan-in integrity: milestone E2E is the integration defense; no standing post-merge delta
  review. If a merge produces conflicts or requires edits, only the affected task is
  re-reviewed.
- Worker: NEW dedicated `general-dev` agent (authored here, backed up in this repo) with a
  research-informed English system prompt oriented to plain, maintainable code; frontend
  code stays with `frontend-dev` unconditionally.

## Research summary (Phase 3)
Artifact: research/dag-parallel-pipeline.md (Codex GPT-5.5 + web, retrieved 2026-07-22)
- Worktree-per-agent isolation is productized industry practice (Claude Code, Cursor,
  OpenHands, Codex) [S2–S8]; DAG scheduling is proven CI/build prior art [S9–S13].
- Strongest against-point: LLM-declared task independence is brittle ("false independence",
  NeurIPS'25 [S20]). Mitigation adopted: the LLM proposes deps/file-ownership, a
  deterministic checker verifies; fail-closed to serial.
- Per-task scoped review cannot see sibling contract changes; semantic conflicts can merge
  cleanly [S23–S25] → milestone-level integration E2E stays mandatory; merge conflicts
  trigger per-task re-review.
- Schema-only rewrites lose judgment nuance [S16, S31] → YAML schema + prose annotations.
- Unverified: wall-clock gains at 2–6 task scale; the 15–25 min review-round figure is
  local lore — the biggest evidence-safe win is overlapping review rounds across tasks.
- Delta research (for T04): coding-agent worker system-prompt practices — artifact
  research/general-dev-prompt.md (gathered during M2).

## Milestones & tasks (Phase 5)
Task rows carry the deps/files declaration format this Goal itself introduces (first use).

### M1 — DAG-parallel pipeline restructure
- [x] **T01** skill-schema-rewrite — rewrite full-cycle SKILL.md as YAML
  phases/gates/scheduling blocks + prose annotations; add the per-task `deps`/`files`
  declaration format to the GOAL.md template; define conditional-parallelism,
  worker-delegation, pause, and fan-in rules. deps: []; files: [claude/skills/full-cycle/SKILL.md, claude/skills/full-cycle/tests/skill-schema.test.sh]
- [x] **T02** disjoint-check-script — deterministic validator for the GOAL.md task
  declarations: `plan` verdict (readiness, acyclicity/transitive incomparability,
  pairwise file-set disjointness) and `scope` verdict (actual-diff containment in the
  declaration); fail-closed.
  deps: [T01]; files: [claude/skills/full-cycle/check-parallel.sh, claude/skills/full-cycle/tests/check-parallel.test.sh]
- [x] **T03** surface-sync — sync claude/CLAUDE.md §0 and the fullcycle-inject.sh directive
  text with the restructured pipeline. deps: [T01]; files: [claude/CLAUDE.md, claude/hooks/fullcycle-inject.sh]

### M2 — general-dev worker agent
- [x] **T04** general-dev-agent — authored worker-agent definition (research-informed
  English system prompt; plain, maintainable code focus); add the .gitignore allowlist
  line, install.sh MAP row, and secret-guard pinned-negation update in the same change.
  deps: []; files: [claude/agents/general-dev.md, .gitignore, install.sh, tests/secret-guard.sh]

## E2E evidence (Phase 11–12)
- M1 (evidence/final-e2e.txt, 2026-07-22): the three tasks exercised TOGETHER —
  skill-schema suite validates the rewritten SKILL.md (yaml blocks parse, frozen hook
  strings intact); the checker self-hosts on THIS GOAL.md's declarations with correct
  PARALLEL/SERIAL verdicts ({T01,T04} was the pair actually built concurrently); the
  live inject hook emits the same DAG directive the skill defines ([quick] silent).
  Review rounds: reviews of different tasks genuinely overlapped during the goal
  (T01 round 001 and T04 round 001 ran concurrently on disjoint bundles).
- M2 (evidence/m2-install.txt, m2-smoke.txt, final-e2e.txt): install.sh linked the
  agent (re-run idempotent, 15 entries up-to-date); headless smoke test loads the
  linked definition and reports its scope STOP rule; the harness registered
  general-dev as a live agent type with the least-privilege tool list; secret-guard
  PASS (allowlist pins updated in the same change).
- GOAL (evidence/final-e2e.txt): one full pass over all four tasks' checks plus the
  secret guard in a single capture, everything green.
- Accepted residuals for ratification: (a) checker base/branch inputs are
  orchestrator-recorded (mistake-tripwire, not anti-falsification — same
  self-attestation scope as the Stop hook); (b) clean disjoint sibling merges are
  defended by milestone E2E, not an extra review round (interview decision); (c)
  gitignored files are outside scope's vision (never enter merges).

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: restructured skill + checker verified together (checker parses this GOAL.md's
  own declarations, verdicts correct on made cases, synced surfaces consistent)
- [x] M2 E2E: general-dev agent verified (install.sh links it, secret-guard green,
  delegation smoke test)
- [x] GOAL E2E: one full end-to-end pass of the whole Goal, captured
