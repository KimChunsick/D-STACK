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
# Lint violation at approval

Every other check passes, so the one refusal is the Korean lint approve runs on the whole file.

## 배경과 문제

이 요청서가 정본이라서 승인한 뒤에는 내용이 바뀌면 안 돼요.

## 목표

1. 승인할 때 요청서 전체를 `dstack lint-ko`로 검사해요.

## 비목표

1. 절이 비었거나 행 문법이 틀린 요청서는 이 고정물에서 다루지 않아요.

## 사용 시나리오

### S1 금지 단어가 남은 요청서를 승인해요

고정물 실행기가 배경 문단에 S1 단어가 남은 요청서로 `dstack request approve`를 실행해요.

## 요구사항

- [ ] **R01** 승인 때 한국어 검사가 S1 단어를 막아요. — accept: request approve가 1로 끝나고 승인 도장을 남기지 않아요.

## 열린 가정

없음.

<!-- selftest-judge: approve -->
