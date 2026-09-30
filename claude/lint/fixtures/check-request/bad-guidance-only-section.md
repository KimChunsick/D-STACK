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
# Section left with its guidance

Every required part-1 section is filled except 목표, which still holds only the template's guidance comment.

## 한눈에 보기

<!-- 승인 전에 확인할 것을 모으는 절이에요. -->

# 1부 요청

## 배경과 문제

A request was approved while a section still held nothing but the template's guidance.

## 목표

<!-- 이번 작업이 끝나면 누가 무엇을 할 수 있게 되는지 번호 목록으로 적어요. 구현 방법은 2부 설계에 둬요. (키: goals) -->

## 비목표

1. The part-2 sections are not judged here.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

- [ ] **R01** a section holding only guidance comments is refused — accept: the output names "## 목표" and the exit code is 1

## 열린 가정

없음.

# 2부 설계

## 지금 구조

<!-- 바꾸기 전의 모듈, 파일, 데이터 흐름을 적어요. (키: current) -->
