# Maintainer response — Round 004

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted. What changed, by finding:

**[high] Cleanup deleted through a prefix glob and an unchecked session path.** Step 4 now
refuses when the session capture directory is a symlink and deletes EXACT enumerated labels. The
glob was worse than it looked: `<goal>-api-r*` also matches a sibling unit's
`<goal>-api-refactor-r004`, so it could delete another OPEN round's evidence, not merely its own.
The `cmd_run_dir` half of the same class — `mkdir -p` following `runs/<sid>` through a symlink —
is fixed in M1's Round 3, since one class gets one fix even when it straddles two milestones.

**[medium] The launch was still split across two fences.** Step 1 no longer carries a runnable
snippet at all; it decides the allowlist in prose. Step 2 is ONE fence from `run-dir` through
assembly, the skip guard, and the launch. A fence that consumes `$RD` and `$IN` without defining
them is not a shorter recipe, it is a broken one.

**[medium] `full-cycle`'s opening still said the gate blocks the turn from ending.** Rewritten to
the one-block-per-user-turn contract, with the reason spelled out: `waits.external` depends on a
turn being able to end, so an opening that says otherwise contradicts the mechanism the milestone
exists to install.

**[medium] The review-unit abstraction was half-applied.** P7-P10 are now `per: review-unit`,
`@all-tasks` became `@all-units`, and `P10-task-e2e` became `P10-unit-e2e`; the schema comment
states that `review-unit` is a PARAMETER and reads as `task` at the default granularity. P10 and
P11 landing on the same folder at milestone granularity is called out as two gates, not one. The
schema check was updated in the same change. The bundle now carries the subordinate records the
unit doc points at — the finding was right that omitting them hides contradictions, and it hid
two: GOAL.md's T06 row claimed `claude/CLAUDE.md` was deliberately left alone, and T06's own
record said "not touched", while the diff shows 15 insertions and 8 deletions. Both corrected to
what actually happened (stale section-0 text replaced, net length flat).

**[low] The schema check could not detect the regression it claimed to pin.** It asserted one
`/.claude/bin/dstack` occurrence plus free-floating ` migrate` / ` unreg` substrings, which never
had to belong to the same call. Now each verb must appear bound to an absolute invocation, plus a
negative scan for a bare `dstack <verb>` inside any runnable fence (prose backticks exempt).

**[low] Bytes reported as characters, and the wrong parser.** Re-measured: 1,850 -> 465 bytes,
1,845 -> 461 characters (75%). The old 466 was neither — it was `jq -r`'s byte count including
its trailing newline. `zsh -n` now checks `ultracode.zsh`; `bash -n` never read that file's
parser.

**[low] `<pending>` design consult.** Filled in as skipped-with-reason, consistent with all three
subordinate records, and noting why a placeholder is worse than either answer.

**Beyond the findings — a premise of this Goal turned out to be false.** GOAL.md and this skill
both assumed a backgrounded round survives the turn that starts it and re-invokes the agent on
completion. It does not: two rounds launched with `run_in_background` were killed the second the
turn ended (same timestamp, 0-byte output, no `codex` process left). Relaunched detached with
`start_new_session=True`, both survived and completed. Step 2 now launches through a detached
`run.sh` that writes an `exit` sentinel, and Step 2a arms a watch on that sentinel with a
VANISHED branch, because silence from a detached process is indistinguishable from progress. The
observation is recorded as an observation of this harness, not a documented platform guarantee.

Verified by direct run (repo policy: no TDD): the skip guard's anchored form against the blocked
bundle (4 bare-substring false positives, 0 anchored) and against a bundle with two genuine skips
(both caught); `bash claude/skills/full-cycle/tests/skill-schema.test.sh` green including the six
new assertions; the byte/character measurement above from the hook's own output; `zsh -n` on
`ultracode.zsh`; and the detached-launch behaviour observed end to end on both milestones.
