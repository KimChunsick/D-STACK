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
# Row outside the requirements section

Every part-1 section is filled, but the one R row sits under 열린 가정 and 요구사항 holds only guidance.

# 1부 요청

## 배경과 문제

A row outside 요구사항 is not a requirement a reader of that section would see.

## 목표

1. check request counts only the rows inside the requirements section.

## 비목표

1. Rows are not moved for the author.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

<!-- dstack req add "<한국어 요구사항>" --accept "<한국어 완료 기준>"으로 행을 추가해요. 직접 작성하지 않아요. -->

## 열린 가정

없음.

- [ ] **R01** a row outside 요구사항 does not fill it — accept: the output names "## 요구사항: no R row" and the exit code is 1
