---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: skip
review: on
codex_effort: high
e2e: cli
unit_tests: on
visual: none
korean_polish: on
---
# 3부에 손으로 쓴 R 행을 검사해요

3부 경계 줄 아래는 승인 해시가 보지 않아요. 거기에 R 행을 쓰면 해시는 맞아도 검사가 그 줄을 알려 줘야 해요.

# 1부 요청

## 배경과 문제

고정물 실행기의 승인 도장과 `dstack check request`가 서로 다른 바이트를 해시하면 승인한 요청서가 불일치로 실패해요.

## 목표

1. 3부 경계 줄이 있는 승인한 요청서가 `dstack check request`를 통과해요.

## 비목표

1. 승인 뒤에 고친 요청서는 이 고정물에서 다루지 않아요.

## 사용 시나리오

### S1 승인한 요청서를 검사해요

고정물 실행기가 도장을 찍은 다음 `dstack check request`를 실행해요.

## 요구사항

- [ ] **R01** 승인한 3부 요청서의 해시가 맞아요. — accept: check request가 "approved: yes"를 출력하고 0으로 끝나요.

## 열린 가정

없음.

# 2부 설계

## 위험

없음.

# 3부 계획과 검증
<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->
<!-- 이 부분은 직접 쓰지 않아요. 계획 대장(plan.json)이 바뀔 때마다 CLI가 Milestone, Plan, Task 분해를 채워요. -->
### M1 첫 묶음

- P1 첫 계획이에요.
- [ ] **R02** 승인 없이 추가한 요구사항이에요. — accept: 새 검사를 통과해요.
<!-- selftest-stamp: approved -->
