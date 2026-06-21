# 02-repo-agents-docs

## 의도 / Why
사용자가 지시한 핵심: **이 저장소의 `AGENTS.md`가 기본 가이드이고 `CLAUDE.md`는 그것을 내부에서 호출(@import)**. 둘 다 루트의 실제 파일이라 상대 import가 안전(심링크 아님 → GH #4754 무관). 새 에이전트가 이 백업 저장소에서 작업할 때의 규칙·구조·안전수칙을 한곳에. → PLAN.md `Milestone 1 / Task 2`.

## 작업 내용 (무엇을 / 왜)
- 루트 `CLAUDE.md` = `@AGENTS.md` 한 줄(+Claude 노트).
- 루트 `AGENTS.md` = 저장소 가이드: 무엇/레이아웃(Layer A·B)/골든룰("Never commit secrets"+제외목록·authored만)/install.sh(심링크+Gemini copy 주의)/새 에이전트 추가법.
- `README.md`(사람용, 한 줄 복원), `claude|codex|gemini/.gitkeep`.

## 변경 파일 (어디를 / 왜 수정)
- `AGENTS.md`·`CLAUDE.md`·`README.md` — Layer A 문서.
- `claude/.gitkeep`,`codex/.gitkeep`,`gemini/.gitkeep` — 에이전트 우선 폴더 골격.
- `tests/test_repo_agents_docs.sh` — `CLAUDE.md`의 `@AGENTS.md` import·안전수칙 문구·폴더 존재 검증.

## 적대적 리뷰 (Codex 한도 → Claude 대체, 사용자 승인)
- Claude 적대적 서브에이전트가 함께 검토. 이 문서 관련 지적:
  - (MED) `install.sh`를 AGENTS.md/README가 이미 존재하는 듯 기술 → **수용**: 같은 작업 범위 M3에서 실제 구현(완료 시 일치). 중간 상태 한정 불일치.
  - (MED) `@AGENTS.md` import은 문자열만 테스트, 런타임 해석은 CI로 검증 불가 → **수용**: 리서치로 안전성 확인된 메커니즘(루트 실제 파일+상대 import), 문서화된 한계.
- 합의 상태: **수정완료** (블로킹 이슈 없음, 수용 사항 명시)

## E2E 검증
- `docs/dstack-backup/evidence/m1-tests.txt` — `bash tests/run.sh` 전체 PASS + 구조 검증(CLAUDE.md `@AGENTS.md` import, 폴더 골격, 추적트리 비밀 0건).

## 게이트 상태
- [x] TDD: Red→Green→Refactor 완료
- [x] 적대적 리뷰 합의완료 (Codex 한도→Claude 대체, 수정완료)
- [x] E2E 캡처 검증 완료
