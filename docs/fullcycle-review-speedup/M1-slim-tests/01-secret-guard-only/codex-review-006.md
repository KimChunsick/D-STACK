# Codex adversarial review — Round 006

## Review scope
Re-review

## GPT findings
[severity:medium][security] The Round 5 matcher repair still permits force-added files beneath protected directory components and protected names with non-alphanumeric backup suffixes.
Evidence: `.gitignore` patterns such as `**/*_token`, `**/*.pem.*`, and `**/*.db-*` match directories or suffixes containing characters like hyphens. `SECRET_RE`, however, anchors `_token` and extension families to the complete pathname and restricts suffix components to `[a-z0-9]+`. Consequently, `claude/skills/full-cycle/api_token/payload.txt`, `private.pem.backup-2026`, and `cache.db-wal.backup-2026` do not match section 4. After `git add -f`, section 5 also cannot see them because they are indexed. The fixed probe battery contains no equivalent cases, so the guard can report success.
Suggested direction: At the `SECRET_RE` boundary, match protected names as complete path components, not only as the final filename. Make suffix handling equivalent to the corresponding `.gitignore` glob semantics, preferably through one component-level policy shared by indexed and addable scans.
Illustrative example:
```text
.gitignore:  **/*_token
index (-f):  claude/skills/full-cycle/api_token/payload.txt
section 4:   `_token...$` does not match
section 5:   already indexed
result:      PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In disposable clones, first confirm the three example paths are ignored, then force-add each and run the guard. Every case must fail; repeat with mixed-case component names and punctuation-bearing backup suffixes.

GPT verdict: reject — The sole retained control still has a reproducible tracked-path bypass within filename families it explicitly claims to protect.

## Maintainer response
**Agreed — fixed by moving to component-level matching, the same semantics
`.gitignore` globs already have.** Sections 4 and 5 now split every pathname into
components (pure parameter-expansion split — `read`-based splitting silently stops at
embedded newlines, a regression the battery caught) and match each component against
one `COMP_RE`: protected names match as directory components (`api_token/payload.txt`
is caught via its dir component), suffix chains accept any characters
(`private.pem.backup-2026`, `cache.db-wal.backup-2026`), and — same-class sweep — the
`.gitignore` substring families `*credentials*` and `*password*` are now mirrored too
(they never were in the old regex). The battery's ignore probes gained the three
example paths (all ignored by existing globs), and scenario V force-adds all three
plus a mixed-case `PASSWORD-list.txt`; every case fails the guard, and the clean tree
still passes. Verified in the 38-scenario battery recorded in task.md §E2E.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Component-level `COMP_RE` is now the single family table for indexed and addable
  scans, mirroring `.gitignore` glob semantics (R6-1).
- All prior dispositions unchanged: content scanning excluded by user decision
  (R2-1); single-user TOCTOU (R2-3); per-machine addable verdict (R3-2);
  case-insensitive nested-ignore prohibition (R4-2); global runtime-dir families per
  the repo-wide deny policy (R5-3 rebuttal).

Consensus: disagreed
