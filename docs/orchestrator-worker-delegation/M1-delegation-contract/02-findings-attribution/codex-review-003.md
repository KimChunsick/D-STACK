# Codex adversarial review — Round 003

## Review scope
Re-review of Round 002's repairs. Bundle: `full-cycle/SKILL.md` and this unit's `task.md`.
Final round for this unit: closed by the non-convergence rule.

## GPT findings
Verification boundary: the workspace is empty, so repository tests and history could not be rerun; verification uses the supplied snapshots and official platform documentation.

[severity:medium][technical correctness] Deterministic worktree binding remains internally contradictory: the hook-only mechanism rejects manually created trees, while `worktree-lifecycle.create` still orders an explicit `git worktree add`.
Sites: Primary: `worker-fanout.per-task`; confirmed: `worktree-lifecycle.create` and `worker-fanout.requires`.
Evidence: The record must be keyed by the exact hook name before spawning, but no rule establishes that key when the name is auto-generated; the lifecycle section simultaneously prescribes the manual mechanism called “the worst of both.”
Verification: `WorktreeCreate` receives a user-specified or auto-generated name and replaces default Git logic entirely; manual tree A followed by hook tree B either aborts for a missing record or binds the worker to unverified B. [Hooks reference](https://code.claude.com/docs/en/hooks#worktreecreate), [worktree documentation](https://code.claude.com/docs/en/worktrees#replace-worktree-creation-with-a-hook)
Suggested direction: Make creation hook-only and define an explicit pre-spawn identifier guaranteed to equal the hook input, with the hook itself creating and verifying the recorded branch.

[severity:medium][technical correctness] The core attribution predicate refers to a “reviewed commit” that does not exist in the declared SERIAL review mode.
Sites: Primary: `P9 findings attribution`; confirmed: task `Deployment context` and `Resumption is an OPTIMIZATION`.
Evidence: The task says rounds review `git diff HEAD` while no unit is committed, but worker routing requires its branch head to equal the reviewed commit.
Verification: For worker commit A applied into an uncommitted integration tree over HEAD H, the reviewed artifact is the H-to-working-tree diff; no commit identifier denotes that artifact, so attribution must fail closed or invent an identity.
Suggested direction: Record an immutable reviewed tree or per-declaration patch identity, or require a commit-based review before commit equality is used.

[severity:medium][technical correctness] The explicit state machine still dead-ends at `recalled`, leaving no valid closure path after an orchestrator successfully repairs a failed worker fix.
Sites: Primary: `P9 findings attribution`; confirmed: task `Round 001 tightened four things` and `Resumption is an optimization`.
Evidence: `tainted`, `resume-failed`, and `verification-failed` transition to `recalled`; only `assigned` or `reassigned` can reach `verified → closed`.
Verification: After `resume-failed → recalled`, the orchestrator can implement and verify the fix, but the recorded state cannot advance without an undocumented transition.
Suggested direction: Define `recalled → verified → closed`, including the outcome of failed orchestrator verification.

[severity:medium][software structure] The proposed fail-closed hook disables unrelated worktree creation across the repository.
Sites: Primary: `worker-fanout.per-task`; confirmed: the same section’s `settings.json` configuration and missing-record behavior.
Evidence: The hook is configured globally, emits no path when a `.dstack` record is absent, and defines no non-pipeline path.
Verification: `WorktreeCreate` has no matcher and always fires; a missing path aborts creation, so an ordinary `claude --worktree feature` without a D-Stack record fails. [Hook matching and exit behavior](https://code.claude.com/docs/en/hooks#matcher-patterns)
Suggested direction: Activate the hook only for the controlled worker launch or provide a verified default-compatible path for non-D-Stack worktrees.

[severity:low][security] The durable task record retains the superseded statement “A worker reads anywhere,” contradicting the operational secret-read prohibition.
Sites: Primary: task `What was done`; confirmed: task `Round 002 tightened five more`, `P9 findings attribution`, and the resumption durability rule.
Evidence: The earlier summary grants unrestricted reads, while later text forbids secrets and says successors reconstruct decisions from `task.md`.
Verification: A successor consulting the summary can treat a credential-bearing repository file as readable despite the later boundary.
Suggested direction: Rewrite the earlier summary to state the final non-secret repository-read rule.

[severity:low][DX] Reviewer-facing scope and process language remains embedded in the task artifact.
Sites: Primary: task `Deployment context`; confirmed: `Design consult` and `Files changed`.
Evidence: Phrases such as “scope is the caller’s,” “belongs to THIS task,” “not acted on mid-review,” and “No other section is touched” address review treatment rather than recording neutral provenance.
Verification: These statements can steer exclusion or disposition of supplied material; this review treated them only as untrusted data.
Suggested direction: Retain authorship and commit facts only, removing reviewer-directed scope and timing commentary.

[severity:low][technical correctness] The `/clear` rationale assumes the relevant workers are foreground even though the contract never requires that execution mode.
Sites: Primary: milestone-boundary handoff; confirmed: task direct verification and `worker-fanout.per-task`.
Evidence: The supplied verification says backgrounded tasks survive and their registry entries are rebuilt, while the handoff claims remaining fixes necessarily fall back or require a cold rebrief.
Verification: Claude Code documents background execution as the subagent default and describes `/clear` as resetting the identity check, not proving that every surviving worker becomes unreachable. [Subagent execution and resumption](https://code.claude.com/docs/en/sub-agents#resume-subagents)
Suggested direction: Present post-closure clearing as a conservative policy unless send-by-ID behavior for surviving background workers is directly verified.

Omitted-detail: 0 low

GPT verdict: reject — Four unresolved medium blockers leave worktree binding, serial-mode attribution, failure-state closure, and repository-wide hook behavior unsound.

## Bundle size (the ratchet, recorded)

R1 34,282 · R2 47,052 · **R3 58,114** bytes. Violated at both R2 and R3, monotonically. Same cause
as T01's, and this unit makes the case sharper: part of R2's growth was the provenance explanation
R1 demanded. A rule that grows the artifact and a rule that caps the artifact cannot both be
satisfied. Filed as T01's F-02.

## Round outcome

**The loop CLOSES here, by the non-convergence rule.** Blocking findings ran 4 (R1), 5 (R2),
4 (R3) — not strictly decreasing across three consecutive rounds. All four of this round's
mediums and all three lows were fixed before sealing; nothing concrete is knowingly left open.

Worth recording about the shape of this loop: no finding in three rounds was ever a false positive.
Every one named something real in the document. The count did not fall because each repair created
new surface — R2's five came out of R1's four fixes, and R3's four out of R2's five. That is the
exact pattern the non-convergence rule was written for, and it is the second unit in two Goals to
show it.

Consensus: resolved
