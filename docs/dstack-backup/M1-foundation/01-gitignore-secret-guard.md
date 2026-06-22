# 01-gitignore-secret-guard

## Intent / Why
This is a public repo, so the top priority is that **no secret/runtime file can ever be committed**. We lay the safety net first — an allowlist (deny-all, then re-include curated paths) plus a hard deny of known secret filenames — before anything else lands. See PLAN.md `Milestone 1 / Task 1`.

## What was done (and why)
- `tests/lib.sh`, `tests/run.sh` (test runner), `tests/test_gitignore_secret_guard.sh` (Red).
- `.gitignore`: `/*` deny → re-include only curated paths → each agent dir is deny-all internally with only curated entries re-included → hard-deny secret names (`auth.json`, `credentials.json`, `id_rsa`, `*.key`, `*.pem`, `*.p12`, `*.token`, `*.sqlite*`, `*.db`, `history.jsonl`, `config.toml`, `.DS_Store`, `sessions/`, `projects/`, `memory/`, `.env*`).

## Files changed (where / why)
- `.gitignore` — block secret ingress (allowlist + per-dir deny-all).
- `tests/lib.sh`, `tests/run.sh` — dependency-free bash test harness.
- `tests/test_gitignore_secret_guard.sh` — assert a battery of secret paths (incl. nested + extensionless) stay ignored and absent from the index; tracked tree matches no secret pattern.

## Adversarial review (Codex limit → Claude substitute, user-approved)
- `codex exec` (GPT-5.5) was refused due to a usage limit → per the user's decision, a Claude adversarial subagent did the review instead.
- **Round 1 (REJECT):** the allowlist re-included whole agent dirs → unanticipated secret names (`claude/id_rsa`, `credentials.json`, `.netrc`, `*.db`, `*.p12`, `*.token`) were trackable (HIGH). Two findings claiming `.env`/`*.key`/`*.pem` match only at root were **empirically refuted** (no-slash patterns match at any depth, confirmed via `git check-ignore`).
  - Action: per-dir deny-all with only curated entries re-included; broadened secret deny list; test gained a leak battery + index check. (commit `b6343ff`)
- **Round 2 (APPROVE_WITH_FIXES):** HIGH closed. Residual MED — names *nested* inside wholesale-re-included `hooks/`/`skills/`/`rules/` were still trackable.
  - Action: pin `hooks/`/`rules/` to exact files and `skills/` to exact skill dirs. Skill-internal files are inherently wholesale → secret-name deny list as backstop, documented in AGENTS.md. Test gained nested-unknown probes. (commit `34fa835`) Verified: all nested-unknown paths ignored, curated content allowed.
- Consensus: **resolved** (HIGH/MED both closed; residual is a documented accepted risk).

## E2E verification
- `docs/dstack-backup/evidence/m1-tests.txt` — `bash tests/run.sh` all PASS + structural checks (`@import`, no secrets in tracked tree, dummy auth.json ignored).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Adversarial review consensus (Codex limit → Claude substitute, resolved)
- [x] E2E capture verified
