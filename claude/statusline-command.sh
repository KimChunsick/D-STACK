#!/bin/sh
input=$(cat)

model=$(echo "$input" | jq -r '.model.display_name // "Unknown"')
cwd=$(echo "$input" | jq -r '.workspace.current_dir // .cwd // "?"')
project_dir=$(echo "$input" | jq -r '.workspace.project_dir // empty')

# Context usage %: prefer the precomputed field, else derive from tokens (older CLI)
used=$(echo "$input" | jq -r '
  if .context_window.used_percentage != null then .context_window.used_percentage
  elif (.context_window.context_window_size // 0) > 0 then
    ((.context_window.total_input_tokens // 0) / .context_window.context_window_size * 100)
  else empty end
')

# Project: basename of project_dir (fallback to cwd)
[ -z "$project_dir" ] && project_dir="$cwd"
project=$(basename "$project_dir")

# Git branch (omitted entirely when not inside a work tree)
branch=""
if git --no-optional-locks -C "$cwd" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  branch=$(git --no-optional-locks -C "$cwd" branch --show-current 2>/dev/null)
  if [ -z "$branch" ]; then
    sha=$(git --no-optional-locks -C "$cwd" rev-parse --short HEAD 2>/dev/null)
    [ -n "$sha" ] && branch="${sha} (detached)"
  fi
fi

# Context bar (10 blocks) + %, colored by pressure
ESC=$(printf '\033')
RESET="${ESC}[0m"
if [ -n "$used" ]; then
  pct=$(printf '%.0f' "$used")
  [ "$pct" -lt 0 ] && pct=0
  [ "$pct" -gt 100 ] && pct=100
  filled=$(( (pct + 5) / 10 ))
  [ "$filled" -gt 10 ] && filled=10
  empty=$((10 - filled))
  bar=""
  i=0
  while [ $i -lt $filled ]; do bar="${bar}█"; i=$((i+1)); done
  i=0
  while [ $i -lt $empty ]; do bar="${bar}░"; i=$((i+1)); done

  if [ "$pct" -ge 80 ]; then color="${ESC}[31m"
  elif [ "$pct" -ge 50 ]; then color="${ESC}[33m"
  else color="${ESC}[32m"
  fi
  ctx_part="🧠 ${color}${bar}${RESET} ${pct}%"
else
  ctx_part="🧠 --"
fi

if [ -n "$branch" ]; then
  printf "📁 %s  🌿 %s  🤖 %s  %s" "$project" "$branch" "$model" "$ctx_part"
else
  printf "📁 %s  🤖 %s  %s" "$project" "$model" "$ctx_part"
fi
