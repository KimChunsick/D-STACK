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
# Edited after approval

A line is appended after approval, so the sha256 in request.approved no longer describes this file.

## 배경과 문제

승인한 뒤에 누가 요청서를 고치면 승인 해시가 파일 내용과 더는 맞지 않아요. 그 변경은 아무도 승인하지 않은 내용이에요.

## 목표

1. 승인 뒤에 고친 요청서는 `dstack check request`가 해시 불일치로 거부해요.

## 비목표

1. 절이 비었거나 행 문법이 틀린 요청서는 이 고정물에서 다루지 않아요.

## 사용 시나리오

### S1 승인한 요청서 끝에 한 줄을 덧붙여요

고정물 실행기가 요청서를 승인하고 파일 끝에 한 줄을 덧붙인 다음 `dstack check request`를 실행해요.

## Requirements

- [ ] **R01** an edit after approval is caught by the hash — accept: check request prints "hash mismatch (edited after approval)"

## 열린 가정

없음.
