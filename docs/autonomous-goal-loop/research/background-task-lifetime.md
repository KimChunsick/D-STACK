## Needed info

- Public docs say `run_in_background: true` starts a Bash command as a background task while Claude continues; `/tasks` can list/stop it. No public docs I found state a maximum lifetime for an interactive background Bash task. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/tools-reference
- A normal Bash command has a 2-minute default timeout and Claude can request up to 10 minutes, configurable with `BASH_DEFAULT_TIMEOUT_MS` / `BASH_MAX_TIMEOUT_MS`; if it times out, Claude Code moves it to the background and lets it run to completion. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/tools-reference
- Documented early-ending conditions exist: `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` disables background tasks; on macOS/Linux, Claude Code may terminate a main-session background shell on OS memory pressure once the session has been idle for 30 minutes and no turn/subagent is running. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/env-vars
- In `claude -p` non-interactive mode, a background Bash shell is terminated about 5 seconds after Claude returns the final result and stdin closes; SIGTERM terminates the process tree of any running Bash command. This is explicitly not the same as the interactive TUI case in the brief. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/headless
- `--resume` / `--continue` restore conversation, model, agent, permission mode, active goal, and unexpired scheduled tasks, but explicitly do not restore Background Bash or Monitor tasks. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/sessions
- `/branch` does preserve in-flight background Bash/subagents in the same running process, and their output appears in the new branch. This is evidence for same-process continuity, but not evidence about auto-compact. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/sessions
- Monitor is documented as running a command in the background, feeding each output line back to Claude, and letting Claude interject when events arrive. It stops when canceled or when the session ends. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/tools-reference
- Monitor WebSocket watches end on socket close or messages larger than 1 MiB; `timeout_ms` and `persistent` control deadline behavior, and `TaskStop` cancels early. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/tools-reference
- Auto-compaction is documented as replacing conversation history with a summary; `PreCompact` and `PostCompact` hooks fire for manual and automatic compaction. These docs do not mention background Bash or Monitor lifecycle. Sources: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/context-window and https://code.claude.com/docs/en/hooks
- `asyncRewake` is documented: a command hook with `asyncRewake: true` runs in the background and wakes Claude on exit code 2; ordinary async hook output waits for the next conversation turn if the session is idle. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/hooks
- Local installed client check: `claude --version` reports `2.1.220 (Claude Code)`. Static strings in the native binary include internal tool/prompt text saying `run_in_background` keeps running across turns and re-invokes on exit, and that the user will be notified when it finishes. Source: primary local artifact, build time observed in binary `2026-07-24T22:17:45Z`, retrieved 2026-07-28; no URL.

## Opposing views

- The strongest counter-argument is that public docs explicitly say resumed sessions do not restore Background Bash or Monitor tasks, so a delivery pipeline cannot treat those tasks as durable across process exit, restart, crash, or explicit resume. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/sessions
- Public docs document several cases where background work can be stopped early: non-interactive exit grace, SIGTERM, memory-pressure reap, disabling background tasks, Monitor session end, Monitor timeout/socket close/large WebSocket frame. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/headless, https://code.claude.com/docs/en/env-vars, https://code.claude.com/docs/en/tools-reference
- Community issue reports show auto-compact behavior has had regressions and operational bugs, so relying on unspoken auto-compact/background-task interactions is risky. Source: secondary/community, published 2026-05-28, retrieved 2026-07-28: https://github.com/anthropics/claude-code/issues/63015
- Community issue reports also show background process multiplication/resource exhaustion concerns in real use. This does not prove current v2.1.220 is unsafe for a 3-25 minute command, but it supports bounding and externalizing long-running pipeline work. Source: secondary/community, published 2025-09-13, retrieved 2026-07-28: https://github.com/anthropics/claude-code/issues/7541

## For the goal

- For a live interactive session that stays in the same process, the goal is plausible: Anthropic docs explicitly support background Bash for long-running dev servers/watch builds and Monitor for long-running scripts, log tails, CI polling, and event streams. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/tools-reference
- The local `2.1.220` binary’s internal text directly matches the measured behavior: background Bash keeps running across turns and re-invokes on exit. That is stronger than hearsay, but weaker than public API documentation. Source: primary local artifact, retrieved 2026-07-28; no URL.
- Same-process continuity is supported indirectly by `/branch` docs: in-flight background Bash keeps running when branching inside the same process. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/sessions
- A 3-25 minute command is below the documented 30-minute idle-plus-memory-pressure reap threshold, though that reap is conditional on OS memory pressure and idle state rather than a general maximum. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/env-vars

## Against the goal

- The exact requirement “completion notification re-invoking the session with no human input” is not publicly documented for background Bash in the docs I found. It is present in the installed client’s internal text, so call it installed-client behavior, not a public documented guarantee. Sources: primary docs, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/tools-reference; primary local artifact, retrieved 2026-07-28; no URL.
- The auto-compact interaction is undocumented: I found docs for compaction and hooks, but no public statement that background Bash/Monitor is killed, preserved, or that pending completion notifications survive compaction. Sources: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/context-window and https://code.claude.com/docs/en/hooks
- For a delivery pipeline, a background Bash task inside an interactive Claude Code process is not a durable job runner. Anthropic’s documented durable alternatives are Routines, Desktop scheduled tasks, GitHub Actions, or external event/polling mechanisms. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/scheduled-tasks and https://code.claude.com/docs/en/routines
- If the requirement is “wake the already-open idle interactive TUI programmatically,” I found no documented mechanism beyond the listed ones: background-task/Monitor notifications, scheduled tasks, Channels, and `asyncRewake`. `claude -p --resume <session-id>` is documented for sending a follow-up prompt from another process, but it is a separate invocation and does not restore background Bash/Monitor tasks after exit. Source: primary, no date, retrieved 2026-07-28: https://code.claude.com/docs/en/headless and https://code.claude.com/docs/en/sessions

## Unverified

- Whether auto-compact preserves a running interactive background Bash task or Monitor.
- Whether a pending background-command completion notification can be lost during auto-compact.
- Any public Anthropic guarantee of background Bash completion re-invocation on exit.
- Any public maximum wall-clock lifetime for an interactive background Bash task. I found documented stop conditions, but no max duration.
- Monitor command-mode default `timeout_ms` / `persistent` schema defaults from public docs. Public docs describe the semantics but not all defaults.
- Whether the installed-client internal text should be treated as contractual API documentation. I would not treat it that way without Anthropic docs or release notes saying so.

## Sources

- https://code.claude.com/docs/en/tools-reference — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/sessions — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/context-window — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/hooks — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/env-vars — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/headless — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/scheduled-tasks — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/channels — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/routines — primary Anthropic Claude Code docs, no date, retrieved 2026-07-28.
- https://code.claude.com/docs/en/changelog — primary Anthropic Claude Code changelog, latest cited release 2.1.220 dated 2026-07-25, retrieved 2026-07-28.
- https://github.com/anthropics/claude-code/issues/63015 — secondary/community GitHub issue, published 2026-05-28, retrieved 2026-07-28.
- https://github.com/anthropics/claude-code/issues/7541 — secondary/community GitHub issue, published 2025-09-13, retrieved 2026-07-28.
- Local installed CLI `$HOME/.local/share/claude/versions/2.1.220` and `$HOME/.local/bin/claude --version` — primary installed artifact, no URL, retrieved 2026-07-28.