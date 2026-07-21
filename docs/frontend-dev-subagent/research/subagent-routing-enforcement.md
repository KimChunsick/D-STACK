## Needed info

- Subagent packaging is currently Markdown plus YAML frontmatter. User-wide agents live in `~/.claude/agents/`; project agents live in `.claude/agents/`; both are still supported, and the docs also list managed settings, `--agents`, and plugin `agents/` as other scopes. Project agents are shareable and discovered up the project tree; user agents are personal across projects. [S1]
- Current required frontmatter fields are only `name` and `description`. Current optional fields include `tools`, `disallowedTools`, `model`, `permissionMode`, `maxTurns`, `skills`, `mcpServers`, `hooks`, `memory`, `background`, `effort`, `isolation`, `color`, and `initialPrompt`. [S1]
- Identity comes from the `name` field, not the filename. Claude Code docs explicitly say hooks receive that value as `agent_type`. [S1]
- Auto-delegation is not a hard guarantee. Claude delegates based on the user request, the subagent `description`, and context; docs say phrases like “use proactively” encourage delegation. The guaranteed one-task routing mechanism is `@agent-<name>` / agent mention, and session-wide routing is `claude --agent <name>` or the `agent` setting. [S1]
- Subagents start with fresh isolated context. They do not see full conversation history, previously-read files, or invoked skills; the parent composes a task/delegation prompt. Non-Explore/Plan custom subagents also receive loaded `CLAUDE.md`/memory, startup git status, and preloaded skills. [S1]
- A current hook input includes common fields such as `session_id`, `prompt_id`, `transcript_path`, `cwd`, `permission_mode`, `effort`, and `hook_event_name` when applicable. When a hook fires inside a subagent, current docs add `agent_id` and `agent_type`; docs explicitly say `agent_id` can distinguish subagent hook calls from main-thread calls. [S2]
- `PreToolUse` receives `tool_name`, `tool_input`, and `tool_use_id`. For `Write` and `Edit`, `tool_input.file_path` is the absolute target path, which is enough for extension/path-based frontend heuristics. [S2]
- `PreToolUse` can deny before execution using `hookSpecificOutput.permissionDecision: "deny"` or exit code `2`; if multiple hooks disagree, `deny` wins over `defer`, `ask`, and `allow`. Permission deny/ask rules are still evaluated even if a hook returns allow. [S2, S4]
- `PreToolUse` fires only for Claude tool calls. It does not fire for `@file` prompt references, and file edits performed indirectly through Bash subprocesses require Bash inspection, permission rules, or sandboxing to cover. [S2, S4, S5]
- Subagent frontmatter can define hooks scoped to that subagent lifecycle; docs say all hook events are supported and `Stop` becomes `SubagentStop`. Plugin subagents ignore `hooks`, `mcpServers`, and `permissionMode`, so use user/project agents for this setup. [S1, S2]
- Settings hooks can observe `SubagentStart` and `SubagentStop` by agent type, but `SubagentStart` cannot block a spawn; exit code `2` only surfaces an error notice. Blocking a subagent type should use `permissions.deny` with `Agent(name)`. [S1, S2, S4]
- There is no documented per-agent environment variable for authorization. The documented agent identity channel is hook JSON `agent_id`/`agent_type`; docs note no `$CLAUDE_MODEL`, and hook commands inherit the parent environment rather than receiving model-specific environment changes. [S2]
- `disableAllHooks` disables all hooks; docs say there is no way to disable one individual hook while keeping it configured. I found no documented per-agent override that disables user/project settings hooks for a subagent. [S2, S3]

## Opposing views

- Instruction-only routing is weak as enforcement. Anthropic’s permission docs state that prompts and `CLAUDE.md` shape what Claude tries, but do not change what Claude Code allows; permissions and hooks enforce boundaries. [S4]
- Permission deny rules alone are a bad fit for “deny main, allow frontend-dev” because deny rules are evaluated before allow rules and have no allowlist exception. A broad `Edit(/src/**/*.tsx)` deny would also block the subagent unless the subagent is outside that permission scope. [S4]
- Auto-delegation phrasing like “MUST BE USED” or “use proactively” should be treated as steering, not enforcement. The docs use “encourage proactive delegation” and reserve “guarantees” for explicit `@` mention. [S1]
- Hard frontend file patterns are lossy. `Write/Edit` hooks see file paths, not semantic ownership; `.md` docs can contain frontend snippets, `.ts` can be backend or frontend, and monorepos may mix UI and non-UI code under shared paths. This is an inference from the documented `file_path`-based hook input. [S2]
- Bash is a bypass path if enforcement only matches `Write|Edit`. Anthropic says Read/Edit rules do not apply to arbitrary subprocesses that read/write files indirectly, and sandboxing is the OS-level layer for Bash child processes. [S4, S5]
- Recent changelog entries show agent/hook behavior has been moving quickly: hyphenated hook matchers were fixed June 26, 2026; `Agent(type)` deny enforcement was fixed June 22, 2026; `SubagentStart` hook stderr visibility was fixed July 2, 2026. That argues for version pinning and a small diagnostic hook before relying on this mechanically. [S6]

## For the goal

- The goal is technically achievable on current docs: a user/project `PreToolUse` hook can match `Edit|Write`, inspect `tool_input.file_path`, and deny frontend paths unless `agent_type == "frontend-dev"`; main-thread calls lack subagent `agent_id` and should be blocked. [S2]
- The model separation is aligned with Claude Code’s intended use: docs list subagents as a way to preserve context, enforce constraints through tool restrictions/permissions, reuse configurations, and specialize behavior. [S1]
- The desired “frontend-dev owns implementation” boundary can be strengthened by giving `frontend-dev` explicit `tools` and/or `permissionMode`, while leaving the main loop to call the `Agent` tool for frontend tasks. [S1]
- Hooks can be scoped inside the subagent too, so `frontend-dev` can have its own validation hooks, lint hooks, or narrower write checks without affecting the main loop. [S1, S2]
- Changelog support for `agent_id`/subagent lifecycle, background permission surfacing, and subagent partial-result fixes suggests Anthropic is actively hardening subagent workflows in 2026. [S6]

## Against the goal

- “ALL frontend implementation work” is hard to define mechanically. Extension-based rules catch `.tsx/.jsx/.css/.scss`, but miss frontend logic in `.ts`, config files, tests, generated code, story files, route files, and mixed utilities unless the path heuristic is broad. Broad heuristics increase false positives. [S2]
- A naive hook can deadlock the workflow by blocking the `frontend-dev` subagent too. The hook must explicitly allow `agent_type == "frontend-dev"` and should log unknown/missing agent fields during rollout. [S2]
- A marker-file or environment “allow token” handshake is inferior to `agent_type`: environment changes do not propagate from a subprocess to the parent, marker files are race-prone with background/nested subagents, and the main agent could create the marker unless separately blocked. This is an inference from documented hook process/environment behavior and subagent concurrency/background behavior. [S1, S2]
- Renames, deletes, and generated edits through Bash are not covered by a `Write|Edit` hook. You would need a `Bash` PreToolUse check for write-capable commands, `Edit` deny rules/sandbox denyWrite, or post-change detection. [S4, S5]
- Subagents add latency and context transfer risk. Anthropic explicitly recommends the main conversation for frequent back-and-forth, shared context across phases, quick targeted changes, and latency-sensitive work. [S1]
- Current public docs and changelog appear slightly inconsistent: subagent docs mention “as of v2.1.205,” while the changelog page I retrieved tops out at v2.1.202 dated July 6, 2026. Treat exact version availability as something to verify locally with `claude --version`. [S1, S6]

## Unverified

- I did not verify community GitHub issue threads showing hook stdin from real installations; the web evidence I could substantiate here is Anthropic docs/changelog plus secondary research papers.
- I could not verify whether every user/project settings `PreToolUse` hook fires for every subagent tool call in every older Claude Code version. Current docs document `agent_id`/`agent_type` for hooks inside subagents, but a local diagnostic hook should confirm behavior on the installed CLI.
- I could not verify a documented `frontend-dev`-style per-agent override that disables inherited/global hooks. The documented mechanism is to branch inside the hook based on `agent_type`.
- I could not verify the actual latest installed version on the target macOS machine; the public changelog retrieved lists 2.1.202 on July 6, 2026, while docs include references to later 2.1.205 behavior.

## Sources

- [S1] Primary: Anthropic Claude Code docs, “Create custom subagents.” URL: https://code.claude.com/docs/en/sub-agents. Publication date: no date. Retrieved: 2026-07-10.
- [S2] Primary: Anthropic Claude Code docs, “Hooks reference.” URL: https://code.claude.com/docs/en/hooks. Publication date: no date. Retrieved: 2026-07-10.
- [S3] Primary: Anthropic Claude Code docs, “Claude Code settings.” URL: https://code.claude.com/docs/en/settings. Publication date: no date. Retrieved: 2026-07-10.
- [S4] Primary: Anthropic Claude Code docs, “Configure permissions.” URL: https://code.claude.com/docs/en/permissions. Publication date: no date. Retrieved: 2026-07-10.
- [S5] Primary: Anthropic Claude Code docs, “Configure the sandboxed Bash tool.” URL: https://code.claude.com/docs/en/sandboxing. Publication date: no date. Retrieved: 2026-07-10.
- [S6] Primary: Anthropic Claude Code changelog. URL: https://code.claude.com/docs/en/changelog. Publication dates cited: June 22, June 24, June 26, July 1, July 2, July 6, 2026 entries. Retrieved: 2026-07-10.
- [S7] Primary digest: Anthropic Claude Code “What’s new.” URL: https://code.claude.com/docs/en/whats-new. Publication dates cited: Week 24 June 8-12, Week 25 June 15-19, Week 26 June 22-26, 2026. Retrieved: 2026-07-10.
- [S8] Secondary: Galster et al., “Configuring Agentic AI Coding Tools: An Exploratory Study.” URL: https://arxiv.org/abs/2602.14690. Publication date: 2026-02-16. Retrieved: 2026-07-10.
- [S9] Secondary: McMillan, “Instruction Adherence in Coding Agent Configuration Files.” URL: https://arxiv.org/abs/2605.10039. Publication date: 2026-05-11. Retrieved: 2026-07-10.