# D-STACK

토큰과 시간을 아끼면서 최고 품질의 결과를 내기 위한 Claude Code·Codex 작업 설정이에요. 요구사항마다
번호(R)를 붙이고, "덮였는가·증명됐는가"를 사람이 아니라 `dstack` 명령이 세서 판정해요. 프론트엔드뿐
아니라 서버, 일반 소프트웨어, 글쓰기, 업무 보조에도 같은 흐름을 써요.

v2는 v1을 통째로 다시 만들었어요. v1은 태그 `v1-final`에만 남아 있어요.

## 5분 시작하기

```bash
git clone git@github.com:KimChunsick/D-STACK.git ~/D-STACK
cd ~/D-STACK
./install.sh --dry-run    # 무엇이 바뀌는지 표로 확인해요
./install.sh              # 바이너리를 빌드해 ~/.claude, ~/.codex에 링크하고 settings.json을 병합해요
export PATH="$HOME/.claude/bin:$PATH"
dstack doctor             # 도구, 에이전트, 훅, 규칙표를 한 번에 점검해요
```

설치는 `cargo build --release`로 `dstack` 바이너리를 먼저 만들어요. cargo가 없으면 아무것도
링크하지 않고 무엇을 설치해야 하는지 알려주고 멈춰요. settings.json을 합칠 때는 jq도 있어야 해요.

설치가 끝나면 새 세션이나 `/clear` 뒤에 출력 스타일 `dstack-korean`이 적용돼요.

## 작업 흐름

| 단계 | 하는 일 | 쓰는 것 |
|---|---|---|
| 갈래 정하기 | 열린 Goal에 합칠지, 새 Goal을 열지, 빠른 작업으로 갈지 | `dstack status`, 스킬 `dstack-workflow` |
| 요청서 | R 행(한 줄 + 관찰 가능한 기준)을 번호 붙여 적고 승인해요 | `dstack req add`, `dstack request approve` |
| 조사와 인터뷰 | sonnet 에이전트가 코드를 읽어 recon.md를 쓰고, 질문은 대장으로 관리해요 | `dstack ask add|answer|assume` |
| 계획 | Milestone → Plan → Task를 등록하고 파도 단위로 돌려요 | `dstack plan add`, `dstack next`, 스킬 `dstack-develop` |
| 구현 | Plan마다 빈 맥락의 opus 워커가 dstack이 만든 worktree에서 일해요 | `dstack plan start --worktree`, `dstack worker report` |
| 리뷰 | Plan이 끝날 때마다 Codex(gpt-6-astra)가 요청서 원문과 diff를 함께 봐요 | `dstack review --scope plan`, 스킬 `codex-review` |
| 검증과 보고 | 증거를 대장에 기록하고 R별 상태를 계산해요 | `dstack evidence add`, `dstack verify`, `dstack report`, 스킬 `dstack-verify` |

빠른 작업은 Goal 밖의 별도 트랙이에요: `dstack quick new <slug>`가 같은 요청서·대장·검사기를
쓰되 선택 단계를 전부 끈 채 시작해요(스킬 `dstack-quick`).

## 요청서 필드

요청서 머리말이 곧 완료 조건이에요. 어떤 단계가 도는지는 이 필드가 정하고, 훅은 여기 적힌
값으로만 검사해요. 자동으로 계산하는 등급이나 프롬프트 토큰은 없어요.

| 필드 | 값 | 뜻 |
|---|---|---|
| `work_type` | web-ui, http-api, cli, library, docs-writing | 검증 프로필과 기본값을 정해요 |
| `route` | merge <run>, new-goal, quick | 요청이 어디에 속하는지 |
| `external_research` | none, one-pass | Codex 리서치 1회 + 감사 1회 |
| `risk_axes` | none, ux, perf, security | recon.md의 위험 표에 넣을 축 |
| `design_review` | required, auto, skip | 설계 확인 라운드 |
| `review` | on, off | 라운드 수와 축만 조절해요. R별 판정은 끄지 못해요 |
| `codex_effort` | medium, high, xhigh | 기본값은 high예요. 리뷰·리서치는 항상 high로 실행해요 |
| `e2e` | capture, cli, none | 어떤 종류의 증거가 필요한지 |
| `unit_tests` | on, off | Red/Green/Refactor와 테스트 증거 |
| `visual` | design, regression, none | 화면 비교. 비교 도구가 들어 있지 않아 지금은 `none`만 증거를 남겨요 |
| `korean_polish` | on, off | 사람용 산문을 승인 전에 한 번 다듬어요 |

저장소 정책(`.dstack/project/PROJECT.md`의 `## Verification policy`)이 상한이고, 요청서는 그
안에서만 좁힐 수 있어요. 넘으면 `dstack verify`가 정책의 `why` 줄과 함께 거부해요.

## 지켜지는 방식

- **판정은 명령이 계산하고 훅이 막아요.** 체크박스를 에이전트가 직접 켜는 일은 없어요. 증거 행은
  `dstack evidence add`만 쓰고, 산출물의 sha256을 `dstack verify`가 다시 계산해요.
- **말 없는 통과는 없어요.** 모든 검사 명령이 무엇을 몇 개 세었는지 출력하고, 검사기마다 붙박이
  예제(`claude/lint/fixtures/`)를 둬서 `dstack doctor --self`가 "잡아야 할 것을 잡는지" 확인해요.
- **훅은 판정을 못 내리면 막아요.** 훅 스크립트는 `claude/hooks/dstack-hook.sh` 하나예요. 이 스크립트는
  바이너리를 찾아 넘겨주기만 하고, 찾지 못하면 exit 2로 끝나요. 반복해서 막힐 때의 탈출구는
  `dstack run pause`예요.
- **서브에이전트는 Fable로 띄우지 않아요.** 에이전트 머리말의 `model`과, model이 빠진 Agent 호출을
  opus로 바꿔 넣는 훅이 두 겹으로 지켜요. Codex는 `codex exec` 플래그로 gpt-6-astra와 추론 강도 high에 고정해요.
- **`.dstack/`은 기기에만 있어요.** 커밋되지 않고, `dstack init`이 `.gitignore`에 넣어요.
- **워커가 겪은 불편은 파일로 남아요.** 구현 워커가 dstack 때문에 막히거나 시간을 버리면
  `dstack issue new`로 한 건씩 적어 두고 하던 일을 이어가요. 파일은 저장소 밖
  `~/Documents/dstack-issues`에 쌓이고, 다시 볼 때는 `dstack issue list`를 써요.

## 저장소 구조

```
dstack-cli/                 명령 하나를 만드는 Rust 크레이트
  src/core/                 인자 읽기, 동사 등록표, 종료 코드, 저장소 경로
  src/store/                meta.tsv·요청서·plan.json 같은 저장 파일의 읽기와 쓰기
  src/verbs/                동사별 구현. 파일 하나가 책임 하나이고 350줄이 상한이에요
  src/selftest/             붙박이 예제를 돌리는 검사기
  parity/                   명시적으로 켰을 때만 과거 셸 구현과 출력을 비교하는 도구
  tests/                    cargo test가 도는 통합 테스트
claude/hooks/dstack-hook.sh 유일한 훅 스크립트 (inject·stop·agent-model·pre-write)
claude/skills/              dstack-workflow, dstack-develop, dstack-verify, dstack-quick,
                            codex-review, codex-research, unit-test
claude/agents/              frontend-dev, general-dev(opus) · recon, e2e-runner, ko-polish(sonnet)
claude/templates/request/   작업 종류별 요청서 틀
claude/lint/                한국어 규칙표(ko-rules.tsv), 범위표(ko-scope.tsv), 붙박이 예제
claude/settings.enforced.json, claude/settings/model-policy.json
                            settings.json에 병합되는 강제 키 (훅, 모델, 출력 스타일)
codex/                      Codex 전역 지침과 역할 스킬(dstack-reviewer, dstack-researcher)
deps.tsv                    외부 실행 파일 목록. dstack doctor가 전부 확인해요
```

## 한국어 규칙

사람에게 쓰는 모든 한국어는 해요체이고, 영어 직역 단어와 AI 티가 나는 표현을 피해요. 규칙표는
`claude/lint/ko-rules.tsv`(fluent-korean과 im-not-ai에서 가져왔고, 출처는 표 머리말에 적혀 있어요)
하나이고, 세 곳에 같은 규칙이 적용돼요.

- 터미널 응답: 출력 스타일 `dstack-korean`이 미리 지켜요. 사후에 고칠 방법은 없어요.
- 파일과 커밋 메시지: `dstack lint-ko`가 정규식 규칙을 검사하고, Write·Edit·Bash 훅이 S1 위반을
  막아요. 범위는 `ko-scope.tsv`가 정하고, 범위표가 없는 저장소에서는 아무것도 막지 않아요.
- README·가이드·요청서·완료 보고: `korean_polish: on`이면 ko-polish 에이전트가 승인 전에 한 번
  다듬어요. 코드 주석과 규칙 파일은 자동으로 다시 쓰지 않아요.

요청서는 일반 작업과 빠른 작업 모두 항상 한국어 해요체로 작성해요. 제목·설명·요구사항·완료
기준까지 포함하고, `korean_polish: off`여도 같아요. 머리말의 필드명과 정해진 값, R 번호,
`accept:`와 상태 표시, 명령어·경로·코드 식별자는 유지해요.
그 밖의 워크플로 산출물(recon, 결정, 계획, 리뷰 기록, 에이전트 간 프롬프트)은 영어예요.
요청서 원문을 인용할 때는 한국어 그대로 옮겨요.

## 검사 실행

기본 검사는 `bash dstack-cli/test.sh`로 실행해요. 일반 단위 테스트와 검사기 예제는 계속
검증하고, 과거 셸 구현이 필요한 테스트는 건너뛴 이유와 함께 `ignored`로 표시해요.
일반 작업에서는 `shell-final` 태그를 요구하거나 복원하지 않아요.

과거 구현과의 비교가 필요할 때만 기준 자료를 준비한 뒤 직접 켜요.

- Rust 비교 테스트: `bash dstack-cli/test.sh --features shell-parity`
- 셸 비교 도구: `bash dstack-cli/parity/run.sh --shell-ref shell-final`
- 다른 기준 실행 파일 사용: `bash dstack-cli/parity/run.sh --shell <실행 파일 경로>`

비교 도구에 기준을 지정하지 않으면 `skipped:`를 출력하고 끝나요. 직접 켠 비교에서 기준 자료가
없거나 실제 차이가 발견되면 실패로 처리해요.

## 자주 걸리는 것

- `dstack run new`가 "already runs"로 거부하면 두 번째 Goal은 `--worktree <경로>`로 열어요. worktree
  하나에 Goal run 하나가 규칙이에요.
- `/clear` 뒤에는 `dstack run adopt`로 이어받아요. 10분 넘게 갱신이 없는 run은 자동으로 넘어와요.
- 승인 뒤에 요청서를 손으로 고치면 `dstack check request`가 해시 불일치로 실패해요. 새 요구는
  `dstack req add`로 붙이고 다시 승인해요. 기존 행은 고치지 않아요.
- 증거를 못 내는 R은 `dstack verify --accept-abstain R0N --why <사유>`로 한 건씩 받아야 Goal이 닫혀요.
  그 사유는 완료 보고에 그대로 실려요.
- 나중에 할 것(§10): `doctor --drift`, MCP 설정 추적, Codex 설정 틀, 응답 변조 정책은 `DEFERRED`로
  따로 세요.
