#!/bin/bash
# UserPromptSubmit hook: inject the full-cycle directive into every user prompt,
# UNLESS the prompt contains the skip token [quick].
input=$(cat)
prompt=$(printf '%s' "$input" | jq -r '.prompt // empty' 2>/dev/null)
case "$prompt" in
  *'[quick]'*) exit 0 ;;
esac
ctx='[full-cycle enforced] If this request touches files — implementation, changes, bugfixes, refactors, configuration, builds — first invoke the full-cycle skill via the Skill tool and follow the whole pipeline to the end: intent capture -> security/UI·UX/technical tri-axis evaluation -> (if uncertain) deep-research (including opposing views) -> deep interview (no obvious questions) -> milestone + PR-sized task decomposition -> docs/ documentation -> Red-Green-Refactor TDD -> codex-review (GPT-5.5 adversarial verification + in-document consensus loop) -> E2E capture verification -> final report. Only check a task doc Gate-status checkbox when it is actually complete (while any is unchecked, the Stop hook blocks the turn from ending). Pure questions / lookups / conversation may skip this.'
jq -n --arg c "$ctx" '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:$c}}'
