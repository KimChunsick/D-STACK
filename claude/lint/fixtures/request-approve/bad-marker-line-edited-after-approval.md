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
# 승인 뒤에 3부 경계 줄을 고쳐요

3부 경계 줄은 줄 전체가 같을 때만 경계예요. 줄 끝에 글이 붙으면 경계가 사라지고 파일 전체를 해시해요.

# 1부 요청

## 배경과 문제

경계 문자열이 줄 안에 들어 있기만 해도 경계로 본다면, 경계 줄을 고친 요청서가 승인 해시를 그대로 통과해요.

## 목표

1. 경계 줄을 고친 요청서는 `dstack check request`가 해시 불일치로 거부해요.

## 비목표

1. 경계 줄 위의 1부와 2부를 고친 경우는 cargo test R15가 맡아요. 고정물 실행기는 파일 끝에만 덧붙여요.

## 사용 시나리오

### S1 경계 줄 끝에 글을 덧붙여요

이 파일은 경계 줄로 끝나고 마지막 줄바꿈이 없어요. 고정물 실행기가 승인한 뒤 파일 끝에 한 줄을 덧붙이면 그 글이 경계 줄에 붙어요.

## 요구사항

- [ ] **R01** 경계 줄을 고치면 해시가 맞지 않아요. — accept: check request가 "hash mismatch (edited after approval)"를 출력하고 1로 끝나요.

## 열린 가정

없음.

# 3부 계획과 검증
<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->