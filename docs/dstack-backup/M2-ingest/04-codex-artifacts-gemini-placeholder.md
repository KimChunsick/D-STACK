# 04-codex-artifacts-gemini-placeholder

## Intent / Why
Ingest only the authored Codex artifacts (`instructions.md`, `rules/default.rules`) and **never back up `config.toml`/`auth.json`/private project paths** (public repo). Gemini doesn't exist yet, so just reserve an extensibility slot. See PLAN.md `Milestone 2 / Task 4`.

## What was done (and why)
- Copied: `~/.codex/instructions.md`, `~/.codex/rules/default.rules`.
- `gemini/README.md`: onboarding steps + the "Gemini ignores symlinked context files (GH #11547) → install.sh uses copy" caveat.

## Files changed (where / why)
- `codex/instructions.md`, `codex/rules/default.rules` — authored.
- `gemini/README.md` — extensibility slot.
- `tests/test_codex_artifacts.sh` — assert artifacts present, `config.toml`/`auth.json` absent, no `/Users/` leakage, and (via a gitignored `.private-denylist`) no private name leaks into tracked content.

## Adversarial review (Codex limit → Claude substitute, user-approved)
- Findings → actions:
  - (HIGH) the guard test and docs **hardcoded private project names** (the guard itself exposed the names) → moved them to a gitignored `.private-denylist`; replaced doc occurrences with placeholders. (fix commit in M2)
  - Codex artifacts (`instructions.md`, `default.rules`) contain no `/Users/`, project names, or secrets (source pre-check + tracked-tree scan).
  - `default.rules`'s `git commit --no-verify` allowance is the user's authored policy → **kept verbatim** (it's a backup target); review noted only.
- Consensus: **resolved**.

## E2E verification
- See `docs/dstack-backup/evidence/m2-tests.txt` — codex/gemini tracked files PII 0; denylist guard fires on an injected name.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Adversarial review consensus (Codex limit → Claude substitute, resolved)
- [x] E2E capture verified
