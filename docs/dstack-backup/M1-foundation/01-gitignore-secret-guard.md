# 01-gitignore-secret-guard

## 의도 / Why
공개 저장소이므로 **비밀/런타임 파일이 단 한 번도 커밋되지 않는 것**이 최우선. 블록리스트가 아니라 "전부 차단 후 허용" 올로우리스트 + 알려진 비밀 파일명 강제 차단으로, 이후 모든 작업의 안전망을 먼저 깐다. → PLAN.md `Milestone 1 / Task 1`.

## 작업 내용 (무엇을 / 왜)
- `tests/lib.sh`, `tests/run.sh`(테스트 러너), `tests/test_gitignore_secret_guard.sh`(Red).
- `.gitignore`: `/*` 차단 → 큐레이트 경로만 `!` 재포함 → `auth.json`/`*.sqlite*`/`history.jsonl`/`config.toml`/`.DS_Store`/`sessions`/`projects`/`memory`/`*.key`/`*.pem`/`.env` 강제 차단(허용 디렉터리 안쪽까지).

## 변경 파일 (어디를 / 왜 수정)
- `.gitignore` — 비밀 유입 차단(올로우리스트).
- `tests/lib.sh`, `tests/run.sh` — 무의존성 bash 테스트 하네스.
- `tests/test_gitignore_secret_guard.sh` — 더미 비밀을 허용 디렉터리에 넣어도 ignore 되는지, `git ls-files`에 비밀 패턴이 없는지 검증.

## 적대적 리뷰 (Codex 한도 → Claude 대체, 사용자 승인)
- `codex exec`(GPT-5.5)이 사용량 한도 초과로 거부 → 사용자 결정에 따라 **Claude 적대적 서브에이전트**로 대체 검증.
- **Round 1 (REJECT):** allowlist가 에이전트 디렉터리를 통째 재포함 → `claude/id_rsa`·`credentials.json`·`.netrc`·`*.db`·`*.p12`·`*.token` 등 미지정 비밀명이 추적 가능(HIGH). 반례로 `.env`/`*.key`/`*.pem`이 루트만 매칭한다는 2건은 **경험적으로 반박**(슬래시 없는 패턴은 모든 깊이 매칭, `git check-ignore`로 확인).
  - 조치: 각 에이전트 디렉터리 deny-all 후 큐레이트만 재포함, 비밀명 deny 대폭 확장, 테스트에 leak 배터리+인덱스 검사 추가. (commit `b6343ff`)
- **Round 2 (APPROVE_WITH_FIXES):** HIGH 닫힘 확인. 잔여 MED — 통째 재포함된 `hooks/`·`skills/`·`rules/` 내부의 *중첩 미지정 이름*은 여전히 추적 가능.
  - 조치: `hooks/`·`rules/`는 정확한 파일로, `skills/`는 정확한 스킬 디렉터리로 핀. 스킬 내부 통째는 불가피 → 비밀명 deny가 backstop, AGENTS.md에 문서화. 테스트에 중첩-미지정 프로브 추가. (commit `34fa835`) 경험 검증: 모든 nested-unknown IGNORED, 큐레이트 콘텐츠 allowed.
- 합의 상태: **수정완료** (HIGH/MED 모두 해소, 잔여는 문서화된 수용 리스크)

## E2E 검증
- `docs/dstack-backup/evidence/m1-tests.txt` — `bash tests/run.sh` 전체 PASS + 구조 검증(@import, 추적트리 비밀 0건, 더미 auth.json ignore 확인).

## 게이트 상태
- [x] TDD: Red→Green→Refactor 완료
- [x] 적대적 리뷰 합의완료 (Codex 한도→Claude 대체, 2라운드, 수정완료)
- [x] E2E 캡처 검증 완료
