# Codex adversarial review — Round 004

## Review scope
Re-review

## GPT findings
[severity:medium][security] The claimed single-sourced sensitive-name policy still misses declared secret and runtime families when they have additional suffixes.
Evidence: `SECRET_RE` recognizes `_token` only at the end of the complete pathname. Consequently, `claude/skills/full-cycle/api_token.json` does not match. It also contains no rule for the “sessions” or “per-project memory” that README claims are never trackable. After `git add -f`, these paths are examined by section 4 but not matched, while section 5 excludes them because they are already indexed. The fixed probe list tests only exact neighboring names.
Suggested direction: At the `SECRET_RE`/`.gitignore` policy boundary, define each protected basename family and its allowed suffix behavior explicitly, then validate the same family table against ignored, addable, and indexed paths. Align the documented runtime families with that table.
Illustrative example:
```text
git add -f claude/skills/full-cycle/api_token.json
section 4: `_token$` does not match
section 5: path is already indexed
result: PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable clone, test untracked and force-added variants such as `api_token.json`, token backup names, session state, and per-project memory paths outside the fixed probes. Every documented protected family must fail in both states.

[severity:medium][security] Mixed-case `.gitignore` aliases bypass the repository-wide nested-ignore prohibition on case-insensitive filesystems.
Evidence: The worktree check uses case-sensitive `find ... -name .gitignore`, and the index loop matches only `*/.gitignore`. On a case-insensitive filesystem, Git’s lookup of `.gitignore` can resolve a file named `.GITIGNORE`, while both guard checks miss that pathname. A staged `claude/agents/.GITIGNORE` containing `!rogue.md` can therefore reopen and stage `rogue.md`; once both files are indexed, the addable scan does not see them and neither pathname matches `SECRET_RE`.
Suggested direction: Reject every ASCII-case variant of `.gitignore` in both the worktree and index, regardless of the current filesystem, so commits remain transferable across machines. Apply this at section 0 before any probes.
Illustrative example:
```text
case-insensitive checkout
claude/agents/.GITIGNORE: !rogue.md
indexed: .GITIGNORE + rogue.md
current nested-source checks: no match
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: On an actual case-insensitive volume, repeat with `.GITIGNORE` and `.GitIgnore`, confirm Git applies their negations, and verify the repaired guard rejects both worktree-only and indexed variants before creating probes.

[severity:low][technical correctness] The recorded divergence and assume-unchanged scenarios do not create the states they claim to test.
Evidence: In scenario A, the rogue copy is staged and then `git checkout -- .gitignore` restores the working tree from that same rogue index entry, not from the safe version. Scenario L makes the identical mistake with `git restore .gitignore`, whose default source is also the index. Both states therefore have matching rogue index/worktree bytes and can fail solely at the negation pin; they do not verify the byte-comparison fix. The runner also hardcodes `/Users/won/Desktop/Workspace/D-STACK`, contradicting “rerunnable as-is” from the artifact alone.
Suggested direction: Preserve a known-safe worktree copy separately, stage the rogue copy, restore the preserved safe bytes, and assert divergence before invoking the guard. Parameterize the source repository instead of embedding a maintainer-specific path.
Illustrative example:
```text
index after git add: rogue
git restore .gitignore: copies rogue index -> worktree
actual state: index == worktree
claimed state: index != worktree
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Before running the guard, assert that `git show :.gitignore | cmp -s - .gitignore` returns nonzero and that only the index contains the rogue rule; then assert failure occurs in section 0. Run the battery from an arbitrary clone without editing its source.

GPT verdict: reject — Reproducible filename-family and mixed-case ignore-source bypasses still allow prohibited paths to enter the index.

## Maintainer response
1. **Agreed (family suffixes and documented runtime dirs) — fixed.** `SECRET_RE` token
   family widened to `_token(\.[a-z0-9]+)?$` and the documented runtime-dir families
   added as `(^|/)(sessions|projects|memory)(/|$)`; `.gitignore` gained the matching
   `**/*_token.*` deny; the ignore battery gained wholesale-subtree probes for
   `api_token.json`, `sessions/state.json`, `memory/notes.md`, `projects/p/x.md`. The
   README-documented families and the guard's table now agree. Verified (scenarios
   P/Q): force-added `api_token.json`, `sessions/state.json`, `memory/notes.md` each
   fail the guard; clean tree passes.
2. **Agreed (mixed-case ignore aliases) — fixed.** Both ignore-source checks are now
   case-insensitive regardless of the current filesystem: the worktree check uses
   `find -iname .gitignore` (root exact-case `./.gitignore` excluded), and the index
   loop lowercases each pathname before matching, rejecting any case variant anywhere
   (including a root-level variant that is not exactly `.gitignore`). Verified
   (scenario R): `claude/agents/.GITIGNORE` fails the guard both worktree-only and
   index-only.
3. **Agreed (battery defects) — fixed.** Scenarios A and L now construct the claimed
   state: the safe bytes are preserved outside the repo before staging the rogue copy,
   restored afterwards, the divergence precondition is asserted
   (`git show :.gitignore | cmp -s - .gitignore` must be nonzero), and the guard's
   failure is required to come from section 0 (its "differs between index and working
   tree" message), not the negation pin. The runner is parameterized
   (`${1:-git rev-parse --show-toplevel}`) instead of hardcoding the maintainer path.
   The updated script and its transcript replace the old ones in task.md §E2E.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Content-level secret scanning excluded by user decision (R2-1; docs narrowed).
- Accepted residuals: single-user TOCTOU window (R2-3); per-machine addable-scan
  verdict (R3-2).
- Prohibition remains the nested-ignore invariant, now case-insensitive (R1, R4-2).

Consensus: disagreed
