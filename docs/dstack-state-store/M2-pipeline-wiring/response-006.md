# Maintainer response — Round 006

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[medium] `Monitor` was armed with its default timeout.** Its default is 5 minutes and a round
takes 15-25, so a default watch expires 10-20 minutes before the sentinel and no completion event
ever arrives — the long-run handoff this milestone exists to build silently does not happen. Step
2a now says `persistent: true` outright.

**[medium] A nonzero reviewer exit was displayed but never enforced.** A `codex exec` that dies
partway can still have written contract-shaped text, so a failed run could be recorded as a round
and receive a positive `Consensus:` — the mandatory review gate satisfied by a run that never
finished. The triage fence reads the sentinel FIRST and refuses to look at the output unless it
is exactly `0`.

**[medium] The completion and recording fences consumed variables no fence defined.** `OUT` came
from an earlier fence and `TASK_DIR` was assigned nowhere in the file, so under `set -u` the
allocator died — and without it, formatted `/codex-review-001.md`, an absolute path at the
filesystem root, forever "allocating" round 001. Both fences reconstruct what they need from the
durable path now, and the reason is written next to the line so it is not tidied away again.

**[medium] `rm-run` accepted any directory under `runs/`.** Prefix containment is not identity:
swapping `runs/<mine>` to a symlink at another session's directory redirected the delete into ITS
captures, removing a concurrent round's evidence. It now requires physical equality with
`$RUNS/$SID` after the chdir, which leaves no window.

**[medium] `review-unit` was not propagated to the summaries.** The schema said per review unit
while `claude/CLAUDE.md`, the skill description, and the review-overlap rules still said task
folders and per-task reviews — two same-authority instruction paths prescribing incompatible
ownership, and the always-loaded one was the wrong one. Propagated through all of them.

**[low] The M2 record omitted `rm-run`**, the destructive CLI API its own Round-5 finding created.
Recorded as a cross-milestone entry: the file is M1's and the implementation is reviewed there,
but a review-unit record that omits an API its findings produced hides it from the next reviewer.
**[low] "net-flat" was false.** `claude/CLAUDE.md` grew 495 bytes / 7 lines. Corrected in GOAL.md,
the T06 record, and the M2 record — as measured, not as intended.
**[low] The schema check leaked its first temp dir** — a second `trap … EXIT` REPLACES the first;
bash has no trap stack. One cleanup line at the end now.
**[low] The runner pid was published non-atomically** and an unreadable pid meant VANISHED, so a
just-launched round could be declared dead. Temp file plus `mv`, and the watch waits before its
first liveness probe.

**Review budget.** This was round 6, the last of the budget. Every round has produced concrete,
reproducible findings rather than nitpicks, so rather than downgrade anything the state was put
to the user in Korean, per the escalation rule. The user extended the budget and chose to keep
going; they also decided that from the NEXT Goal on, review units should be per task rather than
per milestone, because a wide bundle is what keeps exposing adjacent defects each round. Both
decisions are recorded in GOAL.md.

Verified by direct run (repo policy: no TDD): the schema check green including the two per-fence
assertions and both their negative controls; a temp-dir leak check before and after; `rm-run`
against a real capture, a traversal label (refused) and a missing label; the corrected
byte/character measurement taken from the hook's own output.
