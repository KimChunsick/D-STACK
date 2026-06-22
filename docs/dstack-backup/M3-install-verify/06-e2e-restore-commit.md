# 06-e2e-restore-commit

## Intent / Why
Hands-on proof that "restore on a new machine in one command" actually works, and an explicit user-confirmed procedure for the risky real-home install. Final initial commit (push after user approval). See PLAN.md `Milestone 3 / Task 6`.

## What was done (and why)
- Full sandbox-HOME E2E: `./install.sh` → captured `find -type l` proving every MAP entry symlinks into the repo.
- Confirmed `$HOME` expansion actually resolves (Claude runs hook commands via `sh -c`).
- Captured the full suite green.
- README/AGENTS.md state the real-home install as a user-confirmed step (recommend `--dry-run` first, `*.bak` backups), plus an uninstall procedure.

## Files changed (where / why)
- `docs/dstack-backup/evidence/*` — E2E evidence.
- `README.md` — restore/uninstall procedure.

## Adversarial review (Codex limit → Claude substitute, user-approved)
- Reviewed the sufficiency of the E2E evidence: all symlink targets resolve inside the repo, and the `$HOME` proof matches the hook execution path (`sh -c`) → satisfied.
- The real-home install is documented as a user-confirmed step (README: `--dry-run` first + `*.bak` backups + uninstall).
- Consensus: **resolved**.

## E2E verification
- `docs/dstack-backup/evidence/m3-install-e2e.txt` — full capture. Idempotent 2nd run up-to-date=9, 5 test files PASS.

## Gate status
- [x] TDD: Red→Green→Refactor complete (the install test encodes the E2E behavior)
- [x] Adversarial review consensus (Codex limit → Claude substitute, resolved)
- [x] E2E capture verified
