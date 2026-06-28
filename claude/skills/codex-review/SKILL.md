---
name: codex-review
description: Adversarial review of a completed task by Codex CLI (GPT-5.5). Use after a task's docs/.md is written and TDD is green, before marking the task complete — Phase 9 of the full-cycle pipeline. Sends the task doc plus the code diff to `codex exec` for a hostile critique (security / technical / UI&UX&DX + software structure + "does it satisfy the real Why"; also challenges the research's assumptions), records the verdict into a separate codex-review.md in the task folder, and runs a Claude<->GPT rebuttal loop until consensus or resolution.
---

# Codex Adversarial Review (GPT-5.5)

Codex is already configured (`~/.codex/config.toml`: `model = "gpt-5.5"`,
`reasoning_effort = "xhigh"`), so `codex exec` runs GPT-5.5 non-interactively.

## Step 1 — Assemble the review material (fail-closed, allowlist)
Review material is built by a **fail-closed allowlist** helper: you name exactly the files this
task changed/created (plus the Goal's research artifacts), and **nothing else is sent** — so an
unnamed secret cannot leak. The helper also gates each named file (symlink skip, secret-name
deny backstop, ≤64KB, binary skip) and emits a *scoped* diff per file (never a repo-wide
`git diff`). The prior `codex-review.md` is folded in so consensus rounds keep the record.
```bash
TASK_DIR="docs/<goal>/<milestone>/<NN-task>"      # the task FOLDER
# Allowlist — the ONLY files sent. List what this task touched + the Goal's research artifacts.
FILES=( path/to/changed1 path/to/new2 docs/<goal>/research/*.md )
IN="$(mktemp)"; chmod 600 "$IN"; trap 'rm -f "$IN"' EXIT
bash "$HOME/.claude/skills/codex-review/assemble-review.sh" "$TASK_DIR" "${FILES[@]}" > "$IN"
```
The helper (`assemble-review.sh`, tested in `tests/test_codex_review_assembler.sh`) is the
enforcement point — do not hand-roll the bundle or pass a repo-wide diff. Feed `"$IN"` to Step 2.

## Step 2 — Run the adversarial review
```bash
codex exec --skip-git-repo-check "You are an adversarial code reviewer. Critically verify the attached task doc and diff from these angles: (1) security (2) technical correctness (3) UI/UX & DX (developer experience) (4) software structure/design (5) whether this work actually satisfies the real intent (Why) written in the doc. If the work rests on prior research, also challenge that research's own assumptions. No praise or summary. Focus on weaknesses, risks, counterexamples, and missed edge cases. Format each point as '[severity:high/medium/low][axis] content'. On the last line write 'GPT verdict: approve | approve-with-fixes | reject' and a one-sentence rationale." --ephemeral < "$IN"
```
- `--skip-git-repo-check` is required, or codex refuses to run outside a trusted git
  repo ("Not inside a trusted directory").
- The configured reasoning effort (`xhigh`) is used automatically — do not lower it for
  real reviews.
- macOS has no `timeout`; if you need a deadline use `gtimeout` (coreutils) or run plain.

## Step 3 — Record and rebut (in a SEPARATE file)
Write GPT's verdict into a **separate `codex-review.md` in the task folder**
(`$TASK_DIR/codex-review.md`) — not inline in `task.md`. Then, for each point, respond
honestly in that same file — verify the claim, don't perform agreement, don't blindly comply:
- Agree → fix it, note the fix.
- Disagree → write your counter-argument with evidence.

Keeping the review in its own file (one per task) preserves the full Codex↔Claude record
without bloating the task doc, and is what the maintainer's step 11/12 require.

## Step 4 — Consensus loop
If disagreements remain, send your rebuttals back to Codex (repeat Step 2 with the updated
`codex-review.md` as input) and iterate. Continue until that file's `Consensus` line reads
**agreed** (both sides agree) or **resolved** (raised issues fixed). Only then tick the Codex
checkbox in the task doc's `## Gate status`.

Keep both sides' arguments in `codex-review.md` — the record of *why* a decision was made is
the point, not just the final answer.
