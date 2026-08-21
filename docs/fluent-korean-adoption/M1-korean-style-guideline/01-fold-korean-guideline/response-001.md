# Maintainer response — Round 001

- F1 (medium): Agree. The Slack-tone rule and the repo-style deference were stated as
  peers with no ordering. Fixed by restructuring: the 해요체 rule and the Slack-tone rule
  now stand alone, followed by an explicit precedence bullet — "위 두 규칙은 기본값이다.
  대상 저장소나 작업 지시가 문체나 표기 규칙을 정해 두었으면 (격식체 문서 규정 등)
  그쪽이 우선한다." A repo prescribing formal Korean documentation now gets exactly one
  instruction. Class-wide sweep: checked the remaining rule pairs for unstated precedence —
  the figurative-vocabulary rule carries its own inline exception; the transliteration and
  proper-noun rules were merged in the original edit; the em-dash and notation rules are
  covered by the precedence bullet's "표기 규칙" wording.
- F2 (medium): Agree with the substance. Two-part fix: (1) the inline credit now carries
  the upstream copyright notice verbatim (Copyright (c) 2026 snflkd, MIT License) with a
  link to the full license text; (2) all three example strings carried from the upstream
  document were replaced with own-authored examples, so the adaptation no longer contains
  any verbatim upstream expression — only rules restated in our own words. The full
  permission-notice paragraph is incorporated by reference rather than reproduced because
  these files are model context on every invocation; with zero remaining verbatim
  expression, a compact notice with the full text linked is proportionate. The suggested
  tracked-notice file would require widening this unit's declaration (.gitignore and the
  secret-guard negation list must change together in this repo); if the next round still
  requires it, that is a separate review unit.
- F3 (low): Agree; folded into the same edit since the exact lines were being rewritten
  for F2. Every good-side example is now a complete 해요체 sentence.
- F4 (low): Agree; this is the unit's E2E step by pipeline order (review consensus →
  live-generation capture → unit close). Recorded as follow-up in findings.md; the unit
  cannot close without the capture.

Verification after fixes: Korean block re-extracted from both files and diffed —
IDENTICAL; `bash tests/secret-guard.sh` → PASS.
