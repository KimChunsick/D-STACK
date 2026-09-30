---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: auto
review: on
codex_effort: high
e2e: cli
unit_tests: on
visual: none
korean_polish: on
---
# Row outside the legacy requirements section

Every part-1 section is filled, but the one R row sits under 열린 가정 and the legacy Requirements heading holds none.

## 배경과 문제

A row outside the legacy heading is not a requirement a reader of that section would see.

## 목표

1. check request counts only the rows under the legacy Requirements heading when it is the one present.

## 비목표

1. Rows are not moved for the author.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## Requirements

<!-- Rows go here. -->

## 열린 가정

없음.

- [ ] **R01** a row outside Requirements does not fill it — accept: the output names "## Requirements: no R row" and the exit code is 1
