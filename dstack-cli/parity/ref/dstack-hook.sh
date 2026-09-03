#!/usr/bin/env bash
# dstack-hook.sh <event> — the ONE hook script registered anywhere (R101).
#
# Claude Code treats only exit 2 as a block. A crashed hook, a timed-out hook and a hook that
# printed malformed JSON all become a notification and the turn continues, so every path that
# cannot compute a verdict has to end in exit 2 on purpose. That is the whole reason this file
# exists instead of four registrations calling the CLI directly: the CLI itself might be the thing
# that is missing, and a missing command exits 127, which Claude Code reads as "carry on".
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
#   #   dstack-hook stop: cannot decide — the CLI … and jq … — fix: …; escape: <abs> run pause
#   # expected: exit=2
#   rm -rf "$H"
# The same run is automated as selftest_hook_fail_closed in the reference's gate.sh, over the
# fixtures in claude/lint/fixtures/hook-fail-closed/.

set -u

EVENT="${1:-}"

# stdin is read exactly once, before anything can fail: the caller writes the whole payload and
# a hook that exits without draining it can leave the client blocked on the pipe.
PAYLOAD="$(cat 2>/dev/null || true)"

# ── locating the tools ──────────────────────────────────────────────────────────────────
DS=""
if [ -n "${DSTACK_BIN:-}" ] && [ -x "${DSTACK_BIN:-}" ]; then
  DS="$DSTACK_BIN"
elif [ -n "${HOME:-}" ] && [ -x "$HOME/.claude/bin/dstack" ]; then
  DS="$HOME/.claude/bin/dstack"
else
  DS="$(command -v dstack 2>/dev/null || true)"
  if [ -z "$DS" ] || [ ! -x "$DS" ]; then DS=""; fi
fi
HAVE_JQ=0
if command -v jq >/dev/null 2>&1; then HAVE_JQ=1; fi

MISSING=""
if [ -z "$DS" ]; then MISSING="dstack (looked at \$DSTACK_BIN, \$HOME/.claude/bin/dstack, then PATH)"; fi
if [ "$HAVE_JQ" -eq 0 ]; then
  if [ -n "$MISSING" ]; then MISSING="$MISSING and jq"; else MISSING="jq"; fi
fi
FIXHINT="install jq (brew install jq) and run D-STACK's install.sh so the CLI is installed at ~/.claude/bin/dstack"
ESCAPE="${DS:-${HOME:-~}/.claude/bin/dstack}"

utc() { date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || printf '%s' '-'; }
worktree() { git rev-parse --show-toplevel 2>/dev/null || true; }

# <worktree>/.dstack/local/hooks/<event>.last — what `dstack doctor` reads back to show the last
# result of every registered hook (R101). The log never decides anything, so it never fails the
# hook: a read-only or absent store just means no line.
log_hook() {
  local ex="$1" note="$2" wt ld
  wt="$(worktree)"; [ -n "$wt" ] || return 0
  ld="$wt/.dstack/local"
  [ -d "$ld" ] || return 0
  mkdir -p "$ld/hooks" 2>/dev/null || return 0
  printf '%s\t%s\t%s\t%s\n' "$EVENT" "$ex" "$(utc)" "$note" >> "$ld/hooks/$EVENT.last" 2>/dev/null || true
  return 0
}

cannot_decide() {
  printf 'dstack-hook %s: cannot decide — %s — fix: %s; escape: %s run pause\n' \
    "$EVENT" "$1" "${2:-$FIXHINT}" "$ESCAPE" >&2
  log_hook 2 "cannot decide: $1"
  exit 2
}

case "$EVENT" in
  inject|stop|agent-model|pre-write) ;;
  *) printf 'dstack-hook %s: cannot decide — unknown event — fix: register one of inject|stop|agent-model|pre-write; escape: %s run pause\n' "${EVENT:-<none>}" "$ESCAPE" >&2; exit 2 ;;
esac

# D-12: inject is a status carrier, not a verdict. It must never block a prompt, so a missing
# the CLI becomes a note the agent can read rather than an exit 2.
if [ -n "$MISSING" ]; then
  if [ "$EVENT" = inject ]; then
    printf 'dstack: status unavailable — missing %s — fix: %s\n' "$MISSING" "$FIXHINT"
    exit 0
  fi
  cannot_decide "missing $MISSING"
fi

# ── payload fields ──────────────────────────────────────────────────────────────────────
jqs() { printf '%s' "$PAYLOAD" | jq -r "$1 // empty" 2>/dev/null || true; }
jqc() { printf '%s' "$PAYLOAD" | jq -c "$1 // {}" 2>/dev/null || printf '%s' '{}'; }

CWD="$(jqs '.cwd')"
if [ -n "$CWD" ] && [ -d "$CWD" ]; then cd "$CWD" || cannot_decide "cannot enter cwd $CWD" "check the directory still exists"; fi

# The store lives beside the main worktree so every linked worktree shares it (design §2).
store_dir() {
  local common main
  common="$(git rev-parse --git-common-dir 2>/dev/null)" || return 1
  [ -n "$common" ] || return 1
  case "$common" in /*) ;; *) common="$(pwd -P)/$common" ;; esac
  main="$(cd "$common/.." 2>/dev/null && pwd -P)" || return 1
  printf '%s\n' "$main/.dstack"
}
has_store() { local s; s="$(store_dir)" || return 1; [ -f "$s/version" ]; }

# ── inject (UserPromptSubmit, R24) ──────────────────────────────────────────────────────
if [ "$EVENT" = inject ]; then
  if ! has_store; then log_hook 0 "no store in this repository"; exit 0; fi
  out="$("$DS" status --oneline 2>/dev/null)"; rc=$?
  if [ "$rc" -eq 2 ] || [ -z "$out" ]; then
    printf 'dstack: status unavailable — the CLI could not read this repository state — fix: run %s run verify\n' "$DS"
    log_hook 0 "status exit $rc"
    exit 0
  fi
  printf '%s\n' "$out"
  log_hook 0 "injected $(printf '%s' "$out" | wc -c | tr -d ' ') bytes"
  exit 0
fi

# ── agent-model (PreToolUse, tool Agent, R22) ───────────────────────────────────────────
if [ "$EVENT" = agent-model ]; then
  tool="$(jqs '.tool_name')"
  if [ "$tool" != Agent ]; then log_hook 0 "tool ${tool:-<none>}: not Agent"; exit 0; fi
  ti="$(jqc '.tool_input')"
  model="$(printf '%s' "$ti" | jq -r '.model // empty' 2>/dev/null || true)"
  case "$model" in
    sonnet|opus) log_hook 0 "model $model: unchanged"; exit 0 ;;
  esac
  # No model means the subagent inherits the session model, which is the Fable this rule exists
  # to keep out of subagents; "inherit", "haiku" and a pinned full id are the same problem.
  was="${model:-(none)}"
  jq -nc --argjson ti "$ti" --arg was "$was" \
    '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"allow",
      permissionDecisionReason:("dstack: model \u0027" + $was + "\u0027 → opus (R22)"),
      updatedInput:($ti + {model:"opus"})}}' \
    || cannot_decide "jq could not build the updatedInput payload"
  hl="${DSTACK_HOOK_LOG:-${HOME:-/tmp}/.claude/dstack-hook.log}"
  mkdir -p "$(dirname "$hl")" 2>/dev/null || true
  printf '%s\tagent-model\tmodel %s → opus (R22)\n' "$(utc)" "$was" >> "$hl" 2>/dev/null || true
  log_hook 0 "model $was → opus"
  exit 0
fi

# ── stop (Stop, R33/R65/R99, D-13) ──────────────────────────────────────────────────────
if [ "$EVENT" = stop ]; then
  if ! has_store; then log_hook 0 "no store in this repository"; exit 0; fi

  # R01 wants the transcript findable from the run folder. meta_set is the only writer of
  # meta.tsv, so the hook borrows it from the resolved dstack's own lib rather than appending
  # a line itself and racing a verb mid-write.
  record_meta() {
    local tp="$1" sid="$2" store wt cur self d libdir
    store="$(store_dir)" || return 0
    wt="$(worktree)"; [ -n "$wt" ] || return 0
    cur="$(cat "$wt/.dstack/local/CURRENT" 2>/dev/null || true)"
    [ -n "$cur" ] || return 0
    [ -d "$store/runs/$cur" ] || return 0
    self="$DS"
    while [ -L "$self" ]; do
      d="$(cd "$(dirname "$self")" && pwd -P)"; self="$(readlink "$self")"
      case "$self" in /*) ;; *) self="$d/$self" ;; esac
    done
    libdir="$(cd "$(dirname "$self")/../lib" 2>/dev/null && pwd -P)" || return 0
    [ -f "$libdir/common.sh" ] || return 0
    . "$libdir/common.sh" 2>/dev/null || return 0
    command -v meta_set >/dev/null 2>&1 || return 0
    [ -n "$tp" ] && meta_set "$store/runs/$cur" transcript_path "$tp"
    [ -n "$sid" ] && meta_set "$store/runs/$cur" owner_session "$sid"
    return 0
  }
  record_meta "$(jqs '.transcript_path')" "$(jqs '.session_id')"

  # D-13: the block is stated once per turn. A second stop in the same turn ends the turn, which
  # is what lets a turn waiting on a background `dstack exec` be re-entered when the run finishes
  # — a gate that could never let a turn end could never be woken up again.
  if [ "$(jqs '.stop_hook_active')" = true ]; then
    log_hook 0 "stop_hook_active: block already stated this turn"
    exit 0
  fi

  out="$("$DS" gate 2>&1)"; rc=$?
  # Written again after the gate: every verb the gate shells out to refreshes ownership through
  # touch_owner, so the payload's session id has to be the last writer or the recorded owner is
  # whatever CLAUDE_CODE_SESSION_ID the checker subprocess happened to carry.
  record_meta "$(jqs '.transcript_path')" "$(jqs '.session_id')"
  case "$rc" in
    0) log_hook 0 "gate clear"; exit 0 ;;
    1)
      reason="${out:0:4000}"
      jq -nc --arg r "$reason" '{decision:"block",reason:$r}' \
        || cannot_decide "jq could not build the block payload"
      log_hook 0 "gate blocked: $(printf '%s' "$out" | tail -1)"
      exit 0
      ;;
    *) cannot_decide "dstack gate exited $rc: $(printf '%s' "$out" | tail -1)" "fix the state named above, or pause the run" ;;
  esac
fi

# ── pre-write (PreToolUse, tools Write|Edit|Bash, R93) ──────────────────────────────────
# Heredoc bodies are the only content a Bash call makes visible; a bare `> file` says nothing
# about what will land there, so it is allowed and the Stop gate's `lint-ko --changed` catches it.
heredoc_bodies() {
  printf '%s\n' "$1" | awk -v q="'" '
    !inh {
      if (match($0, "<<-?[ ]*[\"" q "]?[A-Za-z_][A-Za-z0-9_]*")) {
        tag = substr($0, RSTART, RLENGTH)
        sub(/^<<-?[ ]*/, "", tag)
        gsub("[\"" q "]", "", tag)
        inh = 1
      }
      next
    }
    inh { if ($0 ~ ("^[ \t]*" tag "[ \t]*$")) { inh = 0; next } print }
  '
}

# -m arguments and heredoc bodies of a `git commit`. Newlines are folded to \001 first because
# grep is line-oriented and a commit message argument routinely spans lines.
commit_text() {
  local c="$1"
  printf '%s' "$c" | tr '\n' '\001' | grep -oE -e '-m +"[^"]*"' 2>/dev/null | sed -e 's/^-m *"//' -e 's/"$//' | tr '\001' '\n'
  printf '%s' "$c" | tr '\n' '\001' | grep -oE -e "-m +'[^']*'" 2>/dev/null | sed -e "s/^-m *'//" -e "s/'\$//" | tr '\001' '\n'
  # A heredoc is the message only when it feeds the git commit itself (`git commit -F - <<EOF`).
  # A heredoc elsewhere in the same command (writing a file, then committing) is that file's
  # content and is judged by the file's own scope, not as a commit message.
  if printf '%s' "$c" | tr '\n' ' ' | grep -qE 'git( +--?[A-Za-z-]+( +[^ ]+)?)* +commit[^;|&]*<<' 2>/dev/null; then
    heredoc_bodies "$c"
  fi
}

# First `> path` / `>> path` / `tee path` of the command. `2>&1` and `>&2` cannot match: the
# character before `>` is a digit and the character after is `&`, both excluded.
redirect_path() {
  local c="$1" p
  p="$(printf '%s' "$c" | tr '\n' ' ' | grep -oE -e '(^|[^0-9&<>])>>?[[:space:]]*[^[:space:];&|<>]+' 2>/dev/null | head -1 | sed -E 's/^.*>>?[[:space:]]*//')"
  if [ -z "$p" ]; then
    p="$(printf '%s' "$c" | tr '\n' ' ' | grep -oE -e 'tee( +-a)? +[^[:space:];&|]+' 2>/dev/null | head -1 | sed -E 's/^tee( +-a)? +//')"
  fi
  case "$p" in /dev/*|-|'') p="" ;; esac
  printf '%s\n' "$p"
}

tool="$(jqs '.tool_name')"
LINT_PATH=""; LINT_CONTENT=""; LINT_FRAG=0; LINT_COMMIT=0
case "$tool" in
  Write)
    LINT_PATH="$(jqs '.tool_input.file_path')"
    LINT_CONTENT="$(printf '%s' "$PAYLOAD" | jq -r '.tool_input.content // ""' 2>/dev/null || true)"
    ;;
  Edit)
    # An Edit shows one fragment of a file; sentence-level rules are deferred to the Stop gate,
    # where the whole file is readable (R93).
    LINT_PATH="$(jqs '.tool_input.file_path')"
    LINT_CONTENT="$(printf '%s' "$PAYLOAD" | jq -r '.tool_input.new_string // ""' 2>/dev/null || true)"
    LINT_FRAG=1
    ;;
  Bash)
    cmd="$(printf '%s' "$PAYLOAD" | jq -r '.tool_input.command // ""' 2>/dev/null || true)"
    if [ -z "$cmd" ]; then log_hook 0 "empty command"; exit 0; fi
    if printf '%s' "$cmd" | tr '\n' ' ' | grep -qE 'git( +--?[A-Za-z-]+( +[^ ]+)?)* +commit' 2>/dev/null; then
      # R66's --no-verify and Codex's own commits skip git's hooks, so the message can only be
      # checked here, on the Bash argument itself.
      LINT_CONTENT="$(commit_text "$cmd")"
      LINT_COMMIT=1
      if [ -z "$LINT_CONTENT" ]; then log_hook 0 "git commit with no visible message"; exit 0; fi
    else
      LINT_PATH="$(redirect_path "$cmd")"
      if [ -z "$LINT_PATH" ]; then log_hook 0 "no file creation detected"; exit 0; fi
      LINT_CONTENT="$(heredoc_bodies "$cmd")"
      if [ -z "$LINT_CONTENT" ]; then log_hook 0 "redirect to $LINT_PATH with no visible content"; exit 0; fi
    fi
    ;;
  *) log_hook 0 "tool ${tool:-<none>}: not linted"; exit 0 ;;
esac

if [ "$LINT_COMMIT" -eq 0 ] && [ -z "$LINT_PATH" ]; then log_hook 0 "no path to scope"; exit 0; fi

# lint-ko needs no store: a commit message in a repository that never ran `dstack init` is still
# checkable, and an unscoped path simply matches no row and blocks nothing (R93).
if [ "$LINT_COMMIT" -eq 1 ]; then
  out="$(printf '%s\n' "$LINT_CONTENT" | "$DS" lint-ko --stdin --scope commit-msg 2>&1)"; rc=$?
elif [ "$LINT_FRAG" -eq 1 ]; then
  out="$(printf '%s\n' "$LINT_CONTENT" | "$DS" lint-ko --stdin --path "$LINT_PATH" --fragment 2>&1)"; rc=$?
else
  out="$(printf '%s\n' "$LINT_CONTENT" | "$DS" lint-ko --stdin --path "$LINT_PATH" 2>&1)"; rc=$?
fi

case "$rc" in
  0) log_hook 0 "allow ${LINT_PATH:-<commit-msg>}"; exit 0 ;;
  1)
    reason="$(printf '%s\n' "$out" | head -20)"
    jq -nc --arg r "$reason" \
      '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:$r}}' \
      || cannot_decide "jq could not build the deny payload"
    log_hook 0 "deny ${LINT_PATH:-<commit-msg>}: $(printf '%s' "$out" | head -1)"
    exit 0
    ;;
  *) cannot_decide "dstack lint-ko exited $rc: $(printf '%s' "$out" | tail -1)" "fix the rule or scope table the message names" ;;
esac
