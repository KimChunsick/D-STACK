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
# Both requirements headings

Every part-1 section is filled, and the rows are split between 요구사항 and the legacy Requirements heading.

# 1부 요청

## 배경과 문제

Two requirements sections leave no single section to count rows and lines in.

## 목표

1. check request refuses a request with both requirements headings.

## 비목표

1. Neither section is chosen for the author.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

- [ ] **R01** both headings are refused — accept: the output names "'## 요구사항' and '## Requirements'" and the exit code is 1

## 열린 가정

없음.

## Requirements

- [ ] **R02** the second section is not merged into the first — accept: the same line names both headings
