# 02-routing-and-ssot-wiring

## Intent / Why
Make the subagent (a) mandatory for frontend work via an instruction-based routing rule in
the global `claude/CLAUDE.md`, and (b) a first-class SSOT artifact: `install.sh` map entry
(link mode), `.gitignore` allowlist pin (deny-all inside `claude/agents/`), and a
`tests/test_claude_artifacts.sh` guard so the backup stays healthy and public-safe.

## What was done (what / why)
- **Red:** extended the guards first, then confirmed 2 failing test files:
  - `tests/test_claude_artifacts.sh`: `.gitignore` must pin `!/claude/agents/frontend-dev.md`,
    the file must be *effectively* un-ignored (`git check-ignore --no-index`, mirroring the
    ultracode guard's later-rule-override lesson), `install.sh` must carry the exact map row,
    and `claude/CLAUDE.md` must contain the routing rule (`frontend-dev` + `MUST be delegated`;
    an initial `MUST|must` assertion was strengthened — it already matched pre-existing text,
    so it could never fail).
  - `tests/test_install_sh.sh`: sandbox HOME must end up with the
    `.claude/agents/frontend-dev.md` symlink pointing into the repo.
  - `tests/test_gitignore_secret_guard.sh`: new nested-unknown probes
    `claude/agents/random_unknownfile` + `claude/agents/auth.json` — the new `agents/` allow
    must not open a wholesale hole (deny-all inside, secret-name backstop).
- **Green:**
  - `.gitignore`: `!/claude/agents/` + `/claude/agents/*` + `!/claude/agents/frontend-dev.md`
    (same allowlist-at-every-level shape as `skills/`).
  - `install.sh`: `claude/agents/frontend-dev.md|.claude/agents/frontend-dev.md|link`.
  - `claude/CLAUDE.md`: new section "0.1 Frontend work → `frontend-dev` subagent (mandatory)" —
    all frontend code work MUST be delegated (Agent tool / `@agent-frontend-dev`); sole
    exception one-line typo/copy/constant fixes; delegation prompt carries full context
    (subagents start fresh); composes with full-cycle (pipeline in main loop, implementation
    delegated); relay the subagent's report.
- **Refactor:** none needed; full suite green.
- **Post-review hardening (Codex rounds 1–2, see codex-review.md):** routing rule scope
  broadened beyond React/TS (any framework's components, templates/markup, frontend
  tests/stories, frontend build config), keyed on the nature of the code rather than repo
  shape (full-stack/monorepo included), with the generated-artifacts case covered; the
  full-cycle bullet now explicitly delegates frontend test code. Guards hardened: routing
  assertion is one co-located phrase; secret-guard battery gained unknown-`.md` and nested
  probes under `claude/agents/`; and `.gitignore` is structurally restricted to exactly the
  two permitted agents-path negation lines.
- Enforcement level is instruction-based by the owner's explicit interview choice; research
  documented that instructions steer but don't enforce, and that a hook-based hard gate
  (branching on `agent_type`) remains available as a follow-up.

## Files changed (where / why)
- `claude/CLAUDE.md` — the mandatory routing rule (the "무조건 이 서브 에이전트로만" half of the Goal).
- `.gitignore` — allowlist pin; `agents/` dir stays deny-all internally.
- `install.sh` — links the agent file into the live `~/.claude/agents/`.
- `tests/test_claude_artifacts.sh` — wiring + routing-rule guards.
- `tests/test_install_sh.sh` — sandbox symlink assertion.
- `tests/test_gitignore_secret_guard.sh` — nested-unknown/secret probes for `claude/agents/`.

## E2E verification
- `bash tests/run.sh` → ALL TESTS PASSED (includes the sandboxed install.sh run that
  actually creates and verifies the `~/.claude/agents/frontend-dev.md` symlink, and the
  secret-guard probe battery over the new `agents/` dir).
- Real-home wiring + live routing observation recorded under M1 E2E in GOAL.md.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus
- [x] E2E capture verified
