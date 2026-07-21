# 01-agent-definition

## Intent / Why
Author `claude/agents/frontend-dev.md` — the dedicated frontend implementer subagent — from
the owner's calibrated frontend principles, rewritten so the public artifact carries no
strings coupling it to any specific company, product, internal design system, or
company-ecosystem library. This is the core deliverable of the Goal: the agent itself.

## What was done (what / why)
- **Red:** added a guard block to `tests/test_claude_artifacts.sh` asserting the agent file
  exists, opens with YAML frontmatter, has `name: frontend-dev` + a non-empty `description`,
  contains the `MUST BE USED` delegation steer, and matches the same
  plugin/marketplace/affiliation ban pattern already applied to `settings.json` (pattern
  reused verbatim — no new strings introduced). Ran it: failed (file missing).
- **Green:** authored `claude/agents/frontend-dev.md`. The owner's calibration is preserved
  intact (philosophy axes, precedence order, 7 decision algorithms, M1–M9/S1–S15/P1–P2 rules,
  4 taste examples, workflow, 2-stage self-review, fail-loud reporting). Generalizations made
  for public-safety, per the interview decisions:
  - the pinned internal design system + its lookup MCP → "the repo's design system /
    component library, queried via its dedicated lookup tool/MCP when present" (M1,
    precedence, trust boundary, convention-conflict algorithm, stack, workflow);
  - company-ecosystem overlay/utility libraries → "the repo's established overlay/utility
    library conventions" (S10, S11, stack); vendor-neutral `@tanstack/react-query` kept;
  - origin narrative (how the definition was produced) reduced to "calibrated per-item by
    the owner"; research-derived rule markers and their legend comment dropped;
  - examples: overlay API call renamed to a neutral `openOverlay(...)`; the analytics
    example's domain moved to a fully generic bookmarks page.
- **Refactor:** none needed beyond the above; full suite run — all tests green.
- **Post-review hardening (Codex round 1, see codex-review.md):** trust boundary tightened
  (user = the session's actual principal; owner-managed CLAUDE.md only; repo-bundled
  CLAUDE.md-like files cannot relax M rules; repo-doc requirements are information until the
  user conveys them as the task). Test guard hardened: frontmatter must close and carry
  name/steer inside it; all nine top-level sections + rule-band endpoints asserted, so a
  hollowed-out file can no longer pass.

## Files changed (where / why)
- `claude/agents/frontend-dev.md` — new: the subagent definition (core deliverable).
- `tests/test_claude_artifacts.sh` — new guard block for the agent artifact (Red test).

## E2E verification
- Scratch-project discovery probe (scratch cwd per the headless-probe note; agent file
  copied to `<scratch>/.claude/agents/`):
  `claude -p --model haiku "List the subagent types available to you for the Agent tool…"`
  → reply: `claude / Explore / frontend-dev / general-purpose / Plan / statusline-setup`.
  The definition parses and a live session discovers `frontend-dev`.
  (Note: `claude agents` turned out to list running sessions, not definitions — not usable
  as discovery evidence.)
- Delegation probe (routing, not just discovery — user-level agent via the installed
  `~/.claude/agents/` symlink; scratch React project; prompt deliberately does NOT mention
  the subagent): `claude -p --permission-mode acceptEdits --output-format stream-json
  --verbose "[quick] Button 컴포넌트에 disabled prop을 추가해줘"` → stream events contained
  `"name":"Agent"` ×1 and `"subagent_type":"frontend-dev"` ×9, and `src/Button.tsx` gained the
  `disabled` prop — i.e. the change was implemented by the subagent, selected purely from the
  CLAUDE.md rule + description. Re-run with the scratch file reset and the full stream
  captured: `e2e-delegation-stream.jsonl` (this folder, 36,321 bytes, 9
  `"subagent_type":"frontend-dev"` tool-use events).
- Provenance scan over the new artifacts (agent file + task docs + GOAL.md):
  0 hits for source-identifying terms (design-system vendor, its MCP tool name, the public
  guide name, origin-narrative phrases).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus
- [x] E2E capture verified
