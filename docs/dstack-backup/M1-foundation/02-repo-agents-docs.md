# 02-repo-agents-docs

## Intent / Why
The user's core instruction: **this repo's `AGENTS.md` is the base guide, and `CLAUDE.md` calls it internally (`@import`)**. Both are real files at the repo root, so the relative import is safe (no symlink → immune to GH #4754). This is where rules/structure/safety for any agent working in this backup repo live. See PLAN.md `Milestone 1 / Task 2`.

## What was done (and why)
- Root `CLAUDE.md` = one `@AGENTS.md` line (+ a Claude note).
- Root `AGENTS.md` = the repo guide: what it is / layout (Layer A·B) / golden rules ("Never commit secrets" + exclusion list, authored-only) / install.sh (symlink + Gemini copy caveat) / how to add a new agent.
- `README.md` (human-facing, one-command restore), `claude|codex|gemini/.gitkeep`.

## Files changed (where / why)
- `AGENTS.md`, `CLAUDE.md`, `README.md` — Layer A docs.
- `claude/.gitkeep`, `codex/.gitkeep`, `gemini/.gitkeep` — agent-first folder skeleton.
- `tests/test_repo_agents_docs.sh` — assert `CLAUDE.md` imports `@AGENTS.md`, the safety wording is present, and the folders exist.

## Adversarial review (Codex limit → Claude substitute, user-approved)
- The Claude adversarial subagent also reviewed this. Findings on this doc:
  - (MED) `install.sh` is described in AGENTS.md/README as if it already exists → **accepted**: it is implemented in M3 within the same body of work (consistent on completion); only an intermediate-state mismatch.
  - (MED) the `@AGENTS.md` import is only string-tested; runtime resolution isn't CI-verifiable → **accepted**: research confirmed the mechanism (real root file + relative import); a documented limitation.
- Consensus: **resolved** (no blocking issues; accepted items noted).

## E2E verification
- `docs/dstack-backup/evidence/m1-tests.txt` — `bash tests/run.sh` all PASS + structural checks (`CLAUDE.md` `@AGENTS.md` import, folder skeleton, no secrets in tracked tree).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Adversarial review consensus (Codex limit → Claude substitute, resolved)
- [x] E2E capture verified
