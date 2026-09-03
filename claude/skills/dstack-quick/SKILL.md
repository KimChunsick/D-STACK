---
name: dstack-quick
description: The quick track - a small, self-contained piece of work done outside any Goal, with the same request, ledger and checkers but nothing optional turned on. Use it when the user asks for one bounded change that needs no Milestone, no Plan and no roadmap entry - a dependency bump, a config fix, a one-file cleanup, a short document - and no Goal run is claiming those files. Korean triggers the user may type - "빠르게 하나만 해줘", "간단한 거 하나", "이것만 잠깐 고쳐요", "골까지는 아니고요", "quick으로 해요". Do NOT use it for urgent work inside a running Goal (that is a decimal plan insert), and do NOT use it to skip verification on work that belongs in a Goal.
---

# dstack-quick — the short path with the same guarantees

Same system, shorter path. A quick task uses the same request format, the same case ledger and
the same checkers as a Goal; what changes is that everything costing a model round trip is off
until a flag turns it on. **`review: off` really skips review here — and only here** (R99): a
quick task has no Plan, so there is no review bundle to build.

You are the main loop; AskUserQuestion works only here (R47). No checkbox is ever ticked by you.

## 1. Quick, or not quick

| Situation | Track |
|---|---|
| Bounded change, outside every Goal, no roadmap entry needed | **quick** — this skill |
| Urgent work discovered *inside* a running Goal | `dstack plan insert --after P<n>` (decimal), never quick |
| New scope that overlaps the open run's declared files | merge route → **dstack-workflow** §2 |
| Independent, large enough to need Milestones | `new-goal` → **dstack-workflow** |

Check first: `dstack status` shows the current run and `quick open N`. Opening a quick task does
**not** touch `CURRENT` — a Goal run stays exactly where it was (R99).

## 2. Open the task

```
dstack quick new <slug> [--type web-ui|http-api|cli|library|docs-writing] [--discuss] [--research] [--review] [--validate] [--full]
```

`--type` defaults to `cli`. The directory is `.dstack/quick/<slug>/`; the record is the
`## Quick tasks` section of `.dstack/quick/STATE.md`, one per worktree. Nothing goes in ROADMAP.

| Flag | Field it turns on | Cost it buys |
|---|---|---|
| *(none)* | `external_research: none`, `review: off`, `e2e: none` | The default: you already know what to do |
| `--discuss` | *(no field)* one interview round before approval | Ambiguity worth resolving up front |
| `--research` | `external_research: one-pass` | **codex-research**: one pass, one audit |
| `--review` | `review: on` | **codex-review** on the diff (§7) |
| `--validate` | `e2e:` = the `work_type` default (`capture` for web-ui, `cli`, `none` for docs) | Real evidence, not a claim |
| `--full` | `--research --review --validate` together | Everything |

Fixed for every quick task: `route: quick`, `risk_axes: none`, `design_review: skip`,
`codex_effort: medium` (R23), `unit_tests: off`, `visual: none`, `korean_polish: on`.
Flags compose: `--research --review --validate` is the same as `--full`.

**Tool refusal (R105)**: `quick new` checks `deps.tsv` before it creates anything and refuses
when a tool needed to close the task is missing, naming the tool, the install command and the
field that would drop the need. Relay it; never work around it:

> "`--validate`를 켜면 캡처 도구가 필요한데 없어요. `<설치 명령>`으로 설치할까요, 아니면
> `--validate` 없이 진행할까요?"

A tool that disappears mid-task is a ledger row: `dstack evidence add --quick <slug>`
with `--status blocked --note "tool-unavailable <name>"`.

## 3. Write the R rows

```
dstack req add "<one line>" --accept "<observable criterion>" --quick <slug>
```

Minimum for a quick task (R99): **at least one R row** (one line + an accept criterion), the
work itself, one `dstack evidence add`, one `dstack report`. Every `req`, `ask`, `evidence`,
`check` and `report` call carries `--quick <slug>`; without it the verb targets `CURRENT` and
writes into the Goal run instead.

`dstack req status --quick <slug>` counts rows and pending markers. More than a handful of rows
is the signal that this was never quick: say so and offer the Goal route.

> "R 행이 7개까지 늘었어요. quick 트랙보다는 Goal 하나로 여는 게 맞아 보이는데, 어떻게 할까요?"

## 4. Discuss, only with `--discuss` (R51)

One round, at most 3 questions, batched into a single AskUserQuestion call. A question is
allowed only when its answer changes an R row. Everything goes through the ledger with
`--quick <slug>`: `dstack ask add`, then `dstack ask answer` or `dstack ask assume`. Leftovers
become assumptions — `ask assume` mints its own R row, so the default is visible on the page the
user approves. Without `--discuss` this whole phase is written down as
`discuss: skipped — no --discuss flag` in the task's report prose (§3-3).

## 5. Research, only with `--research` (R54, R97)

One research pass and one audit through the **codex-research** skill at
`codex_effort: medium`, output at `.dstack/quick/<slug>/research.md`. There is no re-audit loop.
Off by default; the skip line is `external research: skipped — external_research=none`.

## 6. Approve, then work

1. With `korean_polish: on`, polish the request prose **once, before approval**, through the
   **ko-polish** subagent (sonnet). It never touches R rows, frontmatter, tables or code spans.
   After approval the hash freezes the file (R46) — never polish it again.
2. `dstack request open --quick <slug>` — opens `code -g <abs>:1`, or prints the path.
3. Ask in Korean with three options plus the tool's built-in **Other**:

   | Option | Meaning | Next move |
   |---|---|---|
   | 승인 | Right as written | §6-5 |
   | 수정 요청 | A specific change | Apply with `req` verbs, ask again |
   | 재작성 | Wrong shape | Back to §3 |
   | *(Other, free text)* | A requirement the draft missed | `dstack req add --from-answer --quick <slug> "<their words>"`, then `dstack req accept R<NN> "<criterion>" --quick <slug>` |

4. **After any answer your first action is a fresh read: `dstack request show --quick <slug>`.**
   The file may have been edited in VSCode while the question was on screen.
5. `dstack request approve --quick <slug>` writes the sha256 and syncs the case ledger.
6. Do the work. A quick target has **no `plan.json`**: `dstack task add` refuses with
   `quick tasks have no plans`. The unit of work is the R row itself, and coverage counts
   evidence only. One commit, Korean 해요체 message, no AI trailer. Frontend code goes to
   **frontend-dev** (opus), everything else to **general-dev** (opus) — always pass `model`
   explicitly (R25). A change small enough to be one obvious edit stays in the main loop.

## 7. Review, only with `--review` (R69, R96)

There is no Plan, so `dstack review --scope` refuses on a quick target. Invoke the
**codex-review** skill directly on the task's own diff plus the R rows verbatim, at
`codex_effort: medium`. The per-R `covered | partial | absent` verdict still applies, and an
`absent` row still blocks the positive close. With `review: off` write the skip line
`review: skipped — review=off` into the report prose; this is the only place in the whole
pipeline where that line is truthful.

## 8. Evidence, report, close

| Step | Command |
|---|---|
| Record what was observed | `dstack evidence add --quick <slug> --r R<NN> --case c-1 --kind test\|capture\|transcript\|cli\|visual --artifact <path> --produced-by "<cmd>"` |
| Check every live R has evidence | `dstack check coverage --quick <slug>` |
| See the ledger | `dstack cases render --quick <slug>` |
| The report the user reads | `dstack report --quick <slug>` |
| Close it | `dstack quick close <slug>` |

`evidence add` is the only writer of a non-open ledger row (R104) and it rejects a missing or
empty artifact, an artifact older than the task, an artifact already claimed by another R
without `--shared <why>`, and a `test`/`cli` artifact whose text never mentions the R id. Do not
argue with a rejection — produce the artifact it asked for.

`dstack report --quick <slug>` prints the R table first and your Korean prose goes underneath
(R79). UNMET exits non-zero; only ABSTAIN or BLOCKED left exits 2, and each of those closes only
after the user accepts it by name with `dstack verify --accept-abstain R<NN> --why "<reason>"`.

## 9. Resuming, and the Stop gate

| Command | Use |
|---|---|
| `dstack quick list` | Every quick task with its status and the counts by status |
| `dstack quick status <slug>` | Fields, R rows, pending, cases for one task |
| `dstack quick resume <slug>` | What this task still needs, item by item, with the command for each |
| `dstack gate` | The Stop-hook verdict: it checks the run at `CURRENT` **and** every open quick task in this worktree, on the same conditions — R rows, evidence, `check coverage` |

After `/clear`, quick tasks need no adoption: they are not owned by a session. Run
`dstack quick list`, then `dstack quick resume <slug>` and continue.

**Tidy at milestone close (R99)**: when a Milestone of the Goal run closes, run
`dstack quick list`, close every finished quick task with `dstack quick close <slug>`, and name
the ones still open so they do not silently accumulate.

## 10. Long external runs (R98)

A research or review call that may take minutes goes in ONE background Bash call whose blocking
step is `dstack exec <label> -- <cmd>`, and then **the turn ends**. The completion notification
resumes you. Never poll, never detach.

## 11. Sentences borrowed from GSD

Taken from the current `gsd-core` checkout (R99 borrows the quick track; R61/R96/R97 require
naming what was taken).

| Source | Sentence taken | Where it landed |
|---|---|---|
| `skills/gsd-quick/SKILL.md` | "Quick mode is the same system with a shorter path" | The opening line: same request, same ledger, same checkers |
| `skills/gsd-quick/SKILL.md` | "**Default:** Skips research, discussion, plan-checker, verifier. Use when you know exactly what to do." | §2's default row — everything costing a round trip is off until a flag turns it on |
| `skills/gsd-quick/SKILL.md` | "Granular flags are composable: `--discuss --research --validate` gives the same result as `--full`." | §2's composition rule |
| `skills/gsd-quick/SKILL.md` | "Quick tasks live in `.planning/quick/` — separate from phases, not tracked in ROADMAP.md" | §2: `.dstack/quick/<slug>/`, recorded in `STATE.md`, never in ROADMAP |
| `skills/gsd-quick/SKILL.md` | "Preserve all workflow gates (validation, task description, planning, execution, state updates, commits)." | §8–§9: the checkers and the Stop gate apply unchanged |
| `commands/gsd/quick.md` | "Use `list` to audit accumulated tasks; use `resume` to continue in-progress work" | §9, including the tidy-at-milestone-close rule |

Stuck quick tasks: a review that cannot finish is closed with `dstack review close --quick <slug>
--scope quick --why "…"` (its R ids become ABSTAIN to accept), and a ledger row whose artifact was
overwritten is retired with `dstack evidence retire --quick <slug> --r R<NN> --case <id> --why "…"`
before recording the replacement under a new case id.
