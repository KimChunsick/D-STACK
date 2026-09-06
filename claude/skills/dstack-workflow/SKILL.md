---
name: dstack-workflow
description: The entry point of the pipeline. Use it when a new piece of work arrives in a repository that has a .dstack/ store — a feature, a fix, a refactor, a document — and it has to be routed (merge into a running Goal, a new Goal, or the quick track), turned into an approved request with numbered R rows, reconnoitred, interviewed and design-confirmed before any code is written. Also use it to resume after /clear (dstack run adopt) or to attach a new idea to a Goal that is already running. Korean triggers the user may type - "새 작업 시작해요", "이거 요청서로 만들어줘", "골 하나 열어요", "기능 추가하고 싶어요", "이 버그 고쳐요", "여기에 붙여줘". Do NOT use it for a pure question, a lookup, or a conversation with no artifact; do not use it to run Plans (dstack-develop) or to verify (dstack-verify).
---

# dstack-workflow — route, request, recon, interview, design

Read the shared `runtime.md` installed in the current provider's agent home before this skill.
Its host check, native worker mapping, main-only questions/state and mandatory CLI gates apply.
The same source is installed for Claude and Codex; provider selection never changes these gates.

You are the **main loop**. The host question tool is main-only (R47), so every human round trip
happens here. Subagents get bounded briefs and return text; the main session records state.

Nothing in this skill ticks a checkbox. `dstack` computes pass/fail and prints what it counted;
you read that output and relay it (§3-1, §3-2).

## 1. When this skill runs

| Situation | Action |
|---|---|
| New work arrives, `.dstack` store exists | Run this skill from §2 |
| No store yet | `dstack init`, then §2 |
| Session resumed / after `/clear` | `dstack run adopt` (or `dstack run adopt <id>`), then continue where `dstack status` says |
| A Plan has to be built or executed | Stop here, hand to **dstack-develop** |
| Evidence, ledger, report | **dstack-verify** |
| Question, lookup, conversation, a one-line typo fix | No pipeline at all |

Claude’s inject hook (R24) supplies `dstack status --oneline`; in either host run it explicitly
when no injection is present. Read it before routing: it names the current run, `type/route/research/review/effort/e2e/tests/visual/
polish`, `R rows N, pending N, approved yes|no`, `Q open N`, `cases met N/N`, plan sets, and
`quick open N`. `pending > 0` on a run you did not open means another session appended rows to
your Goal (R48) — approve them before you plan anything else.

## 2. Route the request (R48)

Read `dstack status` first. Then propose exactly one route in the draft's `route:` field.

| Signal | Route | Meaning |
|---|---|---|
| The new work touches files a Plan of the open run declared, or restates one of its R rows | `merge <run-id>` | Append R rows to that Goal |
| Independent scope, its own branch and worktree | `new-goal` | A second run (R37) |
| Small, self-contained, outside every Goal | `quick` | **dstack-quick** |

Urgent work found *inside* a Goal is never `quick`: it is a decimal `plan insert` (R99, §9).

**Merge route, in order.** `dstack req add --run <id> "<line>" --accept "<criterion>"` appends
rows as `status: pending-approval`; existing rows are never edited. `dstack check request` fails
while any row is pending and prints the count. `dstack request approve --run <id>` is the only
writer of the new hash, and its own `cases sync` step appends ledger rows for the new R ids while
keeping recorded evidence. Then size the follow-up work: small → `dstack plan insert --after
P<n>` (decimal id), large → `dstack milestone add <slug> --after M<n>`. Sealed reviews and
finished Plans are never reopened. If the target run lives in another worktree, stop after
`req add`: that session sees `pending` in its next `status --oneline` and approves it itself.

## 3. Open the run (R30, R37, R52, R105)

```
dstack run new <slug> --type <work_type>
```

- **Second Goal in the same worktree** → refused, with a `--worktree ../<repo>-<slug>` example.
  Use it; it makes the worktree and the `goal/<slug>` branch (R37). Do not delete `CURRENT`.
- **Overlap warning**: `run new` prints how many files other open runs declared. It warns, it
  does not block (R38). `dstack next` repeats the warning per Plan pair; `dstack verify` at close
  refuses a Goal branch that does not contain the base HEAD ("rebase first").
- **Style line (R52)**: `run new` resolves `<repo>/.claude/style/team.md` → PROJECT.md
  `team_style:` → `~/.claude/style/<org>.md`. With none of the three it prints
  `No team style — in this repository existing code wins`. Copy that exact line
  into the recon brief and keep it as the first line of `recon.md`. Precedence is
  team/repo > existing code.
- **Tool refusal (R105)**: `run new` reads `deps.tsv` and refuses when a tool needed to *close*
  this run is missing, naming the tool, its install command, and the field that would drop the
  need. Never work around it and never edit `deps.tsv` to pass. Relay it and ask:

  > "화면 캡처 도구(ego-browser)가 없어서 run을 못 열어요. `<설치 명령>`으로 설치할까요,
  > 아니면 `e2e: capture`를 `cli`로 낮출까요?"

  A tool that disappears mid-run is a ledger row, not a silent skip: `dstack evidence add`
  with `--status blocked --note "tool-unavailable <name>"`.

## 4. Draft the request (R40–R44)

Always write the request in Korean 해요체: title, headings, description, R-row text and acceptance
criteria. This applies to every work type, route and `korean_polish` setting, including new rows,
splits and rows created by `ask assume`. Use Korean for the requirement and `--accept` arguments
from the start; polishing never translates R rows. Keep frontmatter keys/enum values, R ids,
`accept:` and status markers, commands, paths and code identifiers unchanged. Read the draft before
approval and correct English prose through the request workflow. Never rewrite an approved request
just to translate it. Copy its frozen R rows into downstream briefs verbatim in Korean.

1. `dstack request new --type <work_type> --title "<한국어 제목>"` copies the template for that type.
2. Fill the frontmatter with the §11 defaults, then narrow anything the user actually asked for.
   The repository policy block in PROJECT.md is the ceiling; a request may only narrow (R75).
3. One row per requirement: `dstack req add "<한국어 요구사항>" --accept "<한국어 완료 기준>"`.
   The CLI mints the number. An accept criterion names what is *observed*, not what is done —
   "401 응답 본문에 스택 추적이 포함되지 않아요", not "오류를 처리해요".
4. `dstack check request` after every batch. It counts rows, pending, withdrawn, deferred,
   superseded and Q states, and warns above **12 rows or 60 lines** (R43). On a warning, propose
   one of two splits and let the user pick:

   > "R 행이 15개예요. 뒤쪽 6개를 두 번째 Milestone으로 미룰까요, 아니면 별도 Goal로 뺄까요?"

   Never split by editing rows: `dstack req split R<NN> --into R<a>,R<b>` is the only splitter,
   and it leaves the parent text untouched with a `superseded-by` marker (R103).

## 5. Recon, before the interview (R50)

Delegate to the **recon** native worker (read-only; model mapping in `runtime.md`). The brief carries: the R rows so far,
`work_type`, `risk_axes` and the style line from §3 verbatim. It returns the text of `recon.md` and writes nothing.

**You write the file with Write, not with a CLI verb** — `recon.md` has no writer verb
(design.md §3 lists it in the run directory but no verb produces it), and R36 forbids workers
from writing under `.dstack/`. Path: `.dstack/runs/<run>/recon.md`.

- First line is the style resolution; `## Risks` covers only the axes in `risk_axes`, at most 5
  rows, each with a `file:line` or URL. With `risk_axes: none` the section is the single line
  `Risks: skipped — risk_axes=none`.
- Append a `## Phases` block and keep it current — this is where a skipped phase becomes visible
  (§3-3). One line each: `recon: on`, `interview: 2 rounds / 5 questions`,
  `external research: skipped — external_research=none`, `design: skipped — design_review=skip`,
  `korean polish: on`, `review: on (rounds by review=on)`, `e2e: cli`, `unit tests: off`,
  `visual: skipped — visual=none`.

## 6. Interview (R51, R61)

A question is allowed **only when its answer changes an R row or a design decision**. Anything
else is not asked. Everything runs through the ledger, so the budget is countable.

| Verb | Use |
|---|---|
| `dstack ask add "<q>" --affects R01,R03` or `--affects design` | Mint Q-NN before asking |
| `dstack ask answer Q-NN "<answer>" --decision "<one line>"` | Record the answer + its D row |
| `dstack ask assume Q-NN "<default>" --accept "<what is observed if the default is wrong>"` | Adopt a default: Q assumed, D assumed, **and one new R row** |
| `dstack ask list` | Counts before you decide to ask another round |

| Budget | Rounds | Questions per round |
|---|---|---|
| First Milestone | 2 | 5 |
| Every later Milestone | 1 | 3 |

Batch questions within the host question tool limit; additional questions use another call in
the same round. When the budget is spent, every leftover question becomes `ask assume` — an
assumption with an R row is auditable, an unasked question is not.

`dstack check request` refuses while any Q is `open`, or when an `assumed` Q has no R row. That
is the real terminator of the interview, not your judgment.

## 7. Design confirmation (R55)

| `design_review` | Behaviour |
|---|---|
| `skip` | No round. Write the skip line in the `## Phases` block. |
| `auto` | Runs **only** when the work hits a trigger below; otherwise skipped, with the reason. |
| `required` | Always one round. |

Auto triggers — any one is enough: **a new module boundary**, **an API contract**, **persistence
or idempotency semantics**, **sanitization across a trust boundary**.

| `work_type` | What is confirmed |
|---|---|
| web-ui | Component tree, state ownership, Suspense and ErrorBoundary boundaries |
| http-api | Request/response sequence and the data model |
| cli, library | Module boundaries and the public surface |
| docs-writing | Outline and the list of decisions |

| Complexity | Mermaid kind |
|---|---|
| One path, a few steps | `flowchart` |
| Two or more actors exchanging messages | `sequenceDiagram` |
| A thing with modes and transitions | `stateDiagram-v2` |
| Persisted entities and their relations | `erDiagram` |
| A UI tree | Component tree in text (indented list), no diagram |

**One human round, maximum.** Record the outcome with
`dstack decision add "<decision>" --affects R01,R02 --design "<why this round was needed>"`.
A second design round needs its reason in the row, or `dstack check decisions` reports it.

## 8. Approval loop (R44, R45, R46)

Approval is last, after the interview and design: `check request` fails while a Q is open, and
assumption rows (§6) must be on the page the user approves.

1. **Polish once, before approve** (R94). With `korean_polish: on`, delegate the request body to
   the **ko-polish** native worker (`runtime.md`). It never touches R rows, the frontmatter, tables, paths
   or code spans; over 15,000 characters it returns `skipped: too-long`. Record the diff and the
   call count in the run folder. After `request approve` the file is frozen by its hash — never
   polish an approved request.
2. `dstack request open` — snapshots `request.agent-draft.md` and opens `code -g <abs>:1`. With
   no `code` on PATH it prints the path and exits 0; say the path out loud.
3. Ask, in Korean, with exactly these three options plus the tool's built-in **Other**:

   | Option label | Meaning | Your next move |
   |---|---|---|
   | 승인 | The document is right | §8-4 |
   | 수정 요청 | A specific change | Apply it with `req` verbs, loop |
   | 재작성 | Wrong shape | Back to §4 with what was wrong |
   | *(Other, free text)* | A requirement the draft missed | `dstack req add --from-answer "<their words>"`, then `dstack req accept R<NN> "<criterion>"` |

4. **After any answer, your first action is a fresh read: `dstack request show`.** The user may
   have edited the file in VSCode while the question was on screen; answering from memory is how
   a pipeline approves a document nobody wrote.
5. `dstack request approve` — validates, clears pending markers, writes the sha256, diffs against
   the agent draft, and syncs the case ledger. A hand edit after this point makes
   `dstack check request` fail on the hash until it is approved again (R46).

## 9. Hand-off

With `request.approved` written and `dstack check request` clean, hand to **dstack-develop**:
milestones, Plans, `dstack next` waves, worker briefs, per-Plan review through **codex-review**.
External research, when `external_research: one-pass`, is one research pass and one audit,
executed by **codex-research** (R54) — never twice, never a re-audit loop. Verification and the
final report belong to **dstack-verify**.

## 10. Long external runs (R98)

Use the host completion mechanism in `runtime.md`. Review/research/audit run through
`dstack mode exec`; other long commands use `dstack exec <label> -- <cmd>`. Keep the process
attached to its managed session, await completion and report actual failures.

## 11. `work_type` defaults (R41, R71)

The values `dstack` itself applies when a field is absent. The agent proposes; the user changes
them on the approval screen. There is no computed tier and no prompt token.

| Field | web-ui | http-api | cli | library | docs-writing |
|---|---|---|---|---|---|
| `external_research` | none | none | none | none | none |
| `risk_axes` | ux | security | none | none | none |
| `design_review` | auto | auto | auto | auto | skip |
| `review` | on | on | on | on | on |
| `codex_effort` | high | high | high | high | high |
| `e2e` | capture | cli | cli | cli | none |
| `unit_tests` | on | on | on | on | off |
| `visual` | none | none | none | none | none |
| `korean_polish` | on | on | on | on | on |
| `route` | proposed by §2; `new-goal` when nothing matches |

## 12. Delegation (R25)

Use the native role mapping in `runtime.md`: `recon` for code reconnaissance, `ko-polish` for
Korean prose, `e2e-runner` for verification, `frontend-dev` for frontend code, `general-dev` for
other implementation. Claude passes its explicit native model; Codex uses fresh native workers
with inherited model/effort. Reviews and external research use the configured `sub` through
the legacy **codex-review** and **codex-research** skills.

A worker starts with an empty context: the brief carries the project summary, the milestone
context, relevant recon rows, R rows verbatim and the D rows they point to (R68).

## 13. Phase on/off by field (R71, §3-3)

| Phase | Runs when | Recorded as when off |
|---|---|---|
| Recon | always | — |
| Risks table | `risk_axes` ≠ none | `Risks: skipped — risk_axes=none` |
| Interview | always, inside the §6 budget | budget spent → assumptions, not silence |
| External research | `external_research: one-pass` | `external research: skipped — external_research=none` |
| Design | `design_review: required`, or `auto` + a trigger | `design: skipped — design_review=skip` / `no trigger` |
| Korean polish | `korean_polish: on`, once before approve | `korean polish: skipped — korean_polish=off` |
| Review | always per Plan; `review` tunes rounds and axes only, never the per-R verdict (R69) | only **dstack-quick** may truly skip it |
| E2E | `e2e: capture` or `cli` | `e2e: skipped — e2e=none` |
| Unit tests | `unit_tests: on` | `unit tests: skipped — unit_tests=off` |
| Visual diff | `visual: design` or `regression` | `visual: skipped — visual=none` |

## 14. Sentences borrowed from GSD

Taken from the current `gsd-core` checkout (R61, R96, R97 require naming them).

| Source | Sentence taken | Where it landed |
|---|---|---|
| `gsd-core/workflows/explore.md` | "Requirement — `REQUIREMENTS.md` (append) — Clear requirements that emerged from discussion" / "New phase — `ROADMAP.md` (append) — Scope large enough to warrant its own phase" | §2's route table: an idea becomes an R row on the running Goal, or its own run |
| `gsd-core/workflows/explore.md` | "**Never write artifacts without explicit user selection.**" | §8: the route and the request are chosen on the approval screen, not by the agent |
| `gsd-core/workflows/insert-phase.md` | "Uses decimal numbering (72.1, 72.2, etc.) to preserve the logical sequence of planned phases while accommodating urgent insertions without renumbering the entire roadmap." | §2's `plan insert --after` and §9's rule that urgent work inside a Goal is a decimal insert |
| `gsd-core/workflows/insert-phase.md` | "Don't renumber existing phases" | R42's numbering rule restated: `req split` marks, it never renumbers |
| `skills/gsd-discuss-phase/SKILL.md` | "Prior context loaded and applied (no re-asking decided questions)" | §6: a question is allowed only when it changes an R row or a design decision |
| `skills/gsd-discuss-phase/SKILL.md` | "CONTEXT.md captures decisions, not vague vision" | §6: every answer becomes a D row with an `--affects` list |
| `skills/gsd-discuss-phase/SKILL.md` | "Scope creep redirected to deferred ideas" | §4: a size warning becomes a later Milestone or a new Goal, never a wider request |
