# Maintainer response — Round 003

Outside the reviewed corpus. Seven findings, all accepted and all fixed before sealing.

**[medium] Worktree binding was internally contradictory.** The contract said creation is hook-only
and `worktree-lifecycle.create` still ordered `git worktree add`, and nothing established the key
the record must be written under when the platform generates the name. Three things now stated:
the orchestrator chooses the worktree name (equal to the task branch slug) so the record can be
written in advance; `git worktree add` is what the orchestrator runs when IT is doing the work
serially with no subagent, not a second route to a worker's tree; and the hook creates the tree from
the recorded base on the recorded branch and verifies what it made.

**[medium] A globally configured fail-closed hook would break unrelated worktrees.** The finding I
am most glad arrived: `WorktreeCreate` has no matcher and fires for every worktree, so "emit nothing
when the record is missing" would kill an ordinary `claude --worktree feature`. A missing record now
means NOT A PIPELINE WORKTREE and falls through to normal creation. Fail closed only when a record
exists and does not match — the case that actually endangers a review.

**[medium] "The reviewed commit" does not exist in SERIAL mode.** Correct, and it exposed an
unstated assumption. In `committed` mode the predicate is literal. In serial mode the reviewed
artifact is a working-tree diff against HEAD with no commit id, so there is nothing to compare and
the predicate now fails closed to the orchestrator. Which is consistent anyway: a serial round is
the orchestrator reviewing its own uncommitted work, where there is no worker to route to.

**[medium] The state machine dead-ended at `recalled`.** Round 002 added `recalled` as the
destination for `tainted`, `resume-failed` and `verification-failed`, and then gave it no exit — so
an orchestrator that successfully repaired a failed worker fix had no way to record it.
`recalled` → `verified` → `closed` now. A failed orchestrator verification stays `recalled`, because
there is nobody further to hand it to.

**[low] The record still contained the superseded "reads anywhere".** A successor reading the
summary would have taken a credential-bearing file as readable despite the later boundary. Rewritten
at the summary.

**[low] Reviewer-directed language survived a third round.** Phrases like "scope is the caller's" and
"No other section is touched" are still addressed at review treatment rather than recording facts.
Stripped. Three drafts of one paragraph were findings in three consecutive rounds, which is recorded
in the task document because the recurrence is the interesting part.

**[low] The `/clear` rationale assumed foreground workers.** The contract never requires that
execution mode, so the rationale should not lean on it.
