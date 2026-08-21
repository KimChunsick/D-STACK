## Carried decisions — Round 002
- F1 (medium, precedence contradiction): verified FIXED by round 002.
- F3 (low, incomplete example sentences): verified FIXED by round 002.
- F4 (low, no live-generation verification): deferred to the unit's E2E step by pipeline
  order; not refiled by round 002; the unit does not close without that capture.
- F2 (medium, MIT notice): ACCEPTED, fixed per the reviewer's suggested direction, split
  across two units because this unit's allowlist may not grow. Task T02 (its own review
  unit, declaration: THIRD-PARTY-NOTICES.md, .gitignore, tests/secret-guard.sh) adds a
  tracked THIRD-PARTY-NOTICES.md whose content is the upstream MIT license verbatim
  (copyright line "Copyright (c) 2026 snflkd" plus the full permission and warranty
  paragraphs, 1,067 bytes fetched from the upstream LICENSE), gitignore-allowlisted with
  the secret-guard pinned list and hash pin updated in the same change, guard green. In
  THIS unit's declared files the credit line now reads "(Copyright (c) 2026 snflkd, MIT
  License. 허가 조항 전문은 이 파일이 있는 저장소의 THIRD-PARTY-NOTICES.md에 있다)" —
  the mutable upstream URL is replaced by the tracked in-repo file reference.
- Standing context: no-new-tests repo policy (direct-run verification recorded in
  task.md); maintainer explicitly authorized the MIT-credited adaptation.

Consensus: disagreed
