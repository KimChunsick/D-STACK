# 05-install-sh

## Intent / Why
The mechanism that actually activates the SSOT. It replaces files in `~/.claude`/`~/.codex` with **symlinks** into the repo, but **backs up any pre-existing real file to `*.bak.<ts>`** and is **idempotent** with a `--dry-run`, so it touches the user's real home safely. See PLAN.md `Milestone 3 / Task 5`.

## What was done (and why)
- A single MAP (`repo_relpath|target_under_$HOME|mode(link|copy)`).
- Per entry: skip if the parent agent dir is absent (e.g. `~/.gemini`) → back up any existing file → `ln -sfn` (copy-mode = `cp -R`, reserved for Gemini).
- `--dry-run` prints the plan only. Re-running is a no-op (idempotent).

## Files changed (where / why)
- `install.sh` — symlink engine + backup + dry-run.
- `tests/test_install_sh.sh` — in a **sandbox HOME** (`mktemp -d`): asserts links created/targets correct/pre-existing file backed up/idempotent/dry-run makes no changes/collision-safe backups. Never touches the real `~`.

## Adversarial review (Codex limit → Claude substitute, user-approved)
- Focused on data loss since the script mutates the real home.
- (HIGH) `ts` was second-granular → **same-second re-runs could overwrite an earlier backup (original lost)** → **fixed**: increment a suffix until the backup name is free; `DSTACK_BACKUP_TS` makes the test deterministic. Verified: `fixedstamp`=ORIGINAL and `fixedstamp.1`=SECOND both survive.
- Other (LOW/MED, no data loss): `set -e` arithmetic `x=$((x+1))` safe; `--dry-run` makes no changes (reviewer retracted); `$HOME` unset → `set -u` aborts immediately; dir backup→link order safe; absolute-path readlink compare correct. Future MAP entries with `..`/`./` noted as an authoring caveat.
- Consensus: **resolved**.

## E2E verification
- `docs/dstack-backup/evidence/m3-install-e2e.txt` — real install into a sandbox HOME: 9 entries all symlinked into the repo, content readable through the symlink, `$HOME` expansion proven, idempotent re-run no-op, suite PASS.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Adversarial review consensus (Codex limit → Claude substitute, resolved)
- [x] E2E capture verified
