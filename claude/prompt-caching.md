# Prompt reuse and cache measurement

D-STACK controls prompt bytes, role selection and CLI invocation flags. The provider/client
controls cache breakpoints, routing, expiry and hidden context. A stable prefix improves the
conditions for reuse; it does not prove a hit or guarantee reuse between fresh sessions.

## Prompt construction

`dstack prompt render --role review|research|audit|worker --context <file>` prints:

1. A fixed wrapper and the canonical role source, verbatim.
2. A fixed task boundary, then mode and the task-context file, verbatim.

Review uses `codex/skills/dstack-reviewer/SKILL.md`; research and audit share
`codex/skills/dstack-researcher/SKILL.md`; Claude implementation briefs use
`claude/templates/prompts/worker.md` alongside the selected agent's existing definition.
Only the second part contains run ids, round numbers, output paths, timestamps and fresh state.
Keep repeated project/frozen context before changing questions, plans and diffs where possible.
Do not translate Korean request rows or reorder evidence to manufacture matching bytes.

The renderer sends the prefix SHA-256 and UTF-8 byte count to stderr, never to the model.
These identify accidental prefix drift, not cached tokens. Source changes intentionally change
the next prefix; no local cache can serve a stale policy. Missing/empty inputs fail before
printing a prompt. Render successfully before starting the model; send the file through stdin
without command substitution so trailing newlines survive and shell argument limits do not apply.

For a given role, keep model, effort, tool schemas/order, environment and worktree stable when
correct. Do not load all optional tools/skills just to make a longer prefix. Do not rewrite
global instructions with progress state; the existing inject hook appends fresh runtime state.
Maintain chronological conversation history within a task. Compact when necessary for quality,
and expect cache rebuilding. Keep reviewer rounds and the research audit independent: never
resume another pass merely to get a better cache rate. No padding, dummy warmup or keepalive.

## Usage

Codex invocations in the skills use `--json`; the `-o` artifact remains plain model output.
`dstack exec` saves `usage.json` next to stdout/stderr for executables named `codex` or `claude`,
including absolute executable paths. Claude CLI capture requires `--print --output-format
stream-json --verbose` (or JSON output). Interactive Claude agents do not go through this runner:
use Claude Code's `/usage` for the main session, with its documented subagent exclusions.
No transcript discovery or user-log scanning is performed.

To aggregate captured invocations, pass each log once, separately per provider and role/model:

```bash
dstack prompt usage --provider codex <capture-1>/out.txt <capture-2>/out.txt
dstack prompt usage --provider claude <capture-1>/out.txt <capture-2>/out.txt
```

The parser accepts JSONL completion summaries (one JSON result line also works), not arbitrary
API responses or Claude transcript files. It counts Codex `turn.completed.usage` events and only
Claude `result.usage`, ignoring assistant/stream events that duplicate those totals. Claude
`result.usage` covers the main agent loop and excludes subagents; this is not whole-tree
accounting (`modelUsage` has that different scope). Each Claude
file must contain one invocation result. Duplicate file paths, malformed summaries, truncated
runs without completion and invalid counts return explicit `skipped` errors, not zero hits.
Telemetry failure never changes the child process's exit status.

| Field | Meaning |
|---|---|
| `input_tokens` | Codex's total input; Claude's uncached input + cache reads + cache creation |
| `cache_read_tokens` | Codex `cached_input_tokens`; Claude `cache_read_input_tokens` |
| `cache_write_tokens` | Codex `cache_write_input_tokens` if reported; Claude `cache_creation_input_tokens`; null when unavailable |
| `cache_read_ratio` | Sum of cache reads / sum of input tokens, null for no input; not an average of percentages |
| `samples` | Completion summaries counted, not API request count |

Compare representative runs with the same provider, model, role, tools and comparable task sizes.
Record input/read/write/output totals alongside elapsed time from the exec capture, distinguishing
cold starts from subsequent calls. Completion summaries can include several model requests; this
metric does not isolate cross-session reuse or demonstrate causality. Cost needs the applicable
provider/model pricing and cache lifetime; a high hit ratio alone is not a cost improvement.

## Provider boundary (checked 2026-09-06)

OpenAI GPT-5.6+ caching requires an eligible breakpoint as well as an exact prefix: the default
implicit breakpoint at the latest eligible user/tool message is not a promise to cache any
arbitrary shared substring. Direct API integrations can place explicit breakpoints at the end
of stable content and use a stable `prompt_cache_key`. Claude's direct API supports automatic
or explicit `cache_control`; explicit blocks can end at stable content before variable data.
These APIs have distinct minimum lengths, lifetimes and write costs. Never send their API
fields as unverified Codex/Claude CLI config keys or paste them into instructions expecting
transport behavior. This implementation does not change the clients' transport, hidden prompt,
cache key or breakpoint configuration, and cannot share caches between vendors/models.

Sources:
- [OpenAI prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)
- [Codex completion event schema](https://github.com/openai/codex/blob/main/codex-rs/exec/src/exec_events.rs)
- [Claude prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching)
- [Claude SDK cost tracking](https://code.claude.com/docs/en/agent-sdk/cost-tracking)
- [Claude Code usage diagnostics](https://code.claude.com/docs/en/costs)
