# Codex adversarial review — Round 002

## Review scope
Batch pass 2 (consolidated round) | `REVIEW_MODE=serial` | bundle 12392 bytes (r1 6762). Allowlist unchanged: the review-unit folder, `claude/CLAUDE.md`.

## GPT findings

[severity:medium][technical correctness] The claimed `reclaim` reconciliation still gives one handoff state two incompatible transitions: stop for a person on every foreign owner, or autonomously reclaim after deliberate `/clear`.
Sites: `claude/skills/full-cycle/SKILL.md:489`; confirmed: `claude/skills/full-cycle/SKILL.md:722`, `claude/CLAUDE.md:61`, `docs/autonomous-goal-loop/M2-unattended-contract/05-standing-rules-alignment/task.md:58`.
Evidence: The stop table says autonomous `reclaim` has no valid path, while lines 734–735 prescribe it for records orphaned by session-ID rotation.
Verification: Following the documented handoff leaves old-owner records after `/clear`; one rule requires asking, while the other requires reclaiming, causing either an unattended stall or an unverifiable ownership takeover.
Suggested direction: Either record verifiable handoff provenance before `/clear`, or classify the handoff as an explicit user-input stop.

[severity:low][technical correctness] Round 001’s lifecycle correction is complete only in the standing file; the full-cycle authority still ambiguously says the background “call does not return,” conflating it with its blocking terminal step.
Sites: `claude/skills/full-cycle/SKILL.md:405`; confirmed: `claude/CLAUDE.md:42`, `docs/autonomous-goal-loop/M2-unattended-contract/05-standing-rules-alignment/codex-review-001.md:36`.
Evidence: The authority first defines a background Bash call using `run_in_background`, then says “the call does not return,” before separately identifying `dstack run` as the blocking operation.
Verification: The installed standing file correctly says the Bash tool call returns immediately; the authority retains the wording Round 001 identified as incorrect.

[severity:low][the real Why] The pre-skill “Honest limits” are incomplete: they omit OS reaping and untrappable-signal orphan handling, leaving a capture without `exit` lacking the liveness guard needed before relaunch.
Sites: `claude/CLAUDE.md:51`; confirmed: `claude/skills/full-cycle/SKILL.md:420`, `docs/autonomous-goal-loop/M2-unattended-contract/05-standing-rules-alignment/task.md:51`.
Evidence: The authority says memory pressure may reap the background shell and `SIGKILL`/`SIGPROF` may orphan its child, requiring a live PID/group check; the standing file lists neither residual.
Verification: A direct phrase comparison found both residuals only in the full-cycle skill; following the standing file alone can relaunch while the original child remains alive.

[severity:low][security] The task artifact attempts to constrain review with “behaviour itself is out of scope here,” an embedded scope directive capable of suppressing verification of the standing contract’s actual intent.
Sites: `docs/autonomous-goal-loop/M2-unattended-contract/05-standing-rules-alignment/task.md:62`; confirmed: `task.md:69`.
Evidence: The disclaimer and checked gate explicitly exclude behavioral review despite the task’s intent being an operational standing rule.
Verification: Treating the disclaimer as untrusted exposed the missing no-terminal-record handling above; accepting it would have hidden that defect.

Omitted-detail: 0 low

GPT verdict: reject — The unresolved `reclaim` contradiction can either stall the promised unattended handoff or authorize an unverifiable ownership takeover.

## Carried decisions
- **`reclaim` still has two transitions, and the second one is in a file this round could not
  touch.** `autonomy.stops` says there is no autonomous path; the milestone-boundary handoff prose
  says to `/clear` and then `reclaim` the records the session-id rotation orphaned. One state, two
  rules. DEFERRED under the freeze-rule — `full-cycle/SKILL.md` is inside an OPEN review bundle
  (unit 04, round 007) — and carried to unit 04's next round rather than edited mid-round. The
  resolution is the reviewer's second option: after `/clear` the operator IS present by
  construction, so the handoff reclaim is an explicit user-input confirmation, not an autonomous
  act. Nothing can verify provenance without a liveness signal, so confirmation is the honest form.
  **DEFERRAL LIFTED IN THIS SESSION, and recorded rather than quietly dropped.** The blocking bundle
  (unit 04, round 007) closed a few minutes later, which unfroze the file, so the fix landed here
  instead of waiting for another Goal: the handoff prose now lists what it intends to reclaim and
  asks, and says why presence rather than provability is what makes this case different.
- **The lifecycle wording was fixed in the standing file and not in its authority.** The skill said
  "the call does not return until the external command has finished" of a `run_in_background` Bash
  call, which round 001 had already corrected in the standing file. Fixed in `waits.external` once
  the same freeze lifted — it now names the STEP as the blocking thing and says the tool call
  returns immediately, because "the call blocks" reads as a stuck harness and invites the
  hand-rolled watcher this whole contract replaced.
- **The standing file was missing two residuals it needs precisely when the skill is not loaded.**
  A background shell may be reaped under OS memory pressure after 30 idle minutes, and `SIGKILL` or
  `SIGPROF` can orphan `dstack run`'s child — so a capture with NO terminal record is not a failed
  run you may relaunch; check for a live pid or group first, or you spend credits twice and let two
  runs write one label. Both now in both files, verified with line wrapping folded out because
  wrapping is not semantic and a naive phrase grep reported false absences.
- **A repair became the next instance of the class it repaired.** Round 001's F004 asked the gate row
  to stop overstating; the row I wrote to fix it said "behaviour itself is out of scope here", which
  is an evaluator directive embedded in reviewed data. The reviewer proved it operationally rather
  than stylistically: treating the disclaimer as untrusted is what surfaced the missing liveness
  residual above. The section now states what the commands establish and what they do not, with no
  instruction to the reader.

**Disposition.** §4 closure with the batch authorisation spent: every finding is fixed, including
the two that were deferred for part of this session under the freeze-rule and completed once their
blocking bundle closed. No concrete HIGH is open. The follow-ups this unit does not own are named in
`findings.md`.

Consensus: resolved
