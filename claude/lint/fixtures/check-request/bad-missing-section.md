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
# Required section missing

The 비목표 heading was deleted, so there is no place a reader could find what this request leaves out.

# 1부 요청

## 배경과 문제

A request without one of its part-1 headings cannot be read the way the layout promises.

## 목표

1. check request names the heading that is not there.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

- [ ] **R01** a missing required heading is refused — accept: the output says "no '## 비목표' heading" and the exit code is 1

## 열린 가정

없음.
