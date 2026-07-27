# Codex adversarial review — Round 002

## Review scope
Re-review of Round 001's repairs. Bundle: `full-cycle/SKILL.md` and this unit's `task.md`.

## GPT findings
Verification boundary: the workspace is empty, so repository tests and history could not be rerun; verification uses the supplied snapshots and primary platform documentation.

[severity:medium][technical correctness] The “explicit” state machine has no success transition for the ordinary in-scope fix: `assigned` can reach `verified` only through `expansion-requested`.
Sites: Primary: `P9 findings attribution`; confirmed: task `What was done`, whose optional-expansion notation contradicts the deployed sequence.
Evidence: The deployed sequence is `assigned → expansion-requested → (reassigned | recalled) → verified → closed`.
Verification: A matching worker that fixes without expansion or failure remains `assigned`, so the orchestrator cannot record `verified` or `closed` without inventing a transition.
Suggested direction: Add `assigned → verified` and define the same-worker approved-expansion transition explicitly.

[severity:medium][security] “A worker may READ anywhere” creates an unqualified secret-read capability despite the same change recognizing that gitignored credentials and keys exist.
Sites: Primary: P9 declaration rule; confirmed: `worktree-lifecycle.bootstrap` secret-deny-list discussion and `honest-scope`.
Evidence: Nothing in the supplied contract exempts secret paths, external credentials, or sensitive local configuration from “anywhere.”
Verification: A worker diagnosing configuration can read an undeclared `.env` or credential file, placing its contents in the persistent tool-call transcript while complying with this rule.
Suggested direction: Limit cross-declaration reads to non-secret repository material and make the secret deny list an unconditional higher-priority boundary.

[severity:medium][technical correctness] Round 001’s taint recovery remains incomplete because recreating a worktree cannot undo the external writes that the rule explicitly includes.
Sites: Primary: P9 declaration rule; confirmed: `honest-scope` and `worker-fanout.requires` resource-isolation rule.
Evidence: The rule names databases and paths outside the repository, then prescribes only discarding the checkout and rerunning.
Verification: Mutating a shared database or an external cache and recreating the Git worktree leaves that state changed; corruption or disclosure can survive closure.
Suggested direction: Separate repository taint from external side effects and require proven cleanup, reprovisioning, or an unresolved-blocker state before sealing.

[severity:medium][software structure] Resource isolation remains an all-or-nothing delegation prerequisite, so the change has not actually decoupled delegation from parallel scheduling.
Sites: Primary: `worker-fanout.requires`; confirmed: `delegate-when`, `parallel-when`, and the worker-fanout fail-closed comment.
Evidence: Every `requires` item must hold or work returns to the orchestrator, while resource isolation says “else serial” even though `parallel-when` is declared the later scheduling decision.
Verification: One otherwise eligible delegated task whose tests use a fixed shared port has no concurrent consumer, yet fails `requires` and must run in the orchestrator.
Suggested direction: Evaluate shared-resource isolation only when concurrent consumers exist, inside the scheduling gate.

[severity:medium][technical correctness] The `WorktreeCreate` mechanism has no defined handoff for the recorded base SHA, task branch, or fixture set, and its timing contradicts an orchestrator check “before briefing.”
Sites: Primary: `worker-fanout.per-task`; confirmed: `worktree-lifecycle.create`, `bootstrap`, and `requires` base-identity rule.
Evidence: The contract requires an exact recorded base but defines no mapping from the hook’s slug to that record or any hook-side verification before returning the path.
Verification: Official documentation shows that `WorktreeCreate` receives only common fields plus `name`, returns the adopted working directory, and runs as part of isolated-session creation; an external fail-closed mapping is therefore necessary but absent. [Claude Code hooks reference](https://code.claude.com/docs/en/hooks#worktreecreate)
Suggested direction: Define a durable name-to-base/branch/fixture record and require the hook to verify it before emitting the path, or launch the worker directly inside a preverified manual worktree.

[severity:low][security] The task document embeds reviewer-facing directives and review-disposition claims inside material consumed by the reviewing model.
Sites: Primary: task `Deployment context`; confirmed: “Read the diff you were given” and the claim that other changes have closed reviews.
Evidence: These statements attempt to influence review behavior and scope from inside the untrusted artifact.
Verification: Obeying the closed-review claim would exclude supplied `worker-fanout` and lifecycle changes and conceal the concrete blockers above.
Suggested direction: Record provenance as neutral metadata without imperatives or claims that determine the current reviewer’s scope.

Omitted-detail: 0 low

GPT verdict: reject — Five unresolved medium blockers leave attribution transitions, secret-read boundaries, taint recovery, delegation decoupling, and deterministic worktree binding unsound.

## Bundle size — the ratchet was VIOLATED again

R1 34,282 · **R2 47,052** bytes. Second violation in this Goal, same cause as T01's: the growth is
in the reviewed artifacts themselves, and one round of it is literally the explanation Round 001
demanded about what the diff carries. The rule prescribes removing carried prose; this unit's
bundle has none to remove. Two independent instances now support the follow-up already filed as
T01's F-02 rather than a one-off.

## Round outcome

Five blocking mediums and one low, all accepted, none rebutted. Blocking count went UP, 4 to 5.

Reasoning in `response-002.md`; ledger in `findings.md`.

Consensus: disagreed
