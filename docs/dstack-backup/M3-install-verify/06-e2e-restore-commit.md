# 06-e2e-restore-commit

## 의도 / Why
"새 머신에서 한 번에 복원"이 실제로 되는지 손으로 검증하고, 위험한 실제-홈 설치는 사용자 확인 절차로 명문화. 마지막에 초기 커밋(푸시는 사용자 승인 후). → PLAN.md `Milestone 3 / Task 6`.

## 작업 내용 (무엇을 / 왜)
- 샌드박스 HOME 전체 E2E: `./install.sh` → 모든 MAP 항목이 저장소로 심링크됨을 `find -type l -ls`로 증거 캡처.
- `settings.json`의 `$HOME` 확장 실동작 확인(또는 리터럴 폴백 결정 기록).
- `tests/run.sh` 전체 녹색 캡처.
- README/AGENTS.md에 실제-홈 설치 = 사용자 확인 후(권장 `--dry-run` 선행, `*.bak` 백업) 명시.

## 변경 파일 (어디를 / 왜 수정)
- `docs/dstack-backup/evidence/*` — E2E 증거.
- `README.md`,`AGENTS.md` — 복원/언인스톨 절차 보강.

## 적대적 리뷰 (Codex 한도 → Claude 대체, 사용자 승인)
- E2E 증거의 충분성 검토. 심링크 타깃이 모두 저장소 내부로 해석되는지, `$HOME` 확장 실증이 hook 실행 경로(`sh -c`)와 일치하는지 확인 → 충족.
- 실제-홈 설치는 사용자 확인 단계(README: `--dry-run` 선행 + `*.bak` 백업 + uninstall 절차)로 명문화.
- 합의 상태: **수정완료**

## E2E 검증
- `docs/dstack-backup/evidence/m3-install-e2e.txt` — 전체 캡처. 멱등 2회차 up-to-date=9, 스위트 5개 PASS.

## 게이트 상태
- [x] TDD: Red→Green→Refactor 완료 (install 테스트가 E2E 동작을 인코딩)
- [x] 적대적 리뷰 합의완료 (Codex 한도→Claude 대체, 수정완료)
- [x] E2E 캡처 검증 완료
