---
name: codex-review
description: Adversarial code review of one finished Plan (and the ledger pass that closes a Milestone or Goal) run by Codex gpt-5.6-sol against the frozen request. Use it after the last Task of a Plan is committed and before `dstack plan done`, and again when a Milestone or the Goal closes. Korean triggers a user may type — "리뷰 돌려줘", "코드리뷰 해줘", "Plan 리뷰", "리뷰 한 라운드 더", "마일스톤 대장 점검", "리뷰 봉인해줘".
---

# codex-review — one Plan, one bundle, sealed rounds

The reviewer is Codex, never this session. This session only builds the bundle, launches Codex,
seals what comes back, answers it and decides whether another round is owed. `<T>` below is the
target directory `dstack status` prints (`.dstack/runs/<run>/`).

You never tick a checkbox and you never write a verdict yourself: `dstack review seal` counts the
verdict rows and `dstack report` computes each R's status (§3-1).

## When it runs

| Moment | Scope | Runs? |
|---|---|---|
| Every Task of a Plan is committed, before `dstack plan done` | `plan` | Always. `review: off` cuts rounds and axes, never the per-R verdict (R69) |
| Milestone closes | `milestone` | Always — ledger pass (R70) |
| Goal closes, before `dstack run close` | `milestone` per Milestone with open findings | Always — ledger pass |
| Quick task (`.dstack/quick/<slug>/`) | — | Never: no `plan.json`, so `dstack review` refuses a bundle. Record `skipped: quick target has no plan.json` (see *Skipped*) |
| A Plan whose diff is empty | `plan` | Never. Record `skipped: no diff in the declared files` |

**Skipped**: write one line into `<T>/review/skipped-<scope>-<id>.md` — `skipped: <reason>` — and say
it in the progress message. A phase that quietly does not run is a phase nobody can audit (§3-3).

## What the request frontmatter changes

| Field | `on` / value | Effect |
|---|---|---|
| `review: on` (default) | — | up to 3 rounds per Plan, all five axes |
| `review: off` | — | 1 round, axes reduced to goal achievement + security. The per-R verdict table is still required and `absent` still blocks a positive seal |
| `codex_effort` | medium\|high\|xhigh | `model_reasoning_effort` for every round (R23) |
| `risk_axes` | ux\|perf\|security | the named axis is called out first in the prompt; it never removes an axis |

## One round, four steps

### 1. Build the bundle

```bash
dstack review --scope plan --plan P1
```

It prints the bundle path and runs `dstack check review-bundle` on itself; a bundle that would
hide a requirement is deleted rather than shipped. Do not hand-edit a bundle, and never review
anything the bundle does not contain — silent mis-scoping is worse than failing loudly.

If the command fails with `bundle exceeds 512KB: split the plan`, split the Plan
(`dstack plan insert --after P1 …`) instead of trimming the diff.

### 2. Launch Codex — one background call, then end the turn (R98)

Write the prompt to a file first (it carries quotes and newlines):

```bash
cat > "$T/review/prompt-P1-001.txt" <<'EOF'
You are the D-STACK reviewer role. Read ~/.codex/skills/dstack-reviewer/SKILL.md and follow it exactly.
Bundle (your only statement of intent — read it before anything else): <abs path to bundle>
Scope: plan P1 — round 001 of at most 3.
Previous sealed round: <abs path to codex-review-NNN.md, or "none">
Our answer to it: <abs path to response-NNN.md, or "none">
Axes this round: goal achievement, security, UI·UX&DX, performance, architecture & code quality
First risk axis to weigh: <risk_axes value, or "none declared">
Write your whole output to the file given by -o. Modify no file in the worktree.
EOF
```

Then ONE Bash call with `run_in_background: true`, and end the turn. The completion notification
is the resume signal; never poll, never emit a "still running" turn.

```bash
dstack exec review-P1-001 -- codex exec --ignore-user-config -m gpt-5.6-sol -c model_reasoning_effort=high -C "$WT" --sandbox read-only -o "$T/review/raw-P1-001.md" "$(cat "$T/review/prompt-P1-001.txt")" </dev/null
```

- `</dev/null` is not optional (D-07). With a stdin that is not a terminal the reviewer waits for
  more prompt input until EOF, and a background Bash call never closes it: seven rounds once sat
  idle for half an hour that way.
- `high` is the default. Replace it with the request's `codex_effort` value (`medium|high|xhigh`)
  on every round — the three flags `--ignore-user-config -m gpt-5.6-sol -c model_reasoning_effort=…`
  are what `dstack doctor` counts (R23).
- `$WT` is the `worktree:` line the bundle prints. `--sandbox read-only`: review modifies nothing.
- The label is `review-<plan>-<round-in-this-plan>`. The sealed file's number is a target-wide
  sequence and may differ; `<T>/review/index.tsv` ties label, round and scope together.

### 3. Seal it

```bash
dstack review seal --from "$T/review/raw-P1-001.md" --scope plan --id P1
```

Seal refuses a file with no verdict rows or no `VERDICT:` line, so a laundered review cannot
enter the record. It prints `covered / partial / absent`. A sealed round is never edited, never
re-run into the same file, never deleted.

If the raw file is missing or truncated, read `.dstack/local/exec/review-P1-001/err.txt` and
`exit`; a non-zero exit is a failed round, not a passed one — rerun the same round number.

### 4. Answer it, then decide

Write `<T>/review/response-<NNN>.md` — the same `<NNN>` as the sealed round. It is never bundled
and never contradicts the sealed file; it records what you did about it.

```
# Response to round 001 — plan P1
| finding | sev | decision | where |
|---|---|---|---|
| [security] command injection in lib/run.sh:88 | HIGH | fixed | <commit sha> |
| [perf] list re-renders per keystroke | MEDIUM | rejected: list is capped at 12 rows | — |
effort: raised high → xhigh for the sealing round because two HIGH findings survived round 002
```

Fixes are delegated, not typed here: frontend code → `frontend-dev` (opus), everything else →
`general-dev` (opus), one Plan's worth of context in the brief (R25, §0.2). A rejected finding
needs a reason a reviewer can attack, not a preference.

When the ledger already holds an open `review` case for an R the latest round marks `covered`:

```bash
dstack evidence add --r R01 --case c-review --kind review --artifact "$T/review/codex-review-001.md" --produced-by "dstack review seal"
```

## Loop rule and the cap

Run another round while **either** is true of the round you just sealed:

- it carries at least one HIGH finding, or
- its verdict table carries at least one `absent` (an `absent` can never seal positively, R69).

Otherwise the Plan's review is done: `partial` rows stay `partial` and `dstack report` counts them
as UNMET, so a `partial` you accept is a decision you write in the response file, not a silence.

| Guard | Rule |
|---|---|
| Cap | 3 rounds per Plan (1 when `review: off`) |
| Fresh context | every round is a new Codex run; never resume a session to "continue" a review |
| Counting | count HIGH and MEDIUM in the round you just sealed only — the review directory accumulates history, and grepping across rounds re-counts findings you already fixed |
| Stall | if HIGH+MEDIUM does not fall between two consecutive rounds, stop before the cap: the loop is stuck and a fourth round will not help |
| Effort | the last (sealing) round may raise `codex_effort` one step (`medium→high`, `high→xhigh`); write the one-line reason in the response file (R23) |

**At the cap or on a stall**: append every unresolved finding, verbatim with its `file:line` and
severity, to `<T>/findings.md` as open list items (the Milestone ledger pass reads exactly those
lines). Do **not** run `dstack plan done`; the Plan stays open and the user decides. Say so — an
unresolved round is presented, never swallowed.

## Milestone / Goal ledger pass (R70)

```bash
dstack review --scope milestone --milestone M1
```

The bundle carries the frozen R rows, the open items of `<T>/findings.md`, and the integration
table of the Milestone's Plans. The contract inside it forbids new scope-wide findings: anything
outside the open items and the integration behaviour comes back as one line under `out of scope`.
Same launch, same seal (`--scope milestone --id M1`), same loop rule and cap. Resolve an item by
appending `resolved: <how>` to its line in `findings.md` — the bundle stops carrying it.

If the milestone bundle exceeds the ceiling, review the Plans one at a time; do not truncate.

## Axes and severity

| Axis | What a finding in it looks like |
|---|---|
| goal achievement | the diff does not do what the frozen R row's `accept:` says |
| security | injection, path traversal, secrets in the diff, missing authz, unsafe deserialization |
| UI·UX&DX | broken state/error/empty path, a11y, a message that cannot be acted on, an API that misleads its caller |
| performance | avoidable re-renders, allocations in render, missing memoization, N+1 queries, unvirtualized long lists, work inside a loop that belongs outside it |
| architecture & code quality | wrong boundary, duplicated logic, dead code the diff created, an abstraction with one caller |

Severity is `HIGH | MEDIUM | LOW`; every finding names `file:line`. HIGH means it is wrong,
unsafe, or loses data — it blocks the seal.

**Signal rule**: noise is deleted, not downgraded. A finding you cannot tie to a line and a
concrete failure does not become LOW; it leaves the round. Downgrading to look thorough is the
same lie as approving to look agreeable.

## What you say to the user (Korean 해요체)

- launch: "P1 리뷰 1라운드를 배경으로 돌려요. 끝나면 알림을 받아서 이어갈게요."
- sealed: "1라운드를 봉인했어요. covered 4, partial 1, absent 0이고 HIGH 지적이 하나 있어요. 고친 뒤 2라운드를 돌릴게요."
- cap: "3라운드를 다 썼는데 HIGH 지적 2개가 남아서 P1을 열어 둔 채로 findings.md에 적었어요. 어떻게 할지 정해 주세요."
- skipped: "이 작업은 Plan이 없어서 리뷰 번들을 만들 수 없어요. `skipped: quick target has no plan.json`으로 적어 뒀어요."

## Borrowed from GSD (github.com/open-gsd/gsd-core, read 2026-09-02)

| Sentence taken | From | Used as |
|---|---|---|
| "Find every bug, security vulnerability, and quality defect — do not validate that work was done." | `agents/gsd-code-reviewer.md` | the reviewer's stance in `dstack-reviewer` |
| "Assume every submitted implementation contains defects." | `agents/gsd-code-reviewer.md` | same |
| "Downgrading findings from BLOCKER to WARNING to avoid seeming harsh" (listed as a failure mode) | `agents/gsd-code-reviewer.md` | inverted into the Signal rule: noise is deleted, not downgraded |
| "DO use line numbers. Never \"somewhere in the file\" — always cite specific lines." | `agents/gsd-code-reviewer.md` | `file:line` on every finding |
| "Do NOT invent a heuristic (e.g., HEAD~5) — silent mis-scoping is worse than failing loudly." | `agents/gsd-code-reviewer.md` | never review outside the bundle |
| "DO NOT modify source files. Review is read-only." | `agents/gsd-code-reviewer.md` | `--sandbox read-only` |
| "`status: clean` means \"reviewed and found no issues.\" `status: skipped` means \"no reviewable files — review was not performed.\"" | `agents/gsd-code-reviewer.md` | the `skipped: <reason>` file |
| "Track the number of BLOCKER + WARNING issues … If the count does not decrease between consecutive iterations, the producing agent is stuck and further iterations will not help. Break early and escalate to the user." | `references/revision-loop.md` | the stall guard on HIGH+MEDIUM |
| "Each iteration gets a fresh agent spawn -- don't try to continue in the same context" | `references/revision-loop.md` | one Codex run per round |
| "Don't silently swallow issues -- always present the final state to the user after exiting the loop" | `references/revision-loop.md` | the cap behaviour |
| "Stops when no unresolved HIGH concerns or actionable MEDIUM/LOW findings remain …, or when max cycles is reached." | `skills/gsd-plan-review-convergence/SKILL.md` | the loop rule and the cap of 3 |
| "Do NOT grep REVIEWS.md for HIGH or actionable counts. REVIEWS.md accumulates history across cycles — resolved findings from prior cycles remain in the file … causing false stall detection." | `workflows/plan-review-convergence.md` | count the current sealed round only |

Changed on purpose: GSD reviews with a Claude subagent over a git diff, warns, and puts
performance out of scope; here Codex reviews a bundle whose REQUEST section travels with the
diff, performance is an axis, severities are HIGH/MEDIUM/LOW, and the verdict is a gate —
`absent` blocks the seal and `partial` reports UNMET instead of warning.

## Ending a review on purpose (`dstack review close`)

A review that cannot continue — the round cap is reached, the last HIGH finding was a harness
defect rather than code and its fix was never re-verified, Codex is unavailable — is closed
with `dstack review close --scope plan|milestone|quick --id <id> --why "<reason>"`. Nothing
sealed changes and no round is invented: `review/closed.tsv` records "after round N: why", and
every R the scope covers reads `ABSTAIN` in verify/report until a newer round seals a verdict
for it. The owner then accepts each with `dstack verify --accept-abstain R,R --why "…"` (R79);
a `partial`/`absent` verdict already sealed stays a failure. Never write a skipped-*.md by hand.
