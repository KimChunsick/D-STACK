#!/usr/bin/env bash
# dstack-hook.sh <event> — the ONE hook script registered anywhere (R101).
#
# Claude Code treats only exit 2 as a block. A crashed hook, a timed-out hook and a hook that
# printed malformed JSON all become a notification and the turn continues, so every path that
# cannot compute a verdict has to end in exit 2 on purpose. That is the whole reason this file
# exists instead of four registrations calling the CLI directly: the CLI itself might be the thing
# that is missing, and a missing command exits 127, which Claude Code reads as "carry on".
#
# Everything the events decide — the payload, the model rewrite, the Stop verdict, the Korean
# check of a pending write — lives in the binary as `dstack hook <event>` (D-01). This script
# finds the binary and hands the payload over unread.
#
# Events: inject (UserPromptSubmit) | stop (Stop) | agent-model (PreToolUse Agent)
#         pre-write (PreToolUse Write|Edit|Bash)
#
# ── R101 evidence procedure (reproduce the exit-2 block by hand) ────────────────────────
#   H=$(mktemp -d)                      # a HOME with no ~/.claude/bin/dstack
#   printf '%s' '{"session_id":"s","transcript_path":"/tmp/t.jsonl","cwd":".",
#                 "hook_event_name":"Stop","stop_hook_active":false}' > /tmp/stop.json
#   env -u DSTACK_BIN PATH=/usr/bin:/bin HOME="$H" \
#     claude/hooks/dstack-hook.sh stop < /tmp/stop.json; echo "exit=$?"
#   # expected on stderr, one line:
#   #   dstack-hook stop: cannot decide — missing dstack … — fix: …; escape: <abs> run pause
#   # expected: exit=2
#   rm -rf "$H"
# The same run is automated as the hook-fail-closed checker of `dstack doctor --self`, over the
# fixtures in claude/lint/fixtures/hook-fail-closed/.

set -u

EVENT="${1:-}"

# ── locating the binary ─────────────────────────────────────────────────────────────────
# A candidate counts only when it is a regular file that can be run. -x alone is not that test:
# a searchable directory satisfies it, and starting one ends in 126 — which Claude Code reads as
# "carry on", the one answer a hook must never give by accident (R101). -f follows symlinks, so a
# link to the binary passes and a dangling one does not.
usable() { [ -f "$1" ] && [ -x "$1" ]; }

DS=""
if [ -n "${DSTACK_BIN:-}" ] && usable "${DSTACK_BIN:-}"; then
  DS="$DSTACK_BIN"
elif [ -n "${HOME:-}" ] && usable "$HOME/.claude/bin/dstack"; then
  DS="$HOME/.claude/bin/dstack"
else
  DS="$(command -v dstack 2>/dev/null || true)"
  usable "$DS" || DS=""
fi

# The payload is not read here: the binary reads stdin exactly once, and a payload this script
# had consumed would reach it empty.
if [ -n "$DS" ]; then
  "$DS" hook "$EVENT"
  rc=$?
  # 126 and 127 are the shell's own "I could not run that": the candidate passed the test above
  # and still never started, so no verdict was computed and the block below is the answer. Every
  # other code is the binary's own and is this hook's.
  case "$rc" in 126|127) ;; *) exit "$rc" ;; esac
fi

# ── the one verdict this script still computes: no binary this script could run ─────────
# stdin is drained first, because a hook that exits without reading the payload can leave the
# client blocked on the pipe.
cat >/dev/null 2>&1 || true
MISSING='dstack (looked at $DSTACK_BIN, $HOME/.claude/bin/dstack, then PATH)'
FIXHINT="run D-STACK's install.sh so the CLI is installed at ~/.claude/bin/dstack"
ESCAPE="${HOME:-~}/.claude/bin/dstack"

# D-12: inject is a status carrier, not a verdict. It must never block a prompt, so a binary that
# is missing or cannot be started becomes a note the agent can read rather than an exit 2.
if [ "$EVENT" = inject ]; then
  printf 'dstack: status unavailable — missing %s — fix: %s\n' "$MISSING" "$FIXHINT"
  exit 0
fi

printf 'dstack-hook %s: cannot decide — missing %s — fix: %s; escape: %s run pause\n' \
  "${EVENT:-<none>}" "$MISSING" "$FIXHINT" "$ESCAPE" >&2
exit 2
