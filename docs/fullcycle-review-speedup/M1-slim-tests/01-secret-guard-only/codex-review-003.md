# Codex adversarial review — Round 003

## Review scope
Re-review

## GPT findings
[severity:medium][security] Line-oriented parsing of quoted `git ls-files` output permits indexed secret files and nested `.gitignore` files to bypass anchored checks.
Evidence: Both `nested_ix="$(git ls-files | grep -E '/\.gitignore$' ...)"` and the tracked-tree scan consume newline-delimited, potentially C-quoted filenames. A valid path containing a newline is emitted with a trailing quote, so an indexed path such as `odd\n/cache.token` ends with `cache.token"` and does not match `\.token$`. Likewise, a staged-then-worktree-deleted `odd\n/.gitignore` does not match `/\.gitignore$`, despite remaining in the commit’s index.
Suggested direction: At the index-inspection boundary, consume `git ls-files -z` records with a NUL-safe Bash loop and match each unquoted pathname individually. Apply the same invariant to nested-ignore detection and the tracked-secret scan.
Illustrative example:
```text
index pathname:  claude/skills/full-cycle/odd<LF>/cache.token
ls-files text:   ".../odd\n/cache.token"
anchored regex:  misses because the serialized line ends with `"`
current result:  PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Force-add secret-suffixed files beneath newline-containing directories and stage nested `.gitignore` files beneath such directories before deleting their worktree copies. Every case must make the repaired guard fail; repeat with ordinary and non-ASCII path components.

[severity:medium][security] The ignore policy remains case-sensitive on case-sensitive systems, while the guard does not inspect actual untracked files outside `claude/agents/`.
Evidence: The supplied deny rules and ignore probes use lowercase forms such as `**/*.sqlite3-*`, `**/*deploy_key*`, and `cache.sqlite3-wal`. With `core.ignoreCase=false`, `claude/skills/full-cycle/CACHE.SQLITE3-WAL` is addable because that named skill directory is wholesale-allowed. Section 1 tests only fixed lowercase names, section 2 enumerates actual untracked files only under `claude/agents/`, and section 4 examines only indexed files. Consequently, the guard can pass while the uppercase runtime file is currently trackable, contradicting the retained control’s central invariant.
Suggested direction: Either make sensitive `.gitignore` patterns explicitly case-fold-safe or NUL-safely inspect every untracked addable pathname with the same case-insensitive matcher used for indexed files. Add mixed-case probes at the `.gitignore`/guard policy boundary.
Illustrative example:
```text
case-sensitive clone
claude/skills/full-cycle/CACHE.SQLITE3-WAL
  -> not ignored
  -> not indexed
  -> absent from every current whole-tree scan
  -> guard passes while the file is addable
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a case-sensitive clone with `core.ignoreCase=false`, create mixed-case variants of every declared secret/runtime family in each wholesale-allowed subtree. The repaired guard must reject them both before staging and after force-adding them.

[severity:low][UI/UX & DX] The claimed sabotage verification is not preserved in the task artifact.
Evidence: The task’s “E2E verification” section is empty and all gate-status boxes remain unchecked, while the maintainer response claims a 19-scenario battery was recorded and describes its implementation as a transient scratchpad. An independent reviewer therefore cannot reproduce or audit the asserted results from the supplied deliverables.
Suggested direction: Record the scenario inputs, expected exit statuses, preservation assertions, and actual results in the task’s E2E section. A compact command table is sufficient; the retired test suite need not be restored.
Illustrative example:
```text
scenario                     expected       observed
staged/worktree divergence   exit 1         exit 1
symlinked ancestor           no writes      no writes
clean repository             exit 0         exit 0
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: A fresh reviewer should be able to recreate every claimed sabotage case solely from the recorded artifact and obtain the documented status and filesystem-preservation results.

GPT verdict: reject — The guard still has reproducible pathname-serialization and case-sensitivity gaps that violate its name/trackability security invariant.
tokens used

## Maintainer response
1. **Agreed (C-quoted pathnames) — fixed.** Both index inspections are now NUL-safe
   per-path loops over `git ls-files -z`: the nested-`.gitignore` index check matches
   each raw pathname with a shell `case` pattern, and the tracked-tree scan greps each
   raw pathname individually — the C-quoted serialized form is never matched. Verified:
   a tracked `odd\n…/cache.token` and an index-only `odd\n…/.gitignore` (staged, then
   worktree-deleted) each fail the guard (battery scenarios M and O).
2. **Agreed (case sensitivity) — fixed via the actual-trackability boundary, and the
   family definition is now single-sourced.** A new section 5 scans every untracked
   ADDABLE pathname (`git -c core.excludesFile=/dev/null ls-files -o --exclude-standard
   -z`, NUL-safe) against the same `SECRET_RE` used for the index scan, with `grep -i` —
   so an upper/mixed-case family variant that case-sensitive ignore rules miss is caught
   the moment it is addable, on any filesystem/`core.ignoreCase` combination, without
   attempting non-portable case-folding inside `.gitignore` patterns. This also
   completes R2-2's "define families once" direction: one `SECRET_RE` drives both scans.
   Verified: `claude/skills/full-cycle/CACHE.SQLITE3-WAL` with `core.ignoreCase=false`
   fails the guard (scenario N; on ignore-case systems git itself already ignores the
   variant, which the scan's addable-boundary framing makes correct per machine).
3. **Agreed (record) — done.** The task's E2E section now embeds the exact rerunnable
   battery script (22 scenarios), the observed transcript, and the clean-run result, so
   a fresh reviewer can reproduce every case from the artifact alone.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Content-level secret scanning excluded by user decision (R2-1; docs narrowed).
- Accepted residuals: single-user TOCTOU window (R2-3); the addable-scan verdict is
  per-machine by construction — on an ignore-case checkout git itself prevents the
  add, on a case-sensitive checkout the scan rejects (R3-2).
- Prohibition remains the nested-ignore invariant (R1).

Consensus: disagreed
