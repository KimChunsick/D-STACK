---
name: full-cycle
description: MANDATORY delivery pipeline for ANY implementation or change task — features, bugfixes, refactors, configuration, anything that edits files or builds something. Invoke at the START of such work. Drives intent capture, security/UX/technical tri-axis evaluation, deep research (incl. opposing views), deep interview, milestone + PR-sized task decomposition, docs/ documentation, Red-Green-Refactor TDD, adversarial Codex (GPT-5.5) review with an in-document consensus loop, E2E capture verification, and a final report. Skip ONLY when the user wrote [quick], or the request is pure Q&A / lookup / conversation with no file changes.
---

# Full-Cycle Delivery

The user's standing process for all real work. Every phase below is a gate. Do not
skip phases. Do not mark a task's gate checkbox until that gate is *actually* satisfied
— a Stop hook blocks the turn from ending while any active task has an unchecked gate.

If the user wrote `[quick]` in the prompt, this skill does not apply — answer directly.

## Phase checklist (create one todo per phase)

1. Intent capture
2. Tri-axis evaluation (security / UI&UX / technical)
3. Deep research (only if needed)
4. Deep interview
5. Milestone + task decomposition
6. docs/ scaffold
7. Per-task TDD (Red → Green → Refactor)
8. Per-task documentation
9. Codex (GPT-5.5) adversarial review + consensus loop
10. E2E capture verification
11. Final report

---

## Phase 1 — Intent capture
State, in your own words, what you believe the user is really trying to achieve and
*why*. Separate the literal request from the underlying goal. Surface assumptions.

## Phase 2 — Tri-axis evaluation
Evaluate the task across three axes before designing anything. Be concrete, not generic.
- **Security**: attack surface, data exposure, authz/authn, injection, secrets, supply chain.
- **UI & UX**: user flow, failure states, accessibility, clarity, friction.
- **Technical**: architecture fit, complexity, performance, maintainability, blast radius.
For each axis note risks and open questions. These feed Phases 3 and 4.

## Phase 3 — Deep research (conditional)
If any axis raised a question you cannot answer with confidence, invoke the
`deep-research` skill. Research thoroughly — not just confirming evidence but
**opposing views and counter-arguments**. Do not stop at the first plausible source.
Skip this phase only when nothing is genuinely uncertain (say so explicitly).

## Phase 4 — Deep interview
Interview the user to close the remaining gaps in intent. Use the `brainstorming`
skill's discipline: one question at a time, prefer multiple choice.
**Do not ask obvious questions.** Ask only what changes the design. Continue until
the intent is fully pinned down.

## Phase 5 — Milestone + task decomposition
Invoke the `writing-plans` skill. Break the work into the largest units (milestones),
then break each milestone into detailed sub-tasks. **Correct task size = roughly one
human-reviewable PR** (a bit larger is fine). The priority is splitting the problem
small. Number milestones and tasks.

## Phase 6 — docs/ scaffold
Under the project root, create:
```
docs/<work-name>/<milestone>/<NN-task-name>.md
```
One `.md` per task. Use the template at the bottom of this file. As each task starts,
append its doc path to `.fullcycle-active` at the project root (one path per line) —
the Stop hook reads this file to know which tasks are in flight.

## Phase 7 — Per-task TDD
For every task, follow the `test-driven-development` skill: **Red** (write a failing
test that encodes *why* the behavior matters) → **Green** (minimum code to pass) →
**Refactor** (clean up, tests stay green). Then tick the TDD checkbox in the task doc.

## Phase 8 — Per-task documentation
In the task `.md`, record: what was done, why it was done, the Why it serves, which
files were changed and why each change was made. Write this *as you work*, not after.

## Phase 9 — Codex adversarial review
Invoke the `codex-review` skill on the task doc. It sends the doc + diff to Codex
(GPT-5.5) for adversarial verification across security/technical/UI&UX, software
structure, and "does this satisfy the real Why". Record GPT's verdict in the doc and
run the rebuttal/consensus loop **inside the document** until **agreed** (both sides
agree) or **resolved** (raised issues fixed). Then tick the Codex checkbox.

## Phase 10 — E2E capture verification
Verify the task actually works, hands-on (invoke `verify` / `run` skills as fitting):
- **Web**: drive a headless browser (agent browser), capture a screenshot, confirm the
  behavior in the capture.
- **Desktop app**: capture the screen and confirm it runs/behaves correctly.
- **CLI/library/program**: run it and confirm it executes correctly.
Save the evidence path into the task doc. Never claim it works without direct evidence.
Then tick the E2E checkbox. When all gates for a task are ticked, remove its line from
`.fullcycle-active`.

## Phase 11 — Final report
When all tasks are done, summarize: which tasks completed, how each finished, what was
verified, what changed, and any follow-ups. This is the close-out the user expects.

---

## Task document template
```markdown
# <NN-task-name>

## Intent / Why
<what this task achieves and why it matters to the overall work>

## What was done (what / why)
<what was done and why>

## Files changed (where / why)
- `path` — <why this change>

## Codex review (GPT-5.5)
<GPT verdict recorded here; Claude<->GPT rebuttals appended until consensus>
- Consensus: in progress | agreed | resolved

## E2E verification
<evidence: screenshot path / run output>

## Gate status
- [ ] TDD: Red→Green→Refactor complete
- [ ] Codex (GPT-5.5) adversarial review consensus
- [ ] E2E capture verified
```

The Stop hook scans active task docs for unchecked `- [ ]` boxes. Tick a box only when
that gate is genuinely satisfied — ticking without doing the work defeats the system
and is exactly the "completed-but-skipped" failure the user forbids.
