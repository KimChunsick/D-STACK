# 02-third-party-notices

## Intent / Why
Satisfy the MIT license condition of the adapted fluent-korean guideline by INCLUSION
rather than by link: T01's review (round 002, finding F2) held that the adaptation retains
the upstream guideline's structure and closely rephrased rules, so the license's
"copyright notice and this permission notice shall be included" condition applies, and a
mutable URL does not include anything. This task adds a tracked `THIRD-PARTY-NOTICES.md`
carrying the upstream MIT license verbatim, which the two adapted config sections then
reference (that reference edit belongs to T01's declaration, not this one).

## Deployment context
Public SSOT repo. The notice file is documentation only — no agent loads it as model
context, so it costs zero tokens per invocation. Repo golden rule: any new tracked path
needs its `.gitignore` `!`-allow line AND the matching entry in `tests/secret-guard.sh`'s
pinned negation list in the same change; the guard must stay green.

## Design consult
Skipped — no trigger (a static notice file plus two pinned-list entries).

## What was done (what / why)
Created `THIRD-PARTY-NOTICES.md` at the repo root: a provenance paragraph naming the two
adapted sections and their upstream, followed by the upstream MIT license reproduced
verbatim (fetched from the upstream repo's LICENSE file; 1,067 bytes; copyright line
"Copyright (c) 2026 snflkd"). Added the `!/THIRD-PARTY-NOTICES.md` allowlist line to
`.gitignore` (after `!/README.md`, keeping top-level files grouped), and updated
`tests/secret-guard.sh` twice in the same change as the golden rule requires: the same
entry at the same position in the pinned `expected_negations` list, and the
`GITIGNORE_SHA_PIN` content hash re-pinned to the edited `.gitignore`.

## Pre-review defect-class self-sweep (codex-review Step 0)
- Secret trackability: the new allowlist line re-includes exactly one named file; no glob,
  no directory. Guard green (see below), which also re-runs the nested-unknown probe
  battery against the widened allowlist.
- Pinned-list drift: `.gitignore` and `expected_negations` were edited in the same change
  at the same position; the guard's closed-set check is the executable proof.
- License-content fidelity: the reproduced license text is byte-for-byte the upstream
  fetch (same copyright line and permission paragraph); nothing was retyped by hand.

## Files changed (where / why)
- `THIRD-PARTY-NOTICES.md` — new; carries the full upstream MIT license so the adapted
  guideline satisfies the license's inclusion condition (T01 review finding F2).
- `.gitignore` — one `!`-allow line so the notice file is trackable in the deny-all
  allowlist scheme.
- `tests/secret-guard.sh` — pinned negation list + `.gitignore` content-hash pin updated
  in the same change (maintenance of an existing check; the set did not grow).

## Direct verification (repo policy: no TDD)
Recorded from actual runs (2026-08-21):
- `git add -n THIRD-PARTY-NOTICES.md` → `add 'THIRD-PARTY-NOTICES.md'` (trackable through
  the allowlist)
- `bash tests/secret-guard.sh` → `✓ PASS: secret guard` (closed-set negation check and
  re-pinned hash both hold; probe battery still green)

## E2E verification
Post-merge (commit 06faeaa), 2026-08-21, run against the landed state:
- `git ls-files THIRD-PARTY-NOTICES.md` → `THIRD-PARTY-NOTICES.md` (tracked in the
  commit, so the allowlist line works end-to-end).
- `git show HEAD:THIRD-PARTY-NOTICES.md | grep -c "Permission is hereby granted"` → `1`
  (the committed blob carries the upstream permission notice verbatim).
- `bash tests/secret-guard.sh` → `✓ PASS: secret guard` (closed-set negation list and
  re-pinned hash hold against the landed tree).

## Gate status
- [x] Verification: behavior confirmed by direct run (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
