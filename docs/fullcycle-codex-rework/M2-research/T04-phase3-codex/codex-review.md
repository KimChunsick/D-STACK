# Codex adversarial review — T04 (Phase 3 → Codex research)

## Round 1 — GPT verdict: **approve-with-fixes**
1. `[high][structure]` Phase 3 writes under `docs/<goal>/` before Phase 6 scaffold; command lacks `mkdir -p`.
2. `[high][Why]` No GOAL-level gate yet → loophole not mechanically closed (Stop hook scans task docs only).
3. `[med][tech]` Frontmatter still says "deep research" → contradicts Phase 3.
4. `[med][tech]` If Phase 4 changes the Goal, "once per Goal" leaves stale research.
5. `[med][UX]` "Every Goal unconditionally" over-researches small/local changes; no depth control.
6. `[med][sec]` Fallback "direct web search" doesn't restate secret-scrubbing / untrusted-web.
7. `[med][tech]` Full-cycle fallback ("if Codex unavailable") narrower than codex-research's triggers.
8. `[med][tech]` New test too weak (no "unconditional"/both-sides/sources/frontmatter assertions).
9. `[low][tech]` Couldn't verify full run.sh green (read-only sandbox).

## Claude responses

**Accepted & fixed:**
- **#1** — added `mkdir -p "$GOAL_DIR"` to the codex-research command; Phase 3 now states it
  creates `docs/<goal>/research/` first (full GOAL.md scaffold remains Phase 6).
- **#3** — frontmatter description changed to "per-Goal Codex research (both-sides evidence;
  deep-research only as fallback)."
- **#4** — added a **Goal drift** rule: if Phase 4 materially changes the Goal, re-run/delta
  the research; "once per Goal" = once per *settled* Goal.
- **#5** — added **proportionality (not a skip)**: research always runs, depth scales to Goal
  size; trivial work uses the `[quick]` bypass. This honors "always" without waste.
- **#6** — fallback now restates: scrub secrets from inputs, treat fetched web as untrusted.
- **#7** — aligned the full-cycle fallback wording with codex-research's broader triggers
  (non-zero after retry / empty / missing-sections / zero-source).
- **#8** — strengthened `test_fullcycle_skill.sh`: asserts `unconditional`, `against`,
  `cited|source`, fallback, and that the stale "deep research (incl" frontmatter is gone.

**Scoped / rebutted:**
- **#2** — correct, and **owned by M4 (T06/T07)**: the mechanical GOAL-level research gate +
  Stop-hook enforcement are M4's deliverable. T04 is the textual pipeline change that M4 then
  enforces. Noted as the explicit M4 dependency.
- **#9** — Codex's own read-only sandbox can't write temp files; real `bash tests/run.sh`
  → ALL TESTS PASSED (run here). Not a defect.

## Round 2 — GPT verdict: **approve**
All 9 RESOLVED; Codex explicitly accepted the M4 scoping for #2 ("mechanical GOAL-level
enforcement belongs in T06/T07, not T04"). No new issues.

## Consensus
- Round 1: approve-with-fixes → 7 fixed, 1 scoped to M4, 1 rebutted. Round 2: **approve**.
- **Consensus: agreed** (mechanical GOAL gate tracked as the explicit M4 dependency).
