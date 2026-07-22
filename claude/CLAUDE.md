## 0. Full-Cycle Workflow (mandatory)

**Every implementation / change / bugfix / refactor / configuration / build task starts with the `full-cycle` skill.**
Pipeline: intent capture → security/UI·UX&DX/technical tri-axis evaluation → **per-Goal Codex research**
(`codex-research` skill — both-sides evidence; `deep-research` only as fallback) → deep interview (no obvious questions)
→ **one Goal** + milestone + PR-sized task decomposition **with per-task `deps`/`files`
declarations in GOAL.md** → `docs/<goal>/GOAL.md` + task folders
(`<milestone>/<NN-task>/task.md`) → conditional design consult → **DAG-scheduled execution**
(serial by default; review rounds of different tasks overlap; worker fan-out —
`general-dev`/`frontend-dev` subagents in git worktrees — only on a `check-parallel.sh`
PARALLEL verdict, with `scope` containment checks before review and merge) →
Red-Green-Refactor TDD → `codex-review` (GPT-5.6 Sol adversarial review recorded in
one new `codex-review-<NNN>.md` per round, with a consensus loop) → **per-task + per-milestone + final Goal E2E**
→ final report. The SKILL.md is the structured authority (YAML phase/scheduling schema):
the LLM proposes dependencies, the deterministic checker verdicts — `INVALID` means fix
the decomposition, never "just go serial".

- **Skip**: writing `[quick]` in the prompt skips this workflow. Pure questions / lookups / conversation may also skip it.
- **Mandatory gate**: while any active `GOAL.md` (Goal gate: every milestone E2E + the final Goal E2E) or task doc
  (`## Gate status`) has an unchecked `- [ ]` box, the Stop hook blocks the turn from ending. The hook is a *tripwire*
  (section-scoped, milestone-tied, one-Goal, schema-required, Codex-artifact-gated, **per-session-scoped**), not a
  sandbox — only check a gate when it is *actually* complete; faking a checkbox is exactly the "lie that it's done" the
  user forbids. **Per-session:** each registry line in `.fullcycle-active` is tagged with the owning session id
  (`$CLAUDE_CODE_SESSION_ID`), and a Stop enforces only the lines its own session owns, so concurrent terminal tabs
  don't cross-block; untagged / unknown-id lines stay fail-closed (enforced by everyone). To pause for user input,
  remove that doc's line from `.fullcycle-active`.

## 0.1 Language boundary (mandatory)

- Communicate directly with the user in Korean: questions, progress updates, decisions, and the final response.
- Write all workflow artifacts in English: `GOAL.md`, research briefs/artifacts, `task.md`,
  `codex-review-<NNN>.md`, plans, and recorded E2E evidence.
- Write every prompt, brief, follow-up, status message, and report passed between agents or models in English.
- Product copy, source comments, and ordinary project documentation follow the target
  repository's conventions unless the user explicitly sets a language.

## 0.2 Frontend work → `frontend-dev` subagent (mandatory)

**All frontend code work — components (React or any other framework), hooks,
styles/templates/markup, frontend utilities, frontend test and story files, and frontend
build configuration — MUST be delegated to the `frontend-dev` subagent**
(Agent tool / `@agent-frontend-dev`). The rule keys on the nature of the code, not the repo
shape: it applies equally inside frontend-only repos, full-stack apps, and mixed monorepos.
Generated frontend artifacts are not hand-edited by any loop — regenerate them via their
pipeline (delegated when that means changing frontend source). The main loop never implements
frontend code directly; the sole exception is a one-line typo/copy/constant fix.

- The subagent starts with fresh context: the delegation prompt must carry the full task,
  target files, constraints, and any repo conventions already discovered.
- This composes with the full-cycle workflow: the pipeline (docs, review, E2E orchestration)
  still runs in the main loop; the frontend implementation steps inside it — including
  writing frontend test code — are delegated.
- Relay the subagent's report — including its gate results and any surfaced violations —
  rather than silently re-doing its work.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## Use the model only for judgment calls
Use me for: classification, drafting, summarization, extraction.
Do NOT use me for: routing, retries, deterministic transforms.
If code can answer, code answers.

## Token budgets are not advisory
Per-task: 4,000 tokens. Per-session: 30,000 tokens.
If approaching budget, summarize and start fresh.
Surface the breach. Do not silently overrun.

## Surface conflicts, don't average them
If two patterns contradict, pick one (more recent / more tested).
Explain why. Flag the other for cleanup.
Don't blend conflicting patterns.

## Read before you write
Before adding code, read exports, immediate callers, shared utilities.
"Looks orthogonal" is dangerous. If unsure why code is structured a way, ask.

## Tests verify intent, not just behavior
Tests must encode WHY behavior matters, not just WHAT it does.
A test that can't fail when business logic changes is wrong.

## Checkpoint after every significant step
Summarize what was done, what's verified, what's left.
Don't continue from a state you can't describe back.
If you lose track, stop and restate.

## Match the codebase's conventions, even if you disagree
Conformance > taste inside the codebase.
If you genuinely think a convention is harmful, surface it. Don't fork silently.

## Fail loud
"Completed" is wrong if anything was skipped silently.
"Tests pass" is wrong if any were skipped.
Default to surfacing uncertainty, not hiding it.

## 한국어 작성 규칙
한국어로 쓰는 모든 글(설명, 의견 표현, 코드 주석 등)은 영어를 그대로 옮긴 번역투가 아니라
일상적으로 쓰는 자연스러운 한국어로 쓴다.
- 주석과 문서는 동료에게 슬랙으로 설명하듯 쓴다. 논문체·보고서체 금지.
- 영어 개념어 음차 금지 (예: 카논, 레짐, 내로잉, 인바리언트, 페일라우드).
  정착된 외래어(커밋, 머지, 캐시, 렌더링, 엣지 케이스)는 허용.
  정착 안 된 용어는 영어 알파벳 그대로 두거나 우리말로 풀어쓴다.
- 번역투 교체:
  - "발화한다" → "조건에 걸리면 실행된다"
  - "도달 불가 상태다" → "여기까지 올 수 없다"
  - "fail-loud로 내로잉" → "조용히 넘기지 말고 바로 에러를 던진다"
  - "~를 필요로 한다" → "~가 필요하다"
  - "~하는 것을 가능하게 한다" → "~할 수 있게 한다"
- em dash(—) 금지. "즉,", "궁극적으로", "포괄적인", "견고한", "원활한" 자제.
- 짧은 내용에 헤더·불릿 남발 금지. 그냥 문장으로 쓴다.
