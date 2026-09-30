---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: auto
review: on
codex_effort: ultra
e2e: cli
unit_tests: on
visual: none
korean_polish: on
---
# Value outside the enum

codex_effort has three legal values and "ultra" is not one of them.

## 배경과 문제

Every part-1 section is filled, so only the condition this fixture names can decide the verdict.

## 목표

1. check request judges the one condition this fixture is about.

## 비목표

1. No other condition of the file fails.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 열린 가정

없음.

## Requirements

- [ ] **R01** a value outside the enum is refused by field name — accept: the output names "field codex_effort"
