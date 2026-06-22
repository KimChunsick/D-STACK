#!/usr/bin/env bash
# install.sh must link the SSOT files into a (sandbox) HOME, back up pre-existing
# files OUTSIDE the live dirs, be idempotent, and make NO changes under --dry-run.
# Never touches the real ~.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
REPO="$(git rev-parse --show-toplevel)"
SBX="$(mktemp -d)"; SBX2="$(mktemp -d)"; trap 'rm -rf "$SBX" "$SBX2"' EXIT
mkdir -p "$SBX/.claude/skills/full-cycle" "$SBX/.codex"
printf 'OLD-USER-CONTENT' > "$SBX/.claude/CLAUDE.md"          # pre-existing real file
printf 'OLD-SKILL' > "$SBX/.claude/skills/full-cycle/SKILL.md" # pre-existing real skill dir

# --dry-run must change nothing.
HOME="$SBX" bash "$REPO/install.sh" --dry-run >/dev/null
if [ -L "$SBX/.claude/CLAUDE.md" ]; then fail "--dry-run created a symlink"; fi
if ! grep -q 'OLD-USER-CONTENT' "$SBX/.claude/CLAUDE.md"; then fail "--dry-run modified an existing file"; fi
if [ -e "$SBX/.dstack-backups" ]; then fail "--dry-run created backups"; fi

# Real run.
HOME="$SBX" bash "$REPO/install.sh" >/dev/null
[ -L "$SBX/.claude/CLAUDE.md" ] || fail "CLAUDE.md not symlinked"
[ "$(readlink "$SBX/.claude/CLAUDE.md")" = "$REPO/claude/CLAUDE.md" ] || fail "wrong CLAUDE.md link target"
[ -L "$SBX/.claude/skills/full-cycle" ] || fail "skill dir not symlinked"
[ -L "$SBX/.codex/instructions.md" ] || fail "codex instructions not symlinked"
[ "$(readlink "$SBX/.codex/rules/default.rules")" = "$REPO/codex/rules/default.rules" ] || fail "wrong codex rules target"

# Backups land OUTSIDE the live dirs, preserving content.
ls "$SBX/.dstack-backups/"*/.claude/CLAUDE.md >/dev/null 2>&1 || fail "pre-existing file not backed up"
grep -rqs 'OLD-USER-CONTENT' "$SBX/.dstack-backups/" || fail "backup lost original content"
grep -rqs 'OLD-SKILL'        "$SBX/.dstack-backups/" || fail "pre-existing skill dir not backed up"

# Regression: a backed-up skill dir must NOT pollute the live skills dir (it would be
# re-discovered as a duplicate skill). Only the symlink may remain.
if ls -d "$SBX/.claude/skills/"*.bak* >/dev/null 2>&1; then fail "backup polluted ~/.claude/skills (would be seen as a skill)"; fi

# Idempotent: a second run creates no new backup and leaves the link intact.
nbak1=$(ls -d "$SBX/.dstack-backups/"*/.claude/CLAUDE.md 2>/dev/null | wc -l | tr -d ' ')
HOME="$SBX" bash "$REPO/install.sh" >/dev/null
nbak2=$(ls -d "$SBX/.dstack-backups/"*/.claude/CLAUDE.md 2>/dev/null | wc -l | tr -d ' ')
[ "$nbak1" = "$nbak2" ] || fail "second run created a redundant backup (not idempotent)"
[ -L "$SBX/.claude/CLAUDE.md" ] || fail "idempotent run broke the symlink"

# Must NOT create an absent agent dir (no ~/.gemini was created).
if [ -e "$SBX/.gemini" ]; then fail "install.sh created an absent agent dir"; fi

# Collision-safe backups: two backup-inducing runs forced to the SAME timestamp must
# not let the second backup clobber the first — the original content must survive.
mkdir -p "$SBX2/.claude" "$SBX2/.codex"
printf 'ORIGINAL' > "$SBX2/.claude/CLAUDE.md"
HOME="$SBX2" DSTACK_BACKUP_TS=fixedstamp bash "$REPO/install.sh" >/dev/null
rm -f "$SBX2/.claude/CLAUDE.md"; printf 'SECOND' > "$SBX2/.claude/CLAUDE.md"
HOME="$SBX2" DSTACK_BACKUP_TS=fixedstamp bash "$REPO/install.sh" >/dev/null
grep -rqs 'ORIGINAL' "$SBX2/.dstack-backups/fixedstamp/" || fail "same-second backup clobbered the original"
grep -rqs 'SECOND'   "$SBX2/.dstack-backups/fixedstamp/" || fail "second backup missing"

pass "install.sh links + backup(out-of-tree) + idempotent + dry-run + collision-safe"
