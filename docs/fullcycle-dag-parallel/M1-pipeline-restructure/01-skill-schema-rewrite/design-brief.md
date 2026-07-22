# Design brief — full-cycle SKILL.md restructure (YAML schema + DAG scheduling)

## Context
`claude/skills/full-cycle/SKILL.md` (~250 prose lines) defines a mandatory 12-phase
delivery pipeline an LLM orchestrator re-reads each session. A bash Stop hook
(`fullcycle-gate.sh`) independently enforces gates by parsing the WORK DOCS the pipeline
produces (GOAL.md `## Goal gate` section, task.md `## Gate status` section, and the
`codex-review-<NNN>.md` series contract). The hook does NOT read SKILL.md. Reviews are
per-task, scoped by a fail-closed file allowlist; a review round is voided if bundled
files change mid-round. Everything currently runs serially.

## Decided requirements (user-settled; not up for re-litigation)
1. Mixed conditional parallelism: main loop stays the default serial implementer;
   adversarial-review rounds for different tasks always overlap; full worker fan-out
   (subagent + git-worktree isolation, one worker per task) only for tasks that pass a
   deterministic disjointness check. Fail-closed to serial.
2. Per-task `deps` (predecessor task ids) and `files` (planned ownership) are declared in
   GOAL.md only — the single source the checker parses.
3. A bash checker (`check-parallel.sh`, separate task) validates: acyclicity, declaration
   completeness, pairwise file-set disjointness for a parallel candidate set.
4. Fan-in: merge in dependency order; a merge conflict or post-merge edit re-reviews only
   the affected task; milestone E2E is the standing integration defense.
5. Document format: YAML blocks (phases / gates / scheduling) + short prose annotations,
   all inside the single SKILL.md.

## Intended design (review this)

### A. Phase schema
One YAML block listing all 12 phases. Per phase: `id` (P1-intent … P12-goal-e2e),
`per` (goal | milestone | task), `needs` (phase ids), `gate` (which doc/checkbox or
artifact closes it), and a short `note`. Load-bearing prose (registry helper bash,
codex invocation guidance, honest-scope caveats) stays as fenced blocks/prose sections
referenced from phase `note`s — the schema points, prose elaborates.

### B. Task-declaration format in GOAL.md
Keep the existing human checklist as the single representation and append a
machine-parseable suffix to each task row:
`- [ ] **T<NN>** <slug> — <free prose>. deps: [T01, T02]; files: [path1, dir2/, glob?]`
Open questions for you:
- Rows naturally wrap across physical lines (they already do in the first real GOAL.md).
  Parser contract options: (a) declaration must sit on the item's LAST physical line;
  (b) checker joins continuation lines of a list item before parsing. Which is less
  fragile for a bash/awk parser maintained by hand?
- `files` entry semantics: literal paths + trailing-slash directory prefixes only, or
  also globs? Glob-vs-glob overlap in bash is error-prone; is prefix-match the right
  ceiling?
- Alternative rejected so far: a separate fenced YAML task-graph block (easier parsing,
  but duplicates the checklist and can drift). Confirm or refute the rejection.

### C. Scheduling semantics block
YAML: modes (serial default / review-overlap always / worker-fanout conditional),
fan-out precondition (checker PASS on the candidate set + no dep edge), worker binding
(frontend files → frontend-dev agent; else general-dev agent; delegation prompt carries
the task.md brief + conventions), worktree mechanics, fan-in rules (topological merge,
conflict→re-review that task, milestone E2E), and pause semantics.
Open questions:
- Worktree mechanics: harness-managed isolation (the Agent tool's worktree option,
  auto-cleanup semantics) vs explicit `git worktree add`/branch/merge steps written into
  the skill. Which is more robust across arbitrary target repos?
- Pause semantics: the Stop hook blocks turn-end while gates are open. Long external
  waits (15–25 min background codex rounds) currently force either a blocked Stop loop
  or unregistering the doc lines ("pause") and re-registering on resume. Intended
  design: bless unreg-pause for `external-wait` alongside `user-input`, with a mandatory
  re-register-on-resume rule, and NO hook change. Is there a materially safer
  alternative that avoids weakening the tripwire (e.g. a hook-visible pause marker),
  given the constraint that hook changes are out of this Goal's scope?

### D. Invariants (must hold after the rewrite)
- No hook-parsed surface changes: `## Goal gate`, `## Gate status`, milestone-heading →
  gate-box coupling, codex-review series contract all stay byte-compatible.
- The codex-review skill contract (allowlist bundle, one round per file, void-on-mutate)
  is referenced, not restated — but the "reviews for different tasks may run in
  parallel" clause becomes load-bearing: parallel tasks MUST have disjoint file sets so
  no sibling edit can void an open round. State this as the bridge invariant.
- Registry semantics: during worker fan-out, every active task.md is registered
  concurrently (multi-line registry already supported).

## What to return
Concrete design risks, failure modes, and a recommendation per open question, in
English. Flag anything in the intended design that will bite during implementation or
that contradicts the decided requirements. No code. This is a one-shot design consult,
not a review round — no verdict line needed beyond your assessment.
