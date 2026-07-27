# Maintainer response — Round 003

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Eight findings, all accepted; one was already fixed before the round was read.

**[medium] The Round-2 recipe was not executable.** Confirmed and embarrassing: `DS_run_dir` and
`$AS` were never defined, `IN` was assigned only in the block I had just contradicted, and the
triage step referenced an `OUT` that dies with the launching shell. Steps 1 and 2 are now one
runnable block that defines `DS`, `AS`, `TASK_DIR`, `LABEL`, `RD`, and `IN`, and the completion
turn gets its own command that rebuilds the path from the repo root and the label.

**[medium] The entry-count guard proved nothing.** Confirmed, and this is the sharper half of the
finding: the assembler emits a `--- ` header for skipped files too, so swapping a real file for a
nonexistent one leaves the count identical and the review blind. Counting is replaced by
grepping for `(SKIPPED`, and any skip is disqualifying — this bundle is the round's entire
evidence base.

**[medium] `run-dir` check-then-create.** Already fixed. The reviewer read `claude/bin/dstack`
from disk (its reads are unconfined, a documented residual) before this turn's repair landed;
the current code does `mkdir -p` on the parent and a plain `mkdir` on the leaf, exactly the
suggested direction. Verified in this turn: a taken label fails loudly instead of allocating a
variant.

**[medium] `"$DS"` was unset in later procedures.** Confirmed — it is defined only inside the P6
block, and the review skill itself says shell variables do not survive tool calls, so I wrote a
recipe that contradicted my own rule one file away. Every independently executable procedure now
uses the literal absolute path; `$DS` survives only inside the block that assigns it.

**[medium] `claude/CLAUDE.md` still described the removed registry.** Confirmed, and it is the
worst of these because that file is loaded for every session: it said state lives in
`.fullcycle-active`, that pausing means removing a line there, and that unchecked work prevents
turn end — all three now false, and the last one actively encourages the wait loop this Goal
removed. Rewritten. It was in no task's `files` declaration, which is why nothing forced me to
look at it; GOAL.md's T06 declaration was corrected before the edit.

**[medium] The convergence fix was not propagated to durable records.** Confirmed — the same
class-wide failure as M1's round. The skill said discovery time never changes blocking status
while the milestone doc, `04-review-io/task.md`, and GOAL.md's T04 row still described the
superseded round-4 downgrade. All three rewritten.

**[low] Pruning at closure prunes nothing.** Confirmed: a just-closed loop's captures are age
zero, so age-based pruning skips exactly the bundles the closing step claimed to remove. Step 4
now removes the closed unit's capture directories explicitly and keeps `prune` for abandoned
runs.

**[low] The subagent-effort claim was false.** Confirmed against the documentation, which states
subagent `effort` "Default: inherits from session" with a frontmatter override. My note said a
subagent never inherits the parent's effort, which is wrong; what does not transfer is ultracode
as a session mode. Corrected, along with the stale `claude -p` sentence in the subordinate
record.
