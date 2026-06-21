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
reason="full-cycle 게이트 미완료 — 다음 태스크 문서의 '게이트 상태' 체크박스(TDD / Codex 합의 / E2E)를 실제로 완료해야 턴을 끝낼 수 있습니다: $pending. 남은 게이트를 처리하세요. 사용자 입력을 기다려야 한다면 해당 태스크를 .fullcycle-active 에서 제거해 일시중지로 표시하세요."
jq -n --arg r "$reason" '{decision:"block",reason:$r}'
