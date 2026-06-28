# Codex adversarial review — M4 enforcement core

## Round 1 — GPT verdict: **reject** (deep, correct critiques of the global core)
1. `[high]` G1 not mechanically enforced (hook didn't require one GOAL.md).
2. `[high]` Hook enforces listed files, not the workflow — missing/empty/typo'd active list bypasses G3/G4.
3. `[high]` GOAL.md parsing too broad: scans every `- [ ]`, so the milestone/task checklist above the gate **deadlocks**.
4. `[high]` Gate ticking still honor-system (can't verify TDD/Codex/E2E).
5. `[high]` Milestone E2E not tied to milestones (1 checked line passes a 5-milestone Goal).
6. `[med]` Escape hatch unsound (removing GOAL.md disables enforcement).
7. `[med]` Hook follows arbitrary paths/symlinks, no `grep --`, no docs/ restriction.
8. `[med]` Stale entry points (inject hook, CLAUDE.md) still describe the old workflow.
9. `[med]` DX only in SKILL.md; codex-review prompt still says "UI/UX" only.
10. `[med]` Tests are string-matching; don't prove enforcement behaviorally.
11. `[low]` Task template has no placeholder body.

## Claude responses

**Fixed (mechanical):**
- **#3 (the real deadlock bug)** — hook now parses **only the `## Goal gate` section** of GOAL.md
  (and `## Gate status` of task docs) via a section extractor. The milestone/task checklist above
  the gate is ignored. New regression test C2 proves it no longer deadlocks.
- **#5** — hook now **ties milestones to gates**: every `### M<n>` heading must have a ticked
  `M<n> E2E` box in the Goal gate. Test C3 proves a missing milestone gate blocks.
- **#1** — hook enforces **exactly one** active GOAL.md (test C7) and **tasks require a Goal**
  (test C4).
- **#4 (partial)** — a ticked **Codex** gate now requires a real `codex-review.md` with an
  agreed/resolved consensus (tests C5/C6). Raises the cost of fake-ticking the most gameable gate.
- **#7** — hook now honors only `docs/**` paths, skips symlinked docs, and uses `grep --`.
- **#9** — codex-review prompt now reviews **"UI/UX & DX (developer experience)."**
- **#10** — rewrote the behavioral hook test to 8 cases: section-scoping, milestone tie,
  one-Goal, task-requires-Goal, Codex-artifact, escape hatch.
- **#11** — added a placeholder body to the task template.

**Scoped:**
- **#8** — the inject hook + `claude/CLAUDE.md` + root docs sync is **M5 (T08)**, the SSOT-sync
  milestone. Tracked there; not dropped.

**Honestly conceded as accepted residuals (defense-in-depth, not a single airtight gate):**
- **#2 / #4 (the fundamental limit)** — a Stop hook over self-attested docs is a **tripwire, not
  a sandbox**: it cannot force registration, nor prove TDD/E2E actually ran. This was stated up
  front when the user chose "Stop-hook enforcement," and the hook header now says so plainly. The
  real enforcement is **layered**: the inject hook forces full-cycle to *start*; the Stop hook
  trips on registered unchecked gates (now section-scoped, milestone-tied, Codex-artifact-gated);
  and the adversarial Codex review + E2E phases catch hollow work. Each layer raises the cost and
  visibility of skipping. Claiming a bash Stop hook can *prove* the work was done would be the
  exact "lie that it's done" the user forbids.

## Evidence
- `bash tests/run.sh` → ALL PASSED, incl. the 8-case Stop-hook behavioral test and the
  full-cycle contract test.

## Round 2 — reject (most resolved; one refined blocker)
#1/#3/#5/#7/#9/#10/#11 RESOLVED. Codex accepted the tripwire framing for registration but
refined #2: a **missing/typo'd gate schema** is mechanically checkable and was still not
fail-closed → G4 bypassable.

## Round 3 — reject (schema check too loose)
Added a schema requirement, but the predicate `grep 'GOAL E2E'` matched **prose**, not a
checkbox row → still bypassable.

## Round 4 — **approve**
Tightened to require a real gate row `^- \[[ x]\] GOAL E2E` (test C11: prose-without-checkbox
blocks). "The M4 enforcement core is acceptable with #4/#6 as accepted residuals and #8
deferred to M5."

## Consensus
- R1 reject → R2 reject (refined schema blocker) → R3 reject (prose-vs-checkbox) → R4 **approve**.
- **Consensus: agreed.**
- Accepted residuals (defense-in-depth, not a single airtight gate): **#4** TDD/E2E truth is
  self-attested (a bash hook can't prove the work ran — the Codex review + E2E phases are the
  other layers); **#6** the escape hatch is an intentional, documented pause. **#8** (sync the
  inject hook + CLAUDE.md to the new pipeline) is **M5 (T08)**.
