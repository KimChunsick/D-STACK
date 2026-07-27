# Maintainer response — Round 001

Outside the reviewed corpus. Six findings, all accepted; nothing rebutted.

**[medium] The record claimed sections were untouched that the diff plainly rewrote.** True as
stated, and worth being precise about the cause. Rounds run in SERIAL mode against
`git diff HEAD`, and no unit of this Goal is committed, so the bundle carries T01's `worker-fanout`
rewrite and T04's `check-parallel.sh` enumeration alongside this task's two additions. The document
now says that outright and lists what THIS task authored, so a reviewer can separate them. The
structural fix — commit each unit as it closes, or review in `committed` mode against a recorded
base — is recorded as a Goal-level follow-up rather than done mid-review, because committing while
a later round may still read `git diff HEAD` would empty the diff that round depends on.

**[medium] Declaration containment does not prove the original worker owns the current defect.**
The finding I most wanted to have thought of myself. `reopen` already says a merge conflict or a
post-merge edit reopens a review, which means the reviewed code can be code the worker never
wrote — and the rule as written would still send the finding to that worker's stale branch and
transcript. Routing now also requires the worker's branch head to equal the reviewed commit;
integration-authored changes go to whoever authored them, which is the orchestrator.

**[medium] The write-capability and taint guarantee were not enforceable.** Accepted, and it is a
contradiction between two things I wrote: `honest-scope` says there is no sandbox and no write
audit, and then P9 claimed every unapproved write taints the worktree. Only writes that reach a
COMMIT are detectable. A tracked file edited and restored before any commit, an ignored file, a
database, anything outside the repository — none of it is. The rule now says which half is enforced
and which half is self-reported policy, and adds the recovery for when the unenforced half surfaces
late: discard the worktree, re-create from the recorded base, re-run the fix.

**[medium] `WorktreeRemove` cannot enforce cleanup-after-closure.** Correct: it fires at subagent
teardown and has no decision control, so it can never hold a tree open until a review closes. I had
written "WorktreeRemove owns teardown" one round after being handed the `WorktreeCreate` mechanism,
and over-generalised from create to remove. The orchestrator now removes the worktree explicitly
after closure; the hook is notification or archiving.

**[low] The `/clear` rationale over-claimed against my own evidence.** Also correct, and slightly
embarrassing: the verification I recorded distinguishes aborted non-backgrounded tasks from
surviving backgrounded ones, and then the prose said "every warm worker". Reworded to what the
evidence supports.

**[low] The state machine omitted the outcomes its own prose defines.** `tainted`, `resume-failed`
and `verification-failed` had no transition. All three now route to `recalled`, meaning the
orchestrator takes the fix.
