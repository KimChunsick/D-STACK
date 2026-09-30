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
# Untouched after approval

Nothing edits the file between dstack request approve and dstack check request, so the hash still matches.

## 배경과 문제

승인 해시는 승인한 순간의 요청서 내용을 담아요. 그 뒤로 파일을 고치지 않았다면 검사할 때도 같은 해시가 나와야 맞아요.

## 목표

1. 승인 뒤에 고치지 않은 요청서는 `dstack check request`를 통과해요.

## 비목표

1. 승인 뒤에 고친 요청서를 막는 경우는 bad-edited-after-approval.md가 맡아요.

## 사용 시나리오

### S1 고치지 않은 요청서를 다시 검사해요

고정물 실행기가 요청서를 승인한 다음, 아무것도 바꾸지 않고 `dstack check request`를 실행해요.

## Requirements

- [ ] **R01** the approval hash matches an unedited file — accept: check request prints "approved: yes" and exits 0

## 열린 가정

없음.
