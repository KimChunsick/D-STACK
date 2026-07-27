## Needed info

Citations use source IDs expanded in `## Sources` with URL, publication date, retrieval date, and primary/secondary status.

- Production prior art does not treat “start a background process and hope the agent notices” as the core abstraction. Temporal, Restate, AWS Step Functions, and Inngest all model the wait as durable state plus an explicit external signal/callback/event that resumes the workflow. [S1][S2][S3][S4][S5]
- Temporal’s relevant primitives are Workflow event history, Signals, Updates, `Workflow.await`, durable timers, and idempotent/validated message handling. The key property is replay from persisted history after crashes. [S1][S2]
- AWS Step Functions’ relevant primitive is `.waitForTaskToken`: the workflow pauses until an external caller returns the task token via `SendTaskSuccess` or `SendTaskFailure`; the wait can last up to the execution quota, commonly one year. [S3]
- Restate’s relevant primitives are Awakeables and Durable Promises; handlers can suspend, survive process failure, and later resume when an ID or named promise is resolved/rejected, including via HTTP API. [S4]
- Inngest’s relevant primitives are `step.run()` checkpoints, `step.sleep()`, and `step.waitForEvent()`; `waitForEvent` is explicitly recommended for human-in-the-loop or external-system waits, with timeout returning `null`. [S5][S6][S7]
- Event-driven completion beats polling when the external job can emit a callback/webhook/event: Inngest argues events fan out, decouple producer/consumer code, and create audit trails; Claude Code’s own docs position Channels for pushed events and scheduled tasks for polling. [S5][S15][S11]
- A sentinel file is only a reliable completion signal if a registered watcher/monitor is itself durable enough, has a timeout longer than the job, and records correlation IDs/idempotency keys; otherwise it is just polling with race and crash gaps. This is an inference from the durable-engine docs, not a directly documented “sentinel file” pattern. [S4][S5][S6]
- Claude Code documented primitives relevant to this harness: Bash background commands via `run_in_background: true`; Monitor for background scripts whose output lines become events; Stop hooks with `background_tasks` and `session_crons`; `/goal`; `/loop`/Cron; Channels; `claude -p`; `--resume`/`--continue`; Agent SDK; and `--max-turns`/`--max-budget-usd`. [S8][S9][S10][S11][S12][S13][S14][S15]
- Claude Code Stop hooks receive `stop_hook_active`, `last_assistant_message`, `background_tasks`, and `session_crons`; Claude Code ends the turn after 8 consecutive Stop-hook continuations. That cap means Stop hooks should gate finite checks, not implement an unbounded wait loop. [S9]
- Claude Code async hooks do not normally wake an idle session: output is delivered on the next conversation turn. The documented exception is `asyncRewake`, where an async hook exiting with code 2 wakes Claude immediately while idle. [S9]
- Claude Code scheduled tasks are session-scoped; `/loop` can poll or dynamically schedule follow-up prompts, but recurring tasks expire after 7 days, missed fires do not catch up, and background Bash/Monitor tasks are never restored on `--resume`/`--continue`. [S11]
- Claude Code `claude -p` is a plausible alternative host for an unattended loop because it is scriptable, resumable by session ID, supports structured output, `--max-turns`, `--max-budget-usd`, explicit allowed tools, and SDK packages. But in `-p`, background Bash tasks are terminated about five seconds after the final result and stdin close; background subagents/workflows wait with a default 10-minute cap unless configured. [S8][S13][S14]
- I could not find official Claude Code documentation for a general “inject a prompt into any idle local interactive session” API outside documented mechanisms such as Channels, scheduled tasks, background session management, Remote Control, `--resume`/`--continue`, Monitor, and `asyncRewake`. [S8][S11][S13][S15][S16]

## Opposing views

- The strongest counter-argument is that the desired behavior is not “unattended” in the workflow-engine sense unless resumption state is durable and idempotent. Claude Code background Bash/Monitor tasks are explicitly not restored on resume, so a crash/restart gap can lose the wake path. [S11]
- A second counter-argument is cost control: Claude Code background sessions run without a terminal, multiple sessions consume quota independently, agent-view summaries are billed, and non-interactive runs need explicit `--max-budget-usd`/`--max-turns` ceilings to prevent runaway spend. [S16][S8][S13]
- A third counter-argument is safety/audit: agentic systems with tool access and irreversible side effects need guardrails, verification, and oversight; formal-verification commentary in CACM frames autonomous tool-using agents as risky when objectives or checks are underspecified. [S21]
- A fourth counter-argument is quality: LLMs still fail instruction following under long/complex constraints. LIFEBench reports sharp degradation on long length constraints; OpenAI’s instruction-hierarchy work states safety/reliability failures can arise when models follow the wrong instruction source; the 2024 survey treats instruction following as still containing unresolved challenges. [S17][S18][S19]
- A fifth counter-argument is evaluator reliability: if the Stop hook or `/goal` condition relies on model judgment rather than deterministic checks, RubricEval reports substantial variance and weak hard-subset accuracy even for advanced LLM judges. [S20]
- Counter-argument to the counter-argument: these concerns argue against unbounded autonomy, not against automatic wake-up between already-approved review/research rounds. Durable engines and Claude Code both support bounded, observable, resumable automation when budgets, timeouts, state, and kill switches are explicit. [S3][S4][S5][S8][S11][S13]

## For the goal

- The goal is achievable in principle: every major workflow system examined has a first-class “pause for external completion, then resume” abstraction, and Claude Code itself has multiple wake primitives. [S2][S3][S4][S5][S9][S10][S11]
- The maintainer’s measured local facts fit the documented shape: Bash background tasks, Monitor events, and `asyncRewake`/scheduled/event mechanisms are the right class of primitive; detached POSIX processes are the wrong class because they survive but are invisible to the harness. [S9][S10][S11]
- The sound design is an atomic launcher: one scripted action should create a run ID, persist state, start the external `codex exec`, arm a watcher with a timeout longer than 25 minutes, and emit one correlated completion event. That reduces the late-step prose failure where the model remembers to launch but forgets or misconfigures the watcher. This is an inference from durable-step/checkpoint patterns. [S4][S5][S6][S7]
- Claude Code `/goal` supports “keep working across turns until condition holds,” but it is model-evaluated; for this pipeline, deterministic Stop-hook checks over task/work documents are more defensible for completion gating. [S12][S9][S20]
- Responsible autonomy baseline: hard dollar cap, max turns, max rounds, per-external-job timeout, watcher timeout, run-state file/database, idempotency key, heartbeat, append-only logs, and a manual kill command. Claude Code and the workflow engines each expose parts of that pattern. [S3][S4][S5][S6][S8][S11][S13]
- Public benchmark evidence shows autonomous coding agents can complete nontrivial tasks without step-by-step human input, but success is partial rather than guaranteed; SWE-bench Verified leaderboard rows around 2025 show top systems resolving roughly 50-60% of tasks, depending on system/date. [S22]

## Against the goal

- The current contract is structurally brittle if it depends on a multi-hundred-line procedural instruction ending with “arm watcher, end turn.” Current research and benchmarks support the maintainer’s observed failure mode: models can omit or misapply late constraints in long procedural contexts. [S17][S18][S19]
- A plain async hook is insufficient for no-human resumption because Claude Code says idle delivery waits until the next user interaction unless `asyncRewake` exits with code 2. [S9]
- A Stop hook cannot be the sole long-wait mechanism because Claude Code caps consecutive Stop-hook continuations at 8. [S9]
- A `claude -p` host cannot rely on long-lived background Bash tasks after exit because Claude Code terminates those shortly after the final result; the host must either keep the process running, use SDK control, or externalize orchestration. [S13]
- Claude Code session-scoped cron/loop is useful for polling, but not crash-durable in the same way as Temporal/Restate/Inngest/Step Functions; background Bash and Monitor tasks are never restored on resume. [S11]
- If the pipeline removes human typing but also removes human review/approval checkpoints, the evidence weakens: agentic systems can compound errors, judges can mis-evaluate completion, and tool-use autonomy raises audit/safety issues. [S20][S21]

## Unverified

- I did not independently verify the maintainer’s local measured facts on Claude Code CLI 2.1.220; they are treated as given, per brief.
- I could not verify a current official Claude Code document stating a default 5-minute Monitor timeout. The docs I found describe `timeout_ms`/`persistent` behavior but not that default. [S10]
- I could not verify a documented maximum lifetime for interactive Claude Code background Bash or Monitor tasks across ordinary turn boundaries, except for `claude -p` exit behavior and resume limitations. [S11][S13]
- I could not verify that context compaction kills or preserves background Bash/Monitor tasks. Claude Code documents `PreCompact`/`PostCompact` hooks, but I found no explicit background-task lifetime rule tied to compaction. [S9]
- I could not verify strong empirical evidence that human checkpoints materially improve general autonomous coding-agent outcomes across tasks; the stronger evidence available here is indirect: safety literature argues for oversight, and judge/instruction-following benchmarks show failure modes. [S18][S20][S21]

## Sources

| ID | Source | Type | Publication date | Retrieved |
|---|---|---:|---|---|
| S1 | https://docs.temporal.io/ | Primary | no date | 2026-07-27 |
| S2 | https://docs.temporal.io/develop/java/workflows/message-passing | Primary | no date | 2026-07-27 |
| S3 | https://docs.aws.amazon.com/step-functions/latest/dg/connect-to-resource.html | Primary | no date | 2026-07-27 |
| S4 | https://docs.restate.dev/develop/go/external-events | Primary | no date | 2026-07-27 |
| S5 | https://www.inngest.com/docs/features/inngest-functions/steps-workflows/wait-for-event | Primary | no date | 2026-07-27 |
| S6 | https://www.inngest.com/docs/guides/handling-idempotency | Primary | no date | 2026-07-27 |
| S7 | https://www.inngest.com/docs/learn/durable-agents | Primary | no date | 2026-07-27 |
| S8 | https://code.claude.com/docs/en/cli-usage | Primary | no date | 2026-07-27 |
| S9 | https://code.claude.com/docs/en/hooks | Primary | no date | 2026-07-27 |
| S10 | https://code.claude.com/docs/en/tools-reference | Primary | no date | 2026-07-27 |
| S11 | https://code.claude.com/docs/en/scheduled-tasks | Primary | no date | 2026-07-27 |
| S12 | https://code.claude.com/docs/en/goal | Primary | no date | 2026-07-27 |
| S13 | https://code.claude.com/docs/en/headless | Primary | no date | 2026-07-27 |
| S14 | https://code.claude.com/docs/en/agent-sdk/overview | Primary | no date | 2026-07-27 |
| S15 | https://code.claude.com/docs/en/channels | Primary | no date | 2026-07-27 |
| S16 | https://code.claude.com/docs/en/agent-view | Primary | no date | 2026-07-27 |
| S17 | https://openai.com/index/instruction-hierarchy-challenge/ | Primary | 2026-03-10 | 2026-07-27 |
| S18 | https://proceedings.neurips.cc/paper_files/paper/2025/hash/c84f00ccee3d35cee1901acb0e258dc7-Abstract-Datasets_and_Benchmarks_Track.html | Primary | 2025 | 2026-07-27 |
| S19 | https://direct.mit.edu/coli/article/50/3/1053/121669/Large-Language-Model-Instruction-Following-A | Primary | 2024-09-01 | 2026-07-27 |
| S20 | https://huggingface.co/papers/2603.25133 | Secondary index of paper | 2026-03-26 | 2026-07-27 |
| S21 | https://doi.org/10.1145/3777544 | Primary | 2025-12-16 | 2026-07-27 |
| S22 | https://swe-agent-bench.github.io/ | Secondary benchmark leaderboard | no date | 2026-07-27 |