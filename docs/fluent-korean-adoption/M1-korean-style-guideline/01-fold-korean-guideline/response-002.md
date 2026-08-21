# Maintainer response — Round 002

- F2 (medium): Accept, and take the reviewer's suggested direction. The permission notice
  must be included, not linked, and the tracked notice file is the right vehicle: it
  costs zero model context (nothing loads it) and survives upstream URL churn.
  Implementation is split because this unit's review allowlist may not grow between
  rounds: the new files (THIRD-PARTY-NOTICES.md, the .gitignore allow line, and the
  secret-guard pinned-list + hash-pin updates that repo golden rule couples to it) are
  task T02, its own declared review unit, implemented and under its own round. This
  unit's own change is confined to its declared files: the credit line in both sections
  now references the tracked THIRD-PARTY-NOTICES.md instead of the mutable upstream URL.
- F1/F3: verified fixed by the reviewer this round; nothing further.
- F4: unchanged — the unit's E2E step captures live Korean generation from both agents
  before the unit closes.

Verification after fixes: Korean block re-extracted from both files and diffed —
IDENTICAL; `bash tests/secret-guard.sh` → PASS (with T02's widened allowlist staged).
