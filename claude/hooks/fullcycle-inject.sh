#!/bin/bash
# UserPromptSubmit hook: inject the full-cycle directive into every user prompt,
# UNLESS the prompt contains the skip token [quick].
input=$(cat)
prompt=$(printf '%s' "$input" | jq -r '.prompt // empty' 2>/dev/null)
case "$prompt" in
  *'[quick]'*) exit 0 ;;
esac
ctx='[full-cycle enforced] If this request touches files — implementation, changes, bugfixes, refactors, configuration, builds — first invoke the full-cycle skill via the Skill tool and follow the whole pipeline to the end: intent capture -> security/UI·UX&DX/technical tri-axis -> per-Goal Codex research (codex-research skill: both-sides evidence; deep-research only as fallback) -> deep interview (no obvious questions) -> one Goal + milestone + PR-sized task decomposition -> docs/<goal>/GOAL.md + task folders -> Red-Green-Refactor TDD -> codex-review (GPT-5.6 Sol; one new codex-review-<NNN>.md per round; each finding includes evidence, a rough suggested direction, a small code/pseudocode/ASCII example, an explicit non-prescriptive reviewer-opinion caveat that it is not a patch to copy verbatim, and verification; continue until genuine consensus) -> per-task + per-milestone + final Goal E2E -> final report. Communicate with the user in Korean; write all workflow artifacts and every agent-to-agent prompt/report in English. Only check a Gate-status / Goal-gate checkbox when it is actually complete (while any is unchecked in an active GOAL.md or task doc, the Stop hook blocks the turn from ending). Pure questions / lookups / conversation may skip this.'
jq -n --arg c "$ctx" '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:$c}}'
