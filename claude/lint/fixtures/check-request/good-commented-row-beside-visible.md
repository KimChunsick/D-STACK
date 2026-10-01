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
# 보이는 행 옆에 주석으로 감춘 행

요구사항 절에 보이는 R01이 있고, 그 뒤의 여러 줄 주석 안에 아직 다듬지 않은 R02를 보관해 둔 요청서예요.

## 한눈에 보기

<!-- 승인 전에 확인할 것을 모으는 절이에요. 직접 쓰지 않아요. -->

# 1부 요청

## 배경과 문제

Markdown 화면은 주석 안의 행을 보여주지 않아요. 검사도 그 행을 없는 행으로 읽어야 화면과 어긋나지 않아요.

## 목표

1. check request가 보이는 행만 세고 주석 안의 행은 검사하지 않아요.

## 비목표

1. 주석 안의 행 번호는 다음 행 번호를 정할 때만 읽어요. 이 고정물은 그 동작을 검사하지 않아요.

## 사용 시나리오

### S1 고정물 실행기가 이 파일을 검사해요

실행기가 파일을 격리된 저장소에 복사한 뒤 dstack check request를 돌려요.

## 요구사항

- [ ] **R01** 주석 밖의 행만 요구사항으로 읽어요. — accept: check request가 종료 코드 0으로 끝나요.
<!--
- [ ] **R02** 아직 다듬지 않은 행이에요. — accept: pending: agent to propose
-->

## 열린 가정

없음.
