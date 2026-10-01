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
# 3부가 있는 요청서를 승인해요

승인 해시는 3부 경계 줄 위까지만 덮고, 그 아래는 CLI가 계획 대장에서 다시 만들어요.

# 1부 요청

## 배경과 문제

3부는 계획이 바뀔 때마다 다시 만들어져요. 그 부분까지 해시에 넣으면 승인한 요청서가 계획을 고칠 때마다 불일치로 바뀌어요.

## 목표

1. 3부 경계 줄이 있는 요청서를 승인한 뒤 고치지 않으면 `dstack check request`를 통과해요.

## 비목표

1. 3부 아래를 고친 경우와 위를 고친 경우는 cargo test R15가 맡아요. 고정물 실행기는 good 고정물을 고치지 않아요.

## 사용 시나리오

### S1 3부가 있는 요청서를 승인해요

고정물 실행기가 요청서를 승인한 다음, 아무것도 바꾸지 않고 `dstack check request`를 실행해요.

## 요구사항

- [ ] **R01** 3부 경계 줄이 있는 요청서도 승인한 그대로면 해시가 맞아요. — accept: check request가 0으로 끝나요.

## 열린 가정

없음.

# 3부 계획과 검증
<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->
<!-- 이 부분은 직접 쓰지 않아요. 계획 대장(plan.json)이 바뀔 때마다 CLI가 Milestone, Plan, Task 분해를 채워요. -->
