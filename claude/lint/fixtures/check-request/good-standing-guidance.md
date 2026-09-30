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
# Standing guidance outside the required sections

Every part-1 section is filled; 요구사항 keeps the template's standing guidance above its row, and part 2 keeps its guidance.

## 한눈에 보기

<!-- 승인 전에 확인할 것을 모으는 절이에요. 직접 쓰지 않아요. -->

# 1부 요청

<!-- 무엇을 왜 바꾸는지 적는 부분이에요. 각 절은 dstack request section <키> --from <파일>로 채우고, 요구사항은 dstack req add로만 추가해요. 주석만 남은 절은 빈 절로 봐요. -->

## 배경과 문제

The guidance under 요구사항 tells how rows are added; it is an instruction, not an unfilled section.

## 목표

1. check request passes a filled request whose other places keep their guidance.

## 비목표

1. The part-2 sections are not judged here.

## 사용 시나리오

### S1 The fixture runner checks this file

The runner copies the file into a sandbox and runs dstack check request on it.

## 요구사항

<!-- dstack req add "<한국어 요구사항>" --accept "<한국어 완료 기준>"으로 행을 추가해요. 직접 작성하지 않아요. -->
<!-- 진행 중인 요구사항은 12개, 요구사항 절은 60줄이 상한이에요(R43). 넘으면 Milestone을 나눠요. -->
- [ ] **R01** standing guidance outside the required sections passes — accept: check request exits 0

## 열린 가정

없음.

# 2부 설계

## 지금 구조

<!-- 바꾸기 전의 모듈, 파일, 데이터 흐름을 적어요. 표나 코드 블록을 써도 돼요. (키: current) -->

## 위험

<!-- 이 변경으로 깨질 수 있는 것, 되돌리기 어려운 것, 확인하지 못한 것을 대비책과 함께 적어요. (키: risks) -->
