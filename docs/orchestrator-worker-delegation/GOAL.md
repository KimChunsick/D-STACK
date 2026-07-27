# GOAL — decide and encode when full-cycle delegates implementation to a worker subagent

## Goal (the one Why)

The orchestrator's context grows monotonically because implementation transcripts accumulate in
it. The Goal that just closed spent 205,400 tokens of conversation messages against 37,000 tokens
of fixed overhead in a single session, and nearly all of that 205k was implementation: tool calls,
file contents, and iteration. Every later turn re-sends the whole thing.

The pipeline already delegates implementation to worker subagents, but only when
`check-parallel.sh` returns a PARALLEL verdict. That gate keys on the wrong property. Whether two
tasks can run *simultaneously* has nothing to do with whether one task's implementation transcript
belongs in the orchestrator's context. This Goal replaces that gate with one that keys on the
property that actually matters, and writes down what happens to an adversarial-review finding once
implementation lives somewhere else.

What this Goal is NOT: a decision to delegate everything. Phase 3 research found no source
supporting blanket delegation, and several measuring its cost. The scope is settled at P4 and
recorded there.

## Interview record (Phase 4)

Four decisions, all settled by the user on 2026-07-27. Every answer took the recommended option,
and three of the four moved AWAY from the request as originally phrased, because Phase 3 research
contradicted it.

**Q1 — How far should delegation go?**
*Answer: cut by TASK SHAPE, not by declaration completeness alone.* Delegate an implementation
that has a complete declaration and is written to a spec. Keep review-fix rounds, exploratory
work (finding out why something behaves as it does), and anything touching `docs/` or the
pipeline's own skill files in the orchestrator. The original request was to delegate every task
with a complete declaration; research found no source supporting that, and Claude Code's own
delegation guidance names "iterative refinement" and "shared context across
planning/implementation/testing" as main-conversation cases — which is exactly the review loop.

**Q2 — Worktree for serially delegated work?**
*Answer: always worktree.* The `scope` containment check requires a clean tree and a committed
`base..HEAD` range; a worker editing the main checkout satisfies neither, so dropping worktrees
would silently delete the only mechanism that catches a worker writing outside its declaration.
The costs are accepted and must be written down rather than discovered: a fresh checkout,
gitignored files needing explicit `.worktreeinclude` copying, dependency installs, per-tree
isolation of shared runtime resources, and the documented default that subagent worktrees branch
from the DEFAULT branch unless configured to use current `HEAD`.

**Q3 — `/clear` destroys warm workers. What wins?**
*Answer: document the constraint.* `SendMessage`'s name/ID scope resets on `/clear`, while the
pipeline's own milestone-boundary handoff instructs a `/clear`. The rule becomes: no `/clear`
while a review unit's loop is open; the handoff happens after the unit closes. This is one line
and it does not fight anything that already works.

**Q4 — Frontend delegation vs the new task-shape gate.**
*Answer: the global rule wins.* `CLAUDE.md` §0.2 requires every frontend code change to go to
`frontend-dev`, with no exception beyond a one-line typo. The new task-shape gate therefore
governs NON-frontend work only. Exploratory frontend work continues to a warm `frontend-dev`
worker via `SendMessage` rather than coming back to the main loop. Keeping §0.2 intact also keeps
this Goal's blast radius inside this repository.

**Recorded limitation, not a decision.** This Goal's own tasks edit the pipeline's skill files,
which the new rule assigns to the orchestrator. So the Goal cannot dogfood its own change: the
E2E verifies the decision procedure against crafted fixtures and the checker's real output, not
by delegating one of these tasks. Stated here so a later reader does not mistake the absence of a
live fan-out for an oversight.

## Research summary (Phase 3)

Artifact: `docs/orchestrator-worker-delegation/research/orchestrator-worker-delegation.md`
(13 sources, all primary). Brief:
`docs/orchestrator-worker-delegation/research/orchestrator-worker-delegation.brief.txt`.

**Findings that change the design.**

- **Resumed subagents keep their full prior context**, including tool calls and reasoning, and
  subagent transcripts are *unaffected by main-conversation compaction*, persisting within the
  same session with default cleanup after 30 days. `Explore` and `Plan` are one-shot and cannot
  be resumed. This is the mechanism the proposed findings-attribution rule depends on, and it
  exists. (Claude Code sub-agents doc, retrieved 2026-07-27)
- **But `SendMessage`'s name/ID scope resets on `/clear`.** The pipeline's own
  milestone-boundary session handoff tells the maintainer to `/clear` and then
  `dstack reclaim`. That handoff therefore *destroys* every warm worker. The two mechanisms this
  Goal wants to combine are in direct tension, and nothing currently says so.
- **Claude Code's own delegation guidance is conditional, and it names this pipeline's review
  loop as a non-delegation case**: use the main conversation for frequent back-and-forth,
  iterative refinement, and shared context across planning/implementation/testing; use subagents
  for verbose, self-contained work that returns a summary.
- **Non-fork subagents start with no parent context** — they do not see the parent conversation,
  invoked skills, or files already read. A complete declaration is therefore *necessary but not
  sufficient*: the brief is a lossy compression boundary, and everything discovered but not
  serialized is lost.
- **Worktrees are documented as the isolation mechanism**, which supports keeping them if
  containment depends on a clean tree and a committed `base..HEAD` range — but their costs are
  concrete: a fresh checkout, gitignored files needing explicit `.worktreeinclude` copying, and
  subagent worktrees branching from the *default branch* unless configured to use current `HEAD`.

**Strongest opposing point.** No reviewed source supports delegating all implementation work
because declarations are complete. Anthropic's own production multi-agent system reported ~15x
chat tokens and stated that most coding tasks have fewer truly parallelizable parts than research;
`Agentless` (32.00% on SWE-bench Lite at $0.70/task) beat contemporary agentic systems by
deliberately removing autonomous planning and tool use; and MAST catalogued 14 failure modes over
1,600+ traces across 7 frameworks, finding multi-agent gains often minimal. The direction of that
evidence is: delegate less than you think, and only where task shape justifies it.

**Second opposing point.** If the orchestrator becomes mostly a router over already-declared
dependencies and file ownership, that is an expensive model doing deterministic work — which the
maintainer's own standing rules already forbid ("Do NOT use me for: routing, retries,
deterministic transforms").

**Evidence for the goal.** `MASAI` is direct prior art: modular sub-agents with well-defined
objectives, gathering repo information from different sources and avoiding long trajectories with
extraneous context, reporting 28.33% on SWE-bench Lite. And the parent-context reduction this Goal
wants is precisely what Claude Code subagents are documented to do — verbose exploration and file
contents stay in the worker, only a summary returns.

**Unverified, carried as risk.** No public evidence compares "orchestrator implements serial
tasks directly" against "orchestrator delegates all serial tasks to subagents in worktrees" for a
one-maintainer pipeline; that comparison does not exist and this Goal cannot appeal to it. No
evidence shows review-fix loops improve when routed back to the original implementation worker —
the closest is general evaluator-optimizer guidance plus the resume semantics above. Local
worktree costs (dependency reinstall, port and test-database contention) are unmeasured here.

## Milestones & tasks (Phase 5)

Review granularity: **per task** (the default). Each task folder carries the registered `task.md`
and its own `codex-review-<NNN>.md` series. That is deliberate — the previous Goal reviewed at
milestone granularity and its loops ran 9 and 10 rounds on 6-11 file bundles.

One milestone: the change is one coherent edit to the pipeline's delegation contract plus the two
places that quote it. Repo policy (`AGENTS.md`) replaces P7's Red-Green-Refactor with direct
verification, and forbids new test files.

### M1 — delegation contract and the places that quote it

T01 and T02 both own `claude/skills/full-cycle/SKILL.md`, so they are dependency-ordered and can
never be a parallel candidate set. That is intended: they are separate review units so each
round's bundle carries one concern, not two.

- [x] **T01** delegation-gate — replace `worker-fanout.requires`' PARALLEL precondition with the task-shape gate: a complete declaration (checker non-INVALID, non-empty files list) AND an implementation written to a spec. Record what stays with the orchestrator (review-fix rounds, exploratory work, `docs/` and pipeline skill files) and that `CLAUDE.md` §0.2 keeps frontend delegation unconditional, so this gate governs non-frontend work only. Keep the worktree mandatory and write down its documented costs, including that subagent worktrees branch from the default branch unless configured to use current HEAD. deps: []; files: [claude/skills/full-cycle/SKILL.md]
- [x] **T02** findings-attribution — add the P9 rule for who fixes an adversarial-review finding once implementation lives in a worker: a finding whose fix is contained in ONE task's declaration returns to that worker via `SendMessage` with its context intact; a finding crossing declarations, or touching `docs/` or a pipeline skill, is the orchestrator's. Record in the same change that `SendMessage` scope resets on `/clear`, so the milestone-boundary handoff may not run while a unit's review loop is open. deps: [T01]; files: [claude/skills/full-cycle/SKILL.md]
- [x] **T04** scope-union — the `scope` check enumerates `git diff --name-only -z --no-renames <base> HEAD`, a NET two-tree comparison, so a path added in one commit and removed in the next is absent from the result while its content stays in branch history. The delegation gate this Goal is writing leans on `scope` being a real containment check, so the checker must validate the UNION of paths changed by every commit in the range instead. Found by T01's design consult, in a file no other task owns. deps: []; files: [claude/skills/full-cycle/check-parallel.sh]
- [x] **T03** quoted-copies — update the two places that restate the old gate: the schema check's pinned assertions, which currently pin `requires:` and the checker script name but nothing about what the gate keys on, and `claude/CLAUDE.md` §0, whose one-line pipeline description still says fan-out happens "only on a `check-parallel.sh` PARALLEL verdict". Both are maintenance of existing files, not new tests. deps: [T01, T02]; files: [claude/skills/full-cycle/tests/skill-schema.test.sh, claude/CLAUDE.md]

## Success criteria, and what would falsify this Goal

Phase 7's design consult landed a fair objection: the motivating measurement is orchestrator
conversation tokens only, so the change could shrink that number while total model tokens, wall
time, and rework all rise, and it would still look like a win. Recorded so a later reader can
check rather than assume:

- **Intended effect** — a delegated task's implementation transcript does not enter the
  orchestrator's context; only the brief and the returned report do.
- **What would falsify it** — total model tokens across orchestrator plus workers rising for
  comparable work; delegated tasks needing more review rounds than orchestrator-run ones; scope
  expansion requests or stale-base rebases becoming routine rather than rare.
- **Not measured here, and honestly so** — this Goal cannot A/B itself. No public evidence
  compares the two arrangements for a one-maintainer pipeline (Phase 3, Unverified), and this
  Goal's own tasks are orchestrator-owned by its own rule. The first Goal that actually delegates
  is the measurement.

## M1 E2E evidence (Phase 11)

Run on 2026-07-27. The milestone's claim is that delegation and parallelism became two separate
questions, and that `scope` is a real containment check the delegation gate can lean on. Three
things had to be shown, against the checker's real output and crafted declarations.

**1. The checker still answers the parallelism question, and answers it three ways.**

```
  T01 T02   disjoint, ready       PARALLEL: T01 T02                                     (exit 0)
  T01 T03   dependent pair        SERIAL: T03 not ready — dep T01 incomplete            (exit 1)
  T02 T04   file overlap          SERIAL: T02 and T04 overlap on 'claude/c.txt' ...     (exit 1)
  T01 T05   empty declaration     SERIAL: T05 has an empty files declaration ...        (exit 1)
  T01 T02   cyclic graph          INVALID: dependency cycle detected                    (exit 2)
```

**2. The delegation answer is now independent of it.** Same fixture, applying `delegate-when`:

- `T01` (one declared file, no verification run of its own) is in a PARALLEL set and is NOT
  delegation-eligible — the third condition, positive isolation benefit, is unmet.
- `T02` (two declared files, own build) is in a SERIAL set with `T04` because of file overlap, and
  IS delegation-eligible.

Under the old gate both statements were impossible: eligibility WAS the PARALLEL verdict, so a
delegable task in a serial set could not exist and a non-delegable task in a parallel set could not
either. Both now do. That inversion is the milestone.

**3. `scope` catches what it previously could not see.** A file added in one commit and removed in
the next, inside the reviewed range:

```
  net two-tree diff (the OLD enumeration):   src/a.txt
  union of every commit (what it collects):  src/a.txt, src/OUTSIDE.txt

  scope T01 (undeclared path in history)   VIOLATION: src/OUTSIDE.txt is not in T01 declaration
  scope T01 (declared paths only)          PASS
```

The undeclared file is absent from the net diff while its content stays in the branch history that
gets merged. Without T04 this milestone would have written a delegation gate on top of a containment
check that could be walked past in two commits.

**4. The two restatements agree with the contract they restate.** Every claim `claude/CLAUDE.md`
section 0 makes about delegation resolves against the PARSED `worker-fanout` node — declaration
complete, write set determined, isolation benefit, PARALLEL decides concurrency only, `docs/` and
skills and exploratory work retained, review fixes retained with the worker exception, and 0.2
outranking the retention list. Full table in `03-quoted-copies/task.md`. Both pinned checks green:
`skill-schema.test.sh` → `== all checks passed`, `secret-guard.sh` → green.

**Not shown, and honestly so.** No task was actually delegated to a worker. Every task in this Goal
edits `docs/` or a pipeline skill file, which the new rule assigns to the orchestrator, so the Goal
cannot dogfood its own change — recorded at P4 as a limitation, not discovered here.

## GOAL E2E evidence (Phase 12)

Run on 2026-07-27. One full pass over what this Goal changed, using the real artifacts rather than
fixtures where a real artifact exists.

**Everything that must run, ran.**

```
  ./install.sh --dry-run    linked=0 copied=0 backed-up=0 up-to-date=18 skipped=0
  skill-schema.test.sh      == all checks passed
  tests/secret-guard.sh     ✓ PASS: secret guard
  fullcycle-gate.sh         block — "GOAL.md has unchecked Goal-gate boxes"
```

The hook's block is the correct answer at the moment it ran: every review unit had been closed and
deregistered and `M1 E2E` ticked, so the only thing it could still name was the box this section
exists to earn. A gate that named nothing at that point would be the broken one.

**The decision procedure walked end to end, on this Goal's own declarations.**

```
  T01  files: [claude/skills/full-cycle/SKILL.md]
  T02  files: [claude/skills/full-cycle/SKILL.md]                        deps: [T01]
  T04  files: [claude/skills/full-cycle/check-parallel.sh]
  T03  files: [claude/skills/full-cycle/tests/skill-schema.test.sh,
               claude/CLAUDE.md]                                         deps: [T01, T02]

  plan T01 T04   INVALID: candidate 'T01' is already checked complete — not schedulable
```

All four are orchestrator-owned, and the reason is the FIRST thing `delegate-when` checks against:
`keep-in-the-orchestrator` retains anything writing a pipeline skill file, and every one of these
declarations names one. `delegate-when`'s own three conditions are never reached. That is the gate
behaving as written, on the only declarations this Goal has.

It is also the sharpest available contrast with what it replaced. `T01` and `T04` declare disjoint
paths with no dependency between them and both were open at the same time, so the OLD gate — where
eligibility WAS the PARALLEL verdict — would have fanned them out to workers, into files no worker
may touch. The new gate refuses them for a reason that has nothing to do with whether they could
have run at once. The `INVALID` above is a second, smaller demonstration: the checker refuses to
schedule closed rows at all, so the plan verdict cannot be reused as a delegation answer even by
accident.

**What this pass did NOT establish, stated plainly.**

- No task was delegated to a worker, so `WorktreeCreate` binding, bootstrap, base identity, branch
  naming and retention are still unrun end to end. Recorded at P4 as a limitation of this Goal and
  carried out of it as F-01, not discovered here.
- The motivating measurement — orchestrator context saved — cannot be taken from a Goal that
  delegates nothing. The first Goal that actually fans out is the measurement, and «Success
  criteria» above says what would falsify this one.

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)

- [x] M1 E2E: the delegation decision procedure exercised against crafted declarations and the checker's real output, with the schema check and CLAUDE.md agreeing on what the gate keys on
- [x] GOAL E2E: one full end-to-end pass of the whole Goal, captured
