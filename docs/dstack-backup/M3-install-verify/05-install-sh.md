# 05-install-sh

## 의도 / Why
SSOT를 실제로 활성화하는 메커니즘. `~/.claude`/`~/.codex`의 파일을 저장소로 향하는 **심링크**로 교체하되, **기존 실파일은 `*.bak.<ts>`로 백업**하고 **멱등·`--dry-run`** 지원으로 사용자의 실제 홈을 안전하게 건드린다. → PLAN.md `Milestone 3 / Task 5`.

## 작업 내용 (무엇을 / 왜)
- 단일 MAP(`repo_relpath|target_abspath|mode(link|copy)`)로 선언.
- 각 항목: 부모 에이전트 디렉터리 없으면 skip(예: `~/.gemini` 부재) → 기존 실파일 백업 → `ln -sfn`(copy 모드는 `cp -R`, Gemini 대비).
- `--dry-run`은 계획만 출력. 재실행 시 무변경(멱등).

## 변경 파일 (어디를 / 왜 수정)
- `install.sh` — 심링크 엔진 + 백업 + dry-run.
- `tests/test_install_sh.sh` — **샌드박스 HOME**(`mktemp -d`)에서 링크 생성/타겟 정확성/기존파일 백업/멱등 검증. 실제 `~`는 절대 건드리지 않음.

## 적대적 리뷰 (Codex 한도 → Claude 대체, 사용자 승인)
- 실제 홈을 변경하는 스크립트라 데이터손실 관점으로 집중 검증.
- (HIGH) `ts`가 초 단위 → **같은 초 재실행 시 백업명 충돌로 이전 백업(원본) 덮어쓰기** → **수정**: 충돌 없는 이름까지 카운터 증가, `DSTACK_BACKUP_TS`로 결정론적 테스트. 검증: `fixedstamp`=ORIGINAL, `fixedstamp.1`=SECOND 둘 다 보존. (commit `fix(m3-t5) collision-safe`)
- 나머지(LOW/MED, 무손실): set -e 산술 `x=$((x+1))` 안전, `--dry-run` 무변경(리뷰어 철회), `$HOME` unset 시 `set -u`로 즉시 중단, 디렉터리 백업→링크 순서 안전, 절대경로 readlink 비교 정확. 향후 MAP에 `..`/`./` 추가 시 주의는 주석/문서로 남김.
- 합의 상태: **수정완료**

## E2E 검증
- `docs/dstack-backup/evidence/m3-install-e2e.txt` — 샌드박스 HOME 실제 install: 9개 전부 저장소로 해석되는 심링크, 심링크 통해 내용 read, `$HOME` 확장 실증, 멱등 재실행 무변경, 스위트 PASS.

## 게이트 상태
- [x] TDD: Red→Green→Refactor 완료
- [x] 적대적 리뷰 합의완료 (Codex 한도→Claude 대체, 수정완료)
- [x] E2E 캡처 검증 완료
