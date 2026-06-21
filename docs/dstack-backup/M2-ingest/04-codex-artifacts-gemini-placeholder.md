# 04-codex-artifacts-gemini-placeholder

## 의도 / Why
Codex 측 authored 산출물(`instructions.md`, `rules/default.rules`)만 수집하고, **`config.toml`/`auth.json`/비공개 프로젝트 경로는 절대 제외**(공개 저장소). Gemini는 아직 없으므로 확장용 자리만 마련. → PLAN.md `Milestone 2 / Task 4`.

## 작업 내용 (무엇을 / 왜)
- 복사: `~/.codex/instructions.md`, `~/.codex/rules/default.rules`.
- `gemini/README.md`: 온보딩 절차 + "Gemini는 심링크된 컨텍스트 파일을 무시(GH #11547) → install.sh는 copy" 주의.

## 변경 파일 (어디를 / 왜 수정)
- `codex/instructions.md`, `codex/rules/default.rules` — authored.
- `gemini/README.md` — 확장 자리.
- `tests/test_codex_artifacts.sh` — 산출물 존재 + `config.toml`/`auth.json` 부재 + `<private-project>|<private-project>|<private-project>` 경로 누출 부재 검증.

## 적대적 리뷰 (Codex 한도 → Claude 대체, 사용자 승인)
- 지적 → 조치:
  - (HIGH) 가드 테스트와 문서에 **비공개 프로젝트명을 하드코딩**(가드가 이름을 노출) → gitignore되는 `.private-denylist`로 분리, 문서는 placeholder로 교체. (commit `868c7a7` 이전 fix 커밋)
  - codex 산출물(`instructions.md`, `default.rules`)엔 `/Users/`·프로젝트명·비밀 0건(소스 사전검증 + 추적트리 스캔).
  - default.rules의 `git commit --no-verify` 허용은 사용자 authored 정책이므로 **원본 유지**(백업 대상 그대로), 리뷰 기록만 남김.
- 합의 상태: **수정완료**

## E2E 검증
- `docs/dstack-backup/evidence/m2-tests.txt` 참조 — codex/gemini 추적 파일 PII 0, denylist 가드 동작 확인.

## 게이트 상태
- [x] TDD: Red→Green→Refactor 완료
- [x] 적대적 리뷰 합의완료 (Codex 한도→Claude 대체, 수정완료)
- [x] E2E 캡처 검증 완료
