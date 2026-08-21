# Codex adversarial review — Round 002

## Review scope
Re-review | serial | bundle 34729 bytes (round 001: 28907)

## GPT findings
Verification limitation: The assigned workspace contains no Git checkout, so this review relies on the supplied snapshots and upstream primary sources.

Round 001 verification: F1 and F3 are fixed. F4 remains the explicitly deferred E2E gate and is not refiled.

[severity:medium][security] F2 remains unresolved: replacing examples does not eliminate the licensing condition for an adaptation that retains the upstream guideline's structure and closely rephrased rules.
Sites: `claude/CLAUDE.md`; confirmed: `codex/AGENTS.md`.
Evidence: The revised sections retain the upstream sequence and substance for scope, terminology, tone, sentence completion, particles, and figurative language from the source guideline (https://raw.githubusercontent.com/snflkd/fluent-korean/main/plugins/fluent-korean/output-styles/fluent-korean-not-coding.md).
Verification: The MIT license (https://raw.githubusercontent.com/snflkd/fluent-korean/main/LICENSE) requires the copyright and permission notices to be included; both diffs contain the copyright line and a mutable URL, but not the permission notice itself.
Suggested direction: Add the complete upstream MIT license to a tracked third-party-notices file and reference that file from both sections.

Omitted-detail: 0 low

GPT verdict: reject — The Round 001 license-compliance blocker remains because the required MIT permission notice is linked but not included.

## Carried decisions
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
