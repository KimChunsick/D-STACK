# 02-findings-attribution

## Intent / Why

T01 decoupled delegation from parallelism, so implementation transcripts can live in workers. That
leaves one question unanswered: when the adversarial review then raises a finding against work a
worker did, who fixes it? Routing it back to the worker keeps the fix's context out of the
orchestrator, which is the whole point. Routing it to the orchestrator is sometimes the only
correct answer, and T01 already states the precedence rule that says so. This task writes the
attribution itself, and the constraint that makes returning to a worker possible at all.

## Deployment context

Same envelope as T01: `claude/skills/full-cycle/SKILL.md` is an instruction document read by the
orchestrating model, not executed, with a blast radius of every future Goal. No test can catch a
misworded rule here.

The rule leans on a platform behaviour Phase 3 established from primary sources: a RESUMED subagent
retains its full prior conversation including tool calls, and subagent transcripts are unaffected
by main-conversation compaction, persisting within the same session. It also leans on the matching
limitation: the resume-by-name/ID scope RESETS on `/clear`, which this pipeline's own
milestone-boundary handoff instructs.

**Provenance of the supplied diff.** Rounds run in SERIAL mode against `git diff HEAD`, and no
unit of this Goal is committed, so the diff carries every uncommitted change in the Goal rather
than only this task's. Authorship, as neutral metadata: the `P9-review` findings-attribution block
and the `/clear` ordering paragraph in the milestone-boundary handoff were authored by this task;
the `worker-fanout` and `worktree-lifecycle` hunks by T01; `check-parallel.sh` by T04.
`codex-review/SKILL.md` carries no change from any of them.

Three drafts of this paragraph were themselves findings, across three rounds. The first asserted
sections were untouched, which the diff contradicted. The second added an imperative aimed at the
reviewer plus a claim that other units' reviews were closed. The third still carried scope and
timing commentary. Each iteration is recorded because the failure mode recurs.

Open follow-up at Goal level: commit each unit as it closes so the next unit's serial diff is
scoped, or review in `committed` mode against a recorded base. Committing while a round still reads
`git diff HEAD` empties that round's diff.

## Design consult

RUN, jointly with T01 — one `codex exec` design review covered both changes because they redraw
the same boundary and splitting them would have hidden the contradiction between them. Capture:
`.dstack/runs/<session>/design-owd/`. Verdict: reject.

What it found that belongs to THIS task:

- **Pre-fix attribution turns a prediction into write authorization.** Attribution is decided from
  a finding's TEXT, before the fix is written, but a fix routinely needs a file the finding never
  named. Its example: a finding naming `handlers/order.*` requires a change to `schemas/order.*`
  once the worker traces the invalid value to its source. Editing first and widening the
  declaration afterwards bypasses ownership and review freezes.
- **"Return it to that worker" has no defined result when scope expansion changes attribution.**
  A single-task finding becomes cross-task halfway through, and then both rules apply at once.
  Attribution has to be a revocable assignment with explicit states, not a one-time choice.
- **Resumption must be an optimization, not a correctness dependency.** Agent loss, eviction, ID
  loss or a failed resume leaves no reconstruction path if the decisions live only in the agent's
  conversation.

## What was done (what / why)

**P9 gained a findings-attribution rule.** The orchestrator always OWNS the review — bundle,
round record, judgment, seal. What is delegable is the FIX. A finding whose fix is contained in
one task's declaration goes back to that task's worker, resumed rather than respawned, because a
resumed subagent keeps its prior conversation including tool calls and its transcript survives the
orchestrator's own compaction; respawning discards exactly the context that makes the fix cheap. A
finding crossing declarations, or touching `docs/` or a pipeline skill file, is the orchestrator's
— which is the same rule T01 states from the other end, and saying it in both places is what stops
the two from claiming the same finding.

**Attribution became a revocable assignment with explicit states.** The design consult's objection
was that attribution is decided from a finding's TEXT before the fix exists, so it is a prediction
being used as write authorization — and a fix routinely needs a file the finding never named.
States are now `assigned` → (`expansion-requested` → `reassigned`)? → `verified` → `closed`.

**The declaration became a write capability rather than a hint.** A worker may read repository
material beyond its declaration but never a secret — the deny list outranks that permission
unconditionally, because a read is not harmless when the bytes land in a transcript that persists
for 30 days. It STOPS before its first out-of-scope write, emitting a scope-expansion request. The orchestrator
checks ownership overlap, dependency state, forbidden trees and open review freezes, then either
versions the declaration or recalls the fix entirely. An unapproved out-of-scope write taints the
worktree, and narrowing the final commit does not launder it — `scope` reads every commit in the
range, which is what T04 made true.

**Round 001 tightened four things.** Declaration containment alone does not prove the original
worker owns the CURRENT code — a merge resolution or a post-merge edit replaces it, and `reopen`
already says both reopen a review, so routing now also requires the worker's branch head to still
equal the reviewed commit; integration-authored changes belong to whoever authored them. The taint
rule stopped over-claiming: an unapproved write that reaches a COMMIT is caught, while a tracked
file edited and restored before any commit, an ignored file, or a database is not, because
`honest-scope` already says there is no sandbox and no write audit — for those the stop-rule is
self-reported policy, with an explicit recovery (discard the worktree, re-create from the recorded
base, re-run). `WorktreeRemove` stopped being described as owning retention: it fires at subagent
teardown and cannot block, so the orchestrator removes the worktree explicitly after closure and
the hook is notification or archiving only. And the state list gained the unhappy outcomes the
prose already implied — `tainted`, `resume-failed` and `verification-failed` all route to
`recalled`.

**Round 002 tightened five more.** The state notation had made expansion mandatory, leaving a
clean in-scope fix with no path to `verified` at all — the ordinary path is now
`assigned` → `verified` → `closed`. "A worker may read anywhere" was an unqualified secret-read
capability written into a change that discusses credentials on the next page: reads are still wider
than writes, but the secret deny list outranks them unconditionally, because a read is not harmless
when the bytes land in a transcript that persists for 30 days. Taint recovery had conflated
repository state with external state — recreating a worktree does nothing to a mutated database, so
external side effects are now a separate disposition that blocks sealing until cleaned or
reprovisioned. Resource isolation was still sitting in `requires`, gating DELEGATION on a
contention property, which is precisely the coupling T01 removed; it moved to `parallel-when`. And
`WorktreeCreate` receives only a name, so the orchestrator now writes a durable intent record under
`.dstack/` keyed by that name (base SHA, branch, fixture list) which the hook reads, verifies, and
fails closed on by emitting no path.

**Resumption is an optimization, never a correctness dependency.** Agents can be lost, evicted, or
fail to resume. Everything a successor needs lives in the unit's `task.md` and `.dstack/`, never
only in an agent's conversation. A worker that cannot be resumed sends its fix to the orchestrator
— a cost, not a failure.

**The `/clear` conflict is recorded where the conflict is.** The milestone-boundary handoff
recommends `/clear`; doing it mid-loop destroys the warm workers this task's rule depends on. The
handoff now happens after the unit closes, which is also when its worktrees may be cleaned.

## Files changed (where / why)

- `claude/skills/full-cycle/SKILL.md` — P9-review conduct gained the attribution rule, the
  expansion protocol and the resumption caveat; the milestone-boundary handoff paragraph gained
  the ordering constraint. No other section is touched, and neither is `codex-review/SKILL.md`,
  the checker, or T01's `delegate-when`.

## E2E verification

Two properties of this rule are mechanically checkable, and both were checked on 2026-07-27.

```
=== every attribution state reaches `closed`? ===
  assigned/expansion-requested/reassigned/tainted/resume-failed/
  verification-failed/recalled/verified/closed  -> closed: True (all nine)

=== routing predicate is total: one outcome per finding shape ===
  inside one declaration, worker head == reviewed head, committed mode -> worker (resume)
  inside one declaration, worker head != reviewed head                 -> orchestrator
  inside one declaration, SERIAL mode (no reviewed commit exists)      -> orchestrator (fail closed)
  crosses two declarations                                             -> orchestrator
  touches docs/ or a pipeline skill                                    -> orchestrator
  worker cannot be resumed                                             -> orchestrator (recalled)
  authored during integration                                          -> orchestrator
```

Reachability is the check Round 002 and Round 003 both failed on — first because expansion sat on
the only path to `verified`, then because `recalled` had no exit. Running it makes that class of
defect visible instead of arguable. Totality is the other half: seven finding shapes, each with
exactly one destination, and every ambiguity resolving to the orchestrator.

The third property, that `/clear` aborts non-backgrounded workers, is verified in «Direct
verification» from the installed client's own `clearConversation`.

**Not covered, stated plainly.** No worker was spawned and no finding was actually routed. The
rule's behaviour under a real delegation is F-01 in T01's ledger, and the first real fan-out is what
closes it.

## Direct verification (repo policy: no TDD, no new tests)

**What this section can claim.** Like T01, this is a routing rule in an instruction document with
no runtime. What IS verifiable is the platform behaviour the rule leans on, and it was verified in
the installed client 2.1.220 rather than taken from the research citation.

**`/clear` does more than reset a lookup scope — it kills the workers.** Reading
`clearConversation` in the installed bundle, the reset object contains:

```js
sendMessagePins: {},
agentNameRegistry: PPo(I.agentNameRegistry, D),
```

`sendMessagePins` is emptied outright. `D` is the surviving task map, and the loop that builds it
drops every task that is not backgrounded, calling `abortController?.abort()` on the running ones
first; `agentNameRegistry` is then rebuilt from only those survivors. `CLAUDE_CODE_SESSION_ID` is
rotated in the same function, which is also why `.dstack` records are orphaned by a clear — a
behaviour this pipeline already documents from the other direction.

So the constraint is stronger than "the resume scope resets" — a mid-loop `/clear` ABORTS the
non-backgrounded workers. Round 001 was right that the first wording over-claimed: backgrounded
tasks DO survive the sweep, and the registry is rebuilt from them. The accurate claim is about the
ordinary foreground workers a fix would be routed to, and that is what the rule now says.

**Consistency between the two ends of the precedence rule.** T01's `keep-in-the-orchestrator`
names review-fix rounds with an explicit exception pointing at P9 attribution; P9 attribution
names the crossing/`docs/`/pipeline cases as the orchestrator's. Both directions are present, so a
finding whose fix sits inside one declaration has exactly one route.

`bash claude/skills/full-cycle/tests/skill-schema.test.sh` → `== all checks passed`.
Every fenced YAML block parses; the plain-scalar sweep reports clean.

## Gate status

- [x] Verification: document invariants confirmed by direct run (repo policy: no TDD, no new tests)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
