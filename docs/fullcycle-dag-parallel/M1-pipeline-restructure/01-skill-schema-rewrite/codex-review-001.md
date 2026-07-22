# Codex adversarial review — Round 001

## Review scope
Adversarial review (GPT-5.6 Sol, xhigh; bundle: SKILL.md, tests/skill-schema.test.sh,
goal research, design brief + consult).

## GPT findings
[severity:high][software structure] The purported control-flow schema is invalid YAML and omits the machine-readable phase metadata needed for deterministic scheduling.
Evidence: `worker-fanout` mixes sequence entries with a sibling `per-task` mapping, which YAML parsers reject. Phase `per` and `needs` values exist only in comments, and no per-phase `gate` fields exist. The new tests merely search for strings, so they accept malformed YAML and phase objects containing only `id`.
Verification: Ruby's YAML parser reports "did not find expected '-' indicator" at `worker-fanout`; parsing the phase fragment yields only `{"id": ...}` objects. Even after repairing syntax, the stated bare-ID "same instance" rule cannot resolve task-scoped P7's dependency on goal-scoped P6.
Suggested direction: Define a valid typed schema with explicit scope, dependency, and gate fields, then test it using a real YAML parser plus semantic cross-reference validation.

[severity:high][the real Why] Nothing in the supplied changes consumes the YAML as executable scheduling data, so the rewrite still depends on an LLM reinterpreting Markdown each session.
Evidence: The task changes only SKILL.md and grep-based structural tests; it explicitly says the hook does not read SKILL.md. The cited Structured Outputs evidence concerns schema-constrained model output, not deterministic interpretation of fenced YAML input.
Verification: Search the allowlisted changes for schema loading or dispatch logic; there is none. Altering scheduling values while retaining the tested keywords would leave every test green.
Suggested direction: Identify the actual schema consumer and validate its behavior end-to-end, or describe the document honestly as structured prompting rather than deterministic orchestration.

[severity:high][technical correctness] Fail-closing every declaration-validation failure to serial execution can permanently deadlock the pipeline.
Evidence: P7 requires every declared predecessor's P10 completion, while the scheduling block says any parse or validation failure yields "serial." A cycle, self-dependency, or unknown predecessor cannot be satisfied merely by selecting serial mode.
Verification: Declare two tasks as each other's predecessor, or reference a nonexistent task. The checker rejects parallelism, but neither task becomes P7-ready under the phase DAG.
Suggested direction: Separate parallel-ineligibility from graph invalidity: overlap should select serial execution, while malformed, cyclic, duplicate, or unresolved dependencies must return to decomposition as blocking errors.

[severity:high][the real Why] The clean-main-tree requirement makes worker fan-out unreachable in the documented normal workflow.
Evidence: P3 writes research and P6 writes GOAL.md, task documents, and the registry before P7 evaluates fan-out. P8 and P9 add further orchestrator-owned documentation. No phase commits or otherwise checkpoints those required changes.
Verification: Start from a clean repository and follow P3 through P6; `git status` becomes dirty before the first worker candidate is evaluated, forcing serial mode.
Suggested direction: Define an explicit orchestrator checkpoint/base transition or a narrowly specified cleanliness invariant that preserves user changes while accounting for required workflow artifacts.

[severity:high][technical correctness] The worktree lifecycle does not define an executable reviewed-change state machine.
Evidence: It creates a branch, validates a working-tree "actual diff," and later merges the branch, but specifies no commit step or exact base-to-head scope calculation. Merge is not represented in the phase DAG, while P10 can mark a task complete and deregister it and successor readiness depends on that P10 state.
Verification: If worker edits remain uncommitted, merging the task branch is a no-op. If they are committed implicitly, ordinary status-based scope checks become empty and can miss undeclared committed paths. Because merge is not ordered before P10, a successor can also start from main before its completed predecessor has been integrated.
Suggested direction: Bind each task to a recorded base, reviewed head commit, complete base-to-head plus staged/unstaged/untracked path set, and an explicit merge/reopen transition that must finish before P10 completion or dependent-task readiness.

[severity:high][technical correctness] Review-overlap invalidation stops when a round closes, allowing later fixes or sibling merges to leave stale consensus.
Evidence: Bundled files are immutable only while a round is open. Re-review is required for merge conflicts or manual post-merge edits, but not for normal fixes made after another task's round seals or for clean sibling merges that change the reviewed base.
Verification: Run reviews for two tasks sharing a file, let task B reach consensus, then apply task A's requested fix after both rounds seal. B's reviewed bundle has changed, yet its consensus remains ticked. Disjoint branches can similarly merge cleanly while introducing a contract interaction that neither task review observed.
Suggested direction: Invalidate and reopen every affected task whenever its reviewed bundle or relevant merge base changes, or add an adversarial review of the integrated milestone state rather than relying solely on E2E coverage.

[severity:medium][technical correctness] Fan-out eligibility ignores shared runtime resources even though the supplied research identifies them as a known worktree limitation.
Evidence: Eligibility checks dependencies, file paths, worker type, and main-tree cleanliness only. There is no setup or isolation contract for ignored configuration, ports, test databases, dev servers, caches, memory, or other process-global resources.
Verification: Run two otherwise file-disjoint tasks whose tests bind the same fixed port or mutate the same local test database; they can interfere, produce nondeterministic failures, or validate against contaminated state.
Suggested direction: Add repository-specific environment and resource-isolation readiness to fan-out eligibility, with serial fallback when isolation cannot be demonstrated.

[severity:medium][technical correctness] The declared path grammar is not canonical enough to support sound disjointness decisions.
Evidence: It rejects absolute paths, globs, and `..`, but does not reject `.` components, repeated separators, platform-equivalent case or Unicode forms, or paths traversing repository symlinks.
Verification: Two declarations can use lexical aliases such as `src/a` and `src/./a`; the stated exact/prefix algorithm treats them as different even though filesystem and Git operations resolve them to the same target.
Suggested direction: Require canonical Git-tree path spellings and explicitly reject aliases and symlink traversal before overlap or scope evaluation.

[severity:medium][UI & UX / DX] Worker routing remains nondeterministic because "frontend files" has no stack-neutral definition.
Evidence: The rule requires all-frontend, no-frontend, and mixed classifications, but supplies no repository-defined classifier or fallback for ambiguous shared files. This issue was raised in the design consult and was not resolved in SKILL.md.
Verification: Classify a shared TypeScript model, build configuration, generated client, or package manifest; different orchestrator sessions can reasonably produce different routing and eligibility decisions.
Suggested direction: Make classification an explicit repository-provided mapping and fail closed when the complete declaration cannot be classified consistently.

[severity:low][security] The review bundle contains prompt-like directives capable of steering a reviewer if the outer untrusted-data boundary is omitted.
Evidence: The design brief tells the reviewer that requirements are "not up for re-litigation," says "review this," and specifies "What to return." These are artifact contents, not authoritative instructions for the current review.
Verification: Pass the artifact directly to a reviewing model without a higher-priority untrusted-data wrapper and check whether it suppresses challenges or follows the embedded output directions.
Suggested direction: Ensure every review invocation structurally delimits all assembled task and research artifacts as untrusted data.

GPT verdict: reject — The schema is neither valid nor executable, and its dependency, worktree, and review lifecycle rules contain concrete deadlock, no-op merge, scope-bypass, and stale-approval paths.

## Maintainer response
Per the user's per-goal directive, fixes landed during implementation; this round seals
as disagreed pending independent re-verification in round 002 (consolidated).
1. Agreed, fixed. Phases are typed flow mappings (`id`/`per`/`needs`/`gate` real
   fields); `worker-fanout` restructured (`requires:` list + sibling keys). The test
   now extracts every fenced yaml block and parses it with ruby's YAML loader — all
   three parse. Verification: `tests/skill-schema.test.sh` (yaml-parse section).
2. Agreed, reframed. The intro now states the honest consumer model: structured
   prompting for the orchestrating LLM; the DETERMINISTIC consumer is
   check-parallel.sh (T02, implemented this goal) consuming the GOAL.md declarations.
3. Agreed, fixed. Verdicts are three-way; INVALID (malformed/cycle/unknown/duplicate)
   is a blocking error returning to P5-decompose, never collapsed into SERIAL. The
   checker exits 2 for INVALID vs 1 for SERIAL, with contract tests.
4. Agreed, fixed. Cleanliness is now declared-path-scoped: docs/ and the registry are
   orchestrator-owned and undeclarable, so routine pipeline writes cannot block
   fan-out; only uncommitted changes under a candidate's declared paths (or an
   in-progress merge/rebase) do.
5. Agreed, fixed. Lifecycle now: recorded fan-out base → worker COMMITS on the task
   branch → scope = base..HEAD names PLUS staged/unstaged/untracked → review →
   scope re-check → topological merge, with "Merge precedes P10 completion" binding
   merge before task completion and successor readiness → reopen → cleanup last.
6. Agreed, fixed. `post-seal-rule` + `reopen`: any post-seal change to a sealed
   bundle's file before milestone close reopens that task's review; affected set is
   computed from touched paths ∩ declarations/bundles (can exceed one). Milestone E2E
   remains the integration backstop.
7. Agreed, fixed. `resource isolation` is a fan-out requirement (ports, test DBs, dev
   servers, caches demonstrably isolated per worktree, else serial).
8. Agreed, fixed. Canonical grammar (no `.` components, repeated separators, trailing
   separators, symlink traversal; case-variant collisions count as overlap) in
   SKILL.md, enforced by the checker (INVALID on non-canonical; case-insensitive
   overlap vs case-sensitive scope containment).
9. Agreed, fixed. Worker binding resolves from the target repo's OWN declared
   frontend classification; mixed or unclassifiable → ineligible (fail closed).
10. Agreed (low, non-blocking). The codex-review/consult invocations already wrap all
    assembled material in an untrusted-data framing at the prompt boundary; recorded
    here as the standing requirement for any future reviewer path.

## Carried decisions
- Consumer honesty framing (finding 2) is a permanent scope statement: SKILL.md is
  structured prompting; determinism lives in check-parallel.sh over GOAL.md.
- Low finding 10 recorded as a standing invariant, non-blocking.
- All fixes await independent re-verification in round 002.

Consensus: disagreed
