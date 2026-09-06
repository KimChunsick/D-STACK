---
name: dstack-verify
description: >-
  Verification and close-out of a Goal run. Expands approved R rows into the cases ledger, runs the
  work_type verification profile (R72) through the e2e-runner subagent, records every artifact with
  `dstack evidence add`, then runs `dstack verify`, `dstack report` and the milestone / Goal close
  checklist. Use it when a worker returns with test artifacts, when a milestone is closing (the e2e
  cases of every Plan in it run once, in one pass), when the Goal is closing, when a case must be
  marked blocked or abstained, or when a completion report is asked for. Korean triggers the user may type: "검증해줘", "증거 남겨줘", "케이스 돌려줘",
  "마일스톤 닫자", "Goal 닫아줘", "완료 보고 만들어줘", "리포트 뽑아줘".
---

# dstack-verify

Read the shared `runtime.md` installed in the current provider's agent home before this skill.
Its host check, native worker mapping, main-only questions/state and mandatory CLI gates apply.
The same source is installed for Claude and Codex; provider selection never changes these gates.

The ledger decides, not the agent. This skill never ticks a box and never writes `cases.tsv`: the
CLI computes every pass/fail and `dstack evidence add` is the only writer of an evidence row (R104).
Borrowed stance, GSD `agents/gsd-verifier.md:15`: *"Verify that the phase goal is actually achieved
in the codebase — SUMMARY.md claims are not evidence."* Here a worker's report is that claim.

## 1. When this runs

| Moment | What runs |
|---|---|
| A worker returns | the `test` rows its Tasks produced (Red/Green artifacts) → `dstack evidence add` at once: the Red output exists only at that moment |
| A milestone closes | ONE e2e-runner pass over the cases of every Plan in the milestone → evidence → `dstack check coverage` → the §7 checklist |
| The Goal closes | §7 checklist + `dstack report --metrics` + `dstack run close` |
| A case cannot be observed | record `blocked` / `abstain` (§5), then continue with the next case |

Evidence has two rhythms. Test evidence is per Task and recorded the moment the worker's report
arrives (develop §7). E2E evidence is per milestone, not per Plan: the Plans of a wave are
independent, so one runner at milestone close covers them all, and the runner is the expensive
part of verification. The price is that a failed case reopens a Plan that is already done and
reviewed — the fix is a decimal Plan (`dstack plan insert --after P<n>`) with its own review
round, and the milestone stays open until its case is re-run and recorded.

Phases are switched on by request fields only — never by a guess about the task's size. Read them
with `dstack request show` and obey the table:

| Field | Value | Effect here |
|---|---|---|
| `e2e` | `capture` | profile artifacts are captures; ledger kind `capture` |
| `e2e` | `cli` | profile artifacts are stdout/stderr/exit captures; ledger kind `cli` |
| `e2e` | `none` | no execution; each R gets a `review` row (claim → source) |
| `unit_tests` | `on` | the `unit-test` skill produces the `test` rows |
| `visual` | any | no comparison tool is bundled: one `skipped` row per rendering R, note `no-visual-surface` |
| `korean_polish` | `on` | the Korean prose under the report goes through `ko-polish` (native mapping in `runtime.md`) |

A phase that does not run is written down, never silently dropped (§3-3): put
`e2e-runner: skipped — e2e=none` (or the real reason) in the message that reports the milestone,
and record the ledger row that carries the same note. There is no unconditional LLM round trip in
this skill: the runner is delegated to only when at least one open case needs execution.

## 2. The cases ledger (R73)

`cases.tsv` in the run directory is the only answer to "is this R covered and proven".

```bash
dstack cases sync                 # expand approved R rows into open rows; keeps recorded evidence
dstack cases render               # the human table: R, case, kind, status, artifact, sha256, note
dstack check coverage             # every live R needs a covering task AND an evidence row
```

- `cases sync` is safe to re-run: it appends only the R ids with no row and prints how many it added.
  Run it after every `dstack request approve` (a merged request adds rows, R48).
- Statuses are `open | met | abstain | blocked | skipped | unreported`. Only `evidence add` writes a
  non-open row. Editing the file by hand is caught later by the sha256 recheck in `dstack verify`.
- `unreported` comes from a worker that did not report an R it was delegated (R68) — treat it as an
  open case and run it yourself before closing anything.
- Quick tasks use the same ledger: add `--quick <slug>` to any command in this file.

## 3. Verification profiles (R72)

One profile per `work_type`. The profile says what one case must produce; the request's `e2e` field
says which kind the ledger row carries.

| work_type | One case produces | Tool | Ledger kind |
|---|---|---|---|
| `web-ui` | one annotated capture per case + a `.txt` naming the R, the URL path, the steps and what was observed | `ego-browser` skill | `capture` |
| `http-api` | request + response (status, the headers that matter, body) for each case, and exactly one tampered-input case whose rejection is recorded | the repo's own client (curl/httpie) | `transcript` |
| `cli` | the exact command, stdout, stderr and the exit code | the command itself, `dstack exec` for long runs | `cli` |
| `library` | build command + an example run against the **built** artifact, its output and exit code | the repo's build | `cli` |
| `docs-writing` | a claim → source checklist: one row per claim with `file:line` or a URL | none (reading) | `review` |

Two rules that apply to every profile:

- GSD `agents/gsd-verifier.md:498` — *"Exit code 0 is PASS. Any non-zero exit is FAILED and must
  include stdout/stderr evidence"*. Record the failing output too; a case that failed is evidence.
- GSD `agents/gsd-verifier.md:472` — *"SUMMARY.md probe pass claims are not evidence. If a phase
  declares or implies probe-based verification, the verifier must run the probe in its own process
  and record the command result."* The runner runs it; the main session records what came back.

### 3.1 web-ui capture engines

**ego-browser** (skill `ego-browser`, installed at `~/.claude/skills/ego-browser` — read its
`SKILL.md` before the first call of a session). Everything runs through
`ego-browser nodejs <<'EOF' … EOF`; the helpers a case needs are `useOrCreateTaskSpace`,
`openOrReuseTab`, `waitForElement`, `click`, `fillInput`, `typeText`, `snapshotText`,
`captureScreenshot`, `cliLog` (the only output channel inside the heredoc). Reuse one task space
for the whole verification pass. A task space owned by the user is the R78 case: `switchTaskSpace`
throws on a user-owned space — that is "user is controlling", stop there (§5.3).

## 4. Running the cases — delegate to the native e2e-runner

The main session prepares the artifact directory and passes it as the only writable location:

```bash
mkdir -p "$(git rev-parse --show-toplevel)/.dstack/local/artifacts/<scope>"
```

`<main-root>/.dstack/local/artifacts/<scope>/` (mode 700, never committed). `<scope>` is the
milestone id for a milestone pass (`M2`) or the quick slug. The runner writes only there; the main
session records the artifacts afterwards. The worker's own test artifacts live elsewhere
(`.dstack/runs/<id>/artifacts/P<n>/`, develop §6) and are recorded when the worker returns.

Delegation follows `runtime.md` (R25): `e2e-runner` executes cases, `ko-polish` handles Korean
prose, and `general-dev` / `frontend-dev` fix failed cases. Each is a native worker of the main
host with a fresh bounded brief. The legacy `codex-review` skill reviews fixes through the
configured `sub`; it does not select the worker engine.

Brief block to send (an empty-context worker gets everything it needs, R68):

```
Run the <work_type> verification profile for run <run-id>, milestone <M> (every Plan in it).
Artifact directory (the ONLY place you may write): <abs path>/.dstack/local/artifacts/<scope>/
How to start the system under test: <command, or "already running at <url>">
Capture engine: ego-browser — instructions: <paste the §3.1 rows>
Cases (every open case of every Plan in the milestone, in one table):
| R | case | acceptance criterion (verbatim from the request) | steps |
| R03 | c1 | <accept: …> | <steps> |
Naming: <artifact-dir>/R<NN>-<case>.<ext>, plus R<NN>-<case>.txt naming the R id.
A "user is controlling" error is a hard stop for that case: write `blocked: user-controlling`
into the text file and move on.
Return only the table | R | case | artifact | outcome (met|blocked|skipped) | note |.
```

A run longer than the foreground cap uses the host completion mechanism in `runtime.md` (R98):

```bash
dstack exec <label> -- <the long command>     # label: e2e-M2, e2e-<quick slug>, …
```

## 5. Recording evidence (R104)

One command per artifact, run by the main session after the runner reports:

```bash
dstack evidence add --r R03 --case c1 --kind capture \
  --artifact .dstack/local/artifacts/M2/R03-c1.png \
  --produced-by "ego-browser nodejs … captureScreenshot → R03-c1.png"
```

```bash
dstack evidence add --r R05 --case c1 --kind cli \
  --artifact .dstack/local/artifacts/M2/R05-c1.txt --produced-by "make test"
dstack evidence add --r R07 --case c1 --kind transcript \
  --artifact .dstack/local/artifacts/M2/R07-c1.txt --produced-by "curl -i -X POST …"
dstack evidence add --r R09 --case c1 --kind review \
  --artifact .dstack/local/artifacts/M2/R09-claims.md --produced-by "claim→source checklist"
```

### 5.1 Why a row is rejected, and what fixes it

| Rejection | Fix |
|---|---|
| `R is not a row of request.md` | `dstack req add` first, or fix the id |
| `R is withdrawn / deferred` | a marked row takes no evidence; record on its replacement |
| `R is superseded by …` | record on the child rows (R103) |
| `unknown kind` / `unknown status` | kinds: `test capture transcript cli visual review`; statuses: `met abstain blocked skipped` |
| `artifact not found` | the runner wrote elsewhere — pass it the artifact directory |
| `artifact is zero bytes` | the command printed nothing; re-run it and tee the output |
| `artifact mtime is earlier than this run started` | a stale file from a previous run; re-produce it now |
| `artifact already recorded under R0x/cN` | add `--shared "<why one artifact proves both>"` |
| `kind test/cli requires the artifact to name R<NN>` | name the R inside the file (test name `R<NN>__<slug>`, or a header line `R<NN>: <command>`) |
| `case is already recorded (status …)` | rows are never overwritten; use a new case id |

### 5.2 Skipped rows

A phase that is off still leaves a trace. Write a one-line note file into the artifact directory
(`R<NN>-<case>-skipped.txt` containing the R id and the reason), then:

```bash
dstack evidence add --r R04 --case c-visual --kind visual \
  --artifact .dstack/local/artifacts/M2/R04-c-visual-skipped.txt \
  --produced-by "visual=none" --status skipped --note no-visual-surface
```

### 5.3 Blocked rows

| Cause | Command | Then |
|---|---|---|
| ego-browser reports the user is controlling the space (R78) | `dstack evidence add … --status blocked --note user-controlling` | **end the turn** — say to the user: "지금 브라우저를 직접 쓰고 계셔서 R04 케이스를 blocked로 적어 뒀어요. 손을 떼시면 이어서 돌릴게요." |
| A tool disappeared mid-run (R105) | `dstack evidence add … --status blocked --note "tool-unavailable ego-browser"` | print the install command from `deps.tsv` and say which request field would avoid it |

A blocked case closes the Goal only the way an ABSTAIN does: the user accepts it one by one (§6).

## 6. verify, accept-abstain, report, metrics

```bash
dstack verify        # policy ceiling, per-field evidence, sha256 recheck, branch containment
dstack report        # the R table: id, text, covering tasks, evidence path, status
```

- **Policy ceiling (R75).** `.dstack/project/PROJECT.md`'s `## Verification policy` block is the
  ceiling; the request may only narrow it. When `verify` rejects a request that widens it, it prints
  the policy's `why` line — quote that line to the user and change the *request*. Never edit
  PROJECT.md to make a run pass.
- **Exit codes.** `0` pass; `1` something failed (the reason is printed); `2` only unaccepted
  ABSTAIN/BLOCKED remain. `report`: `1` on any UNMET, `2` when only ABSTAIN/BLOCKED remain,
  and `--metrics` exits `1` if any metric is `unavailable`.
- **Status is read, never invented (R79).** `report` combines `check coverage`, the ledger, `verify`,
  `check decisions` and the latest sealed round's verdict. A review `partial` counts as UNMET with
  its round number. WITHDRAWN, DEFERRED, SKIPPED and BLOCKED are counted separately, never as MET.
- **ABSTAIN is a real verdict, not a soft pass.** GSD `gsd-core/references/honest-verifier.md`:
  *"Never silent, never a hard halt."* and *"Explicit evidence = a wired held-out/property-based test
  that passes, or a behavior the verifier directly observed."* Presence of the code is not evidence.
- **Accepting them.** One R per call, with the reason that goes into the report verbatim:

```bash
dstack verify --accept-abstain R04 --why "결제 샌드박스가 닫혀 있어 이번 Goal에서는 확인 불가"
```

  The decision is the user's. Ask in the main loop with the host question tool (never inside a
  subagent, R47), one question carrying up to four R ids: "R04·R07은 증거로 판정이 안 나요. 사유를
  붙여 받아들일까요, 아니면 케이스를 다시 돌릴까요?"

- **The report shape.** The completion message starts with the `dstack report` table pasted as-is;
  Korean prose goes underneath it and never restates a status the table already prints. With
  `korean_polish: on` that prose (not the table, not the R rows) goes through `ko-polish` (native mapping in `runtime.md`).

## 7. Milestone and Goal close checklist

Run in order. Every step prints counts; paste the counts, do not summarise them away.

| # | Step | Command |
|---|---|---|
| 1 | All plans of the milestone are done | `dstack plan render` |
| 2 | New R rows expanded | `dstack cases sync` |
| 3 | The milestone's e2e pass: every open case of every Plan, one runner, then one `evidence add` per artifact (§4, §5). A failed case → `dstack plan insert --after P<n>` for the fix, its own review round, back to step 1 | `dstack cases render` shows what is still open |
| 4 | Ledger check for the milestone (R70): open findings and integration behaviour only, no new scope | `dstack review --scope milestone --milestone M2`, then the `codex-review` skill seals the round |
| 5 | Coverage | `dstack check coverage` |
| 6 | Interview decisions covered | `dstack check decisions` |
| 7 | Evidence and policy | `dstack verify` |
| 8 | Report | `dstack report` |
| 9 | Finished quick items tidied (R99) | `dstack quick list`, `dstack quick close <slug>` |

Goal close records no new evidence: every case was run at its milestone. It adds, in this order:

```bash
dstack report --metrics    # R01: wall clock, tokens, review rounds, concurrent runs, R met rate
dstack run close           # runs verify, stamps closed_at, clears CURRENT
```

**Rebase rule (R38).** `dstack verify` refuses to close a Goal whose branch does not contain the base
branch HEAD, and says "rebase first". Then: rebase onto the base branch; for every file that
conflicted, re-run the ledger check for the plan that declares it
(`dstack review --scope plan --plan <P>`) before closing again. `dstack run close --abandon "<why>"`
is for a Goal that is being dropped, not for one that will not pass.

**Never** close by hand-editing a document: `dstack run verify` at the start of a worker's turn and
`dstack verify` at the end are the only two statements about the world this pipeline trusts.

## GSD sentences borrowed

| Source (gsd-core checkout) | Sentence used here |
|---|---|
| `agents/gsd-verifier.md:15` | "Verify that the phase goal is actually achieved in the codebase — SUMMARY.md claims are not evidence." |
| `agents/gsd-verifier.md:472` | "SUMMARY.md probe pass claims are not evidence. If a phase declares or implies probe-based verification, the verifier must run the probe in its own process and record the command result." |
| `agents/gsd-verifier.md:498` | "Exit code 0 is PASS. Any non-zero exit is FAILED and must include stdout/stderr evidence in VERIFICATION.md." |
| `gsd-core/references/honest-verifier.md` | "Never silent, never a hard halt." / "Explicit evidence = a wired held-out/property-based test that passes, or a behavior the verifier directly observed." |

## A recorded row that must stop counting (`dstack evidence retire`)

When an artifact was overwritten after recording (verify: `sha256 mismatch`) or proved the wrong
thing, do not edit the ledger and do not overwrite the row: `dstack evidence retire --r R<NN>
--case <id> --why "<reason>"` sets the row to `retired` (its artifact and old sha stay as
history), then record the replacement under a NEW case id with `dstack evidence add`. Re-runs
of a harness must write new file names, never the recorded ones.
