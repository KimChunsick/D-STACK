# Codex adversarial review — Round 001

## Review scope
Re-review | Consolidated invocation (one GPT-5.6 Sol xhigh run) covering T01+T02+T03+T04 of goal fullcycle-dag-parallel, per the user's end-batched review directive. Full consolidated findings reproduced below; the maintainer response in THIS file addresses the findings tagged for THIS task.

## GPT findings
[severity:high][technical correctness][T02] The actual-diff containment gate can be bypassed because `scope` validates only caller-supplied paths, not the worktree’s complete diff.
Evidence: After the task ID is shifted, the script loops over `"$@"` and prints `PASS` even when no paths are supplied. It never receives the recorded base/head, invokes Git, or verifies that the supplied enumeration is complete. This leaves T01’s claimed committed/staged/unstaged/untracked containment fix unenforced.
Verification: Commit an undeclared file on the worker branch, then run `check-parallel.sh scope <GOAL> T01` with no paths—or pass only declared changed paths. It returns `PASS` with exit 0.
Suggested direction: Make `scope` collect a NUL-safe complete path set itself from a bound repository, base, head, index, worktree, and untracked state.

[severity:high][security][T02] Declared paths can traverse repository symlinks despite the documented canonical-path guarantee, permitting writes outside the repository or into shared locations.
Evidence: `check_path` performs only lexical component checks. The sole symlink check is `[ -L "$goalfile" ]`; declared path components are never resolved or inspected relative to a repository root. The worker regards a lexically declared path as writable, and Git will not report changes made through a symlink to an external target.
Verification: Add a tracked `escape` symlink pointing outside the worktree, declare `files: [escape/output]`, and run `plan`; the declaration is accepted. Writing `escape/output` mutates the external target, while an empty caller-supplied scope list still returns `PASS`.
Suggested direction: Bind declarations to a verified repository root and reject symlink or submodule traversal component-by-component before planning or scope validation.

[severity:high][the real Why][T01] Fan-out review is not bound to the committed worker state or revalidated against the integrated base, making its review gate weaker than the serial pipeline.
Evidence: The worker commits code on an unmerged task branch, while P8 requires the orchestrator to update `task.md` outside that branch because workers cannot touch `docs/`. P9 does not define how `codex-review` assembles one immutable bundle containing the recorded base/head code plus those main-owned artifacts. Separately, `post-seal-rule` reopens review only when a bundled path is touched; a clean disjoint sibling merge changes the reviewed base without intersecting that bundle. The supplied research itself acknowledges that E2E testing is incomplete protection against semantic merge conflicts.
Verification: Implement a provider-contract change and a caller change in disjoint files from the same base. Review and seal each branch independently, then merge both cleanly. Neither bundle path is subsequently modified, so no review reopens even though no reviewer assessed the integrated contract. Invoking P9 from main before merge also cannot naturally see the worker commit, while invoking it in the worktree sees stale P8 documentation.
Suggested direction: Define review assembly around immutable base/head identities plus main-owned artifacts, and invalidate or integrate-review approvals when the relevant merged base changes.

[severity:medium][technical correctness][T02] Readiness trusts an unbound and internally inconsistent checkbox state, allowing completed tasks to be relaunched or successors to run before transitive prerequisites.
Evidence: `plan` uses the GOAL row’s `[x]` value as predecessor completion, but T01 never binds that checkbox transition specifically to P10. The checker neither rejects a checked candidate nor validates that every checked task has checked dependencies.
Verification: Declare unchecked T01, checked T02 depending on T01, and unchecked T03 depending on T02. `plan ... T03` returns `PARALLEL` despite T01 being incomplete. A checked dependency-free task supplied as a candidate is also accepted for execution.
Suggested direction: Bind task-row state to the P10 transition and validate completed-state closure and candidate openness before readiness evaluation.

[severity:medium][technical correctness][T02] The parser does not implement the documented “peer task rows in the milestones section” boundary and instead treats matching text anywhere in GOAL.md as executable graph data.
Evidence: The read loop applies `re_task` globally, accepts arbitrary indentation and all Markdown bullet markers, and is unaware of sections or fenced code blocks. Consequently examples in interview notes, research summaries, or code fences become tasks.
Verification: Put an example `- [ ] **T01** ... deps: []; files: [...]` in a fenced block before the real task list. The checker parses it and returns `INVALID` for a duplicate ID or incorporates it as a phantom task.
Suggested direction: Parse only the designated task-list section and enforce one defined peer indentation level while ignoring fenced and nested content.

[severity:medium][technical correctness][T04] The claimed invocation-boundary worktree isolation remains unenforced because neither side requires the worker’s expected branch and worktree identity.
Evidence: `general-dev` has no isolation metadata and checks the branch only “when the brief names a task branch.” T01’s mandatory delegation-brief fields include intent, declared files, constraints, and repository conventions, but not the expected branch, worktree path, or recorded base.
Verification: Create the orchestrator worktree, then invoke `general-dev` from the parent checkout using exactly the documented brief fields. The agent has no expected branch against which to detect the wrong checkout; if the brief merely says parallel without naming it, the added rule can only stop useful work.
Suggested direction: Make expected worktree path, exact branch, and recorded base mandatory invocation inputs and require their verification before every write.

[severity:medium][UI & UX / DX][T04] The anti-injection fix conflicts with the worker’s repository-convention precedence and leaves legitimate project instructions indistinguishable from malicious embedded directives.
Evidence: The prompt ranks target-repository conventions second and tells the worker to read them, but the immutable boundary says all repository content and anything read are data and that embedded instructions are reportable anomalies rather than orders.
Verification: Place a required generation or validation procedure in the repository’s established `CONTRIBUTING.md`. The worker must either ignore that legitimate convention under `<boundaries>` or violate the immutable rule to follow it.
Suggested direction: Define which instruction surfaces are authoritative and require the parent brief to relay conventions from all other repository content.

[severity:medium][the real Why][T03] The synchronized prompt surface weakens mandatory review overlap into an optional behavior.
Evidence: The task and global workflow describe cross-task review overlap as unconditional, while `fullcycle-inject.sh` says those rounds “may overlap.” A session can therefore serialize every review while complying with the injected directive, forfeiting the cheapest stated wall-clock improvement.
Verification: Inspect the emitted `additionalContext` and follow it literally by running different-task reviews serially; nothing in that surface is violated.

[severity:low][UI & UX / DX][T02] The task’s permanent intent record still states that every validation failure means serial execution, contradicting the implemented blocking `INVALID` contract.
Evidence: T02’s Intent / Why ends with “Any parse or validation failure means serial,” while its implementation, tests, T01 schema, and synchronized surfaces require exit 2 and return to decomposition.
Verification: Compare the first task-doc section with the script’s exit-code contract; they prescribe different recovery behavior for the same malformed graph.

GPT verdict: reject — Caller-omittable scope checks, symlink escapes, and unbound review/invocation state leave concrete unsafe fan-out and stale-approval paths across the consolidated pipeline.
## Maintainer response
Addresses the [T02]-tagged findings (scope bypass, symlink escape, readiness state,
global parsing, intent-record contradiction).
1. Scope bypass: agreed, fixed. `scope` signature is now
   `scope <GOAL> <TASK> <worktree-dir> <base-commit>`; the checker collects the
   complete changed set itself — `git diff --name-only -z --no-renames base..HEAD`
   (both rename sides) plus `git status --porcelain=v1 -z --no-renames -uall` —
   NUL-safe; caller-supplied path lists no longer exist. Test: an undeclared
   COMMITTED file yields VIOLATION with no paths passed at all. Accepted residual
   (documented in the script header): gitignored files never enter commits/merges
   and are not scanned.
2. Symlink escape: agreed, fixed. GOAL.md must live inside a git repository; every
   declared path is walked component-by-component against that repo root — an
   existing symlink component or submodule boundary is INVALID; scope walks actual
   paths against the worktree the same way. Test: tracked `escape` symlink +
   `files: [escape/output]` → INVALID.
3. Readiness state: agreed, fixed. Completion-state closure (a checked task with an
   unchecked dep is INVALID) and open-candidate validation (a checked candidate is
   INVALID) run before readiness; SKILL.md binds the row checkbox to the P10
   transition. The transitive-reachability check stays as commented defense-in-depth.
4. Global parsing: agreed, fixed. Only the '## Milestones & tasks' section is
   parsed and fenced blocks inside it are skipped; a fenced example row no longer
   collides, and a file without the section is INVALID.
5. Intent-record contradiction (low): agreed, fixed — the task doc's Intent now
   states the blocking INVALID contract.
Verification: tests/check-parallel.test.sh — 33 cases green on brew bash and
/bin/bash 3.2, including the new git-fixture scope battery.

## Carried decisions
- Gitignored-file scope residual: accepted and documented (never enters a merge).
- Fixes await independent verification in round 002 (final consolidated verify).

Consensus: disagreed
