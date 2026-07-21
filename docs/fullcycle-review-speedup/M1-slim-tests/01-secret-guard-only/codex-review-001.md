# Codex adversarial review — Round 001

## Review scope
Adversarial review

## GPT findings
[severity:high][security] The guard overwrites and deletes existing secrets, runtime state, or tracked files at its probe paths.
Evidence: Every probe executes `: > "$f"` without checking whether the path already exists, records it in `created`, and later runs `rm -f "${created[@]}"`. Running the guard with an ignored `claude/auth.json` therefore truncates and removes that file. A tracked probe file is damaged before the index check reports failure, and shell redirection follows symlinks, potentially truncating a target outside the repository.
Suggested direction: Test ignore behavior without creating files where possible, and run probes requiring physical files inside an isolated disposable fixture. Never modify a pre-existing path, and clean up only objects proven to have been created by the guard.
Illustrative example:
```text
pre-existing path -> preserve and inspect only
absent probe       -> create inside isolated fixture -> remove fixture
symlink            -> reject without following
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable repository, place sentinel bytes at an ignored probe path, repeat with a tracked probe and a symlink to an external sentinel, then run every success and failure path. Assert that file contents, symlink targets, index state, and directory structure remain unchanged.

[severity:high][technical correctness] The pre-commit check validates working-tree ignore rules rather than the rules staged for the commit.
Evidence: `grep '^!' .gitignore` and `git check-ignore` consume the working-tree `.gitignore`, while `git ls-files` examines the index. A dangerous negation can be staged and then removed only from the working-tree copy; the guard sees the safe copy and passes, but `git commit` records the dangerous staged version.
Suggested direction: Define the staged index as the checked invariant. Either construct the ignore test from the staged snapshot or reject worktree/index divergence for the guard and every applicable ignore file before validation.
Illustrative example:
```text
index .gitignore:    !/claude/agents/rogue.md
working .gitignore:  pinned safe rules
current guard:       PASS
subsequent commit:   unsafe staged rule is recorded
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable fixture, stage an extra negation, restore only the working-tree copy to the pinned rules, and run the guard. The repaired guard must reject this state and must validate a clean staged safe state successfully.

[severity:medium][software structure/design] The claimed closed negation set covers only the root `.gitignore`; lower-level ignore files can reopen protected paths without changing the pinned text.
Evidence: `got_negations="$(grep '^!' .gitignore)"` examines exactly one file. The supplied policy says files inside named skill directories are included wholesale, so a nested `.gitignore` can exist there; Git gives lower-level rules precedence for paths beneath that directory. None of the code enumerates nested ignore files or pins their negations.
Suggested direction: Establish a repository-wide invariant: either prohibit nested `.gitignore` files in allowed trees or enumerate and validate every effective ignore source. Apply that invariant to the commit snapshot rather than only the current root file.
Illustrative example:
```text
root .gitignore: unchanged
claude/skills/full-cycle/.gitignore: !local.sqlite-wal
result: protected nested path becomes addable without root-pin drift
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add a nested ignore file and an otherwise ignored probe in a disposable repository. Confirm that Git considers the probe addable, then verify that the repaired guard rejects the nested negation.

[severity:medium][security] The tracked-tree scan misses runtime-file variants that the guard itself identifies as sensitive.
Evidence: The probe battery explicitly treats `x.sqlite-wal` as runtime state, but the final tracked-path regex accepts only names ending in `.sqlite`, `.sqlite3`, `.db`, or `.db3`. An already tracked or force-added path such as `codex/cache.sqlite-wal` is not one of the exact probe paths and does not match the final regex, so the guard can pass while that runtime file is in the index.
Suggested direction: Centralize the sensitive-path definitions so ignore probes and tracked-tree scanning cover the same filename families, including SQLite journal/WAL/SHM variants. Add regression fixtures for every declared family rather than relying on one exact pathname.
Illustrative example:
```text
tracked: codex/cache.sqlite-wal
exact probe: claude/x.sqlite-wal
final regex: no match
current result: PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable repository, stage representative `.sqlite-wal`, `.sqlite-shm`, and `.sqlite-journal` paths outside the hard-coded probe locations. Each must make the repaired guard fail.

GPT verdict: reject — The retained safety control can destroy protected data and can approve commit states that bypass its core secret-trackability invariant.

## Maintainer response
All four findings are agreed. All are inherited verbatim from the deleted
`test_gitignore_secret_guard.sh`, but the task promotes that script to the repo's only
control, so hardening it is in scope. Each fix was applied class-wide, and one adjacent
same-class defect the review did not name was found and fixed during the sweep.
`tests/secret-guard.sh` was rewritten:

1. **Agreed (destructive probes).** Ignore-battery probes no longer create files at
   all — `git check-ignore` evaluates pattern coverage for nonexistent paths, so
   section 1 is now purely name-based. Physical files remain only for the
   `git ls-files -o` addable-check under `claude/agents/` (existence is required
   there); each such probe first *refuses* any pre-existing path or symlink
   (`[ -e ] || [ -L ] → fail`, before any redirection), and cleanup removes exactly
   the files/dirs the run created. Verified per the requested scenario battery in
   disposable clones: pre-existing `claude/agents/auth.json` with sentinel bytes →
   guard fails, contents intact; symlinked probe to an external sentinel → guard
   fails, target intact; clean run leaves zero residue (files and dirs).
2. **Agreed (worktree vs staged).** New section 0 rejects staged/worktree divergence
   of `.gitignore` (`git diff --quiet -- .gitignore`) before any validation, so the
   verdict always describes the rules a commit would record. Verified: rogue negation
   staged + safe worktree copy → guard fails. (The pin itself still reads the worktree
   copy, which divergence-rejection has just proven identical to the index.)
3. **Agreed (nested ignore sources).** The repo-wide invariant chosen is *prohibition*:
   exactly one ignore file may exist. Section 0 fails on any nested `.gitignore` in
   the worktree (`find`) or in the index (`git ls-files`, which also catches
   staged-then-deleted). Class-swept beyond the finding: local `.git/info/exclude`
   content is rejected and every `check-ignore`/`ls-files -o` call runs with
   `core.excludesFile=/dev/null`, so machine-local rules can neither mask a missing
   repo rule nor hide an addable probe — without this, a global gitignore could fake a
   pass that does not transfer to other machines/CI. Verified: nested worktree file,
   index-only nested file, info/exclude rule, globally-masked structural deny, and
   globally-hidden addable probe are all rejected.
4. **Agreed (journal variants).** Tracked-tree regex now covers the DB-journal family
   via `\.(db[0-9]?|sqlite[0-9]?)(-[a-z]+)?$`; probe battery gained
   `claude/y.sqlite-shm` and `claude/data.db-wal`; `.gitignore` deny list gained
   `**/*.db-*` (the sqlite variant already existed — same class, other spelling).
   Verified: force-added `codex/cache.sqlite-wal`, `cache.db-wal`,
   `cache.sqlite-journal` each fail the guard.

Evidence: sabotage battery (13 scenarios, all behaving) recorded in the task's E2E
section; clean-tree run passes; probe cleanup verified.

Claimed fixes are not yet independently reviewed — sealing this round for re-review.

## Carried decisions
- All four R1 findings fixed in `tests/secret-guard.sh` (rewrite); verification
  battery exists as a transient scratchpad script (the meta suite is retired by
  user decision — interview Q3; the battery is rerunnable and recorded in task.md).
- Prohibition (not enumeration) chosen as the nested-ignore invariant.
- Residual accepted (pre-existing, unchanged): the guard is a manual pre-commit
  tripwire, not a hook; running it is still a convention.

Consensus: disagreed
