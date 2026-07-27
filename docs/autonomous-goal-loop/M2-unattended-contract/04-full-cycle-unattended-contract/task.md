# 04-full-cycle-unattended-contract

## Intent / Why

The pipeline file describes the wait, but not the mechanism, and says nothing about who is expected
to be sitting there. Two gaps, one symptom.

`waits.external` says "background the run, END THE TURN, and act on the completion notification"
without naming what backgrounds it. That is exactly the shape that failed: with no named mechanism,
each round improvised one, and the improvisations that detached the process made it invisible to
the harness, so no notification could ever arrive and the pipeline sat idle until the maintainer
typed. T01 built `dstack run` and T02/T03 put the two skills on it; this file still points at
nothing.

Separately, nothing in the pipeline states that it is supposed to RUN UNATTENDED. The Goal's
interview settled that: conversation through P4, then no human input until the final report, with
the existing escalation paths as the only stops. Without that written down, every round is free to
end its turn with a question, which is the same idle state by a different route.

## Deployment context

Instruction document read by the orchestrating model, symlinked into `~/.claude/skills/` by
`install.sh`. It is the structured authority for the pipeline: a YAML schema the model reads for
control flow, plus prose for the judgment the schema cannot carry. Repo policy: no TDD, no new test
files; verification is a direct run whose output is recorded here.

## Design consult

Skipped — no trigger. This states a contract that T01 already built and that T02/T03 already use.

## What was done (what / why)

- **`waits.external` names the mechanism.** One background Bash call whose BLOCKING TERMINAL STEP is
  `"$HOME/.claude/bin/dstack" run <label> --stdin <file> -- <cmd…>`. Setup before it is required and
  expected; what is forbidden is work AFTER it whose result you need. The completion notification IS
  the resume signal; there is no watcher to arm.
- **`waits.external-residuals`,** from this Goal's follow-up research round and from measurement: no
  background task is restored on `--resume`/`--continue`; `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1`
  removes the mechanism entirely; a main-session background shell may be reaped under OS memory
  pressure after 30 minutes idle; completion re-invocation is observed installed-client behaviour
  (2.1.220), not a documented public guarantee. `<run-dir>/exit` is the run's status and the
  notification is a hint, because a signalled wrapper reports 143 over a completed run. Scratch
  cleanup is conditional on that file existing, since `dstack run` publishes it only after the
  child's process group is confirmed gone.
- **A new `autonomy` block** — `rule`, `internal-recoveries`, `stops`, `bounds`, `notify`. After P4
  the pipeline runs to completion without asking. What goes wrong splits into `internal-recoveries`
  (a defined next move, no human) and `stops` (a person is required), a `stops` entry always wins,
  and `notify` names `PushNotification` and calls it best effort.
- **P6 registration is fail-closed, and its expected set is DERIVED.** A failed `reg` used to print
  a warning and continue, which was survivable only while a human read the transcript. Then a
  hand-listed array turned out to be its own proof — review passed `DOCS=(GOAL u1 u1)` through it.
  The set now comes from `find` at the depth the granularity fixes, cross-checked against GOAL.md's
  task rows, and each document must appear in `status` as an EXACT line saying `(this session)`.
- **P6 names no failure outcomes of its own.** Routing lives in `autonomy` alone. Prose here that
  also routed failures is what kept `reclaim` alive as an automatic answer to foreign ownership
  after the stop table had already forbidden it.

## Files changed (where / why)

- `claude/skills/full-cycle/SKILL.md` — `scheduling.waits.external` and a new `external-residuals`,
  the new `scheduling.autonomy` block, the P6 registration fence and its surrounding prose, and the
  prose section explaining the unattended rule.

## E2E verification

Repo policy: no TDD. Direct runs, with what each establishes.

**The P6 fence, against six scenarios plus the scaffolding cross-check.** This is the part review
broke four times, so it is exercised rather than described:

```
happy path, 3 task rows, 3 scaffolded units          confirmed 4 documents, rc=0
a unit is foreign-owned                              BLOCKED, rc=1
`reg` refuses one unit                               stops on the refusal, rc=3
milestone granularity with no milestone task.md      BLOCKED, rc=1
granularity neither task nor milestone               BLOCKED, rc=1
2 scaffolded units vs 3 task rows in GOAL.md         BLOCKED, rc=1
4 scaffolded units vs 3 task rows in GOAL.md         BLOCKED, rc=1
```

The last two are what closes the hole `find` alone leaves: it proves what was SCAFFOLDED, which is
not what P5 DECOMPOSED.

**The schema parses, and this change broke it four times.** The file calls itself the structured
authority, so a `scheduling:` block that no longer loads is a real defect. Every break was one bare
`: ` inside a plain multi-line scalar, and the last was a colon at end of line, which my own grep
filter missed and only the parser caught:

```
block 0  OK  [pipeline, version, skip, phases]
block 1  OK  [scheduling]  autonomy=[rule, internal-recoveries, stops, bounds, notify]
block 2  OK  [hook-contract]
```

**Pinned checks, and the recipes this contract governs:**

```
bash tests/secret-guard.sh                                ✓ PASS
claude/skills/full-cycle/tests/check-parallel.test.sh       PASS
claude/skills/full-cycle/tests/skill-schema.test.sh         PASS
  ^ FAILS with any of the four YAML breaks reintroduced — verified by reintroducing one
all 8 fenced bash blocks across full-cycle + both codex skills, placeholders substituted
  /bin/bash -n  OK      /bin/zsh -n  OK
conditional scratch trap: terminal record present → removed; absent → KEPT
./install.sh --dry-run                                    = up to date: .claude/skills/full-cycle
```

**What this does NOT verify.** All of the above is about the document loading, its new keys being
well-formed, and its fences behaving. Whether the orchestrator actually *behaves* unattended is not
something a parse proves. The evidence for that is this Goal itself — every round since T01 launched
as one background `dstack run` with no watcher and no human input — and the M2 E2E records it.

### Round 006 (batch pass 1) — what changed

Two highs, three mediums, all agreed and fixed; `findings.md` F023-F027, `response-006.md`.

- **The P6 fence registered task-depth paths whatever the granularity.** The granularity table sits
  four lines above it and warns that registering the wrong level silently un-gates the milestone's
  own document; the fence then iterated a literal `<Mn>/<NN-task>/task.md`. It now calls
  `check-registration.sh --depth`, so the level comes from the same GOAL.md parse the check uses and
  the two cannot disagree — and a Goal with a missing or ambiguous granularity fails at the first
  line rather than after the wrong documents are registered.
- **The checker did not parse the same declaration source as the scheduler.** Fixed in T06 and
  demonstrated on a fixture: with a fenced decomposition example in an earlier section, the old
  parser read that block's granularity and task ids and none of the real ones.
- **P9 had a second cap-closure rule.** It escalated to the user for all "blockers" — high AND
  medium — while §4 and `autonomy.stops` close concrete mediums without one. P9 now defers to §4 and
  states no rule of its own.
- `RUNDIR="$RD"` before `RD` exists: real, fixed in `codex-review`'s Step 2 where the recipe lives.
- The exit-2 fail-loud claim: fixed in T06 by checking every transformation's status.

Also removed here, not from a finding: `autonomy.stops` carved out an autonomous `reclaim` for a
"provably orphaned" handoff whose owner is this session. `reg` returns 0 for a document this session
already owns, so that state is never reached, and the rest is unprovable without a liveness signal.
The carve-out is gone, which also makes `CLAUDE.md`'s stricter summary correct rather than merely
stricter.

Re-verified after these edits:

```
claude/skills/full-cycle/tests/skill-schema.test.sh    PASS  (blocks 0,1,2 all parse)
claude/skills/full-cycle/tests/check-parallel.test.sh  PASS
bash tests/secret-guard.sh                             ✓ PASS
P6 fence, placeholders substituted                     /bin/bash -n OK   /bin/zsh -n OK
check-registration.sh --depth docs/autonomous-goal-loop -> 3
check-registration.sh docs/autonomous-goal-loop        -> confirmed, rc=0
```

**The P6 fence scenarios recorded above are superseded.** They exercised the shell fence this task
removed; the check now lives in `check-registration.sh` and its battery is in T06's `task.md`. The
line kept here is the one about this file: the schema still parses, and it broke four times while
this task was written.

## Gate status
- [x] Verification: behavior confirmed by direct run (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified

### Round 007 (batch pass 2) — the closing round

One high and two mediums, plus two findings deferred here from unit 05; `findings.md` F028-F033,
`response-007.md`.

- **Checker failures had no autonomy transition** — a wrong-depth or closed registration halts P6
  under `set -e` matching neither `internal-recoveries` nor `stops`, which unattended is a silent
  stall. Three transitions added, including `unreg` for a same-session record that must not exist.
- **`find -exec` masks the failure of the command it runs** (measured: `find . -exec false {} \;`
  exits 0), so one failed `reg` was invisible while later documents kept being claimed. That was my
  own round-006 fix trading a wrong level for a swallowed status.
- **`<goal>` reached shell source unvalidated** — `safe; printf INJECTED` executed under both shells.
- Registering before classifying is what made "safe to re-run" false; `--list` fixes both.
- From unit 05: the `/clear` handoff prescribed `reclaim` against the stop table, and
  `waits.external` still said "the call does not return" of a backgrounded Bash call. Both fixed
  once this bundle closed and the freeze lifted.

Sealed `Consensus: resolved` under §4 cap closure.

## P10 closure evidence (batch pass 2)

Recorded at the close of the review loop, against the tree as it ships. Repo policy: no TDD, so
these are direct runs.

```
tests/secret-guard.sh                            PASS
full-cycle/tests/skill-schema.test.sh            PASS
full-cycle/tests/check-parallel.test.sh          PASS
9 fenced bash blocks across the three skills     /bin/bash -n and /bin/zsh -n, 0 failures
check-registration.sh --depth                    3
check-registration.sh --list                     6 documents
check-registration.sh (full)                     confirmed, rc=0
assemble-review.sh                               all 6 units assemble
./install.sh --dry-run                           19 entries up to date
```

**The whole-Goal behavioural evidence, which is what this Goal was actually about:** 38 run captures
under `.dstack/runs/<sid>/`, 33 with a terminal record — 31 `exit=0` and 2 `exit=143` (both harness
kills, both torn down with no orphan). Every one of those rounds was launched as ONE background Bash
call whose blocking terminal step was `dstack run`, with no watcher armed anywhere, and every one
woke this session on completion with no human input.

The 5 captures with no terminal record were exercised against the rule this Goal added for exactly
that state: none has a `.launch` claim, so none was ever launched, and `pgrep -f 'codex exec'`
returns nothing. Abandoned bundle allocations, not orphans — which is the distinction the rule
exists to force you to make instead of relaunching over a live run.

No worktree fan-out for this unit, so there is nothing to merge; the change is in the working tree
of `agent/harden-codex-review-workflow` and committing is the maintainer's call.
