#!/bin/bash
# UserPromptSubmit hook: nudge the full-cycle skill into play on every user prompt,
# UNLESS the prompt contains the skip token [quick].
#
# WHY THIS IS SHORT. This text is prepended to EVERY user prompt, including pure questions, so
# it is the one string in this setup whose cost scales with turn count. It used to restate the
# entire pipeline in 1,850 bytes / 1,845 characters — a near-verbatim copy of section 0 of
# `~/.claude/CLAUDE.md`, which is already loaded for the whole session. It is now 465 bytes /
# 461 characters. Both units are given because they differ here (the text contains multi-byte
# characters) and because an earlier note reported "1,857 characters", which was neither — it
# was a byte count that also included jq's trailing newline. The duplicate bought nothing: the
# skill body carries the real detail either way, so the standing rules were being paid for on
# every turn to say what was already in context.
#
# What is left is the only thing a per-prompt injection can do that an always-loaded file
# cannot: keep the trigger in front of the model at the moment it decides how to answer. Add
# nothing here that CLAUDE.md or the skill already says; point at them instead.
#
# Honest scope on the saving: once this block is stable it sits inside the cached prefix, so on
# a cache hit its direct cost is cache-read rate rather than full input rate. The context window
# it occupies is real regardless, and that is the part that mattered.
input=$(cat)
prompt=$(printf '%s' "$input" | jq -r '.prompt // empty' 2>/dev/null)
case "$prompt" in
  *'[quick]'*) exit 0 ;;
esac
ctx='[full-cycle enforced] If this request touches files — implementation, change, bugfix, refactor, configuration, build — invoke the `full-cycle` skill via the Skill tool BEFORE doing anything else, and follow it to the end. The skill carries the pipeline; CLAUDE.md carries the standing rules (Korean to the user, English artifacts, frontend delegation, and never ticking a gate box that is not actually done). Pure questions, lookups, and conversation skip this.'
jq -n --arg c "$ctx" '{hookSpecificOutput:{hookEventName:"UserPromptSubmit",additionalContext:$c}}'
