#!/usr/bin/env bash
# D-STACK installer — link this repo's authored configs into the live agent dirs.
#
# The repo is the single source of truth; ~/.claude, ~/.codex (etc.) get symlinks
# pointing back here. Idempotent. Any pre-existing real file is backed up to
# <target>.bak.<timestamp> before linking. `--dry-run` prints the plan only.
#
# Per-agent policy: Claude and Codex follow symlinked config files → `link`. Gemini
# CLI intentionally ignores symlinked context files (GH google-gemini/gemini-cli#11547)
# → use `copy` for any future gemini entry, and re-run after editing.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"

DRY_RUN=0
if [ "${1:-}" = "--dry-run" ]; then DRY_RUN=1; fi

# MAP rows: repo_relpath | target_under_$HOME | mode(link|copy)
MAP="
claude/CLAUDE.md|.claude/CLAUDE.md|link
claude/settings.json|.claude/settings.json|link
claude/statusline-command.sh|.claude/statusline-command.sh|link
claude/hooks/fullcycle-inject.sh|.claude/hooks/fullcycle-inject.sh|link
claude/hooks/fullcycle-gate.sh|.claude/hooks/fullcycle-gate.sh|link
claude/skills/full-cycle|.claude/skills/full-cycle|link
claude/skills/codex-review|.claude/skills/codex-review|link
claude/skills/codex-research|.claude/skills/codex-research|link
codex/AGENTS.md|.codex/AGENTS.md|link
codex/instructions.md|.codex/instructions.md|link
codex/rules/default.rules|.codex/rules/default.rules|link
"

# Timestamp for backup names (overridable for deterministic tests).
ts="${DSTACK_BACKUP_TS:-$(date +%Y%m%d-%H%M%S)}"
# Backups go OUTSIDE the live agent dirs so a backed-up skill/hook dir is never
# re-discovered as a skill/hook. Structure under the root mirrors the live path.
backup_root="$HOME/.dstack-backups/$ts"
linked=0; copied=0; backed=0; noop=0; skipped=0

note() { printf '%s\n' "$*"; }
run()  { if [ "$DRY_RUN" = 1 ]; then note "    [dry-run] $*"; else "$@"; fi; }

note "D-STACK installer  (repo: $REPO_DIR)"
if [ "$DRY_RUN" = 1 ]; then note "** DRY RUN — no changes will be made **"; fi

while IFS='|' read -r rel target mode; do
  if [ -z "$rel" ]; then continue; fi
  src="$REPO_DIR/$rel"
  dst="$HOME/$target"
  agent_root="$HOME/$(printf '%s' "$target" | cut -d/ -f1)"

  if [ ! -e "$src" ]; then
    note "  ! source missing, skip: $rel"; skipped=$((skipped + 1)); continue
  fi
  if [ ! -d "$agent_root" ]; then
    note "  - agent dir absent ($agent_root) — skip: $target"; skipped=$((skipped + 1)); continue
  fi

  # Idempotent: already the exact symlink we want.
  if [ "$mode" = link ] && [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then
    note "  = up to date: $target"; noop=$((noop + 1)); continue
  fi

  # Back up anything already there into the backup root (outside live dirs). Pick a
  # collision-free name so a backup never overwrites an earlier backup.
  if [ -e "$dst" ] || [ -L "$dst" ]; then
    bak="$backup_root/$target"; n=1
    while [ -e "$bak" ] || [ -L "$bak" ]; do bak="$backup_root/$target.$n"; n=$((n + 1)); done
    run mkdir -p "$(dirname "$bak")"
    run mv "$dst" "$bak"
    note "  ~ backed up existing → ${bak#"$HOME/"}"; backed=$((backed + 1))
  fi

  run mkdir -p "$(dirname "$dst")"
  if [ "$mode" = copy ]; then
    run cp -R "$src" "$dst"
    note "  + copied: $target"; copied=$((copied + 1))
  else
    run ln -sfn "$src" "$dst"
    note "  + linked: $target → $rel"; linked=$((linked + 1))
  fi
done <<EOF
$MAP
EOF

note ""
note "Summary: linked=$linked copied=$copied backed-up=$backed up-to-date=$noop skipped=$skipped"
if [ "$DRY_RUN" = 1 ]; then note "(dry-run — re-run without --dry-run to apply)"; fi
