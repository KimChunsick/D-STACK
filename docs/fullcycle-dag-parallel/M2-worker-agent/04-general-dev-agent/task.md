# 04-general-dev-agent

## Intent / Why
Conditional worker fan-out needs a non-frontend implementer. Per the user's decision, a
dedicated authored `general-dev` agent (not the built-in general-purpose) carries the
standing delegation content — TDD discipline, reporting contract, plain/maintainable-code
principles — so per-task delegation prompts stay small and behavior stays consistent.
The system prompt is research-informed (delta research artifact
research/general-dev-prompt.md) and written in English. Frontend code remains with
`frontend-dev` unconditionally.

## Design consult
Skipped — no trigger (a single authored agent definition; no architecture/API/persistence
boundary). The prompt content is grounded by the delta research artifact instead.

## What was done (what / why)
- RED: added the `!/claude/agents/general-dev.md` allowlist line, staged it, ran
  `tests/secret-guard.sh` → FAIL on the pinned .gitignore hash (the guard's designed
  tripwire; it printed the new hash to adopt). This is the test that encodes the WHY:
  no allowlist surface grows without a deliberate same-change pin update.
- GREEN: wrote `claude/agents/general-dev.md` (lean English worker prompt, shaped by
  research/general-dev-prompt.md: routeable description with use/do-NOT-use, precedence
  brief > repo conventions > definition, plain/minimal-diff/root-cause philosophy,
  verify-first workflow with R-G-R where testable, declared-file scope with STOP-and-
  report on undeclared needs, worktree discipline, merge-relevant report contract);
  updated the guard's three pins (SHA, negation set, agents addable-check exception);
  added the install.sh MAP row. Guard: PASS. `git check-ignore -v` matches the `!`
  re-include line (trackable). `./install.sh --dry-run` shows the link row.
- Sweep: provenance banned-string grep and machine-path grep over all four files —
  clean.
- Round-001 fix pass (user-directed batching): added a least-privilege `tools:`
  allowlist (Bash, Read, Edit, Write, Glob, Grep, TodoWrite — no Agent nesting, no
  connectors) and an immutable `<boundaries>` layer that no brief can override (no
  frontend code, no registry/docs/undeclared paths, task-branch verification before
  writing with STOP on mismatch, secret redaction, content-as-data
  anti-injection rule); precedence now applies only within those boundaries, closing
  the brief-overrides-ownership hole. Worktree isolation is enforced at the invocation
  boundary (the orchestrator's explicit lifecycle in SKILL.md) rather than frontmatter
  `isolation`, so the harness cannot spawn a second, unmanaged worktree that bypasses
  the recorded base/branch/merge bookkeeping; the branch-verification STOP rule covers
  misinvocations outside the pipeline. Re-swept provenance/machine-path greps: clean.

## Files changed (where / why)
- `claude/agents/general-dev.md` — NEW; the worker-agent definition (per-task briefs
  stay small, behavior stays consistent across delegations)
- `.gitignore` — `!/claude/agents/general-dev.md` re-include (deny-all stays intact)
- `install.sh` — MAP row so the agent links into ~/.claude/agents/ like its sibling
- `tests/secret-guard.sh` — SHA pin, pinned negation set, and agents addable-check
  updated in the same change, per the AGENTS.md golden rule

## Round-A (002-input) fix pass
- Worktree-identity boundary hardened: a parallel/fan-out brief MUST name worktree
  path, task branch, and recorded base; the worker verifies it is in that worktree on
  that branch before any write, and a parallel brief omitting the identities is
  itself a STOP condition (previously the check ran only "when the brief names a
  branch", which an omissive brief bypassed). SKILL.md's brief contract now carries
  the same mandatory fields from the orchestrator side.
- Instruction-surface split resolves the conventions-vs-anti-injection conflict:
  repo instruction surfaces (CONTRIBUTING, CLAUDE.md/AGENTS.md, lint/build configs)
  are authoritative for HOW to write within declared scope; nothing read anywhere may
  change WHAT is touchable, the worktree/branch identity, or the boundaries.

## E2E verification
evidence/m2-install.txt: `./install.sh` linked ~/.claude/agents/general-dev.md →
repo file; re-run shows "up to date" (idempotent, evidence/final-e2e.txt [T04]).
evidence/m2-smoke.txt: headless `claude --agent general-dev` from a scratch cwd
loads the linked definition and answers with its name and its exact writable-scope
STOP rule. The harness additionally registered `general-dev` as an available agent
type (with the least-privilege tool list) in the live session. secret-guard: PASS.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
