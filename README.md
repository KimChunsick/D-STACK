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
export PATH="$HOME/.claude/bin:$HOME/.codex/bin:$PATH"
dstack doctor             # 도구, 에이전트, 훅, 규칙표를 한 번에 점검해요
```

설치는 `cargo build --release`로 `dstack` 바이너리를 먼저 만들어요. cargo가 없으면 아무것도
링크하지 않고 무엇을 설치해야 하는지 알려주고 멈춰요. settings.json을 합칠 때는 jq도 있어야 해요.

양쪽 환경에 같은 워크플로 스킬과 실행 지침을 연결해요. Claude는 새 세션이나 `/clear` 뒤에
출력 스타일 `dstack-korean`을 적용하고, Codex는 설치된 지침에서 같은 한국어 규칙을 읽어요.
개인 스킬과 관계없는 설정은 유지하고, 바꾸는 파일은 먼저 백업해요. `~/.codex/config.toml`은
변경하지 않아요. 두 환경의 `bin/dstack`은 같은 실행 파일을 가리켜요.

## 메인과 서브를 선택해요

`main`은 작업을 진행하고 구현·탐색·검증 에이전트를 부르는 환경이에요. `sub`는 리뷰·리서치·감사를
맡아요. 두 값은 독립적이고, 설정이 없으면 메인은 Claude, 서브는 Codex예요.

프로젝트에서 `dstack init`을 실행한 뒤 원하는 행의 명령을 선택해요.

| 메인 | 서브 | 설정 명령 |
|---|---|---|
| Claude | Codex | `dstack mode set --main claude --sub codex` |
| Claude | Claude | `dstack mode set --main claude --sub claude` |
| Codex | Claude | `dstack mode set --main codex --sub claude` |
| Codex | Codex | `dstack mode set --main codex --sub codex` |

설정은 `.dstack/project/mode.json`에 저장하고, 새 Goal과 quick 작업은 시작할 때 그 값을
복사해요. `dstack mode show`에서 선택값과 적용 시점을 확인해요. 설정을 바꿔도 이미 열린
작업과 현재 대화의 엔진은 바뀌지 않아요. 선택한 앱이나 CLI에서 새 세션을 열어야 해요.

```bash
dstack mode show --host codex                  # Codex 세션에서 실제 메인 환경을 확인해요
dstack mode show --host claude --run <run-id>   # Claude 세션에서 지정한 Goal을 확인해요
dstack mode show --host codex --quick <slug>   # quick 작업은 저장된 조합을 확인해요
dstack run adopt                              # 기존 조합을 유지하며 이어받아요
dstack run adopt --refresh-mode               # 현재 프로젝트 설정으로 명시적으로 바꿔 받아요
```

일반 작업에서 실행 중인 앱과 메인 설정이 다르면 인계 방법을 출력하고 작업을 멈춰요.
사용자가 인계를 명시적으로 요청했을 때는 아래 절차로 준비와 재개만 진행할 수 있어요.
Claude 메인은 `Agent`로 구현에 opus, 탐색·검증에 sonnet을 쓰고, Codex 메인은
기본 `spawn_agent`로 새 맥락의 워커를 실행해요. Codex 워커는 메인 세션의 모델과 추론 강도를
물려받아요. D-STACK 호출 기준은 gpt-6-astra와 high이고, 실제 관찰한 엔진을 기록해요.

서브 실행은 다음 명령이 맡아요. `--dry-run`은 실행 인자를 JSON으로 보여주고 파일을 쓰거나
모델을 호출하지 않아요. 실제 실행 때는 이 옵션을 빼요.

```bash
dstack mode exec review-P1-001 --role review --context context.md --output review.md --dry-run
dstack mode exec research-001 --role research --context research-context.md --output research.md --run <run-id>
dstack mode exec research-audit-001 --role audit --context audit-context.md --output audit.md --quick <slug>
```

서브가 Codex면 gpt-6-astra, Claude면 opus를 high 강도로 실행해요. 메인과 서브가 같아도 매번
별도의 읽기 전용 세션에서 검토해요. 성공한 구조화 결과를 받은 뒤에만 출력 파일을 만들고,
실패하면 해당 오류를 남겨요. 다른 환경으로 자동 전환하지 않아요. 선택한 환경의 CLI와 로그인이
필요하고, `dstack doctor`가 필요한 CLI를 확인해요. 기존 스킬 이름 `codex-review`·`codex-research`와
봉인된 파일 이름 `codex-review-NNN.md`는 그대로 유지하며 선택한 서브 환경을 사용해요.

## 다른 메인 환경에 이어서 맡겨요

예를 들어 Codex에서 “기존 Claude 작업을 Codex로 인계해 줘”라고 요청하면 공유 스킬
`dstack-handoff`가 기존 Goal의 기록을 모아요. 메인 설정과 현재 앱이 달라도 인계 준비와
재개만 허용해요. 일반 작업은 새 메인 세션에서 재개 명령이 성공한 뒤에 시작해요.
사용량을 자동 감시하거나 실행 중인 대화의 엔진을 바꾸지는 않아요.

기존 작업과 같은 worktree에서 실행해요. 먼저 실행 계획을 확인한 뒤 인계 자료를 만들어요.

```bash
dstack handoff --to codex --run <run-id> --dry-run
dstack handoff --to codex --run <run-id>
```

반대 방향은 `--to claude`를 써요. `--dry-run`은 파일을 쓰거나 모델을 호출하지 않아요.
실제 요약은 목적지 환경의 모델을 high 강도로 호출해 만들어요. 기존 메인이나 서브 설정에
따라 요약 모델을 바꾸지 않고, 실패했을 때 다른 모델로 대신 실행하지 않아요.

기록은 저장된 `owner_session`과 `transcript_path`를 기준으로 찾아요. 경로가 없으면 같은
세션의 로컬 Claude/Codex JSONL을 찾아요. 다른 세션의 최신 기록을 대신 쓰지 않아요.
사용자 지정 홈이나 자동 탐색을 지원하지 않는 보관 경로는 파일을 직접 지정해요.

```bash
dstack handoff --to codex --run <run-id> --session <source-session-id> --history <source.jsonl>
```

`--session`은 저장된 소유 세션과 같아야 해요. 파일이 없거나 세션·worktree가 다르고,
형식이나 크기 제한을 어기면 오류를 내요. 기록 일부를 생략했다는 경고는 인계 자료에도 남아요.
준비 중에는 기존 세션과 워커의 작업, 실행 중인 명령을 멈춰 기록이 바뀌지 않게 해요.

CLI만 `.dstack/runs/<run-id>/handoffs/<handoff-id>/`에 `context.md`, `packet.json`,
`summary.json`, `RESUME.md`와 준비 완료 해시를 써요. 요약이 성공해도 아직 메인과 소유자는
바뀌지 않아요. 출력된 정확한 worktree에서 목적지 앱의 새 메인 세션을 열고 `RESUME.md`를 읽어요.
새 세션에는 기존 세션과 다른 비어 있지 않은 세션 식별자가 있어야 해요.

기존 세션과 그 세션의 모든 워커가 종료됐다는 명시적인 확인을 받은 뒤 재개해요.

```bash
dstack handoff resume <handoff-id> --host codex --source-stopped --run <run-id>
dstack mode show --host codex --run <run-id>
dstack status
```

재개는 원본 기록·대장·Git 파일 내용이 준비 시점과 같고, 인계 자료가 변조되지 않았으며,
미완료 실행 기록이 없는지 검사해요. 해당 Goal의 메인·소유자 정보와 `CURRENT`만 바꾸고,
프로젝트 기본값·서브·기존 증거는 유지해요. 오래되거나 실패한 자료는 새로 준비해야 해요.
인수 도중 프로세스가 끊겨 `resuming` 표시가 남으면 자동 재시도를 막아요. 처리 기록과 실제
메타데이터를 확인하고, 막는 표시를 임의로 지우거나 강제로 재개하지 않아요.

이후에는 `dstack-develop`·`dstack-verify`의 기존 검사를 이어가요. 구현 완료 기록만으로
요구사항 검증까지 통과했다고 판단하지 않고, 실패한 시도·남은 검사·근거 위치를 함께 넘겨요.
일반 `dstack run adopt --refresh-mode`는 기존처럼 프로젝트 설정을 다시 받아오는 명령이에요.
이 인계 절차는 기존 Goal에 적용하고 quick 작업에는 적용하지 않아요.

## 작업 흐름

| 단계 | 하는 일 | 쓰는 것 |
|---|---|---|
| 갈래 정하기 | 열린 Goal에 합칠지, 새 Goal을 열지, 빠른 작업으로 갈지 | `dstack status`, 스킬 `dstack-workflow` |
| 요청서 | R 행(한 줄 + 관찰 가능한 기준)을 번호 붙여 적고 승인해요 | `dstack req add`, `dstack request approve` |
| 조사와 인터뷰 | 메인 환경의 recon 에이전트가 코드를 읽고, 질문은 대장으로 관리해요 | `dstack ask add|answer|assume` |
| 계획 | Milestone → Plan → Task를 등록하고 파도 단위로 돌려요 | `dstack plan add`, `dstack next`, 스킬 `dstack-develop` |
| 구현 | Plan마다 메인 환경의 워커가 빈 맥락으로 dstack이 만든 worktree에서 일해요 | `dstack plan start --worktree`, `dstack worker report` |
| 리뷰 | Plan이 끝날 때마다 선택한 서브가 요청서 원문과 diff를 함께 봐요 | `dstack review --scope plan`, `dstack mode exec`, 스킬 `codex-review` |
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
| `external_research` | none, one-pass | 선택한 서브의 리서치 1회 + 감사 1회 |
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

- **판정은 명령이 계산해요.** 체크박스를 에이전트가 직접 켜는 일은 없어요. 증거 행은
  `dstack evidence add`만 쓰고, 산출물의 sha256을 `dstack verify`가 다시 계산해요.
  Codex 메인도 요청·증거·한국어 검사와 `dstack gate`를 직접 실행해요. Claude 훅이 없어도
  같은 완료 조건을 지켜요.
- **말 없는 통과는 없어요.** 모든 검사 명령이 무엇을 몇 개 세었는지 출력하고, 검사기마다 붙박이
  예제(`claude/lint/fixtures/`)를 둬서 `dstack doctor --self`가 "잡아야 할 것을 잡는지" 확인해요.
- **훅은 판정을 못 내리면 막아요.** 훅 스크립트는 `claude/hooks/dstack-hook.sh` 하나예요. 이 스크립트는
  바이너리를 찾아 넘겨주기만 하고, 찾지 못하면 exit 2로 끝나요. 반복해서 막힐 때의 탈출구는
  `dstack run pause`예요.
- **서브에이전트는 Fable로 띄우지 않아요.** 에이전트 머리말의 `model`과, model이 빠진 Agent 호출을
  opus로 바꿔 넣는 Claude 훅이 두 겹으로 지켜요. Codex의 기본 워커는 메인 설정을 물려받고,
  별도 검토 세션은 `dstack mode exec`가 환경별 모델과 high 강도를 고정해요.
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
                            dstack-handoff, codex-review, codex-research, unit-test
claude/agents/              frontend-dev, general-dev(opus) · recon, e2e-runner, ko-polish(sonnet)
claude/runtime.md           양쪽 환경이 함께 읽는 실행·위임·검사 지침
claude/templates/request/   작업 종류별 요청서 틀
claude/lint/                한국어 규칙표(ko-rules.tsv), 범위표(ko-scope.tsv), 붙박이 예제
claude/settings.enforced.json, claude/settings/model-policy.json
                            settings.json에 병합되는 강제 키 (훅, 모델, 출력 스타일)
codex/                      Codex 메인 진입 지침과 양쪽에 설치하는 리뷰·리서치 역할 스킬
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

`cargo test r02_main_runtime`은 두 메인 지침의 실행 경로를 확인하고,
`cargo test r05_mode_install`은 임시 홈에서 설치 링크·반복 설치·개인 설정 보존과 네 조합을
검사해요. 실제 홈에 설치하지 않아요. 실제 모델 세션을 실행하지 못한 검증은 환경별로
`skipped: <사유>`를 남겨요. 설치 테스트나 실행 인자 확인은 실제 모델 실행 기록을 대신하지 않아요.

`cargo test r10_handoff_workflow`는 임시 홈의 양쪽 인계 스킬 설치·반복 설치와 역할 출력을
확인해요. 인계 검증에는 CLI의 stdout·stderr·종료 코드를 남기고, 실제 Claude/Codex 모델
검증은 환경별 실행 기록이나 `skipped: <사유>`를 별도로 남겨요. 전체 회귀 검사는
`bash dstack-cli/test.sh`와 `dstack doctor --self`로 확인해요.

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

### 프롬프트 재사용과 캐시 사용량

리뷰·리서치·감사는 `dstack mode exec`가 내부에서 `dstack prompt render`로 지시를 만들어요.
인계 요약은 `dstack handoff`가 `--role handoff`로 같은 경계를 지켜요.
구현 작업 지시는 `dstack prompt render`를 직접 사용해요. 역할 지침 원문을
앞에 고정하고, 회차·경로·작업 내용은 뒤에 붙여요. 요청서의 한국어 원문도 그대로 보존해요.
Codex 호출은 JSON 로그를 남기고, `dstack exec`가 `usage.json`에 캐시 읽기·쓰기 토큰과
전체 입력 대비 읽기 비율을 기록해요. Claude의 CLI 결과 로그도 같은 기준으로 집계해요.
사용량을 받지 못하면 측정 생략으로 표시해요.

```bash
dstack prompt render --role review --context context.md > prompt.txt
dstack prompt usage --provider codex <실행-로그-경로>/out.txt
```

실제 적중 여부는 클라이언트의 캐시 경계·유효시간·요청 구성에도 달려 있어요.
[적용 범위와 측정 방법](claude/prompt-caching.md)을 확인해 주세요.
