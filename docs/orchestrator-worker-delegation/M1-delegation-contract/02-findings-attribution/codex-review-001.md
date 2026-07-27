# Codex adversarial review — Round 001

## Review scope
First round for this unit. Bundle: `full-cycle/SKILL.md` and this unit's `task.md`.

## GPT findings
Verification boundary: the workspace is empty, so repository tests and history could not be rerun; findings use the supplied snapshot and official platform documentation.

[severity:medium][software structure] The durable task record falsely says T01’s `delegate-when` and all other sections are untouched, while the supplied “tracked, scoped diff” rewrites delegation, lifecycle, bootstrap, and handoff rules. This leaves task-level finding ownership—the feature being introduced—indeterminate.
Sites: Primary: task `Deployment context`; confirmed: both `Files changed` sections and the `SKILL.md` diff beginning at `worker-fanout`.
Evidence: The task explicitly calls its statement “a claim about the diff,” but the diff directly contradicts it and the second `Files changed` section remains `<pending>`.
Verification: Literal comparison of the supplied task snapshot with the supplied scoped diff confirms the mismatch.
Suggested direction: Assign the T01 hunks to their actual unit or expand this task’s declaration and change inventory before review.

[severity:medium][technical correctness] Declaration containment is treated as proof that the original worker owns the current defect, although merge resolution or a manual post-merge edit can replace that worker’s code. A finding can therefore be routed to a worker whose retained branch and transcript do not contain the reviewed implementation.
Sites: Primary: `P9 findings attribution`; confirmed: `worktree-lifecycle.reopen`, `cleanup`, and the recorded-base resumption rule.
Evidence: `reopen` explicitly includes merge conflicts and manual post-merge edits, while P9 routes every in-declaration finding to the original task worker.
Verification: Worker branch A can be integrated as resolution B; a review finding against B is still assigned to the worker on A under the written predicate.
Suggested direction: Route integration-authored changes to the orchestrator and assign a worker only when its checkout matches the exact reviewed commit/tree.

[severity:medium][technical correctness] The declared “WRITE CAPABILITY” and taint guarantee are not enforceable by the described checker. An undeclared tracked file can be modified and restored before commit, or an ignored/external path can be mutated, while both the clean-tree check and commit-path union pass.
Sites: Primary: P9 declaration rule; confirmed: `worker-fanout.honest-scope`, `per-task`, and the task’s checker-unchanged claim.
Evidence: `honest-scope` admits there is no filesystem sandbox or write audit, yet P9 says every unapproved write taints the worktree and cannot be laundered.
Verification: A restored uncommitted write produces no commit path for `scope` to enumerate; build/test side effects can likewise occur before the worker can stop them.
Suggested direction: Either add process-level write auditing/confinement or explicitly make this a self-reported policy with side-effect and taint-recovery rules.

[severity:medium][software structure] `WorktreeRemove` cannot by itself enforce cleanup only after review closure: its lifecycle event occurs during session/subagent removal and cannot block, whereas closure happens after the orchestrator receives and verifies the worker result. The contract supplies no post-closure trigger, leaving either premature deletion or indefinite retention.
Sites: Primary: `worker-fanout.per-task`; confirmed: `worktree-lifecycle.cleanup`.
Evidence: The diff says “WorktreeRemove owns teardown” while separately requiring that teardown occur only after the owning review closes.
Verification: Claude’s official hook contract states that `WorktreeRemove` runs when a subagent finishes/removal occurs and has no decision control. [Hooks reference](https://code.claude.com/docs/en/hooks#worktreeremove)
Suggested direction: Make the orchestrator perform explicit post-closure removal and treat `WorktreeRemove` only as cleanup notification/archiving.

[severity:low][technical correctness] The `/clear` rationale says it destroys “every warm worker,” but its own reverse-engineering says backgrounded tasks survive and the registry is rebuilt from those survivors.
Evidence: The supplied verification distinguishes aborted non-backgrounded tasks from surviving backgrounded tasks.
Verification: Official documentation establishes only that the `SendMessage` name check resets on `/clear`, not that every subagent is killed. [Subagent resumption documentation](https://code.claude.com/docs/en/sub-agents#resume-subagents)

[severity:low][DX] The advertised explicit state machine omits `tainted`, resume-failed, verification-failed, and recalled/abandoned states despite prose defining those outcomes.
Evidence: The only listed states are `assigned`, `expansion-requested`, `reassigned`, `verified`, and `closed`.
Verification: A reported out-of-scope write or failed verification has no valid recorded transition in the supplied graph.

Omitted-detail: 0 low

GPT verdict: reject — Four unresolved medium blockers leave review scope, fix attribution, write-boundary enforcement, and retained-worktree lifecycle unsound.

## Bundle size (the ratchet, recorded)

Round 001 bundle: 34,282 bytes.

## Round outcome

Four blocking mediums and two lows, all accepted. One of them is about the review setup rather than
the change: serial mode reads `git diff HEAD`, nothing in this Goal is committed, so the bundle
carried three tasks' work against one task's record.

Reasoning in `response-001.md`; ledger in `findings.md`.

Consensus: disagreed
