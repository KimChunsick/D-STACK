# 03-surface-sync

## Intent / Why
Two other surfaces describe the pipeline and must not drift from the restructured
SKILL.md: claude/CLAUDE.md §0 (the global workflow contract) and
claude/hooks/fullcycle-inject.sh (the per-prompt directive string). This task syncs both
with the DAG-scheduling/structured-schema pipeline so a session never receives two
conflicting process descriptions.

## Design consult
Skipped — no trigger (text synchronization of surfaces defined by T01).

## What was done (what / why)
- RED: grep confirmed neither surface mentioned check-parallel / deps-files
  declarations / DAG scheduling (count 0 in both).
- GREEN: claude/CLAUDE.md §0 pipeline line now carries the deps/files declaration
  step, DAG-scheduled execution (serial default, review overlap, checker-gated worker
  fan-out with scope checks), and the "INVALID means fix the decomposition" rule;
  fullcycle-inject.sh's directive string gained the same stages so every prompt
  injection describes the same pipeline as SKILL.md.
- Verified: `bash -n` clean; the hook emits valid JSON whose context contains
  "DAG-scheduled"; the `[quick]` bypass still produces zero output.

## Files changed (where / why)
- `claude/CLAUDE.md` — §0 pipeline description synced with SKILL.md v2
- `claude/hooks/fullcycle-inject.sh` — per-prompt directive synced with SKILL.md v2

## Round-A (002-input) fix pass
The injected directive said different-task review rounds "may overlap", which lets a
session serialize every review while technically complying. Reworded to "overlap by
default", matching SKILL.md's rule (concurrent by default, serialization needs a
stated reason).

## E2E verification
evidence/final-e2e.txt ([T03] section, 2026-07-22): the live hook emits valid JSON
whose directive carries the full DAG-scheduled-execution clause (overlap by default,
checker-gated fan-out, INVALID-means-fix-decomposition), and `[quick]` still
produces zero bytes. Live proof: this session's own UserPromptSubmit injections
switched to the new directive text mid-goal. claude/CLAUDE.md §0 carries the same
stages (grep evidence in the capture).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
