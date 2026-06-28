# 03-claude-artifacts

## Intent / Why
Ingest only the **authored** Claude artifacts into the SSOT; install.sh later symlinks them into `~/.claude`. Rewrite settings.json's three hardcoded `/Users/<user>` paths to `$HOME` for portability (works on other machines/users). See PLAN.md `Milestone 2 / Task 3`.

## What was done (and why)
- Copied: `CLAUDE.md`, `settings.json`, `statusline-command.sh`, `hooks/fullcycle-inject.sh`, `hooks/fullcycle-gate.sh`, `skills/full-cycle`, `skills/codex-review`.
- Portability: rewrote the hook/statusline command paths in `settings.json` from `/Users/<user>/.claude/...` to `$HOME/.claude/...`. (Claude expands `$HOME` in hook commands via `sh -c` — confirmed via claude-code-guide; statusLine inferred, cosmetic if not.)

## Files changed (where / why)
- `claude/**` — the 7 authored artifacts.
- `tests/test_claude_artifacts.sh` — assert presence/non-empty, no `/Users/` anywhere under `claude/`, `$HOME` present, valid JSON (jq), and no third-party plugin/marketplace refs.

## Adversarial review (Codex limit → Claude substitute, user-approved)
- The Claude adversarial subagent grepped all ingested content. **No API keys/tokens/private keys/emails in any actual config file** (also confirmed by a deterministic scanner).
- Findings → actions:
  - (MED) `settings.json` carried third-party plugin/marketplace config (Toss disclosure) → **removed per the user's decision** (commit `868c7a7`); a guard test blocks re-introduction.
  - (MED) `test_claude_artifacts` scanned only `settings.json` → strengthened to scan all of `claude/` for `/Users/` and to validate JSON via `jq`.
  - (HIGH, cross-cutting) username/project names lingered in git history → user decision: **scrub history before push** (final M3 step); commit author set to the maintainer's public email.
- Consensus: **resolved** (code fixes done; history scrub reserved for the final step).

## E2E verification
- `docs/dstack-backup/evidence/m2-tests.txt` — suite PASS; `settings.json` valid JSON, `$HOME`×3, `/Users/` 0, third-party 0; tracked tree PII 0.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Adversarial review consensus (Codex limit → Claude substitute, resolved)
- [x] E2E capture verified
