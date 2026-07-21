# GOAL — Dedicated frontend-dev subagent + mandatory instruction-based routing

## Goal (the one Why)
Back up, in this SSOT repo, a personal `frontend-dev` Claude Code subagent that encodes the
owner's calibrated frontend implementation principles in a fully generic, public-safe form
(no strings coupling the definition to any specific company, product, or internal tooling),
and route ALL frontend code work to that subagent via instructions (CLAUDE.md rule + strong
agent description) so the main loop never implements frontend code directly.

## Interview record (Phase 4)
- **Enforcement level:** instruction-based only (CLAUDE.md rule + "MUST BE USED" description).
  No PreToolUse blocking hook, even though research confirmed `agent_type` in hook input makes
  hard enforcement feasible — owner explicitly chose the softer mechanism.
- **Design-system rule:** the source material pinned a specific named design system as the top
  UI priority; replaced with a generic rule — "use the repo's established design system /
  component library first (via its dedicated lookup/MCP tooling when present)".
- **Library names:** keep only vendor-neutral ones (e.g. TanStack react-query); libraries
  traceable to a specific company ecosystem are generalized to "follow the repo's established
  overlay/utility library conventions".
- **Frontend-file judgment:** by project nature (is it a frontend project / frontend code),
  not by file-extension lists — fits instruction-based routing.

## Research summary (Phase 3)
Artifact: [research/subagent-routing-enforcement.md](research/subagent-routing-enforcement.md)
- Subagents: `~/.claude/agents/*.md`, required frontmatter `name` + `description` only;
  identity for hooks is the `name` field (`agent_type`). Guaranteed routing is `@agent-<name>`
  mention; description phrasing ("use proactively"/"MUST BE USED") *encourages* delegation.
- Strongest against-point: Anthropic docs state instructions steer but do not enforce —
  only permissions/hooks are boundaries. Owner accepted this tradeoff knowingly (see interview);
  hard enforcement via `agent_type`-branching PreToolUse hook remains a documented follow-up option.
- Subagents get fresh context (no conversation history) — the delegation prompt must carry
  full task context; latency and context-transfer cost are the known downsides.
- Unverified: exact behavior on the locally installed CLI version (docs referenced a slightly
  newer version than the retrieved changelog); not load-bearing for instruction-based routing.

## Milestones & tasks (Phase 5)
### M1 — frontend-dev subagent, routed and backed up
- [x] **T01** Agent definition: authored `claude/agents/frontend-dev.md` (generic, public-safe)
- [x] **T02** Routing + SSOT wiring: CLAUDE.md routing rule, install.sh map entry,
  .gitignore allowlist pin, tests guard

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: subagent discovered by a live `claude` session and delegation observed end-to-end;
  full test suite green; provenance scan of all tracked changes = 0 hits
  — Evidence: discovery probe listed `frontend-dev` among available subagent types; the
  delegation probe (prompt not naming the subagent, scratch React project, user-level agent
  via the installed symlink) produced 9 `"subagent_type":"frontend-dev"` Agent-tool events
  and a subagent-authored diff (captured:
  `M1-frontend-dev-subagent/01-agent-definition/e2e-delegation-stream.jsonl`); this exercised
  T01 (definition) + T02 (installed symlink + CLAUDE.md routing rule) together.
  `bash tests/run.sh` → ALL TESTS PASSED; provenance scan over the commit-bound tree
  (agent file, CLAUDE.md, .gitignore, install.sh, tests) → 0 hits.
- [x] GOAL E2E: one full end-to-end pass — fresh install.sh run links the agent file, a frontend
  request routes to frontend-dev, and the public tree contains no source-identifying strings
  — Evidence: sandbox-HOME install run (inside `tests/test_install_sh.sh`) creates the
  `.claude/agents/frontend-dev.md` symlink from scratch; real-home run is idempotent
  (`up-to-date=13, linked=0` on re-run); live routing captured as above; final provenance
  scan → 0 hits. Full pass: define → wire → install → route → verify, all green.
