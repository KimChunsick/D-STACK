---
name: codex-review
description: Adversarial review of a completed task by Codex CLI (GPT-5.5). Use after a task's docs/.md is written and TDD is green, before marking the task complete — Phase 9 of the full-cycle pipeline. Sends the task doc plus the code diff to `codex exec` for a hostile critique (security / technical / UI&UX + software structure + "does it satisfy the real Why"), records the verdict into the doc, and runs a Claude<->GPT rebuttal loop until consensus or resolution.
---

# Codex Adversarial Review (GPT-5.5)

Codex is already configured (`~/.codex/config.toml`: `model = "gpt-5.5"`,
`reasoning_effort = "xhigh"`), so `codex exec` runs GPT-5.5 non-interactively.

## Step 1 — Assemble the review material
Gather the task doc and the diff into one input. From the project root:
```bash
TASK_DOC="docs/<work>/<milestone>/<NN-task>.md"
{
  echo "=== TASK DOC ==="; cat "$TASK_DOC"
  echo; echo "=== CODE DIFF ==="; git diff HEAD 2>/dev/null || echo "(no git diff available)"
} > /tmp/fc-review-input.txt
```
If the project is not a git repo, substitute the relevant changed-file contents for the
diff section.

## Step 2 — Run the adversarial review
```bash
codex exec --skip-git-repo-check "You are an adversarial code reviewer. Critically verify the attached task doc and diff from these angles: (1) security (2) technical correctness (3) UI/UX (4) software structure/design (5) whether this work actually satisfies the real intent (Why) written in the doc. No praise or summary. Focus on weaknesses, risks, counterexamples, and missed edge cases. Format each point as '[severity:high/medium/low][axis] content'. On the last line write 'GPT verdict: approve | approve-with-fixes | reject' and a one-sentence rationale." < /tmp/fc-review-input.txt
```
- `--skip-git-repo-check` is required, or codex refuses to run outside a trusted git
  repo ("Not inside a trusted directory").
- The configured reasoning effort (`xhigh`) is used automatically — do not lower it for
  real reviews.
- macOS has no `timeout`; if you need a deadline use `gtimeout` (coreutils) or run plain.

## Step 3 — Record and rebut
Paste GPT's verdict into the task doc under `## Codex review (GPT-5.5)`. Then, for each
point, respond honestly in the same section — verify the claim, don't perform
agreement, don't blindly comply:
- Agree → fix it, note the fix.
- Disagree → write your counter-argument with evidence.

## Step 4 — Consensus loop
If disagreements remain, send your rebuttals back to Codex (repeat Step 2 with the
updated doc as input) and iterate. Continue until the doc's `Consensus` line reads
**agreed** (both sides agree) or **resolved** (raised issues fixed). Only then tick
the Codex checkbox in `## Gate status`.

Keep both sides' arguments in the document — the record of *why* a decision was made is
the point, not just the final answer.
