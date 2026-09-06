# D-STACK shared main runtime

This is the canonical runtime for both main providers. The installer links this same file,
the workflow skills and native worker definitions into both agent homes. A supplied role
prompt (reviewer, researcher, audit, implementation worker, recon, e2e-runner/verification or
ko-polish) follows that bounded role only;
it must not start this main workflow or delegate another main session.

## Enter from the actual host

The user starts the selected app or CLI. In Claude, run `dstack mode show --host claude`;
in Codex, run `dstack mode show --host codex`. Include `--run <id>` for an explicit Goal or
`--quick <slug>` for a quick task, including on resume. For a new target, inspect project mode
first; after creating it, check its saved mode before any delegation. A host mismatch stops
main work: show the handoff instructions and have the user open the selected environment.
A command cannot change the engine of an already running conversation.

Read `dstack status` at entry and after adopting a run. `dstack-workflow` routes the request;
`dstack-develop` executes Plans; `dstack-verify` records evidence and closes; `dstack-quick`
handles quick tasks. These names select the same installed source in either home.
Configuration changes apply to new runs and quick tasks. An existing run keeps its snapshot
until `dstack run adopt --refresh-mode` explicitly refreshes it. Ordinary adoption preserves it.
After refresh, repeat the host check in the selected new session before continuing work.

## Native implementation, reconnaissance and verification

`main` selects the native worker mechanism below. `sub` is reserved for independent review,
research and audit sessions; it never changes an implementation worker's engine.

| Worker role | Claude main | Codex main |
|---|---|---|
| `recon`, `e2e-runner`, `ko-polish` | native `Agent`, explicit `model: sonnet` | native `spawn_agent`, inherited model/effort |
| `frontend-dev`, `general-dev` | native `Agent`, explicit `model: opus` | native `spawn_agent`, inherited model/effort |
| Other bounded delegated work | native `Agent`, explicit `model: opus` | native `spawn_agent`, inherited model/effort |

Claude reads the corresponding installed agent definition and uses its native subagent type.
Never pass a full model id, `fable`, `haiku` or `inherit` to Claude's Agent model field. Its
hooks enforce `sonnet`/`opus`; Workflow `agent()` calls also pass that model explicitly.

Codex reads `~/.codex/agents/<role>.md` as a role specification, not as executable frontmatter.
Include its specialization and tool/write restrictions in the bounded task context, then use
`dstack prompt render --role worker --context <context-file>` for implementation briefs.
Native Codex workers use the available `spawn_agent` tool with a fresh bounded context and the
complete rendered brief. When the host schema exposes `fork_turns` and `message`, use
`fork_turns: "none"` and put the brief in `message`; otherwise use that host's documented
equivalent for fresh context. Read the actual tool schema before calling it. Omit model and
effort overrides where inheritance is supported: workers inherit the main session's
configured engine (the D-STACK invocation is `gpt-6-astra` / `high`). Claude model aliases in
the shared definition do not override Codex. Do not claim that inheritance pins an unknown
host: record the observed engine/model/effort; unavailable details are `skipped: unavailable`.

For recon and verification, send the role definition body, the bounded R rows and the required
output/artifact contract as the fresh brief. A worker never asks the user a question or writes
pipeline state. Return ambiguity to the main session. Reports and prompt context stay English;
quoted request rows remain Korean. Read the installed Korean output style before user prose.

The main session runs `dstack next --max 3` and `dstack plan start P<n> --worktree <path>`;
only the CLI creates worktrees and records them. Supply the actual cwd, common-dir, branch,
HEAD, declared files, R rows and artifact path. Never enable native worktree isolation.
Workers first run `dstack run verify`, compare those values and stop on a location mismatch.
Spawn only the schedulable disjoint Plans, then wait using the host's native completion tool.
Claude may launch its Agent calls as one wave; Codex uses bounded `spawn_agent` calls and
the host's native completion/wait tool (`wait_agent` where available). Both mechanisms must
return the worker report before evidence is recorded. If the host has no native delegation,
report that capability as blocked; do not claim a worker ran or turn it into a sub review call.

## Main-only questions and state

Only the main session asks questions and operates request, decision, Plan, evidence and review
state through `dstack`. Use Claude's AskUserQuestion or the available Codex input tool; when
that tool is unavailable, ask one concise plain-text question. Keep the same question ledger
and approval semantics. Existing user authorization persists; do not ask for it again.
No agent hand-edits machine state under `.dstack/`. Workers write only declared source files
and their named artifact directory; the main session records their evidence with the CLI.
An explicitly designated prose artifact follows its workflow's writer contract.

## Checks apply in both hosts

Claude hooks are an additional enforcement layer. Codex has no dependency on those hooks:
the main session runs `dstack check request` before planning; after worker results it runs
the required repository tests and `dstack lint-ko --changed`, records evidence, then runs
`dstack check coverage`, `dstack check decisions`, `dstack verify` and `dstack gate` before
reporting completion. The same checks also apply when Claude hooks are absent. Run
`dstack gate` before ending a work turn; a nonzero result reports the outstanding work rather
than claiming completion. Checks for a quick target carry `--quick <slug>` where supported;
`dstack gate` checks CURRENT and every open quick task in the current worktree.
Every required but unexecuted step is `skipped: <reason>`, never a pass. No live model calls
are implied by a passing instruction/installer test: record each host's actual execution
transcript separately, including observed engine, native delegation and resulting checks.

## Independent sub sessions

The legacy skill names `codex-review` and `codex-research` remain compatible entry points.
They select the target snapshot's `sub` through `dstack mode exec`, which internally renders
the canonical role prompt and starts a fresh read-only provider session. Even when main and
sub are identical, never review or audit in the main conversation or resume the previous pass.
The CLI fixes provider model/effort, finite prompt input, logs, usage and structured completion;
it writes the final output only after success. Missing provider or failed completion stops
that pass with its actual error. There is no implicit fallback to another provider.
Codex sub uses `gpt-6-astra` with `model_reasoning_effort=high`; Claude sub uses `opus` with
`--effort high`. Captures live under `.dstack/local/exec/<label>/`; completed retries receive
numbered suffixes. Check the recorded output, error, exit and usage for the specific attempt.

```bash
dstack mode exec review-P1-001 --role review --context <context-file> --output <raw-file> --worktree <plan-worktree>
dstack mode exec research-001 --role research --context <context-file> --output <pass-file> --run <run-id>
dstack mode exec research-audit-001 --role audit --context <context-file> --output <audit-file> --quick <slug>
```

Use `--dry-run` to inspect provider, role, model, argv, cwd and output without launching or
writing. It is configuration evidence only. Keep legacy `codex-review-NNN.md` sealed names and
the per-R verdict, finding axes, claim table and audit contracts regardless of selected sub.
For long runs, Claude uses one background Bash call and resumes from its completion event;
Codex uses its terminal's yielded session and completion/wait tool. Do not detach the process,
poll in a tight loop, or represent unfinished work as a final result.
