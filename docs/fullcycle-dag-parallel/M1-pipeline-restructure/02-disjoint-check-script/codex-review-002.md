# Codex adversarial review — Round 002

## Review scope
Re-review | Consolidated verify invocation (one GPT-5.6 Sol xhigh run) covering T01+T02+T03+T04, per the user's end-batched review directive (round cap ~2). Full consolidated findings reproduced below; this file's response addresses the findings tagged for THIS task. The user-set round cap means fixes below are verified mechanically (tests/suites), not by a further reviewer round.

## GPT findings
[severity:high][the real Why][T01] The reviewed HEAD is recorded but never enforced before merge, allowing declared yet unreviewed changes to enter the integrated branch.
Evidence: `before-review` permits staged, unstaged, and untracked changes when they remain within the declaration, while the review bundle contains only `base..HEAD`. Before merge, the lifecycle reruns containment but never requires a clean worktree or equality with the sealed HEAD. The path-based reopen rule covers only files already inside the sealed bundle.
Verification: Declare `src/`, commit `src/a`, leave `src/b` untracked, and seal review of the committed HEAD. Commit `src/b` afterward. The second scope check passes, `src/b` does not trigger the sealed-file rule, and the newer unreviewed HEAD can be merged.
Suggested direction: Require a clean worktree before review and require the pre-merge HEAD to equal the exact sealed HEAD; any later tree change must reopen review.

[severity:high][technical correctness][T02] Scope validation remains bypassable through caller-selected repository and base identities.
Evidence: `scope` accepts any Git directory and any commit resolving there. It does not bind the directory to the GOAL repository’s common Git directory, require the worktree root, verify the expected task branch, authenticate the recorded base, or require that base to precede HEAD. T04 names a recorded base but concretely requires verification only of the worktree path and branch.
Verification: Put undeclared commits in worker worktree A, then invoke scope against a clean sibling worktree B, or invoke it against A using A’s current HEAD as the base. Both produce an empty committed diff and can return `PASS`.
Suggested direction: Bind scope to immutable orchestrator-owned task state containing repository identity, canonical worktree root, exact branch, recorded base, and expected HEAD.

[severity:high][security][T02] A worker-created symlink still permits traversal outside the worktree.
Evidence: Declaration checks inspect only components existing in the GOAL repository before work starts. In scope mode, `check_contained` explicitly exempts the final path component from symlink rejection, and a directory declaration accepts its bare directory name as contained. Git cannot enumerate files subsequently written through that symlink.
Verification: Declare `files: [escape/]` while `escape` does not exist, then create and commit `escape` as a symlink to an external directory and write `escape/output`. Scope sees the declared `escape` entry, accepts it, and never sees the external output.
Suggested direction: Reject newly created symlinks wherever a declaration grants descendant ownership, and validate actual path types against the reviewed tree.

[severity:medium][technical correctness][T02] The parser still does not reliably isolate peer task rows from Markdown examples or nested content.
Evidence: Fence state is tracked only after the parser has entered the milestones section, so a `## Milestones & tasks` heading inside an earlier fenced example starts parsing. `re_task` also accepts arbitrary indentation and all three list markers despite the documented column-zero peer-row grammar.
Verification: Place a complete example milestones section inside a fenced block before the real section, or place an indented task-shaped checkbox beneath a real task. The example or nested row becomes executable graph data, potentially producing phantom tasks or blocking `INVALID` results.
Suggested direction: Track Markdown fences globally and accept task rows only at one exact peer indentation and marker level within one exact section.

[severity:medium][technical correctness][T02] Git enumeration failures fail open in scope mode.
Evidence: Both `git diff` and `git status` run inside process substitutions with stderr discarded. Their exit statuses are never checked, and the script prints `PASS` unconditionally after the loops.
Verification: Corrupt or make the worktree index unreadable while an undeclared working-tree file exists. `git status` exits nonzero without records, the loop observes no offending path, and scope can return `PASS`.
Suggested direction: Treat any nonzero diff or status enumeration result as `INVALID` before issuing a containment verdict.

[severity:medium][security][T04] The HOW/WHAT split makes command-bearing repository content authoritative without an immutable side-effect boundary.
Evidence: `CONTRIBUTING`, instruction files, and lint/build configurations are declared authoritative for “HOW,” and the workflow directs the agent to run their checks through unrestricted Bash. The immutable boundaries prohibit out-of-scope file writes and limited forms of secret reproduction, but do not prohibit publishing, deployment, network transmission, destructive commands, or other external side effects.
Verification: A repository instruction or build script can require an upload, deployment, or network command as a validation step without changing the declared file scope. The worker is instructed to follow it as authoritative HOW.
Suggested direction: Limit repository authority to coding conventions and require explicit parent authorization for command-bearing instructions with destructive, publishing, credential, or external effects.

GPT verdict: reject — unreviewed commits, caller-substitutable scope identities, and newly created symlink traversal leave concrete paths for unsafe fan-out and unreviewed code to pass the gates.
## Maintainer response
Addresses the [T02]-tagged findings (identity substitution, worker-created symlinks,
parser isolation, fail-open enumeration).
1. Identity substitution: fixed for every mechanically checkable identity — scope now
   REQUIRES the task branch argument and verifies the worktree is on it; verifies the
   worktree belongs to the GOAL.md's repository (same resolved git common dir);
   verifies the recorded base is an ancestor of HEAD; and requires a CLEAN tree, so
   the reviewed identity is exactly committed base..HEAD. Tests: wrong branch →
   INVALID; foreign-repo GOAL → INVALID; non-ancestor base → INVALID; unclean tree →
   VIOLATION. Residual, explicitly accepted and recorded in the script header and
   SKILL.md: the base/branch VALUES come from the orchestrator's own records — the
   checker is a mistake-tripwire for one honest orchestrator, not a security boundary
   against that orchestrator falsifying its own records; this is the same
   self-attestation scope the repo's Stop hook already documents. (Disposed under the
   user's round cap; flagged for ratification in the final report.)
2. Worker-created symlinks: agreed, fixed. Directory-ownership containment now
   rejects a committed path that is a symlink (`VIOLATION: ... symlink under
   directory ownership`) — ownership grants files, never link indirection; an
   EXACT-path declaration may still be a knowingly-declared symlink. Clean-tree plus
   committed-only enumeration closes the uncommitted variant. Test: committed
   `lib/util/esc` symlink under declared `lib/util/` → VIOLATION.
3. Parser isolation: agreed, fixed. Fences are tracked globally from line one (a
   section heading inside a fenced example can no longer start parsing) and task rows
   are accepted only at column zero with the `-` marker (the documented peer-row
   grammar); indented or alternate-marker lookalikes are inert. Tests: fenced
   pre-section example ignored; indented lookalike inert and its id unknown.
4. Fail-open enumeration: agreed, fixed. Both git enumerations write to temp files
   with exit-status checks — any failure is INVALID before any containment verdict.
Verification: tests/check-parallel.test.sh — 40 cases green on brew bash and
/bin/bash 3.2.

## Carried decisions
- Orchestrator-record authenticity residual: accepted, recorded (see above).
- Gitignored-file residual stands as recorded (round 001).
- Per the user's round cap, no further reviewer round; fixes verified by the suite.

Consensus: resolved
