# 01-skill-schema-rewrite

## Intent / Why
The full-cycle SKILL.md is ~250 lines of prose the orchestrator re-interprets every
session; it defines no task-level scheduling, so every milestone runs strictly serially
even when tasks are provably independent. This task rewrites the document as YAML
phase/gate/scheduling blocks with short prose annotations, and introduces the per-task
`deps`/`files` declaration format in the GOAL.md template — the substrate the
disjointness checker (T02) and the worker fan-out rules consume. Serves the Goal's Why:
less wall-clock without weaker gates.

## Design consult
Done (GPT-5.6 Sol, read-only, one-shot — full output in design-consult.md). Adopted:
- Task rows parse as logical items (checkbox line through next peer item/heading, joined);
  exactly one `deps` + one `files` per row; duplicates/missing/malformed → invalid.
- `files` grammar: repo-relative literal paths + trailing-slash directory prefixes only;
  no globs/absolute/`..`; overlap = exact match or component-boundary prefix containment;
  renames declare both paths; empty `files` → fan-out ineligible; docs/<goal>/ paths are
  orchestrator-owned and never declared by tasks.
- Actual-diff containment gate (checker `scope` mode): a worker's real changed paths
  (tracked + untracked) must sit inside its declaration, checked before review and again
  before merge — declared disjointness alone proves nothing about worker output.
- Review overlap and worker fan-out are distinct rule sets: overlap is unconditional, but
  files inside any OPEN review bundle freeze until the round seals; fan-out additionally
  needs checker PASS (readiness + pairwise transitive incomparability + file disjointness).
- Explicit git-worktree lifecycle, orchestrator-owned (recorded base commit, unique
  branch, dirty-state fail-closed, merge in topological order, cleanup only after merge +
  deregistration); affected-task set on conflict computed from touched paths ∩ bundles
  (can exceed one).
- Phase `needs` get instance-scope semantics (same-instance default; quantified forms for
  milestone/goal rollups).
- Pause semantics REVERSED from the brief: external waits keep docs registered and keep
  the orchestration turn alive; unregistration stays a manual user-input/recovery escape
  hatch only, and is not presented as preserving the tripwire.
- Disjointness is an eligibility check, not an independence proof — milestone E2E remains
  the semantic-integration defense.

## What was done (what / why)
- Wrote `tests/skill-schema.test.sh` FIRST (Red): 38 structural assertions — schema
  presence (12 phase ids, 8 scheduling keys, declaration grammar, checker name) and
  byte-frozen hook surfaces (gate headings, gate boxes, review-series token, registry
  helpers, [quick], language boundary, waits-keep-registration). Ran: 23 failures
  against the v1 prose file (new-schema assertions), preservation assertions green.
- Rewrote SKILL.md (Green): YAML `pipeline` block (12 phases, `needs` scope-instance
  semantics), `scheduling` block (declaration grammar per design consult: joined
  logical items, literal-paths+dir-prefixes only, component-boundary overlap;
  checker `plan`/`scope` verdicts, fail-closed; serial default / unconditional
  review-overlap with the open-bundle freeze-rule / conditional worker-fanout with
  worker binding + clean-tree precondition; explicit orchestrator-owned
  worktree-lifecycle; fan-in with milestone-E2E integration defense; waits: external
  waits keep registration, unreg reserved for user input), `hook-contract` block
  (frozen strings), condensed per-phase conduct prose, updated GOAL.md template
  (deps/files rows) + task.md template (Design consult section), registry helpers
  preserved verbatim. Test: all green.
- Refactor: found the skill LOADER had rendered `$T$1` as `$Tthe` in the launched
  content I initially copied — restored from git HEAD; helper block verified
  byte-identical to HEAD via diff. Frontmatter description updated to name DAG
  scheduling so skill routing keys on it.

## Files changed (where / why)
- `claude/skills/full-cycle/SKILL.md` — prose v1 → structured v2 (YAML schema blocks +
  conduct prose); the deliverable
- `claude/skills/full-cycle/tests/skill-schema.test.sh` — NEW; encodes the rewrite's
  structural invariants and hook-surface freeze so future edits can't silently drop
  either

## Round-001 fix pass (user-directed batching: fixes landed during implementation)
Per the user's per-goal override (implement everything, consolidated review at the end),
round 001's findings were fixed immediately instead of ping-ponging:
- Schema validity: phases became real typed flow-mappings (id/per/needs/gate); the
  scheduling block's worker-fanout became `requires:` list + sibling keys; ALL yaml
  blocks now parse (test gained a ruby YAML.load check — the round-1 failure mode,
  grep-able keywords inside broken YAML, is now mechanically impossible).
- Honest consumer framing added (structured prompting for the LLM; the deterministic
  consumer is the checker reading GOAL.md declarations, never SKILL.md).
- Deadlock fix: verdicts are three-way — INVALID (blocking, return to P5) is no longer
  collapsed into SERIAL.
- Clean-tree precondition replaced with declared-path cleanliness (docs/ + registry are
  orchestrator-owned and undeclarable, so pipeline writes can't block fan-out).
- Worktree lifecycle became an executable state machine: worker COMMITS on the task
  branch; scope = base..HEAD names + status; Merge precedes P10 completion and
  successor readiness; reopen covers merge conflicts, post-merge edits, AND post-seal
  changes to sealed bundles (stale-consensus fix); cleanup last.
- Resource isolation added to fan-out requires; canonical-path rules added to the
  files grammar; frontend classification anchored to the target repo's own declaration.

## Pre-review defect-class self-sweep (before round 001)
Classes checked (from this repo's prior review history): frozen-string drift,
fail-closed gaps, blind verbatim copies, machine/absolute path rules, internal
contradictions, shell-interpretation of doc content. Findings, fixed class-wide:
1. Blind-copy class: loader-rendered `$Tthe` (already fixed in Refactor); swept ALL
   fenced bash against git HEAD — helper block byte-identical; no other verbatim block
   pre-existed.
2. Path-rule class: checker referenced by D-STACK-repo-relative path, wrong for
   consumers and against the public-safe-path rule → `$HOME/.claude/skills/...`; swept
   all path references in the new file — no other machine/repo-specific path.
3. Contradiction class: `P7-tdd@deps-done` clashed with the quantifier definition →
   `P10-task-e2e@deps-done` + tightened definition wording; fan-out task.md authorship
   was unstated while workers are barred from docs/ → P8 now names the orchestrator.
Test re-run after fixes: all green.

## Pre-review self-sweep (before consolidated round 002)
Cross-artifact contract sweep over the whole goal scope: SKILL.md grammar vs checker
behavior vs synced surfaces. One mismatch found and fixed class-wide: SKILL.md said
only docs/<goal>/ was undeclarable while the checker conservatively rejects everything
under docs/ — SKILL.md now states the stricter rule and its rationale. Both test
suites re-run green; provenance/machine-path grep across all ten changed files clean.

## Round-A (002-input) fix pass
- Declaration `where:` added — checker parses only the '## Milestones & tasks'
  section, fences ignored.
- `scope` verdict rewritten: the checker collects the complete changed set itself
  (worktree dir + recorded base inputs); callers cannot narrow it.
- Fan-out review binding: before-review now pins the bundle to the recorded base and
  committed HEAD (both ids recorded in the round file) plus main-owned task.md.
- fan-in `accepted-residual` added: clean disjoint sibling merges are defended by the
  milestone E2E per the recorded user decision; reopen rules unchanged otherwise.
- P10 now flips the GOAL.md row checkbox (the checker's readiness signal) — bound to
  P10 and nowhere else.
- Delegation brief must carry worktree path, task branch, and recorded base.
- review-overlap wording: concurrent BY DEFAULT when multiple tasks are review-ready
  (serialization is the exception with a stated reason).

## E2E verification
evidence/final-e2e.txt ([T01] section, 2026-07-22): skill-schema suite fully green —
all three fenced YAML blocks parse under ruby's loader, all 12 phase ids, 8
scheduling keys, hook-frozen strings, and hardening tokens present. The live skill
is this file via symlink (~/.claude/skills/full-cycle/SKILL.md), and the session's
inject hook echoes the same pipeline stages.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
