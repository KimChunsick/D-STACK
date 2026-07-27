# GOAL — a Goal runs to completion after the interview, without the maintainer typing

## Goal (the one Why)

Once the P4 interview is done, a Goal must carry itself to the final report with no human
keystroke in between. Today it stalls at every long external run (a `codex exec` review round, a
research round, CI). The maintainer comes back to a session that finished its 20-minute round
fifteen minutes ago and is sitting idle, and only a typed message makes the pipeline pick the
result up.

The cause is structural, not a missing platform capability. Measured on this machine (client
2.1.220) both wake primitives work: a `run_in_background` Bash command survives the end of an
assistant turn and its exit notification re-invokes the session with no human input, and a
persistent `Monitor` event does the same. What breaks is the *shape* of the current contract:
`codex-review` launches the round as a fully DETACHED process, which the harness cannot see, and
then relies on the model reaching Step 2a of a 600-line skill file and arming a `Monitor` by hand
with `persistent: true`. Two separate actions, the second one late in a long document and easy to
skip or to arm with its 5-minute default timeout against a 15-25 minute job. When it is skipped or
mistimed, nothing ever fires and the session waits forever.

So the fix is to make the wake structural: fuse "launch the external run" and "wake me when it
finishes" into ONE harness-tracked action, so "forgot to arm the watch" stops being a reachable
state. This is the maintainer's own standing rule applied to the pipeline itself — a deterministic
transform belongs in code that can be run and checked, not in prose the model must re-execute
correctly every round.

## Interview record (Phase 4)

- **How much human involvement stays?** → *Interview, then unattended.* The P4 deep interview
  stays exactly as it is. After it, decomposition, implementation, review, E2E and the final
  report run with no human input. The existing escalation paths remain: a genuine product or risk
  choice, and the review loop's concrete-HIGH-at-closure rule, still stop and ask in Korean.
- **What guarantees the resume?** → *Fuse launch and watch.* Not "keep the current shape and
  verify harder", and not a periodic heartbeat safety net. The launch itself becomes the thing the
  harness tracks, so its completion notification IS the resume signal and no separate watch step
  exists to forget.
- **What bounds an unattended run?** → *No ceilings; progress push notifications.* No token
  budget, no wall-clock cap, no round cap beyond the ones the review loop already carries. Instead
  the run reports outward at real branch points (milestone closed, review round sealed, blocked)
  via push notification, so the maintainer can walk away and still know where it is. Explicitly
  NOT chosen: halting at high-risk points as a separate mechanism — the pipeline's existing
  escalation rules already cover that and a second one would just add stops.

## Decisions taken mid-Goal (user, during M2)

- **P6's registration proof moves into a script (T06).** The shell fence inside `SKILL.md` took
  five review rounds and every repair introduced the next defect — a hand-listed array that was its
  own proof, a `find` derivation that compared only cardinality, a loop that returned 1 on its
  success path. Asked whether to keep patching prose or move the check into code, the maintainer
  chose code. That is also what this repo's standing rule says: if code can answer, code answers.
- **The remaining work is reviewed in a batch.** T05 and T06 are implemented, then E2E runs, then
  roughly two consolidated review rounds cover everything still open — including T02 and T03, which
  the post-seal rule reopened. Per-unit loops had reached five and six rounds each. Findings from
  the rounds already run stay in each unit's `findings.md`; the Stop hook still requires a sealed
  positive latest round per registered unit, so nothing is skipped, only consolidated.

## Research summary (Phase 3)

Artifact: `docs/autonomous-goal-loop/research/autonomous-resume.md` (22 sources, all sections
present, `codex exec` exit 0).

**Key findings.** Every production system examined — Temporal, AWS Step Functions, Restate,
Inngest — models "wait for an external job, then continue" as a first-class primitive backed by
durable state plus an explicit signal (`waitForTaskToken`, Awakeables, `step.waitForEvent`,
Signals). None of them treats "start a process and hope the agent notices" as the abstraction. The
research names the current design's exact flaw: a detached POSIX process is the WRONG CLASS of
primitive because it survives but is invisible to the harness, so it can produce no completion
event at all. Its prescribed shape is the one chosen here — one scripted action that allocates a
run id, persists state, starts the external command, and emits a single correlated completion
event, specifically to remove the late-prose-step failure where the model launches but forgets or
misconfigures the watcher.

It also surfaced platform facts worth recording: Claude Code caps consecutive Stop-hook
continuations at 8, so a Stop hook can never be the wait mechanism (the gate already assumes
this); an ordinary async hook does not wake an idle session, the documented exception being
`asyncRewake` exiting with code 2; and scheduled/`loop` tasks are session-scoped, expire after 7
days, and do not catch up missed fires.

**Strongest point against the goal.** Two, both accepted rather than dismissed:

1. *This is not "durable" in the workflow-engine sense.* Claude Code background Bash and Monitor
   tasks are documented as never restored on `--resume`/`--continue`. If the session crashes or is
   resumed, the wake path is lost even though the detached round keeps running. Real limit,
   consciously accepted: the detached round still completes and writes its sentinel, so the work
   is not lost — only the automatic pickup is, and a crashed session was going to need a human
   anyway.
2. *Removing typing is fine; removing review is not.* The evidence against unattended loops
   (error compounding, model judges mis-evaluating completion, instruction-following degradation
   in long procedural contexts) argues against unbounded autonomy, not against automatic wake-up
   between rounds that are already gated. This Goal removes only the mechanical wait. Every gate,
   the adversarial review loop, and the escalation paths stay. The research's own counter-counter
   makes exactly this distinction.

Notably the research cites instruction-following degradation under long/complex constraints as
direct support for the maintainer's observed failure mode — the late "arm the watcher" step in a
long skill file is precisely the constraint class models drop.

### Follow-up round — background-task lifetime

Artifact: `docs/autonomous-goal-loop/research/background-task-lifetime.md` (13 source entries — 12
unique URLs plus one local installed-CLI artifact; all sections present, `codex exec` exit 0). Run
during M1 to answer this Goal's open question about how
long a background task actually lives, and simultaneously to serve as T03's direct-run evidence for
the fused recipe.

**What it establishes.** No public documentation states a maximum lifetime for an interactive
background Bash task, but several documented ways to end one early do exist:
`CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` turns the mechanism off entirely; on macOS/Linux a
main-session background shell may be reaped under OS memory pressure once the session has been idle
30 minutes with no turn or subagent running; in headless `claude -p` the shell is terminated about
5 seconds after the final result. `--resume`/`--continue` restore conversation, model, permission
mode and unexpired scheduled tasks, but explicitly NOT background Bash or Monitor tasks. `/branch`
does preserve in-flight background tasks, which is same-process continuity, not durability.

**Strongest point against.** The exact property this Goal is built on — a completion notification
re-invoking the session with no human input — is NOT publicly documented. It is present in the
installed client's own internal text (2.1.220) and it is what every run in this Goal actually did,
but that makes it observed installed-client behaviour, not a contractual guarantee. The honest
framing is: verified on this client, revalidate on upgrade. The research also names the durable
alternatives (Routines, scheduled tasks, GitHub Actions) for anyone who needs a real job runner.

**Still unverified.** Whether auto-compaction preserves a running background task, and whether a
pending completion notification can be lost across it. One first-hand data point was collected
during this Goal: a compaction occurred while `autonomous-goal-loop-lifetime` was in flight, and
both the task and its notification survived. One observation is not a guarantee.

**Unverified / open.** No official documentation was found for a maximum lifetime of interactive
background Bash or Monitor tasks across ordinary turn boundaries, nor for what context compaction
does to them. The first is now closed by local measurement — a 25-minute background command
survived four intervening turn boundaries and its exit notification re-invoked the session — and
T01's five review rounds each did the same for 5–11 minutes. The compaction question stays an
accepted unknown.

**Where the design departed from the research.** The research's prescribed shape had the launcher
*detach* the external command. T01's design consult, and then its review loop, showed that
detaching is the wrong half of the prescription for this harness: a detached process is invisible,
so it can never notify, and Claude Code restores no background task on `--resume`, so there is no
recovery path for the orphan it leaves. The launcher runs the command as a direct child instead.
What survives from the research is the part that mattered — one scripted action, not a late prose
step the model must remember.

## Milestones & tasks (Phase 5)

Review granularity: **per task** (the default), per the recorded preference that wide review units
lengthen the codex loop.

### M1 — One deterministic launch-and-wait for every long external run

- [x] **T01** dstack-run-subcommand — add `dstack run <label> [--stdin <file>] -- <cmd...>`: allocate or adopt the run label, run the command as a direct child in its own process group, capture stdout/stderr/exit, tear that group down on any catchable exit, and publish a terminal record only once the group is confirmed gone. Run it under a harness-tracked background call and its exit notification becomes the resume signal. Document the new subcommand where the store is described. deps: []; files: [claude/bin/dstack, AGENTS.md]
- [x] **T02** codex-review-fused-launch — replace Step 2's detached-launcher fence and delete Step 2a's hand-armed `Monitor` contract, so a round is one background `dstack run` call. Correct the stale claim that background commands are killed at turn end, and record the honest residual (no restore across `--resume`). deps: [T01]; files: [claude/skills/codex-review/SKILL.md]
- [x] **T03** codex-research-fused-launch — same fusion for the research round, which today has no backgrounding recipe at all and so is run either in the foreground or with a hand-rolled launcher. deps: [T01]; files: [claude/skills/codex-research/SKILL.md]

### M2 — The unattended contract, stated once and stated where it is read

- [x] **T04** full-cycle-unattended-contract — rewrite `waits.external` onto the fused launcher, and add the unattended-run rule: after P4 the pipeline runs to completion without asking, the existing escalation paths being the only stops, with push notifications at real branch points. deps: [T01]; files: [claude/skills/full-cycle/SKILL.md]
- [x] **T05** standing-rules-alignment — carry the same two rules into the standing instruction file so they hold outside the skill's own text. deps: [T04]; files: [claude/CLAUDE.md]
- [x] **T06** registration-check-script — move P6's registration proof out of a shell fence in prose and into a deterministic script: parse the granularity and the task identities from GOAL.md, derive the expected review-unit set, and compare it against `dstack status` including ownership. The fence in `full-cycle` shrinks to invoking it. deps: [T04]; files: [claude/skills/full-cycle/check-registration.sh]

## M1 E2E — a long external run resumes the session by itself

**Captured across this Goal's own execution, which is the only honest way to test it.** Every codex
round from T01 onward was launched as ONE background Bash call whose blocking terminal step was
`"$HOME/.claude/bin/dstack" run <label> --stdin <bundle> -- codex exec …`, with no watcher armed
anywhere and no human input between launch and the session picking the result back up.

```
run captures under .dstack/runs/<sid>/          38
  with a terminal record                        33
    exit=0                                      31
    exit=143                                    2   (both harness kills of the wrapper)
  with no terminal record                        5   none has a .launch claim -> never launched
                                                     pgrep -f 'codex exec' -> nothing alive
round durations                                 3-25 minutes; several far exceed the foreground
                                                Bash cap, so none of them could have run inline
```

The two `exit=143` captures are the failure path exercised in production rather than simulated:
`t01-r3` and `t02-r4a`/`t04-r2` were killed mid-run by the harness, the process group was torn down,
`143` was published, no orphan survived, and each round was discarded and relaunched under the next
label. One completed round (`t03-r4a`) was reported 143 by its wrapper and kept, because
`<run-dir>/exit` said 0 — which is the rule this milestone installed and the reason it exists.

The 5 record-less captures were checked against the rule this Goal added for exactly that state:
none was ever launched, and nothing is alive. Abandoned bundle allocations, not orphans. That
distinction is the whole point of the rule — the alternative is relaunching over a live paid run.

**What this does NOT prove.** Completion re-invocation is observed behaviour of the installed client
(2.1.220), not a documented public guarantee; `--resume`/`--continue` restore no background task;
and `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` removes the mechanism outright. All three are written
into `scheduling.waits.external-residuals` and into `claude/CLAUDE.md`, because a contract that
hides its own preconditions is how this failure came back the first time.

## M2 E2E — the written contract matches what the code does

Checked by running the contract's own recipes and its checker against the tree that ships, not by
reading them.

```
tests/secret-guard.sh                            PASS
full-cycle/tests/skill-schema.test.sh            PASS   (the schema broke 4x while M2 was written;
                                                         each break was one bare ': ' in a scalar)
full-cycle/tests/check-parallel.test.sh          PASS
9 fenced bash blocks across the three skills     /bin/bash -n and /bin/zsh -n, 0 failures
check-registration.sh --depth / --list / full    3 / 6 documents / confirmed rc=0
the P6 fence, run end to end against this Goal   registers 6, then confirms; rc=0
  with a hostile <goal> substituted              refused before anything ran
assemble-review.sh                               all 6 units assemble; the documented
                                                 REVIEW_FULL_ROUND_IDS="1 3" now returns rc=0
./install.sh --dry-run                           19 entries up to date
```

**No step the fused launcher made obsolete survives in the instruction files.** The `run.sh`
heredoc, the `python3 start_new_session` detach and the hand-armed `Monitor` are gone from
`codex-review`; `codex-research` had no backgrounding recipe at all and now has the same one; and
`full-cycle`'s `waits.external` names the mechanism instead of describing a wait with no verb.

**And the deregistration path works, which is what closes a Goal.** All five units were deregistered
after their gates were ticked, and the checker — which refuses a closed unit that is still
registered — confirms the end state: `6 scaffolded units (0 open and registered to this session) +
GOAL.md`.

## GOAL E2E — one full pass, and exactly what "no typed input" turned out to mean

This Goal is its own end-to-end test, and the result has to be stated precisely rather than
flattered.

**The mechanical wait is gone, completely.** Across ~33 completed external rounds — codex reviews, a
design consult, three research rounds — the number of times a human typed to make the pipeline pick
up a finished run is **zero**. Each round was one background call, no watcher, and the completion
notification re-entered the session by itself. That is the failure this Goal existed to remove, and
it is removed.

**The maintainer did type twice, and both were `autonomy.stops` escalations, not stalls.** Honesty
matters more than a clean number here:

1. **P6's registration proof — keep patching prose, or move it into code?** Asked after the fifth
   consecutive round in which a repair introduced the next defect. A genuine engineering-direction
   choice with a real cost either way, which is exactly what `stops` reserves for a person.
2. **How to review the remaining work — per-unit loops, or one batch?** Per-unit loops had reached
   five and six rounds. How much review to buy is the maintainer's call, not the pipeline's.

Neither was a question the pipeline could have answered by taking "the reading a careful colleague
would". Neither was a stall waiting on a mechanism. And neither could have been asked at P4, because
both arose from evidence that did not exist until the work was underway — which is the one thing the
unattended rule asks you not to do, and it was not done.

**What ran with no input at all:** decomposition into 6 tasks across 2 milestones, all 6
implementations, 38 run allocations and 33 completed rounds, 5 review loops totalling 24 rounds
(including 2 post-seal reopenings), every findings ledger and response, the milestone E2Es, and this
report. Two harness kills were absorbed and relaunched without a human noticing.

**Cost, stated because it is real.** 24 review rounds is a lot, and the ledgers say why: the review
kept finding defects in the previous round's FIX, in the same few lines — the skip gate took four
matchers, the P6 fence took five prose repairs before moving into code, and the registration script
still took two rounds and seven highs after that. The pipeline's own §5 names this: prefer splitting
the unit over extending the loop. It held here only because each round's findings were real.

**Not verified by this pass.** A Goal whose interview and final report happen in the same session,
in a repository where the work is NOT the pipeline itself. Every round here reviewed the very files
that govern reviewing, which is a harsher test in some ways and a narrower one in others.

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: a real long external run launched through `dstack run` under a background call resumes the session by itself, with the round's output intact
- [x] M2 E2E: the written contract matches what the code actually does — the instruction files carry no step the fused launcher made obsolete
- [x] GOAL E2E: one full pass — a Goal that includes a real codex round runs from interview to final report with no typed input in between
