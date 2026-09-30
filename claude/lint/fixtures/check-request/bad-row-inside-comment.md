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
# Requirements whose only row sits in a comment

Every part-1 section is filled, but the one R row under 요구사항 is wrapped in a multi-line HTML comment.

# 1부 요청

## 배경과 문제

A row a reader never sees is no requirement, however well formed the hidden line is.

## 목표

1. check request refuses a request whose only R row is inside an HTML comment.

## 비목표

1. A commented row beside a live one is not itself a failure.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

<!--
- [ ] **R01** a row inside a comment is not a requirement — accept: check request exits 1
-->

## 열린 가정

없음.
