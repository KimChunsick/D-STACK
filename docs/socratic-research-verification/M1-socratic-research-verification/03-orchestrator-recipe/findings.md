# Findings ledger — 03-orchestrator-recipe

| id | round raised | severity | finding | status |
|----|-------------|----------|---------|--------|
| F1 | 001 | high | Step 2b leaf paths unprotected: symlinked audit-input truncated via `>`, symlinked inputs read via `cat`/`-s` | fixed (round 001 response: leaf guards both fences, stdin assembled under $SCRATCH) |
| F2 | 001 | high | Research-fallback path finishes Phase 3 without contract blocks, data-checks record, or audit | fixed (round 001 response: fallback artifact carries the nine sections, Steps 2a–2c resume unchanged) |
| F3 | 001 | medium | Audit acceptance checks only `## Verdict summary`; other required sections/coverage unchecked | fixed (round 001 response: all seven sections + per-H-item summary row, mirrored in fallback trigger) |
| F4 | 001 | medium | New-check results reconciled only when decision-critical; noncritical stale verdicts survive | fixed (round 001 response: superseded-line reconciliation for every check; criticality gates only escalation) |
| F5 | 001 | low | Intro overclaims "evidence-backed" against the research's own Unverified section | fixed (round 001 response: evidence-informed wording, residue owned by E2E) |
| F6 | 002 | high | `-L` guards pass hard-linked leaves; orchestrator-written artifacts lack pre-write leaf protection | fixed (round 002 response: link-count-1 guard both fences + prose rule for orchestrator writes; probes recorded) |
| F7 | 002 | medium | Audit-input assembly fail-open: group status ignored, readability unchecked | fixed (round 002 response: `-r` guards, `&&` chain, nonzero status refuses launch; probe recorded) |
| F8 | 002 | medium | Structural acceptance trusts producer-declared H coverage; F/data-check coverage unvalidated | fixed (round 002 response: ledger/deferred reconciliation + empty-F breakage + all-`none`-over-claims trigger; auditor keeps claim-level contract) |
| F9 | 002 | medium | Orchestrator-assigned superseded verdicts bypass audit's data-reading probes | fixed (round 002 response: delta audit for every verdict-changing outcome; criticality gates only P4 re-entry; GOAL.md amended); replaced by F12's always-delta rule |
| F10 | 003 | high | Deferred-check specs are a confused-deputy path: untrusted data selects local inputs; verbatim output shipped to Codex | fixed (round 003 response: input authorization — public sources only — and bounded recording) |
| F11 | 003 | medium | Coverage predicates incomplete: per-item ledger dodge, token F coverage, unresolved-column parking | fixed (round 003 response: per-item reading, substantive per-target coverage from artifact-derived sets) |
| F12 | 003 | medium | Orchestrator classifying a check as "confirming" is itself the audited judgment | fixed (round 003 response: every executed new check returns to the auditor; bounded termination) |
| F13 | 003 | medium | Every Step 2b run writes the same audit.md path, so a delta audit overwrites the original | fixed (round 003 response: ATTEMPT suffixes label + artifact; existing artifact refused; fallback writes next attempt) |
| F14 | 003 | medium | Empty brief passes the leaf guard and dstack, yielding goal-free research | fixed (round 003 response: full input test on the brief; probe recorded) |
| F15 | 003 | low | Assembly failure exits before the cleanup trap is armed, leaking scratch | fixed (round 003 response: unconditional trap at mktemp, gated swap at launch; probe recorded); verified closed round 004 |
| F16 | 004 | high | Authorized public inputs can still be EXECUTED — no execute/import/source/install ban on fetched content | fixed (round 004 response: inert-data rule, readable-not-runnable, credential-free scratch) |
| F17 | 004 | medium | Verdict-summary predicate covers H-items only; audited F-item verdicts can vanish before Step 3/P5 | fixed (round 004 response: one summary row per H-item AND per examined F-item; fallback mirrored) |
| F18 | 004 | low | Scratch preserved when dstack refuses pre-launch (no exit file ever) | fixed (round 004 response: clean on no-.launch OR exit-present; three-state probe recorded; foreign-claim race preserves fail-closed) |
| F19 | 004 | low | This Goal's own research artifact predates the contract (no three blocks) | recorded follow-up (non-blocking: regenerate + audit via the new pipeline; intro already evidence-informed) |
| F20 | 005 | low | `--output-schema` bullet contradicts the Markdown-only downstream flow (Step 2a/fallback gate on literal headings) | fixed at the round-005 cap seal (option forbidden for this flow); NOT reviewer-re-verified (§4 cap closure) |
