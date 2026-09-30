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
# Requirements without a row

Every part-1 section is filled, but 요구사항 still holds only the template's standing guidance and no R row.

# 1부 요청

## 배경과 문제

A request with no row has nothing a verification could observe, whatever its prose says.

## 목표

1. check request refuses a request that has no R row.

## 비목표

1. The standing guidance under 요구사항 is not itself a failure.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

<!-- dstack req add "<한국어 요구사항>" --accept "<한국어 완료 기준>"으로 행을 추가해요. 직접 작성하지 않아요. -->
<!-- accept:에는 확인할 명령의 출력과 종료 코드를 적어요. 단순히 코드를 작성했다는 설명은 완료 기준이 아니에요. -->

## 열린 가정

없음.
