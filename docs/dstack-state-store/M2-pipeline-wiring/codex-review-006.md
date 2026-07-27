# Codex adversarial review — Round 006

## Review scope
Re-review (Rounds 1-5 rejected; all findings accepted and fixed). This was the last round of the original six-round budget.

## GPT findings
[severity:medium][technical correctness] Step 2a omits Monitor persistence, so the watch expires before the detached review normally finishes.
Sites: primary: `claude/skills/codex-review/SKILL.md:128`; confirmed: Step 2a at line 213.
Evidence: Reviews take 15–25 minutes, but Monitor defaults to 300,000 ms unless `persistent` or a longer timeout is supplied. [Official Agent SDK reference](https://code.claude.com/docs/en/agent-sdk/python)
Verification: The default watch ends after five minutes, 10–20 minutes before the expected sentinel, so no completion event reaches Claude.
Blocking: Yes — genuinely blocking; it breaks the milestone’s long-run handoff.
Suggested direction: Invoke Monitor with `persistent: true` or an explicit timeout exceeding the documented maximum duration.

[severity:medium][technical correctness] A nonzero reviewer exit is displayed but never enforced, allowing a failed invocation’s contract-shaped output to be sealed.
Sites: primary: `claude/skills/codex-review/SKILL.md:200`; confirmed: Steps 2a–3 at lines 226 and 303–350.
Evidence: `run.sh` publishes the exit code and the watcher echoes it, but Step 2b validates only output shape before Step 3 records the round.
Verification: A command that prints all required markers and exits 1 produces `DONE exit=1`; that output still passes Step 2b and can receive positive `Consensus:`.
Blocking: Yes — genuinely blocking; the adversarial-review gate can accept a failed run.
Suggested direction: Require a numeric sentinel value of exactly zero before inspecting, recording, or sealing output.

[severity:medium][technical correctness] The completion and recording recipes still consume variables that are unavailable in their runnable fences.
Sites: primary: `claude/skills/codex-review/SKILL.md:240`; confirmed: Step 3 at lines 325–330.
Evidence: `OUT` is defined only in a separate earlier fence, while `TASK_DIR` is never assigned anywhere in the current skill.
Verification: Fresh execution cannot open the empty `OUT`; the Step-3 allocator deterministically produces `/codex-review-001.md`.
Blocking: Yes — genuinely blocking; normal completion cannot reliably triage or record the round.
Suggested direction: Make each procedure one self-contained fence that reconstructs `OUT` and defines the review-unit directory before use.

[severity:medium][security] `rm-run` prevents repository escape but still permits a session-directory swap to redirect deletion into another session.
Sites: primary: `claude/bin/dstack:681`; confirmed: `claude/skills/codex-review/SKILL.md:115`.
Evidence: After `cd -P`, the guard accepts any resolved directory under `$RUNS/*`, rather than requiring the current session’s exact directory.
Verification: Swapping `runs/session_a` to a symlink targeting `runs/session_b` after validation yields an accepted path; matching exact labels in session B are then recursively removed.
Blocking: Yes — genuinely blocking; this is a concrete cross-session evidence-deletion race.
Suggested direction: After entering the directory, require physical-path equality with the expected `$RUNS/$SID`, not prefix containment.

[severity:medium][technical correctness] The Round-4 review-unit conversion remains inconsistent across authoritative procedural instructions.
Sites: primary: `claude/CLAUDE.md:6`; confirmed: `claude/skills/full-cycle/SKILL.md:3`, lines 139–142 and 386–390; `claude/skills/codex-review/SKILL.md:320`.
Evidence: The schema runs P7–P10 per review unit and this Goal selects milestone granularity, while the summaries still prescribe task folders, per-task reviews, overlap, and E2E.
Verification: Following the always-loaded summary creates or reviews subordinate task units instead of the milestone-root unit, repeating the gate-placement failure accepted in Round 4.
Blocking: Yes — genuinely blocking; two same-authority instruction paths prescribe incompatible pipeline ownership.
Suggested direction: Propagate `review-unit` through every review/E2E summary and procedure, reserving “task” for task execution and worker fan-out.

[severity:low][software structure] The durable M2 record omits the destructive CLI API added to satisfy Round 5.
Evidence: M2’s design-consult and files-changed sections claim no new API or sanitization path and omit `claude/bin/dstack`, while Round 5 added `rm-run`.
Verification: The supplied allowlisted changes omit that implementation; it was visible only through the actual checkout.
Blocking: No — documentation and review-coverage drift, but the implementation was inspected in this round.

[severity:low][DX] The claim that `claude/CLAUDE.md` stayed net-flat is false.
Evidence: `GOAL.md` and the T06 record say it was not grown.
Verification: The file grew from 8,670 to 9,165 bytes, from 8,019 to 8,510 characters, and by seven lines.
Blocking: No — the injection still shrank overall, but the recorded accounting is inaccurate.

[severity:low][technical correctness] The schema check leaks its first temporary directory.
Evidence: `skill-schema.test.sh:68` installs an EXIT trap for `ftmp`; line 102 replaces that trap with one cleaning only `tmp`.
Verification: With Ruby available, normal shell trap semantics leave `ftmp` behind after every run.
Blocking: No — bounded temporary-file leakage only.

[severity:low][technical correctness] Runner PID publication is non-atomic while absence or emptiness is interpreted as process death.
Evidence: `codex-review/SKILL.md:193` redirects directly to `pid`; lines 224–226 immediately classify an unreadable or empty PID as `VANISHED`.
Verification: `Popen` can return before the child completes its first write, allowing the newly armed monitor to report a live review as vanished.
Blocking: No — it can waste a review attempt but does not corrupt completed output.

Check execution: syntax checks passed; the pinned checks were attempted but could not complete because the enforced read-only sandbox denied their required temporary directories and probe files. Those environmental failures are not repository findings.

Omitted-detail: 0 low

GPT verdict: reject — Five concrete medium blockers still break review completion, failure gating, capture isolation, and milestone-level pipeline ownership.

## Carried decisions — Round 006
Rounds 1-5 decisions stand. Added in Round 6:

- **A watch's timeout must exceed what it waits for.** A default that expires early is not a
  watch, and its silence is indistinguishable from patience.
- **A nonzero exit is a failed round, not a round with bad news.** Check the sentinel before
  reading a single line of output.
- **Every fence reconstructs what it needs.** A variable from another fence is an unset variable.
- **Containment is not identity.** "Under the right parent" permits the wrong sibling.
- **Renaming a schema field is not propagating it.** The always-loaded summary is the one a model
  actually follows.
- **Escalate at the budget; never downgrade to fit it.** Recorded here because it is what
  happened.

Consensus: disagreed
