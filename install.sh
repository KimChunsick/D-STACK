#!/usr/bin/env bash
# D-STACK installer — wires this repository into ~/.claude and ~/.codex (R10–R12, R17, R52, R102).
#
# The repository is the source of truth: every managed target is a symlink back here, so an edit
# in the repo is live immediately and `git clone && ./install.sh` on an empty machine restores
# hooks, model policy, skills, agents, the output style and the status line. The CLI is the one
# target that is compiled first: ~/.claude/bin/dstack links to the release binary of the crate,
# so the install builds it and needs cargo (R08). Settings are the one exception:
# ~/.claude/settings.json is MERGED (jq) from claude/settings.enforced.json and
# claude/settings/model-policy.json, because the live file also carries machine-only blocks
# (autoMode …) that never belong in a repository.
#
# Safety (R102): live config is backed up to ~/.dstack-backups/<ts>/ before it is touched, every
# merge goes through a temp file that must parse before it replaces the original, and a failure
# leaves the original in place and exits non-zero. `--dry-run` prints the row table and the
# settings key diff and changes nothing.
#
# Not managed, on purpose: ~/.claude.json (MCP servers — R15, later), ~/.codex/config.toml
# (provider model flags belong to dstack mode exec, never this file), ~/.zshrc.
set -eu

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"
BIN_REL="dstack-cli/target/release/dstack"   # what ~/.claude/bin/dstack points at (R08)
TAB="$(printf '\t')"
DRY_RUN=0
case "$#:${1:-}" in
  0:) : ;;
  1:--dry-run|1:-n) DRY_RUN=1 ;;
  1:--help|1:-h) printf 'usage: ./install.sh [--dry-run]\n'; exit 0 ;;
  *) printf 'install.sh: usage: ./install.sh [--dry-run]\n' >&2; exit 2 ;;
esac

command -v jq >/dev/null 2>&1 || { printf 'install.sh: jq is required (brew install jq)\n' >&2; exit 2; }

ts="${DSTACK_BACKUP_TS:-$(date +%Y%m%d-%H%M%S)}"
case "$ts" in ''|*/*|.|..|*..*) printf 'install.sh: DSTACK_BACKUP_TS must be a plain name\n' >&2; exit 2 ;; esac
backup_root="$HOME/.dstack-backups/$ts"
backup_used=0
backup_of() {   # move $1 (a live path) into the backup root, collision-free
  local target="$1" rel="${1#"$HOME/"}" bak n=1
  bak="$backup_root/$rel"
  while [ -e "$bak" ] || [ -L "$bak" ]; do bak="$backup_root/$rel.$n"; n=$((n + 1)); done
  if [ "$DRY_RUN" = 0 ]; then mkdir -p "$(dirname "$bak")"; mv "$target" "$bak"; fi
  backup_used=1
  printf '%s' "${bak#"$HOME/"}"
}

say() { printf '%s\n' "$*"; }
row() { printf '  %-44s → %-40s  %s\n' "$1" "$2" "$3"; }
deps_install() {   # the install column deps.tsv gives for the tool $1
  local want="$1" name probe install rest
  while IFS="$TAB" read -r name probe install rest; do
    [ "$name" = "$want" ] && { printf '%s' "$install"; return 0; }
  done < "$REPO_DIR/deps.tsv"
}

# ── MAP: repo path | target under $HOME | mode ─────────────────────────────────────────
# link: symlink, replacing (after backup) whatever is there.
# seed: symlink only when the target is absent; an existing file that is not our link is the
#       user's own and is reported as "exists, kept".
MAP="
claude/CLAUDE.md|.claude/CLAUDE.md|link
$BIN_REL|.claude/bin/dstack|link
$BIN_REL|.codex/bin/dstack|link
claude/runtime.md|.claude/runtime.md|link
claude/runtime.md|.codex/runtime.md|link
claude/hooks/dstack-hook.sh|.claude/hooks/dstack-hook.sh|link
claude/statusline-command.sh|.claude/statusline-command.sh|link
claude/output-styles/dstack-korean.md|.claude/output-styles/dstack-korean.md|link
claude/output-styles/dstack-korean.md|.codex/output-styles/dstack-korean.md|link
claude/lint/ko-scope.tsv|.claude/lint/ko-scope.tsv|link
claude/lint/ko-rules.tsv|.claude/lint/ko-rules.tsv|link
codex/AGENTS.md|.codex/AGENTS.md|link
"
for host in claude codex; do
  for f in "$REPO_DIR"/claude/agents/*.md; do [ -e "$f" ] && MAP="$MAP
claude/agents/$(basename "$f")|.$host/agents/$(basename "$f")|link"; done
  for d in "$REPO_DIR"/claude/skills/*/; do [ -d "$d" ] && MAP="$MAP
claude/skills/$(basename "$d")|.$host/skills/$(basename "$d")|link"; done
  for d in "$REPO_DIR"/codex/skills/*/; do [ -d "$d" ] && MAP="$MAP
codex/skills/$(basename "$d")|.$host/skills/$(basename "$d")|link"; done
done

say "D-STACK installer  (repo: $REPO_DIR)"
[ "$DRY_RUN" = 1 ] && say "** DRY RUN — nothing is changed **"

# ── the CLI binary (R08) ───────────────────────────────────────────────────────────────
# ~/.claude/bin/dstack is a link to the compiled binary, so it is built before anything is
# linked; a build that fails leaves the live config untouched. --dry-run compiles nothing.
# --target-dir is passed because the link names one fixed path: without it CARGO_TARGET_DIR or a
# build.target-dir in a cargo config would put the binary elsewhere and the row would silently
# become "source missing". The binary is checked after the build for the same reason.
build_binary() { cargo build --release --manifest-path "$REPO_DIR/dstack-cli/Cargo.toml" --target-dir "$REPO_DIR/dstack-cli/target"; }
say ""; say "binary: $BIN_REL"
if [ "$DRY_RUN" = 1 ]; then
  say "  would build: cargo build --release --manifest-path '$REPO_DIR/dstack-cli/Cargo.toml' --target-dir '$REPO_DIR/dstack-cli/target'"
else
  command -v cargo >/dev/null 2>&1 || { say "  ABORT: cargo builds the dstack binary and is not on PATH — install: $(deps_install cargo)"; exit 1; }
  build_binary || { say "  ABORT: the build failed (see above) — nothing was linked"; exit 1; }
  [ -f "$REPO_DIR/$BIN_REL" ] && [ -x "$REPO_DIR/$BIN_REL" ] || { say "  ABORT: the build left no executable at $BIN_REL — nothing was linked"; exit 1; }
  say "  built"
fi

# R11: the agent roots are created, never used as a reason to skip.
for d in "$HOME/.claude" "$HOME/.codex"; do
  if [ ! -d "$d" ]; then
    [ "$DRY_RUN" = 1 ] && say "  would create $d" || { mkdir -p "$d"; say "  created $d"; }
  fi
done

linked=0; uptodate=0; kept=0; skipped=0; backed=0
say ""; say "rows (source → target, status):"
while IFS='|' read -r rel target mode; do
  [ -n "$rel" ] || continue
  src="$REPO_DIR/$rel"; dst="$HOME/$target"
  if [ ! -e "$src" ]; then row "$rel" "$target" "skipped: source missing in the repository"; skipped=$((skipped + 1)); continue; fi
  if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then row "$rel" "$target" "up-to-date"; uptodate=$((uptodate + 1)); continue; fi
  if [ "$mode" = seed ] && { [ -e "$dst" ] || [ -L "$dst" ]; }; then row "$rel" "$target" "exists, kept (your own file; the repo copy is a template)"; kept=$((kept + 1)); continue; fi
  status="linked"
  if [ -e "$dst" ] || [ -L "$dst" ]; then
    bak="$(backup_of "$dst")"; status="linked (previous moved to $bak)"; backed=$((backed + 1))
  fi
  if [ "$DRY_RUN" = 0 ]; then mkdir -p "$(dirname "$dst")"; ln -sfn "$src" "$dst"; fi
  row "$rel" "$target" "$status"; linked=$((linked + 1))
done <<EOF
$MAP
EOF

# ── settings.json merge (R12, R17, R21, R102) ──────────────────────────────────────────
say ""; say "settings: ~/.claude/settings.json ← claude/settings.enforced.json + claude/settings/model-policy.json"
live="$HOME/.claude/settings.json"
enf="$REPO_DIR/claude/settings.enforced.json"; pol="$REPO_DIR/claude/settings/model-policy.json"
[ -f "$live" ] && live_json="$(cat "$live")" || live_json='{}'
printf '%s' "$live_json" | jq -e . >/dev/null 2>&1 || { say "  ABORT: $live is not valid JSON — fix it by hand (nothing was changed)"; exit 1; }
# Hooks are not deep-merged: entries that point at personal-os or at an older dstack/full-cycle
# hook are dropped, everything else the user registered is kept, and the enforced entries are
# appended per event. A deep merge would leave a stale hook registered forever (R17).
merged="$(printf '%s' "$live_json" | jq --slurpfile e "$enf" --slurpfile p "$pol" '
  def keep_hook: ((.hooks // []) | map(.command // "") | all(test("personal-os|dstack-hook\\.sh|fullcycle-") | not));
  def clean: with_entries(.value |= map(select(keep_hook))) | with_entries(select(.value | length > 0));
  ($e[0]) as $enf | ($p[0]) as $pol
  | ((.hooks // {}) | clean) as $lh
  | (. * $enf * $pol)
  | .hooks = ($enf.hooks | to_entries | reduce .[] as $x ($lh; .[$x.key] = (($lh[$x.key] // []) + $x.value)))
')" || { say "  ABORT: merge failed (jq error) — $live untouched"; exit 1; }
printf '%s' "$merged" | jq -e . >/dev/null 2>&1 || { say "  ABORT: merged result does not parse — $live untouched"; exit 1; }
if [ "$DRY_RUN" = 1 ]; then
  say "  key diff (live → merged):"
  diff -u <(printf '%s' "$live_json" | jq -S .) <(printf '%s' "$merged" | jq -S .) | sed -n '3,200p' | sed 's/^/    /' || true
else
  if [ -f "$live" ]; then bak="$(backup_of "$live")"; say "  backup: $bak"; [ -f "$live" ] || cp "$backup_root/${bak#*/}" "$live" 2>/dev/null || true; fi
  # backup_of moved the live file; restore a working copy from the backup so a failure below leaves it intact
  [ -f "$live" ] || { [ -n "${bak:-}" ] && cp "$HOME/$bak" "$live"; }
  tmp="$live.tmp.$$"
  printf '%s\n' "$merged" > "$tmp" && jq -e . "$tmp" >/dev/null 2>&1 || { rm -f "$tmp"; say "  ABORT: could not write $tmp — $live untouched"; exit 1; }
  mv -f "$tmp" "$live"; chmod 600 "$live"
  say "  merged"
fi
say "  registered hooks:"
printf '%s' "$merged" | jq -r '.hooks | to_entries[] | .key as $ev | .value[] | (.matcher // "*") as $m | .hooks[] | "    \($ev) [\($m)] → \(.command)"'
say "  outputStyle: $(printf '%s' "$merged" | jq -r '.outputStyle // "unset"')  (applies to new sessions and after /clear — R92)"
say "  env.CLAUDE_CODE_SUBAGENT_MODEL: $(printf '%s' "$merged" | jq -r '.env.CLAUDE_CODE_SUBAGENT_MODEL // "unset"')"

say ""
say "summary: linked=$linked up-to-date=$uptodate exists-kept=$kept skipped=$skipped backed-up=$backed"
[ "$backup_used" = 1 ] && say "backups: $backup_root"
say "not managed: ~/.claude.json (R15 later), ~/.codex/config.toml (provider flags belong to dstack mode exec), ~/.zshrc"
case ":$PATH:" in *":$HOME/.claude/bin:"*|*":$HOME/.codex/bin:"*) ;; *) say "note: add \$HOME/.claude/bin or \$HOME/.codex/bin to PATH so dstack resolves; both links use the same binary" ;; esac
if [ "$DRY_RUN" = 0 ] && [ -x "$HOME/.claude/bin/dstack" ]; then
  say ""; say "dstack doctor:"
  "$HOME/.claude/bin/dstack" doctor || say "doctor reported problems (see above); the install itself is complete"
fi
