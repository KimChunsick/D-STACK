#!/bin/bash
# UserPromptSubmit hook: inject the full-cycle directive into every user prompt,
# UNLESS the prompt contains the skip token [quick].
input=$(cat)
prompt=$(printf '%s' "$input" | jq -r '.prompt // empty' 2>/dev/null)
case "$prompt" in
  *'[quick]'*) exit 0 ;;
esac
ctx='[full-cycle 강제] 이 요청이 구현·변경·버그수정·리팩터·설정·빌드 등 파일을 만지는 작업이면, 먼저 Skill 도구로 full-cycle 을 호출해 전체 파이프라인을 끝까지 따르세요: 의도캡처 → 보안/UI·UX/기술 3축 평가 → (불확실하면) deep-research(반대의견 포함) → 심층 인터뷰(뻔한 질문 금지) → 마일스톤+PR단위 태스크 분해 → docs/ 문서화 → Red-Green-Refactor TDD → codex-review(GPT-5.5 적대적 검증 + 문서 내 합의 루프) → E2E 캡처 검증 → 최종 보고. 태스크 문서의 "게이트 상태" 체크박스는 실제로 완료했을 때만 체크하세요(미완료 시 Stop 훅이 턴 종료를 차단합니다). 순수 질문·조회·대화면 생략 가능.'
jq -n --arg c "$ctx" '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:$c}}'
