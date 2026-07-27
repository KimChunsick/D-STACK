# 05-skill-wiring

## Intent / Why
Make `full-cycle` call `dstack` instead of carrying registry bash inline, restate the
byte-frozen hook contract for the new state location, and write down two procedures that do
not exist yet: ending a session at a milestone boundary (the durable state is entirely in the
Goal and task documents, so a `/clear` is safe once `reclaim` can re-tag the orphaned records),
and the migration-filename plus worktree guidance that replaced the claim lock dropped in
Phase 4.

## Design consult
Skipped — no trigger. Instruction document plus one obsolete test assertion; no new boundary.

## What was done (what / why)
**The embedded registry bash is gone**, replaced by `dstack` calls. That block was a mistake
twice over: an interrupted deregistration stranded a `.fullcycle-active.tmp` at a repo root, and
the skill loader substitutes positional-parameter references with the skill's own name, so the
old helper's argument reference arrived as the literal string `full-cycle` and a model following
the text verbatim registered *that* instead of a document path — ungated, with only a warning
nobody reads. The replacement documents the semantics that genuinely changed: one owner per
document with a loud refusal instead of silent replacement, explicit `reclaim` because no
liveness signal exists, fail-closed attribution, and a fail-loud legacy cutover.

**The hook contract block now describes the real gate surface**: `.dstack/active/` records,
`task.md` as the review-unit document name (both the gate and `assemble-review.sh` bind to it),
and the cutover trigger.

**Milestone-boundary session handoff is now a documented move.** After a milestone's E2E, every
piece of durable state is in the Goal and task documents, so `/clear` there is safe; resume with
`dstack status`, `reclaim` the records the id rotation orphaned, and re-read `GOAL.md`. This is
the answer to a Goal whose context grows monotonically across days.

**`waits.external` was self-contradictory and is fixed.** It ordered "background the run and act
on its completion notification" while the gate made turn-end impossible, so the notification
could never arrive. It now says to end the turn, and says why.

**Concurrent-streams guidance** replaces the claim lock dropped in Phase 4: never number
migration files sequentially (timestamp+slug instead, which is what Rails moved to and for this
reason), reach for a worktree only when a stream is long-lived enough to earn its setup and
resource isolation, and treat `dstack status` as visibility with no enforcement behind it rather
than pretending otherwise.

## Files changed (where / why)
- `claude/skills/full-cycle/SKILL.md` — P6 registry section, hook-contract block,
  `waits.external`, and a new concurrent-streams section.
- `claude/skills/full-cycle/tests/skill-schema.test.sh` — assertions 8. They pinned
  `.fullcycle-active` and its `mkdir` lock, both deliberately removed, so the test was demanding
  a mechanism that no longer exists. Replaced with assertions on what now has to be true
  (`.dstack/active/`, `dstack reg`, `dstack migrate`, `dstack unreg`), not deleted.
