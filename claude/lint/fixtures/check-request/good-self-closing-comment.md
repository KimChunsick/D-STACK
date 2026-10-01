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
# 스스로 닫히는 주석 뒤의 글

배경과 문제 절은 스스로 닫히는 주석으로 시작하지만 그 뒤에 글이 있어서 채운 절이에요.

## 한눈에 보기

<!-- 승인 전에 확인할 것을 모으는 절이에요. 직접 쓰지 않아요. -->

# 1부 요청

## 배경과 문제

<!--> Markdown 화면은 이 주석을 바로 닫고 뒤의 글을 보여줘요. 검사도 같은 방식으로 읽어요.

## 목표

1. check request가 스스로 닫히는 주석 뒤의 글을 채운 절로 읽어요.

## 비목표

1. 2부 절은 여기서 검사하지 않아요.

## 사용 시나리오

### S1 고정물 실행기가 이 파일을 검사해요

실행기가 파일을 격리된 저장소에 복사한 뒤 dstack check request를 돌려요.

## 요구사항

- [ ] **R01** 스스로 닫히는 주석 뒤에 글이 있는 절은 채운 절로 봐요. — accept: check request가 종료 코드 0으로 끝나요.

## 열린 가정

없음.
