#!/bin/bash
# Stop hook: block the turn from ending while any active full-cycle task still has an
# unchecked gate checkbox ("- [ ]"). Active tasks are listed (one doc path per line) in
# .fullcycle-active at the project root. Grounded in real artifacts, not LLM judgment.
#
# Escape hatch (avoids deadlock): if a task genuinely needs to pause for user input,
# the model removes that task's line from .fullcycle-active. The unchecked boxes remain
# in the doc as a visible record that the task is incomplete.
f=".fullcycle-active"
[ -f "$f" ] || exit 0
pending=""
while IFS= read -r doc; do
  [ -z "$doc" ] && continue
  [ -f "$doc" ] || continue
  if grep -qE '^- \[ \]' "$doc"; then
    pending="${pending:+$pending, }$doc"
  fi
done < "$f"
[ -z "$pending" ] && exit 0
reason="full-cycle gate incomplete — you can end the turn only once the 'Gate status' checkboxes (TDD / Codex consensus / E2E) in these task docs are actually complete: $pending. Resolve the remaining gates. If you must wait for user input, remove that task from .fullcycle-active to mark it paused."
jq -n --arg r "$reason" '{decision:"block",reason:$r}'
