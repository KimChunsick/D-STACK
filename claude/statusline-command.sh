#!/usr/bin/env bash
# Claude Code status line:  model │ project │ branch (wt: worktree) │ goal: <dstack slug> │ ctx bar
#
# Reads the Status JSON Claude Code pipes in (model.display_name, workspace.project_dir,
# workspace.current_dir, context_window.used_percentage). Every part degrades on its own — "?"
# for a missing field, "-" outside git or without a run — so one absent value never blanks the
# whole line. used_percentage is null before the first API call and right after /compact; the
# bar then shows "?" instead of pretending 0%. The bar is built as ${bar}█, never $bar█: the stock
# bash of macOS swallows the first byte of a multibyte character after a bare $name expansion.
input="$(cat 2>/dev/null)"
have_jq=0; command -v jq >/dev/null 2>&1 && [ -n "$input" ] && have_jq=1
j() { [ "$have_jq" = 1 ] && printf '%s' "$input" | jq -r "$1" 2>/dev/null; }

model="$(j '.model.display_name // .model.id // empty')"; model="${model:-?}"
cwd="$(j '.workspace.current_dir // .cwd // empty')"; cwd="${cwd:-$PWD}"
proj="$(j '.workspace.project_dir // empty')"; proj="${proj:-$cwd}"
project="$(basename "$proj")"

# Branch, plus the worktree name when this checkout is a linked worktree (its git-common-dir
# lives in another checkout).
branch="-"; wt=""; top=""
if top="$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null)"; then
  branch="$(git -C "$cwd" rev-parse --abbrev-ref HEAD 2>/dev/null || echo detached)"
  common="$(git -C "$cwd" rev-parse --git-common-dir 2>/dev/null)"
  case "$common" in /*) ;; *) common="$cwd/$common" ;; esac
  main="$(cd "$common/.." 2>/dev/null && pwd -P)"
  [ -n "$main" ] && [ "$main" != "$(cd "$top" && pwd -P)" ] && wt="$(basename "$top")"
fi
loc="$branch"; [ -n "$wt" ] && loc="$branch (wt: $wt)"

# The dstack Goal of this worktree: the slug of .dstack/local/CURRENT (timestamp stripped).
goal="-"
if [ -n "$top" ] && [ -s "$top/.dstack/local/CURRENT" ]; then
  goal="$(cat "$top/.dstack/local/CURRENT")"; goal="${goal#*_}"
fi

# Context bar: ten cells, filled = used_percentage rounded to the nearest 10%. Colour turns
# yellow from 50% and red from 80%, the range where /compact or a fresh session is worth it.
ctx="ctx ░░░░░░░░░░ ?"
pct="$(j '.context_window.used_percentage // empty')"
p="${pct%%.*}"
case "$p" in ''|*[!0-9]*) p="" ;; esac
if [ -n "$p" ]; then
  [ "$p" -gt 100 ] && p=100
  filled=$(( (p + 5) / 10 )); bar=""; i=0
  while [ "$i" -lt 10 ]; do
    if [ "$i" -lt "$filled" ]; then bar="${bar}█"; else bar="${bar}░"; fi
    i=$((i + 1))
  done
  if [ "$p" -ge 80 ]; then col='\033[31m'; elif [ "$p" -ge 50 ]; then col='\033[33m'; else col='\033[32m'; fi
  ctx="$(printf 'ctx %b%s\033[0m %s%%' "$col" "$bar" "$p")"
fi

printf '%s │ %s │ %s │ goal: %s │ %s' "$model" "$project" "$loc" "$goal" "$ctx"
