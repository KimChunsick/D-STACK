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
claude/ultracode.zsh|.claude/ultracode.zsh|link
claude/hooks/fullcycle-inject.sh|.claude/hooks/fullcycle-inject.sh|link
claude/hooks/fullcycle-gate.sh|.claude/hooks/fullcycle-gate.sh|link
claude/skills/full-cycle|.claude/skills/full-cycle|link
claude/skills/codex-review|.claude/skills/codex-review|link
claude/skills/codex-research|.claude/skills/codex-research|link
claude/agents/frontend-dev.md|.claude/agents/frontend-dev.md|link
claude/agents/general-dev.md|.claude/agents/general-dev.md|link
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

# ── Ultracode zsh hook: ensure ~/.zshrc sources the alias fragment ──
# The fragment (claude/ultracode.zsh → ~/.claude/ultracode.zsh) is only live if
# ~/.zshrc sources it. That hook line used to be a manual, machine-local step; a
# zshrc rewrite silently dropped it once (2026-07), losing the ultracode default.
# Managed here, `./install.sh` becomes the standard remedy after any zshrc churn.
# Exact-line grep keeps it idempotent; absent ~/.claude skips (same rule as the map).
ZSHRC="$HOME/.zshrc"
HOOK='[ -f "$HOME/.claude/ultracode.zsh" ] && source "$HOME/.claude/ultracode.zsh"'
if [ ! -d "$HOME/.claude" ]; then
  note "  - agent dir absent ($HOME/.claude) — skip zshrc hook"; skipped=$((skipped + 1))
elif [ -f "$ZSHRC" ] && grep -qxF "$HOOK" "$ZSHRC"; then
  note "  = zshrc hook up to date: ultracode.zsh"; noop=$((noop + 1))
elif [ "$DRY_RUN" = 1 ]; then
  note "    [dry-run] append ultracode source hook to ~/.zshrc"; linked=$((linked + 1))
else
  printf '\n# D-STACK: ultracode-by-default (hook managed by install.sh)\n%s\n' "$HOOK" >> "$ZSHRC"
  note "  + zshrc hook appended: source ~/.claude/ultracode.zsh"; linked=$((linked + 1))
fi
# Presence is not effectiveness: a later ~/.zshrc line can override the alias, and
# `unsetopt aliases` disables expansion while `alias` still prints the definition.
# Verify the OUTCOME in a bounded, stdin-detached interactive zsh. That probed shell
# is CONTAMINATED by ~/.zshrc — startup aliases rewrite unquoted words in anything it
# parses, including (in an earlier design) the verifier's own `exit` guards — so NO
# decision logic runs inside it: it emits a single state-dump line and the PARENT
# compares that line byte-for-byte against the one expected value. The dump embeds a
# nonce, so an early-exiting `.zshrc` (e.g. `[[ -t 0 ]] || exit 0`) can fake an exit
# status but not the dump. Timeout is reported as explicitly UNVERIFIED, and the
# whole probe process TREE is killed (interactive zsh ignores TERM, and killing the
# shell alone would orphan a blocking startup child). Warn loudly, never hard-fail
# the installer on a shell-environment quirk. (Residuals, accepted: the startup files
# run once here — same class as opening a terminal; and a FUNCTION shadowing `print`
# or `zmodload`, or a startup file that reads $DSTACK_VERIFY_OUT and forges the full
# expected line, is out of the trust model — ~/.zshrc already executes arbitrary
# code; this check catches accidental breakage and straightforward overrides, and it
# DOES detect a `claude` wrapper function.)
kill_tree() { local c; for c in $(pgrep -P "$1" 2>/dev/null); do kill_tree "$c"; done; kill -KILL "$1" 2>/dev/null || true; }
if [ "$DRY_RUN" = 0 ] && [ -d "$HOME/.claude" ] && command -v zsh >/dev/null 2>&1 \
   && command -v script >/dev/null 2>&1; then
  vout="$(mktemp)"
  vnonce="dstack-$$-$(od -An -N4 -tx4 /dev/urandom | tr -d ' ')"
  # Channel design: the OUTPUT PATH travels via the environment and is expanded
  # inside the zsh source as a quoted variable, so any valid TMPDIR (spaces,
  # brackets, metacharacters) stays inert; the NONCE travels only inside the argv
  # (never the environment), so a startup file cannot casually read it and forge
  # success — and, being a safe unique token, it doubles as the process-ownership
  # key the traps use to find the verifier tree even before $! is recorded
  # ($! itself is the fallback inside the handlers). (A startup file that parses
  # `ps` for the nonce is out of the trust model — ~/.zshrc already executes
  # arbitrary code in every shell; this check catches ACCIDENTAL breakage.)
  # Handlers must be unconditionally zero-returning: under set -e a handler whose
  # last test is false would abort mid-cleanup, skip the re-raise, and turn a
  # cancellation into exit 1. Ownership has three redundant identities so no
  # cancellation instant escapes: (1) the child SELF-REGISTERS its PID to $vpidf as
  # its first action before exec (handlers grace-wait ~150ms for that write, which
  # covers the fork-to-exec window); (2) the unique nonce in the exec'd argv;
  # (3) $vpid once the parent records it. Re-raising never depends on any of them.
  vpidf="$(mktemp)"
  vpid=""
  vkill() {
    local p t i
    for i in 1 2 3; do [ -s "$vpidf" ] && break; sleep 0.05; done
    t="$(cat "$vpidf" 2>/dev/null || true)"
    if [ -n "$t" ]; then kill_tree "$t"; fi
    for p in $(pgrep -f "$vnonce" 2>/dev/null); do kill_tree "$p"; done
    if [ -n "$vpid" ]; then kill_tree "$vpid"; fi
    return 0
  }
  vsig() { vkill; rm -f "$vout" "$vpidf"; trap - "$1" EXIT; kill -s "$1" $$; }
  trap 'vsig INT' INT; trap 'vsig TERM' TERM; trap 'vsig HUP' HUP
  trap 'vkill; rm -f "$vout" "$vpidf"' EXIT
  # script(1) allocates a REAL pseudo-terminal, so the probed startup takes the same
  # `[[ -t 0 ]]`-true path a human terminal takes — a detached probe would follow the
  # non-TTY branch and could bless a hook a real terminal then overrides. The probe
  # NEVER EXECUTES the alias body (an eval-based capture was rejected in review: a
  # compound body like `command claude --effort high; claude …` would really launch
  # Claude during install and still look verified) and carries NO decision logic —
  # startup aliases apply to anything this shell parses, so an in-shell `… && exit 7`
  # guard could itself be neutralized by `alias exit=true` (found in review). It
  # emits one state-dump line built purely from zsh/parameter metadata expansions —
  # alias-expansion state, the managed alias text, `claude` wrapper-function
  # presence, global aliases over any word of the expansion — using exactly two
  # backslash-escaped (alias-immune) command words; the parent compares the line
  # byte-for-byte below. Any construct that cannot be proven equivalent without
  # execution surfaces as a mismatched line and is conservatively reported
  # ineffective.
  DSTACK_VERIFY_OUT="$vout" \
    bash -c 'printf "%s" "$$" > "$1"; shift; exec "$@"' _ "$vpidf" \
    script -q /dev/null zsh -ic '\zmodload zsh/parameter 2>/dev/null
      \print -r -- "n='"$vnonce"' o=${options[aliases]-} f=${+functions[claude]} g1=${+galiases[claude]} g2=${+galiases[--effort]} g3=${+galiases[ultracode]} a=${aliases[claude]-}" > "$DSTACK_VERIFY_OUT"' \
    </dev/null >/dev/null 2>&1 &
  vpid=$!
  i=0
  while kill -0 "$vpid" 2>/dev/null && [ "$i" -lt 100 ]; do sleep 0.1; i=$((i + 1)); done
  if kill -0 "$vpid" 2>/dev/null; then
    vkill
    wait "$vpid" 2>/dev/null || true                           # reap; 137 must not trip set -e
    note "  ! WARNING: zsh startup exceeded 10s — ultracode hook effectiveness UNVERIFIED. Something in ~/.zshrc blocks interactive startup; inspect and fix that first, then re-run ./install.sh for a bounded re-check"
  else
    wait "$vpid" 2>/dev/null || true
    vexpect="n=$vnonce o=on f=0 g1=0 g2=0 g3=0 a=claude --effort ultracode"
    if [ "$(cat "$vout" 2>/dev/null)" != "$vexpect" ]; then
      note "  ! WARNING: interactive zsh does not effectively expand 'claude' to the ultracode invocation — an override, 'unsetopt aliases', a global alias rewrite, a pre-existing 'claude' wrapper function, or an early-exiting ~/.zshrc defeats or hides the hook"
    fi
  fi
  rm -f "$vout" "$vpidf"
  trap - INT TERM HUP EXIT
elif [ "$DRY_RUN" = 0 ] && [ -d "$HOME/.claude" ]; then
  note "  - zsh or script(1) unavailable — ultracode hook effectiveness not verified"
fi

note ""
note "Summary: linked=$linked copied=$copied backed-up=$backed up-to-date=$noop skipped=$skipped"
if [ "$DRY_RUN" = 1 ]; then note "(dry-run — re-run without --dry-run to apply)"; fi
