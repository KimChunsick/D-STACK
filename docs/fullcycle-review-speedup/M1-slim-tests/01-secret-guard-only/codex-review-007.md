# Codex adversarial review — Round 007

## Review scope
Re-review

## GPT findings
[severity:medium][security] The Round 6 component matcher still does not mirror `.gitignore` wildcard semantics, leaving a force-add bypass for protected database-journal names.
Evidence: `.gitignore` uses `**/*.db-*` and equivalent SQLite patterns, where `*` accepts any sequence other than `/`, including additional hyphens and punctuation ([Git documentation](https://git-scm.com/docs/gitignore)). `COMP_RE` instead permits only `-[a-z0-9]+` followed by an optional dot-prefixed suffix. Therefore `cache.db-wal-backup-2026` is ignored by `.gitignore`, but after `git add -f` it fails section 4’s matcher because `-backup-2026` cannot be consumed; section 5 cannot inspect it once indexed.
Suggested direction: At the `COMP_RE` boundary, make journal suffix handling equivalent to the corresponding component-level glob rather than enumerating permitted characters. Keep one predicate for indexed and addable scans and add adversarial cases for every wildcard boundary.
Illustrative example:
```text
.gitignore: **/*.db-*
index:      cache.db-wal-backup-2026
COMP_RE:    matches through ".db-wal", then stops at "-"
result:     guard passes
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In disposable clones, confirm `cache.db-wal-backup-2026`, `cache.sqlite-shm_copy`, and `cache.db-` are ignored; force-add each and require the guard to fail. Repeat with mixed case and embedded newlines in the wildcard portion.

[severity:medium][technical correctness] The staged-state integrity checks compare blob bytes but not index entry types, so a staged symlink can disable the ignore policy while the guard passes against a regular worktree file.
Evidence: Both section 0 checks use `git show :path | cmp`, which observes blob content but not the index mode. Git records file type separately and permits symbolic-link entries; `git update-index --cacheinfo` can directly stage such an entry ([Git index format](https://git-scm.com/docs/index-format.html), [update-index documentation](https://git-scm.com/docs/git-update-index/2.43.0)). Staging `.gitignore` with mode `120000` and its existing safe blob while retaining an identical regular worktree file makes the byte comparison pass. The guard then validates the regular worktree rules, but the commit records a symlink; Git explicitly does not follow symlinks as working-tree `.gitignore` files ([Git documentation](https://git-scm.com/docs/gitignore)). The same omission can commit the sole guard as an unusable symlink.
Suggested direction: In section 0, require exactly one stage-zero regular-file index entry with the intended mode for both `.gitignore` and `tests/secret-guard.sh`, then perform the existing byte comparison. Treat symlink, gitlink, and unmerged modes as failures.
Illustrative example:
```text
index:     120000 <safe-blob> .gitignore
worktree: regular file with <safe-blob>
byte cmp:  PASS
commit:    .gitignore becomes a symlink
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable clone, reuse each policy file’s current blob while staging mode `120000` via `git update-index --cacheinfo`; the repaired guard must reject both files before probing. Confirm ordinary stage-zero regular files with identical bytes still pass.

GPT verdict: reject — The retained control still permits a protected journal pathname into the index and can approve a commit that changes either policy file into an ineffective symlink.

## Maintainer response
1. **Agreed (glob-equivalent journal suffixes) — fixed.** The db/sqlite family in
   `COMP_RE` now consumes ANY tail once the family extension is followed by `.` or
   `-` (`\.(db[0-9]?|sqlite[0-9]?)([.-].*)?$`), exactly mirroring the `*.db-*` /
   `*.db.*`-class globs. Verified (scenario W): `cache.db-wal-backup-2026`,
   `cache.sqlite-shm_copy`, and `cache.db-` — each proven ignored by policy first,
   then force-added — all fail the guard; the clean tree still passes.
2. **Agreed (staged symlink mode) — fixed.** Section 0 now requires each policy file
   (`.gitignore` AND `tests/secret-guard.sh`) to be exactly one stage-0 regular-file
   index entry (100644/100755) before the byte compare counts; symlink (120000),
   gitlink, and unmerged entries fail. Verified (scenario X): staging each file's own
   safe blob with mode 120000 via `git update-index --cacheinfo` fails the guard with
   the entry-check message; ordinary stage-0 regular entries with identical bytes
   pass (baseline). One battery assertion updated accordingly: a staged guard
   DELETION is now caught by this earlier entry check rather than the byte compare —
   still a rejection, one check earlier.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- All prior dispositions unchanged (user-decided content-scan exclusion; TOCTOU,
  per-machine addable verdict; case-insensitive nested-ignore prohibition; global
  runtime-dir families; component-level COMP_RE as the single family table).

Consensus: disagreed
