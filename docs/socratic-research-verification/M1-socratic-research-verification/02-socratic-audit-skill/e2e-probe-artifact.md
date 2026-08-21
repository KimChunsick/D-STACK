# E2E probe artifact (synthetic, authored by the orchestrator with deliberate defects)

## Hypotheses
- H1: Project X's test suite pass rate is 80%. [ledger L1]
- H2: The Python walrus operator (`:=`) was introduced in Python 3.9.

## Data-check ledger
L1: H1 | source: recorded local run (see recorded results below) | unit: tests | denominator: 10 | transformation: passed/total | value: 5/10 = 80% | status: recomputed | how sure: high

## Deferred executable checks
- D1 (the artifact links this to H2): count the number of lines in Project X's README file.

## Recorded executable-check results
- Local test run output: "5 passed, 5 failed, 10 total"
