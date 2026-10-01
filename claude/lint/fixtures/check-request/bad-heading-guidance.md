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
# 제목 줄에 남은 안내문

배경과 문제 절에 글이 있어 보이지만, 그 글은 템플릿 안내문을 주석으로 단 소제목 한 줄뿐이에요.

## 한눈에 보기

<!-- 승인 전에 확인할 것을 모으는 절이에요. 직접 쓰지 않아요. -->

# 1부 요청

## 배경과 문제

### 참고 <!-- 배경을 적어요. (키: background) -->

## 목표

1. check request가 제목 줄 안에 닫힌 주석으로 남은 안내문을 찾아내요.

## 비목표

1. 2부 절은 여기서 검사하지 않아요.

## 사용 시나리오

### S1 고정물 실행기가 이 파일을 검사해요

실행기가 파일을 격리된 저장소에 복사한 뒤 dstack check request를 돌려요.

## 요구사항

- [ ] **R01** 제목 줄 안의 주석에 안내문이 남은 절은 실패로 봐요. — accept: check request가 "## 배경과 문제"를 짚고 종료 코드 1로 끝나요.

## 열린 가정

없음.
