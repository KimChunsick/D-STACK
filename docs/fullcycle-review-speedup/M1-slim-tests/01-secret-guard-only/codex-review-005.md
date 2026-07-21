# Codex adversarial review — Round 005

## Review scope
Re-review

## GPT findings
[severity:medium][security] The Round 4 suffix repair still allows protected backup filenames to bypass both ignore and tracked-tree checks.
Evidence: `SECRET_RE` accepts `_token` with at most one alphanumeric extension and requires `.pem`, `.token`, `.key`, database suffixes, and similar families at the end of the pathname. Meanwhile, `.gitignore` has only exact rules such as `**/*.pem` and `**/*.token`. Thus `claude/skills/full-cycle/private.pem.bak` is addable and missed by section 5. `api_token.json.bak` is ignored by `**/*_token.*`, but after `git add -f` it is missed by section 4. Round 4’s verification requested token backup names but tested only `api_token.json`.
Suggested direction: Define explicit suffix-chain semantics at the sensitive-basename boundary and apply them consistently to `.gitignore`, indexed paths, and addable paths. Include representative backup and compound-extension variants in both ordinary-add and force-add verification.
Illustrative example:
```text
private.pem.bak     -> not ignored -> section 5 misses -> PASS
api_token.json.bak -> force-added -> section 4 misses -> PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable clone, create zero-byte `private.pem.bak`, `secrets.token.old`, `api_token.json.bak`, and database-journal backup variants inside a wholesale-allowed subtree. Test each while untracked and after `git add -f`; every protected variant must fail.

[severity:medium][software structure/design] The sole retained guard does not verify that the script being executed is the version staged for the commit.
Evidence: Section 0 compares only `.gitignore` with its index blob. A safe worktree copy of `tests/secret-guard.sh` can therefore be executed while the index contains a weakened version or a staged deletion. The safe copy’s pathname does not match `SECRET_RE`, so an untracked restored copy also survives section 5. The guard can report success immediately before a commit removes or weakens the repository’s only remaining meta control.
Suggested direction: At section 0, require `tests/secret-guard.sh` to exist in the index and compare its index blob byte-for-byte with the executing worktree file. Intentional guard edits should pass once the same bytes are staged.
Illustrative example:
```text
index:     tests/secret-guard.sh deleted
worktree:  restored safe copy
execution: safe worktree copy reports PASS
commit:    records no guard
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Separately stage deletion and a pass-only weakened version, restore the safe script only in the worktree, and run it. Both states must fail before probing; an intentional identical staged/worktree edit must still pass.

[severity:low][UI/UX & DX] The new runtime-directory matcher globally prohibits ordinary repository directories named `projects`, `memory`, or `sessions`.
Evidence: `(^|/)(sessions|projects|memory)(/|$)` is applied to every indexed and addable pathname without limiting it to agent runtime-state roots. Consequently, a benign file such as `docs/projects/review.md` or `docs/memory/model.md` fails the guard even though it is documentation, not runtime state. This introduces avoidable maintenance friction into a change whose stated purpose is reducing such friction.
Suggested direction: Scope runtime-directory patterns to the repository locations where those names represent agent state, while retaining filename-family checks globally.
Illustrative example:
```text
docs/projects/roadmap.md       -> allow
claude/runtime/projects/state  -> reject
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add benign documentation beneath each generic directory name and corresponding runtime-state paths beneath supported agent roots. Confirm only the runtime-state cases fail.

GPT verdict: reject — Compound backup names can bypass the security invariant, and the commit can remove or weaken the sole retained guard while a different worktree copy reports success.

## Maintainer response
1. **Agreed (compound/backup suffixes) — fixed with explicit suffix-chain semantics.**
   Every name family in `SECRET_RE` now tolerates a dot-suffix chain
   (`(\.[a-z0-9]+)*$` on the pem/key/p12/pfx/token/secret family, the `_token`
   family, and the db/sqlite families), and `.gitignore` gained the matching chain
   denies (`**/*.pem.*`, `**/*.key.*`, `**/*.p12.*`, `**/*.pfx.*`, `**/*.token.*`,
   `**/*.secret.*`, `**/*.sqlite.*`, `**/*.sqlite3.*`, `**/*.db.*`, `**/*.db3.*`);
   the ignore battery gained `private.pem.bak`, `secrets.token.old`,
   `api_token.json.bak`, `cache.db.bak` as wholesale-subtree probes. Verified
   (scenario S): all four backup variants force-added each fail the guard; as
   untracked files they are now ignored (probed by the battery), so both requested
   states are protected.
2. **Agreed (guard self-integrity) — fixed.** Section 0 now byte-compares
   `tests/secret-guard.sh` itself against its index blob, so a staged deletion or
   weakened staged copy cannot hide behind a safe worktree copy; intentional edits
   pass once identical bytes are staged. Verified (scenario T): staged deletion and
   weakened-staged-copy both fail with the self-check message; the baseline (same
   bytes staged) passes.
3. **Rebutted with repo evidence (scoping) — reverted to global matching, which the
   repo's own pre-existing policy requires.** The benign-file premise does not hold
   here: `.gitignore` has hard-denied `**/sessions/`, `**/projects/`, `**/memory/`
   repo-wide since before this task, so `docs/projects/review.md` cannot even be
   added without `-f` — there is no benign tracked file for the scoped regex to
   spare, and an agent-root-scoped regex would be NARROWER than the deny policy it
   mirrors (a force-added `docs/projects/x` would pass the guard while violating the
   policy). The global family matcher is the consistent shape. Verified (scenario U):
   `git check-ignore` confirms the pre-existing deny; the force-added file fails the
   guard.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Runtime-dir families (sessions/projects/memory) match GLOBALLY, mirroring the
  repo's pre-existing repo-wide deny policy (R5-3 rebuttal; friction, if it ever
  matters, is a policy change to propose to the user, not a guard exception).
- Content-level scanning excluded by user decision (R2-1); single-user TOCTOU and
  per-machine addable-scan verdicts accepted (R2-3, R3-2); prohibition remains the
  nested-ignore invariant, case-insensitively (R4-2).

Consensus: disagreed
