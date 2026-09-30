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
# Rows without a requirements heading

A request written before either requirements heading existed keeps its rows under a heading of its own.

## 배경과 문제

Requests older than the requirements headings still carry rows, only under another heading.

## 목표

1. check request reads rows anywhere in a request that has neither requirements heading.

## 비목표

1. Such a request is not asked to rename its heading.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 열린 가정

없음.

## R rows

- [ ] **R01** rows under another heading still count when neither requirements heading exists — accept: check request exits 0
