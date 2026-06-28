# T03 — codex-research skill (Codex as researcher)

## Intent / Why
Phase 3 research must move from "Claude decides if it researches" (the self-judgment
loophole) to "Codex researches every Goal." This task creates the `codex-research` skill:
the mechanism that drives `codex exec` + `web.run` to gather **both-sides** evidence
(needed info, opposing views, evidence for/against the stated goal), with sources and
recency, saved as a research artifact and summarized into GOAL.md — with a graceful
fallback to Claude's `deep-research` if Codex is unavailable.

## How (plan, before coding)
1. **Red**: guard in `tests/test_claude_artifacts.sh` — skill present + names live web
   research, evidence *against* the goal, sources, and a fallback; allowlisted + installer-mapped.
2. **Green**: write `claude/skills/codex-research/SKILL.md`; add `.gitignore` allow +
   `install.sh` map row.
3. **Refactor**: tighten; public-safe.

## What was done (what / why)
- Created `claude/skills/codex-research/SKILL.md`: Codex drives a hardened `codex exec` +
  `web.run` research pass gathering both-sides evidence (needed info, opposing views,
  for/against the goal) with dated sources, saved via `-o`, summarized into GOAL.md.
- Hardened after adversarial review: brief via **stdin** (no shell injection), `--ephemeral`,
  `-s read-only`, `-C scratch`, pinned `-m gpt-5.5 -c model_reasoning_effort=xhigh`, concrete
  fallback triggers, copy-paste-safe command block.

## Files changed (where / why)
- `claude/skills/codex-research/SKILL.md` (new) — the researcher skill (deliverable).
- `.gitignore` — `!/claude/skills/codex-research/` allow line.
- `install.sh` — `claude/skills/codex-research|...|link` map row.
- `tests/test_claude_artifacts.sh` — guards: presence + both-sides + hardening flags
  (stdin/ephemeral/read-only/-o/pinned-effort) + copy-paste-safety (`\␠#` forbidden) + wiring.

## E2E verification
- Two real `codex exec` research runs → all 6 sections with cited, dated sources.
- The documented bash block **extracted from SKILL.md and run verbatim** → exit 0, 6 sections
  (proves functional invocation, the Round-2 blocker). `bash tests/run.sh` → ALL PASSED.
- (Raw transcripts removed: they embed machine paths and this repo is public.)

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus (3 rounds → agreed)
- [x] E2E capture verified
