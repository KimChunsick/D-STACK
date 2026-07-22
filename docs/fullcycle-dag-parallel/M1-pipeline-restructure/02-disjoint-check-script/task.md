# 02-disjoint-check-script

## Intent / Why
Research verdict: LLM-declared task independence is brittle ("false independence") —
parallel fan-out must be gated by a deterministic check, not model judgment. This task
ships `check-parallel.sh` with the full-cycle skill: it parses the GOAL.md task
declarations (`deps: [...]; files: [...]`), validates the graph (acyclic, complete), and
answers which task sets may run in parallel (pairwise-disjoint file sets, no dependency
edge). Parse and validation failures are BLOCKING (INVALID, exit 2 — return to
decomposition and fix the declarations); only eligibility failures on a valid graph
fall back to serial. Fail-closed in both directions.

## Design consult
Covered by T01's design consult round (same declaration-format contract).

## What was done (what / why)
- RED first: wrote `tests/check-parallel.test.sh` — 27 contract cases (independent →
  PARALLEL; direct/transitive dependency, unready deps, exact/prefix/case-variant file
  overlap, empty files → SERIAL; cycle, unknown/duplicate ids, duplicate fields, glob,
  `..`, `./`, missing files, goal-docs paths, unknown candidates → INVALID exit 2 —
  never collapsed into SERIAL; scope PASS/VIOLATION incl. dir-prefix containment;
  usage errors). Ran with no implementation: failing as expected.
- GREEN: implemented `check-parallel.sh` — macOS bash 3.2 compatible (no associative
  arrays), LC_ALL=C, declarations treated as inert data (bash `=~` extraction, no
  expansion/eval). Logical-item joining per the SKILL.md grammar; strict terminal
  `deps: [...]; files: [...]` shape; per-entry path ceiling (globs, absolute, `.`/`..`,
  repeated separators, whitespace/shell metacharacters, docs/ all rejected); Kahn cycle
  detection; readiness from ticked dep rows; pairwise transitive-reachability
  incomparability; case-INSENSITIVE prefix-aware overlap (collision-conservative) vs
  case-SENSITIVE scope containment (strict) — both directions fail-closed.
- One test fixture was corrected during Green: the original "transitive" case declared
  a genuinely incomparable pair (T03/T04), which the checker rightly called PARALLEL —
  the fixture, not the checker, was wrong; replaced with a real chain (T01←T02←T03,
  candidates {T01, T03}).
- Verified on both `bash` (brew) and `/bin/bash` 3.2. Self-hosting check: run against
  this Goal's own GOAL.md, it verdicts {T01,T04} PARALLEL (the pair actually built in
  parallel) and {T01,T02} SERIAL (dep incomplete) — correct on real data.

## Files changed (where / why)
- `claude/skills/full-cycle/check-parallel.sh` — NEW; the deterministic fan-out gate
  (LLM proposes, this verdicts — the research's false-independence mitigation)
- `claude/skills/full-cycle/tests/check-parallel.test.sh` — NEW; the 27-case contract

## Round-A (002-input) fix pass
Consolidated round A found three checker holes; all reworked with tests-first updates:
- `scope` no longer trusts caller-supplied paths (the bypass: empty list → PASS). New
  signature `scope <GOAL> <TASK> <worktree-dir> <base-commit>`: the checker collects
  the complete set itself — `git diff --name-only -z --no-renames base..HEAD` (both
  rename sides listed) plus `git status --porcelain=v1 -z --no-renames -uall` —
  NUL-safe. Tests now build real git repos; the undeclared-COMMITTED case passes no
  path list at all and still gets VIOLATION. Accepted residual (documented): ignored
  files never enter commits/merges and are not scanned.
- Symlink/submodule traversal: declared paths are walked component-by-component
  against the repo containing GOAL.md (which must be inside a git repo now);
  an existing symlink or submodule-boundary component is INVALID. Actual paths get
  the same walk against the worktree in scope mode. Test: tracked `escape` symlink +
  `files: [escape/output]` → INVALID.
- Completion-state closure: a checked task with an unchecked dep is INVALID
  (checkbox = P10 signal, bound in SKILL.md); a checked candidate is INVALID
  (not schedulable). The transitive-reachability check remains as cheap
  defense-in-depth (unreachable while closure+readiness hold) with a comment.
- Parser is now section-scoped ('## Milestones & tasks' only) and fence-aware;
  matching text elsewhere is never graph data. Tests: fenced example row ignored;
  missing section → INVALID; GOAL.md outside a git repo → INVALID.
All 33 cases green on brew bash and /bin/bash 3.2.

## E2E verification
evidence/final-e2e.txt ([T02] section, 2026-07-22): 40-case contract suite green on
both brew bash and /bin/bash 3.2 (identity binding, clean-tree, symlink rejection,
fence/section isolation, fail-closed enumeration all exercised on real git
fixtures). Self-hosting: against this goal's own GOAL.md the checker verdicts
{T01,T04} PARALLEL (the pair actually built concurrently) and {T01,T02} SERIAL
(dependency incomplete at capture time), exit codes 0/1 as contracted.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
