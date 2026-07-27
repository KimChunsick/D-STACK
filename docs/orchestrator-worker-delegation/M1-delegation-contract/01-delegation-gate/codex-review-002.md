# Codex adversarial review — Round 002

## Review scope
Re-review of Round 001's repairs. Bundle: `full-cycle/SKILL.md` and this unit's `task.md`. The
Goal research artifact was dropped from the allowlist this round (it may shrink, never grow).

## GPT findings
[severity:high][security] `honest-scope` falsely claims committed-deliverable containment although the endpoint-only check misses transient out-of-scope commits that remain in integrated history.
Sites: `worker-fanout.honest-scope`; confirmed: task doc “Routed elsewhere”, `worktree-lifecycle.work`.
Evidence: The supplied checker uses `git diff "$base" HEAD`; Git documents that `diff` compares endpoints, while commit/object traversal requires history inspection such as `rev-list`. [Git diff](https://git-scm.com/docs/git-diff.html), [Git rev-list](https://git-scm.com/docs/git-rev-list)
Verification: Commit an undeclared credential file, delete it in a later commit, then run the stated check: the path disappears, but merging the branch preserves the earlier commit and blob in reachable history.
Suggested direction: Block this broader delegation gate on a history-aware scope check covering every commit from base through HEAD.

[severity:medium][technical correctness] The declared single worktree mechanism remains broken: a manually created checkout neither binds a non-isolated subagent’s working directory nor receives automatic `.worktreeinclude` bootstrap.
Sites: `worker-fanout.per-task`; confirmed: `worktree-lifecycle.create`, `worktree-lifecycle.bootstrap`.
Evidence: Claude documents that non-isolated subagents start in the parent directory and do not retain `cd`, while `.worktreeinclude` applies only to worktrees Claude creates. [Subagents](https://code.claude.com/docs/en/sub-agents), [Worktrees](https://code.claude.com/docs/en/worktrees)
Verification: A worker can report the correct identity using `cd <worktree> && ...`, but its next Bash call returns to the parent checkout; meanwhile `git worktree add` never performs the described fixture copy.
Suggested direction: Ensure worker session cwd, base creation, bootstrap, and retention all derive from one platform-isolated or explicitly launched worktree mechanism.

[severity:low][DX] The task narrative says `delegate-when` has two conditions although the edited contract has three, and `per-task` contains the duplicated fragment “the worker runs.”
Sites: task doc “What was done”; confirmed: `worker-fanout.per-task`.
Evidence: `POSITIVE ISOLATION BENEFIT` is a third predicate, while the final `per-task` sentence reads “re-pointed. the worker runs the worker runs P7-tdd.”
Verification: Direct comparison of the supplied task snapshot and scoped diff reproduces both inconsistencies.

Omitted-detail: 0 low

GPT verdict: reject — The endpoint-only containment check permits secret-bearing history to pass, and the worker still is not reliably bound or bootstrapped in the verified checkout.

## Bundle size (the ratchet, recorded)

R1 40,728 · **R2 36,821** bytes. Down, as required.

## Round outcome

Two blocking and one low, all accepted. One of the two blocking findings was STALE relative to the
repository and accurate relative to this document — the defect it names had already been fixed and
sealed as T04, but this unit's record still described it in the present tense and the fixed file
was not in the bundle. That is a record defect, not a false positive, and it is fixed here.

Reasoning in `response-002.md`; ledger in `findings.md`.

Consensus: disagreed
