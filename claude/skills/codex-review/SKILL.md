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
codex exec --skip-git-repo-check "당신은 적대적(adversarial) 코드 리뷰어입니다. 첨부된 태스크 문서와 diff를 다음 관점에서 비판적으로 검증하세요: (1) 보안 (2) 기술적 정확성 (3) UI/UX (4) 소프트웨어 구조/설계 (5) 이 작업이 문서에 적힌 진짜 의도(Why)를 실제로 충족하는가. 칭찬·요약 금지. 약점, 리스크, 반례, 놓친 엣지케이스 위주로 지적하세요. 각 지적은 '[심각도:높음/중간/낮음][축] 내용' 형식. 마지막 줄에 'GPT 판정: 승인 | 조건부승인 | 거부' 와 한 문장 근거." < /tmp/fc-review-input.txt
```
- `--skip-git-repo-check` is required, or codex refuses to run outside a trusted git
  repo ("Not inside a trusted directory").
- The configured reasoning effort (`xhigh`) is used automatically — do not lower it for
  real reviews.
- macOS has no `timeout`; if you need a deadline use `gtimeout` (coreutils) or run plain.

## Step 3 — Record and rebut
Paste GPT's verdict into the task doc under `## Codex 리뷰 (GPT-5.5)`. Then, for each
point, respond honestly in the same section using the `receiving-code-review` skill's
discipline — verify the claim, don't perform agreement, don't blindly comply:
- Agree → fix it, note the fix.
- Disagree → write your counter-argument with evidence.

## Step 4 — Consensus loop
If disagreements remain, send your rebuttals back to Codex (repeat Step 2 with the
updated doc as input) and iterate. Continue until the doc's `합의 상태` line reads
**합의완료** (both sides agree) or **수정완료** (raised issues fixed). Only then tick
the Codex checkbox in `## 게이트 상태`.

Keep both sides' arguments in the document — the record of *why* a decision was made is
the point, not just the final answer.
