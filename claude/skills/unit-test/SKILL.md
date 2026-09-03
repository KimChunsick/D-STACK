---
name: unit-test
description: >-
  Writes and runs unit tests for one dstack Task with the target repository's own test runner, names
  every test after the R id it proves, and records the Red and Green runs as ledger evidence with
  `dstack evidence add --kind test`. Use it only when the approved request says `unit_tests: on` and
  the work_type is code (web-ui, http-api, cli, library) — the CLI knows no test framework and wraps
  none. Korean triggers the user may type: "테스트 짜줘", "단위 테스트 추가해줘", "테스트 돌려줘",
  "Red 증거 남겨줘".
---

# unit-test

## 1. When this runs

| Condition | What happens |
|---|---|
| `unit_tests: on` and work_type is code | Red → Green → Refactor inside one Task, one commit (R62) |
| `unit_tests: off` | write `unit tests: skipped — unit_tests=off` in the task report; no test row |
| work_type `docs-writing` | no Red/Green; each acceptance criterion is checked one by one instead |
| This repository (D-STACK) | always `unit_tests: off` — the check commands' own output is the evidence |

The test code is written by the implementation worker inside its Task: `frontend-dev` (opus) for
frontend tests, `general-dev` (opus) for everything else. This skill is the contract they follow.

## 2. Use the repository's own runner

The CLI has no test verb and never wraps a framework (R80). Find the runner before writing anything:

| Signal | Runner |
|---|---|
| `package.json` `scripts.test` | that script, verbatim |
| `Makefile` target `test` | `make test` |
| `Cargo.toml` / `go.mod` / `pyproject.toml` | `cargo test` / `go test ./...` / `pytest` |
| an existing `tests/` shell suite | that suite's own entry point |

Match the file layout, naming and assertion style already in the repository, even where you would
choose differently. If no runner exists, say so and ask before introducing one — adding a framework
is a product decision, not a verification detail.

## 3. Name every test after its R

`R<NN>__<slug>`, e.g. `R03__rejects_expired_token`. Two reasons, both mechanical: the R id has to
survive into the run output, and `dstack evidence add --kind test` **rejects an artifact that does
not name the R as a whole word**. A file-level header comment naming the R does not help if the
runner does not print it — the printed test name is what the ledger reads.

## 4. Red evidence, then Green (R62)

Both runs are captured into the artifact directory the brief names
(`<main-root>/.dstack/local/artifacts/<plan-or-scope>/`), and both become ledger rows:

```bash
# Red — the test must fail before the implementation exists.
<runner> 2>&1 | tee .dstack/local/artifacts/P3/R03-red.txt   # exits non-zero
dstack evidence add --r R03 --case c-test-red --kind test \
  --artifact .dstack/local/artifacts/P3/R03-red.txt --produced-by "<runner>" \
  --note "red: fails without the implementation"

# Green — after the implementation, in the same Task, before the single commit.
<runner> 2>&1 | tee .dstack/local/artifacts/P3/R03-green.txt
dstack evidence add --r R03 --case c-test --kind test \
  --artifact .dstack/local/artifacts/P3/R03-green.txt --produced-by "<runner>"
```

- A "Red" capture whose run exited 0 is not Red evidence — it proves the test cannot fail. Re-write
  the test until the run is red for the stated reason, then capture again.
- The two artifacts are different files, so no `--shared` is needed. `c-test` is the case id
  `dstack cases sync` already opened for the R; `c-test-red` is added alongside it.
- A long suite goes through ONE background call and the turn ends; the completion notification
  resumes it: `dstack exec <label> -- <runner>` (label: `suite-P3`).
- GSD `agents/gsd-verifier.md:467`: *"Run the full workspace test command at most once per
  verification. Never filter a full run per must-have."* Prove a test exists by enumeration
  (`--list` / `--collect-only`); prove one passes with a single named test.

## 5. Test rigor — what makes a row honest

Borrowed from GSD `TESTING-STANDARDS.md`:

| Contract | Sentence borrowed |
|---|---|
| §2 No vacuous-truth assertions | "Assertions must be capable of failing given a plausible defect in the SUT." |
| §3 No pass-always tests | "A test that passes regardless of whether the feature it describes is implemented is worse than no test: it inflates the count while providing false confidence." |
| §1 Exercise real code | "Tests call exported functions or run the CLI and parse structured output. They do not `readFileSync` a source file and assert on its text content." |

Each test encodes *why* the behaviour matters — the acceptance criterion of its R row, verbatim, as
the test's own description. A test that cannot fail when the business rule changes is decoration.

## 6. What this skill never does

- Never ticks a checkbox or writes `cases.tsv` directly: `dstack evidence add` is the only writer.
- Never commits per test: one Task is one commit, with Red, Green and Refactor inside it (R60).
- Never runs when `unit_tests: off` "just to be safe" — a phase runs only when a field turns it on,
  and the skip is written down with its reason (§3-3).
