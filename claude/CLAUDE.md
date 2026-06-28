## 0. Full-Cycle Workflow (mandatory)

**Every implementation / change / bugfix / refactor / configuration / build task starts with the `full-cycle` skill.**
Pipeline: intent capture → security/UI·UX&DX/technical tri-axis evaluation → **per-Goal Codex research**
(`codex-research` skill — both-sides evidence; `deep-research` only as fallback) → deep interview (no obvious questions)
→ **one Goal** + milestone + PR-sized task decomposition → `docs/<goal>/GOAL.md` + task folders
(`<milestone>/<NN-task>/task.md`) → Red-Green-Refactor TDD → `codex-review` (GPT-5.5 adversarial review recorded in a
separate `codex-review.md` + consensus loop) → **per-task + per-milestone + final Goal E2E** → final report.

- **Skip**: writing `[quick]` in the prompt skips this workflow. Pure questions / lookups / conversation may also skip it.
- **Mandatory gate**: while any active `GOAL.md` (Goal gate: every milestone E2E + the final Goal E2E) or task doc
  (`## Gate status`) has an unchecked `- [ ]` box, the Stop hook blocks the turn from ending. The hook is a *tripwire*
  (section-scoped, milestone-tied, one-Goal, schema-required, Codex-artifact-gated), not a sandbox — only check a gate
  when it is *actually* complete; faking a checkbox is exactly the "lie that it's done" the user forbids. To pause for
  user input, remove that doc from `.fullcycle-active`.

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