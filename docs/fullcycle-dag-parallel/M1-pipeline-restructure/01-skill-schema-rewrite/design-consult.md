## Assessment

The design is directionally workable, but it is not implementation-ready in three areas:

1. The phase DAG does not define how dependencies behave across `goal`, `milestone`, and `task` instances.
2. Review concurrency is conflated with worker concurrency. Disjoint files are required for worker fan-out, but the decided requirement makes cross-task review overlap unconditional.
3. The checker validates declared ownership, but nothing currently ensures a worker’s actual changes remain inside that declaration.

The YAML should describe the pipeline, while the existing hook-parsed documents remain the enforcement authority. Phase and gate references therefore need closed identifiers and explicit scope semantics rather than descriptive strings that can drift from the hook contract.

## Answers to open questions

1. **Wrapped task rows: join continuation lines before parsing.**

Choose option (b). Requiring the declaration to remain on the last physical line makes editor reflow and manual wrapping semantically significant.

Define a logical task item as beginning at a peer-level task checkbox and ending at the next peer-level task item or enclosing heading. Normalize its continuation whitespace, then require exactly one terminal `deps`/`files` declaration. A dedicated indented continuation line is a useful style convention, but it should not be a parser invariant.

The parser should reject duplicate fields, missing fields, malformed task IDs, ambiguous declaration delimiters, and nested content it cannot classify. Free prose must not be allowed to masquerade as the structured suffix.

2. **Use literal repository paths and trailing-slash directory prefixes only. Do not support globs.**

Prefix matching is the appropriate ceiling for a hand-maintained Bash/AWK checker. Glob intersection is difficult to make deterministic, especially when paths do not exist yet.

The contract should additionally require:

- Repository-relative, normalized paths.
- No absolute paths or parent traversal.
- Exact path equality as an overlap.
- Directory prefixes matching only at path-component boundaries.
- Ancestor/descendant directory prefixes counting as overlap.
- Both old and new paths being declared for renames.
- Added, deleted, renamed, and untracked output being included in later actual-scope validation.

A task whose scope cannot be expressed with literals and directory prefixes should remain serial. Empty `files` may be valid as a declaration, but it must make the task ineligible for fan-out.

3. **Confirm the rejection of a separate duplicated YAML task graph.**

The rejection is correct as framed. A checklist plus a second authoritative-looking graph creates two representations and no stated mechanism guarantees synchronization.

This does not mean YAML itself is unsuitable. A YAML graph that completely replaced the checklist could be a single representation, but that would be a different document design. For the current design, keep one logical checklist item containing both the human description and structured declaration.

4. **Use explicit Git worktree lifecycle semantics.**

Explicit worktree and branch management is more robust across target repositories because the pipeline can persist and inspect the base commit, task branch, worktree path, merge state, and cleanup state. Harness-managed isolation may be used only as an adapter when it exposes the same lifecycle guarantees.

Required invariants include:

- Every worker starts from a recorded base commit.
- Worktree and branch identities are unique and recoverable after interruption.
- No worktree is cleaned before its changes are validated, merged, and deregistered.
- A dirty or unsupported repository state fails closed.
- The orchestrator, not individual workers, owns merge and cleanup.
- Actual changed paths are validated before review and again before merge.

5. **Do not bless deregistration as the normal `external-wait` mechanism.**

Unregistration removes the only state the Stop hook enforces. A prose-only “re-register on resume” rule cannot protect against session termination, forgotten resumption, or a new session that does not know what was removed.

The safer in-scope behavior is to keep task documents registered and keep the orchestration turn alive while polling or waiting for the external reviews. If the environment must end the turn, there is no hook-equivalent safe pause without changing the hook. A hook-visible pause marker would only be enforceable if the hook understood it, which is explicitly out of scope.

If deregistration must remain available, classify it as a manual recovery escape hatch, persist the suspended task identities and external run identifiers, and do not present it as preserving the tripwire.

## Risks

[severity:high][technical correctness] Declared file disjointness is not connected to actual worker output.

Evidence: The checker examines GOAL.md declarations, while worker behavior is governed only by a delegated brief and conventions.

Verification: A worker can modify an undeclared shared file, allowing the candidate set to pass while creating a merge conflict or invalidating another task’s open review.

Suggested direction: Add a fail-closed actual-diff scope gate before review and before merge, covering tracked and untracked path changes.

[severity:high][the real Why] The bridge invariant contradicts unconditional review overlap unless review and implementation parallelism are distinguished.

Evidence: The requirements say reviews for different tasks always overlap, while full worker fan-out alone is conditional on disjointness; the proposed bridge says “parallel tasks” must be disjoint.

Verification: Two tasks sharing a file must execute serially but must still have overlapping reviews. Applying a fix for one while the other round remains open mutates the sibling review bundle and voids that round.

Suggested direction: Reserve file disjointness for worker fan-out; conduct overlapping review rounds against a frozen union of their bundles and defer mutations until all open rounds in that review batch finish.

[severity:high][technical correctness] “No dependency edge” is insufficient as a fan-out precondition.

Evidence: The design mentions an edge but does not specify direct versus transitive dependency or readiness of predecessors outside the candidate set.

Verification: Two candidates may have no direct edge while one is reachable from the other through an intermediate task, or a candidate may have an unfinished predecessor outside the set.

Suggested direction: Require every candidate to be ready and every pair in the candidate set to be incomparable under transitive dependency reachability.

[severity:high][software structure] Phase dependencies lack instance-level scope semantics.

Evidence: Each phase has `per: goal | milestone | task`, but `needs` contains only phase IDs.

Verification: A task-scoped phase depending on a milestone-scoped phase could mean its own milestone instance, every milestone, or merely the existence of one completed instance; each interpretation produces a different schedule.

Suggested direction: Define phase instances and dependency quantification explicitly across goal, milestone, and task boundaries.

[severity:medium][technical correctness] “Re-review only the affected task” is underdefined for conflict resolution and shared paths.

Evidence: Conflict resolution or a post-merge edit can touch paths belonging to more than one task or review bundle.

Verification: Assigning such a change to one task would leave another reviewed bundle changed without re-review.

Suggested direction: Compute the affected task set from changed paths and review bundles; “only affected” must not be interpreted as “exactly one.”

[severity:medium][security] The task declaration is untrusted text consumed by a shell-based checker.

Evidence: Free prose and path fields share one logical item, and the checker is planned in Bash/AWK.

Verification: Shell expansion, evaluation, word splitting, or filesystem glob expansion would turn document content into executable or environment-dependent behavior.

Suggested direction: Treat parsed values strictly as inert data, use a restricted grammar, and reject unsupported characters or ambiguous quoting rather than interpreting them.

[severity:medium][technical correctness] Concurrent registration is not guaranteed merely because a multiline registry format exists.

Evidence: Multiple task documents will be active concurrently, but no writer or lifecycle ownership is defined.

Verification: Independent workers updating the registry can lose entries, and automatic worktree cleanup can leave registered paths pointing to missing documents.

Suggested direction: Make registry mutation an atomic, orchestrator-owned operation and tie deregistration to completed merge and worktree lifecycle transitions.

[severity:medium][technical correctness] Review artifact names may collide during fan-in.

Evidence: Every task uses the `codex-review-<NNN>.md` series contract, while isolated worktrees eventually merge into one branch.

Verification: If task-specific review series do not occupy distinct directories, independently created files with the same round number conflict despite source-file disjointness.

Suggested direction: Define a stable per-task review namespace and include review artifacts in merge-scope accounting.

[severity:medium][software structure] Gate descriptions in the phase YAML can drift from the hook contract.

Evidence: The hook does not read SKILL.md, while the new schema independently names the document or checkbox closing each phase.

Verification: A rewritten gate description can appear valid to the orchestrator while the unchanged hook continues expecting the old heading or checkbox coupling.

Suggested direction: Use stable gate identifiers mapped once to preserved contract text, and verify all hook-visible fixtures remain byte-compatible.

[severity:medium][DX] Worker role selection is not deterministic across arbitrary repositories.

Evidence: “Frontend files” has no repository-neutral definition, and tasks may contain mixed frontend and non-frontend paths.

Verification: The same declaration can be routed differently depending on unstated filename assumptions.

Suggested direction: Use repository-defined path classifications; select the specialized worker only when the complete declared scope matches, otherwise use the general worker or fail closed.

[severity:medium][the real Why] File disjointness does not establish semantic independence.

Evidence: The checker validates paths and dependencies declared by the planner, not runtime coupling between disjoint files.

Verification: Independent edits can still change a shared interface or assumption without touching the same files.

Suggested direction: Describe disjointness as an eligibility check rather than a correctness proof, with declared dependencies and milestone E2E retaining responsibility for semantic integration.

## Recommendation summary

- Keep one logical checklist representation and parse wrapped items as units.
- Limit ownership expressions to normalized literal paths and directory prefixes.
- Add actual-diff containment as a mandatory fan-out invariant.
- Separate worker fan-out rules from immutable overlapping-review rules.
- Define transitive readiness and cross-scope phase-instance semantics.
- Use explicit, recoverable Git worktree lifecycle management.
- Keep registrations active during external waits; defer a safe pause protocol until the hook can enforce it.
- Preserve hook surfaces through stable gate identifiers and compatibility validation.