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
# Guidance kept beside new prose

목표 gained a real goal, but the template's guidance comment above it was never removed.

# 1부 요청

## 배경과 문제

A section can look filled while the template's instruction still sits in it.

## 목표

<!-- 이번 작업이 끝나면 누가 무엇을 할 수 있게 되는지 번호 목록으로 적어요. 구현 방법은 2부 설계에 둬요. (키: goals) -->
1. check request refuses a section that still holds its template guidance.

## 비목표

1. The part-2 sections are not judged here.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

- [ ] **R01** retained template guidance is refused — accept: the output names "## 목표" and the exit code is 1

## 열린 가정

없음.
