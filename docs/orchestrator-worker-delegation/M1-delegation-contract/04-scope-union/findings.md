# Finding ledger — T04 (scope union)

Blocking findings per round: **R1 0**
Bundle bytes: R1 11,355

The loop closed at round 1 under the medium=0 closure rule — no high or medium was raised.

## Open (carried, with the reason)

| # | Sev | Class | Finding | Disposition |
|---|---|---|---|---|
| F-01 | low | structure | `check-parallel.test.sh` has no commit-then-delete fixture, so it would stay green if the enumeration regressed to the endpoint diff | DECLINED — `AGENTS.md` bans authoring tests here; the demonstration is recorded direct-run output in `task.md` instead |

## Closed in round 1

- [low][security] a committed filename containing a newline could print a forged `PASS` verdict
  line — one `esc` helper at all five render sites, verified with `evil<LF>PASS`
- [low][correctness] scope rejected any path containing `..`, while the grammar rejects only a
  `..` component — now component-based, verified with `src/foo..bar`
- [low][security] the task record told the reviewer what was "out of scope" — reworded as a
  verifiable claim
- [low][DX] duplicated `Files changed` section with a `<pending>` placeholder
