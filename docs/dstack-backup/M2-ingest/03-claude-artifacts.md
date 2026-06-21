# 03-claude-artifacts

## 의도 / Why
Claude 측 **내가 직접 만든** 산출물만 SSOT로 수집. 이후 install.sh가 이걸 `~/.claude`로 심링크. `settings.json`의 `/Users/<user>` 하드코딩 3곳을 `$HOME`로 바꿔 이식성 확보(다른 머신/유저에서도 동작). → PLAN.md `Milestone 2 / Task 3`.

## 작업 내용 (무엇을 / 왜)
- 복사: `CLAUDE.md`, `settings.json`, `statusline-command.sh`, `hooks/fullcycle-inject.sh`, `hooks/fullcycle-gate.sh`, `skills/full-cycle`, `skills/codex-review`.
- 이식성: `settings.json`의 hook·statusline command 경로 `/Users/<user>/.claude/...` → `$HOME/.claude/...`. (Claude의 `$HOME` 확장 여부는 M3 E2E에서 검증, 미확장 시 리터럴+install.sh 재작성 폴백.)

## 변경 파일 (어디를 / 왜 수정)
- `claude/**` — authored 산출물 7종.
- `tests/test_claude_artifacts.sh` — 존재/비어있지않음, `settings.json`에 `/Users/` 부재·`$HOME` 존재, 비밀 부재 검증.

## 적대적 리뷰 (Codex 한도 → Claude 대체, 사용자 승인)
- Claude 적대적 서브에이전트가 ingest 내용 전체를 grep 검증. **실제 설정 파일엔 API키/토큰/개인키/이메일 0건**(결정론적 스캐너로도 확인).
- 지적 → 조치:
  - (MED) `settings.json`에 third-party 플러그인/마켓플레이스(Toss 노출) → **사용자 결정으로 제거** (commit `868c7a7`), 가드 테스트로 재유입 차단.
  - (MED) `test_claude_artifacts`가 `settings.json`만 스캔 → `claude/` 전체 `/Users/` 스캔 + `jq` JSON 검증으로 강화.
  - (HIGH, 공통) 사용자명/프로젝트명이 git 히스토리에 잔존 → 사용자 결정: **push 직전 히스토리 스크럽**(M3 최종 단계), author 이메일 `<maintainer-email>`로 변경.
- 합의 상태: **수정완료** (코드 수정 완료, 히스토리 스크럽은 최종 단계 예약)

## E2E 검증
- `docs/dstack-backup/evidence/m2-tests.txt` — 스위트 PASS, `settings.json` valid JSON·`$HOME` 3·`/Users/` 0·third-party 0, 추적트리 PII 0.

## 게이트 상태
- [x] TDD: Red→Green→Refactor 완료
- [x] 적대적 리뷰 합의완료 (Codex 한도→Claude 대체, 수정완료)
- [x] E2E 캡처 검증 완료
