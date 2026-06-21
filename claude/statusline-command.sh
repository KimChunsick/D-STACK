#!/bin/sh
input=$(cat)

model=$(echo "$input" | jq -r '.model.display_name // "Unknown"')
cwd=$(echo "$input" | jq -r '.workspace.current_dir // .cwd // "?"')
used=$(echo "$input" | jq -r '.context_window.used_percentage // empty')
remaining=$(echo "$input" | jq -r '.context_window.remaining_percentage // empty')

# Context bar (10 blocks)
if [ -n "$used" ]; then
  filled=$(printf '%.0f' "$(echo "$used / 10" | bc -l)")
  [ "$filled" -gt 10 ] && filled=10
  empty=$((10 - filled))
  bar=""
  i=0
  while [ $i -lt $filled ]; do bar="${bar}█"; i=$((i+1)); done
  i=0
  while [ $i -lt $empty ]; do bar="${bar}░"; i=$((i+1)); done
  pct=$(printf '%.0f' "$used")
  ctx_part="🧠 ${bar} ${pct}%"
else
  ctx_part="🧠 --"
fi

# Project dir: show last 2 path components
short_cwd=$(echo "$cwd" | awk -F'/' '{if(NF>=2) print $(NF-1)"/"$NF; else print $NF}')

printf "🤖 %s  📁 %s  %s" "$model" "$short_cwd" "$ctx_part"
