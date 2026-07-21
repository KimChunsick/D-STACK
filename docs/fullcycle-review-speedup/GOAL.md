# GOAL — Cut full-cycle review wall-clock without lowering review quality

## Goal (the one Why)
The full-cycle loop's dominant cost is serial Codex review rounds (15–25 min each; real
tasks have reached 21 rounds). Keep the quality bar (GPT-5.6 Sol at xhigh — fixed, not
negotiable) while removing avoidable time: (a) fewer review rounds *by construction*
(pre-review defect-class self-sweep, class-wide fixes, slimmer finding format, strict
closure, conditional design consult, wait-time parallelization); (b) the ultracode default
must survive machine churn (it silently broke when `~/.zshrc` was rewritten — the source
hook was machine-local and unmanaged); (c) this repo's own meta test suite must stop
taxing every config change (slim to a single secret guard — the one asymmetric-risk
control worth keeping on a public repo).

## Interview record (Phase 4)
- Q: Finding format — drop "suggested direction + illustrative example"? → **A: slim to
  optional one-liner**: drop `Illustrative example:` + `Reviewer caveat:` boilerplate
  entirely; `Suggested direction:` becomes optional, one sentence, only when the repair is
  not obvious from the evidence; `Evidence:` and `Verification:` stay mandatory.
- Q: Joint Codex+Claude upfront design? → **A: conditional** — only for tasks matching a
  trigger list (new architecture, API contracts, persistence/logging consistency,
  cursor/idempotency semantics, partitioning, rendering boundaries, multi-path
  sanitization); one timeboxed Codex design consult before implementation; skipped
  otherwise.
- Q: Test deletion scope (research flagged asymmetric secret-leak risk)? → **A: keep one
  secret guard only** — delete the suite (run.sh, lib.sh, all test_*.sh) but keep a single
  standalone secret-trackability guard script, run manually before commit.
- Q: Meaning of "reflect the ultracode setting in the review"? → **A: it simply meant
  "apply ultracode by default to every session"** — no separate review-doc behavior; the
  durable default itself is the ask. Mechanism (stated in the question, not objected to):
  `install.sh` manages the `~/.zshrc` source hook idempotently, since the breakage cause
  was an unmanaged zshrc rewrite.
- Settled by prompt (not asked): review model/effort stay pinned (GPT-5.6 Sol, xhigh);
  the recurring defect-class checklist derives from actual prior-round findings (user's
  observed classes), not a generic list.

## Research summary (Phase 3)
Artifact: [research/review-loop-speedup.md](research/review-loop-speedup.md)
- **Strongest intervention:** builder pre-review self-pass scoped as a *defect-class
  sweep* tied to actual defect history (PSP evidence: personal reviews remove 60%+ of
  defects pre-test). Generic checklists show no benefit (Hatton, 308 inspections) —
  the checklist must stay evidence-derived and be pruned when classes stop firing.
  Blind spot: LLM self-correction is unreliable without external feedback → anchor the
  sweep on executable checks, not introspection.
- **Finding format:** both-sides evidence — inline suggestions are the top usefulness
  predictor (adoption data), but *mandating* explanations/repairs increases LLM reviewer
  overcorrection/false rejections. Supports the chosen middle: optional one-line
  direction, no mandatory examples.
- **Design consult:** design-level review pays only where defects can't be cheaply
  inferred from a diff (architecture/API/invariants); most observed round-cascades were
  implementation-level edge cases it would not catch → conditional gate with trigger
  list + timebox, else it is ceremony that moves latency earlier.
- **Strongest against-point (tests):** secret exposure on a public repo is
  asymmetric-downside; GitHub push protection has documented pattern/legacy/bypass
  limits. Resolved by user decision: keep exactly one standalone secret guard.
- **Unverified:** no controlled study measures "external adversarial LLM review rounds"
  before/after these interventions — generalization from human-review/PSP/LLM-agent
  literature is plausible but unproven; treat round-count reduction as expected, not
  guaranteed.

## Milestones & tasks (Phase 5)
### M1 — Slim the meta test suite to a single secret guard
- [x] **T01** `01-secret-guard-only` — delete run.sh/lib.sh/all test_*.sh; keep one
  standalone `tests/secret-guard.sh` (inlined helpers, same probe battery + pinned
  negation set); update every tests reference (AGENTS.md, README.md, gemini/README.md,
  codex-review SKILL.md assembler note).

### M2 — Review-loop speedup rules (skills + agent docs)
- [ ] **T01** `01-speedup-rules` — codex-review SKILL.md: new pre-review defect-class
  self-sweep step, slimmed finding format in the review prompt, class-wide fix + prior
  round-citation rebuttal discipline, wait-time parallelization note, explicit
  "medium=0 ⇒ close as approve-with-fixes" closure; conditional design-consult step in
  full-cycle SKILL.md (trigger list, timeboxed); matching format/discipline updates in
  codex/AGENTS.md Mode 2 and the fullcycle-inject.sh pipeline text.

### M3 — Durable ultracode default
- [ ] **T01** `01-installer-zshrc-hook` — install.sh gains an idempotent, dry-run-aware
  "ensure zshrc source hook" step for `~/.claude/ultracode.zsh`; ultracode.zsh header
  updated (hook now installer-managed); run installer to re-wire the broken default.

## Milestone E2E evidence
- **M1 (2026-07-22):** `bash tests/secret-guard.sh` → `✓ PASS: secret guard` on the
  real tree; 53-scenario sabotage battery in disposable clones all behaving (full
  transcript embedded in the task doc §E2E); `tests/` contains exactly
  `secret-guard.sh`; repo-wide grep shows zero references to the retired suite
  (run.sh/lib.sh/test_*). Review closed at round 010, `Consensus: agreed`, two
  recorded non-blocking follow-ups.

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: slimmed suite verified — `bash tests/secret-guard.sh` green on clean tree, fails on planted probe; no stale tests references
- [ ] M2 E2E: full skill-doc pipeline consistency verified across the four changed surfaces (no drift between SKILL.md files, codex/AGENTS.md, inject hook)
- [ ] M3 E2E: fresh interactive zsh resolves `claude` to `claude --effort ultracode`; install.sh re-run is a no-op
- [ ] GOAL E2E: one full end-to-end pass — install.sh idempotent re-run, secret guard green, review-pipeline docs mutually consistent, ultracode alias live in a fresh shell
