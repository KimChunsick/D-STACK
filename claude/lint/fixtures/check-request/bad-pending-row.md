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
# Pending accept criterion

R02 came from a free-text answer and its criterion was never filled in with dstack req accept.

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

- [ ] **R01** the first row is complete — accept: it carries an observable criterion
- [ ] **R02** the second row is still waiting on its criterion — accept: pending: agent to propose
