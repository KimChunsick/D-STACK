## Needed info

- Claude Code `Stop` currently receives `stop_hook_active` and `last_assistant_message`; `Stop` blocks with top-level `{"decision":"block","reason":"..."}`. `PreToolUse` denial should use `hookSpecificOutput.permissionDecision:"deny"`; top-level `decision` for `PreToolUse` is deprecated. [S1]
- Hook matchers can target file-writing tools: exact/list matchers such as `Edit|Write`, regex matchers such as `^Notebook`, and tool events including `PreToolUse` match on `tool_name`. `Stop` has no matcher support and always fires. [S1]
- Claude’s hook docs explicitly warn about process-spawn overhead: omitted/`*` matchers fire every occurrence, while an `if` condition can avoid spawning the hook handler when a tool call does not match. That supports a narrow `Edit|Write|NotebookEdit` hook, but argues against broad hooks. [S1]
- Claude Code has a documented infinite-loop guard: after too many consecutive `Stop` blocks it overrides the hook, and the guide tells hook authors to check `stop_hook_active`. This is directly relevant to the observed stop-spin. [S2]
- Background Bash commands are documented as asynchronous, return a background task ID, write output to a file, and can continue while Claude responds to new prompts. The Agent SDK also exposes `run_in_background` and emits task-completion notifications for background Bash, Monitor, and background subagents. [S3], [S4]
- Claude docs do not document the exact cross-product claim “a `Stop` hook block prevents the background-completion re-invocation path”; what is documented is that `Stop` `decision:"block"` prevents Claude from stopping and continues the conversation. [S1]
- POSIX `open(..., O_CREAT|O_EXCL, ...)` makes name creation atomic, but it does not make the subsequent file contents appear atomically. For a concurrent registry reader, create-final-name-then-write can expose empty or partial content if the reader opens the file before the writer finishes or if the writer dies. [S8]
- POSIX `rename()` is the portable primitive for atomically replacing a directory entry on the same filesystem; atomic-write libraries therefore use same-directory temp files, write/fsync, then rename. [S7], [S11], [S12]
- Directory iteration is not a snapshot: POSIX says if entries are added or removed after `opendir()`/`rewinddir()`, whether later `readdir()` returns them is unspecified. A `.dstack/active/` scanner must tolerate `ENOENT`, stale entries, and disappearing files. [S9]
- APFS supports atomic safe-save primitives and copy-on-write metadata, but Apple also warns that rapid file creation/deletion and atomic writes add filesystem metadata writes. The proposed directory-of-files is sound at this scale, but it is not zero-cost. [S10]
- Git’s own guidance is: shared generated files belong in committed `.gitignore`; local, workflow-specific ignores belong in `.git/info/exclude`; global personal ignores belong in `core.excludesFile`. Auto-editing a repo `.gitignore` should therefore be conditional, not unconditional. [S13], [S14]
- Prior art favors per-project state/cache directories: Terraform uses `.terraform/` for providers/modules/backend metadata and tells users not to commit it; pytest creates `.pytest_cache/` and puts its own internal `.gitignore` plus `CACHEDIR.TAG` inside the cache directory. [S15], [S16], [S17]
- Migration prior art is mixed. Rails uses UTC timestamp migration filenames and historically moved away from sequential integers because multiple developers clashed. Flyway allows integers, dotted versions, and timestamp-like versions, but requires unique versions and marks out-of-order migrations as a state that can make rerunning history produce different results. [S18], [S19]
- Anthropic prompt caching caches full prefixes in `tools -> system -> messages` order. Changing any block at or before a breakpoint changes the cumulative hash; the system does not magically cache stable content behind a varying block unless a prior breakpoint wrote that stable prefix. [S20]
- Prompt-cache cost savings are not simply “characters removed = proportional savings”: 5-minute writes cost 1.25x base input, 1-hour writes cost 2x, cache reads cost 0.1x, and prompts below the model/platform minimum are processed without caching. A stable injected block on cache hits costs mostly cache-read tokens; a changing injected block before the cache point can destroy the hit. [S20]

## Opposing views

- Keep the flat file: for a single maintainer and a few tabs, one registry file plus one well-tested lock is easier to reason about than a mini state database. A flat file gives one snapshot read; a directory set requires list/open race handling, stale cleanup, and per-entry atomic write discipline. [S7], [S9]
- Use existing Claude mechanisms first: Claude Code already has background Bash, `/tasks`, agent view, worktrees, `Stop` block caps, and `stop_hook_active`. A wait-ticket protocol may duplicate platform behavior unless the specific “background completion blocked by Stop” path is reproduced and measured. [S1], [S2], [S3], [S5]
- Prefer worktrees to claim hooks for broad concurrency. Claude’s own parallel-work docs say worktrees isolate file edits so parallel sessions do not edit the same files; a `PreToolUse` claim hook only sees tool calls, not every possible filesystem mutation by external commands. [S5]
- A `PreToolUse` hook on every write is a latency tradeoff. Claude docs recommend matchers/`if` to avoid unnecessary spawns; if migration collisions are rare, manual conflict resolution or a migration allocator command may cost less than gating every edit. [S1]
- PID tickets are a weak liveness primitive. PIDs recycle; stale pidfiles survive crashes/SIGKILL; `kill(pid, 0)` checks existence/permission, not identity. Daemon tooling warns that pidfiles must be updated race-free and not trusted alone. [S21], [S22], [S23]
- Auto-appending `.dstack/` to a tracked `.gitignore` is intrusive in arbitrary target repos. Git’s own split suggests `.git/info/exclude` for local workflow artifacts, while pytest shows an alternative: put `.gitignore` inside the generated cache directory. [S13], [S17]

## For the goal

- The deadlock diagnosis matches documented semantics: if `Stop` returns `decision:"block"`, Claude continues instead of ending. Letting the turn end while a live wait ticket exists is a coherent way to avoid repeated Stop-block turns, provided the hook blocks again after the external run dies without gate completion. [S1], [S2], [S3]
- Replacing generated bash snippets with a `dstack` CLI is sound engineering. It centralizes locking, atomic writes, stale cleanup, JSON parsing, and migrations into testable code instead of requiring a model to reproduce shell logic in a Markdown skill each run. This is an inference from the documented complexity and fragility of hook JSON/stdout/exit semantics. [S1]
- Directory-of-files can eliminate the current registry-wide read-modify-write critical section if each logical record owns a distinct path and record publication uses temp-write-then-rename or write-temp-then-link. That removes stranded `.tmp` and lock-directory failure modes for different keys. [S7], [S8], [S11]
- A `.dstack/` per-repo directory aligns with strong prior art: `.terraform/` and `.pytest_cache/` keep runtime/cache state close to the project, separate from source, and ignored from version control. [S15], [S16], [S17]
- A claim-check hook is feasible at the API level: `PreToolUse` can block tool calls before execution, and matchers can restrict it to `Edit|Write` plus notebook edits. [S1]
- Cutting prompt injection can save money and context, but the best argument is not only raw character count. The stronger case is moving stable pipeline instructions into already-loaded files or stable cache prefixes and keeping volatile per-turn context out of cached prefixes. [S20]

## Against the goal

- The proposed `active/<sha1-of-docpath>` shape may change semantics: the old line registry could represent multiple active documents/session lines, while a single filename per docpath creates last-writer-wins unless `reg` detects an existing path and decides whether that is conflict, idempotence, or multi-session state. This needs an explicit rule.
- “No mutex” is only true for different keys. Same key registration, claim collisions, cleanup of empty session directories, and migration from old dotfiles still need atomic conflict handling. Use `O_EXCL`, hard-link publication, or equivalent no-overwrite semantics where overwrites would be wrong. [S8], [S11]
- The directory store will not give consistent snapshots. `status` and `reclaim` must be designed as eventually consistent sweeps: list entries, open defensively, ignore disappeared files, validate content, and retry only where needed. [S9]
- PID wait tickets can false-allow if a PID is recycled, false-block if a killed process leaves a ticket, and fail across remote/background supervisors. At minimum, tickets should include host, PID, process start time if available, command/session id, and captured run path; better, prefer Claude task IDs or SDK task notifications where available. [S4], [S21], [S22], [S23]
- A file-writing `PreToolUse` hook does not stop writes performed inside `Bash` commands, code generators, package scripts, or migration CLIs. If migration collisions are the motivating case, the stronger control is a migration allocator or a Bash/migration-command policy, not only `Edit`/`Write` interception. [S1], [S5]
- Auto-editing `.gitignore` can create unrelated diffs and team policy churn. A safer default is `.dstack/.gitignore` containing `*` plus optional `.git/info/exclude` insertion, with tracked `.gitignore` edits only after opt-in. [S13], [S17]
- Prompt-cache savings must be measured with usage fields. If the 1,857-character injected block is stable and already cached, deleting it saves mostly 10% cache-read cost on hits; if it changes per prompt or sits before the cache point, deleting/moving it may save much more by restoring cache hits. [S20]
- Migration filenames are not solved by “timestamp” alone. Rails timestamps reduce developer clashes, and Flyway accepts timestamp-like versions, but any ordered migration system can still hit semantic conflicts or out-of-order history; ULID-style names reduce filename collision but do not decide schema dependency order by themselves. [S18], [S19]

## Unverified

- I could not verify from primary docs that a blocked `Stop` hook specifically prevents Claude Code’s background Bash completion from re-invoking the agent. The docs separately confirm Stop blocking and background task notifications, but not their interaction. [S1], [S3], [S4]
- I did not verify the installed local Claude Code version; current docs mention version-gated matcher and background-session behavior, so local behavior may differ. [S1], [S3], [S5]
- I found qualitative hook performance guidance, but no primary benchmark for per-write `PreToolUse` latency. This should be measured in the target setup. [S1]
- I did not fully verify current primary docs for `.ruff_cache`, `.venv`, Liquibase, or Prisma due source retrieval limits; Terraform, pytest, Rails, Flyway, Git, POSIX, Apple, and Anthropic sources were verified.
- I did not test APFS behavior under network folders, cloud-sync folders, or cross-volume temp directories. Atomic rename claims should be limited to same-filesystem local paths unless tested. [S7], [S10], [S11]

## Sources

- [S1] Primary, no date, retrieved 2026-07-27 KST: Claude Code Hooks Reference, https://code.claude.com/docs/en/hooks
- [S2] Primary, no date, retrieved 2026-07-27 KST: Claude Code Hooks Guide, https://code.claude.com/docs/en/hooks-guide
- [S3] Primary, no date, retrieved 2026-07-27 KST: Claude Code Interactive Mode, https://code.claude.com/docs/en/interactive-mode
- [S4] Primary, no date, retrieved 2026-07-27 KST: Claude Agent SDK Python Reference, https://code.claude.com/docs/en/agent-sdk/python
- [S5] Primary, no date, retrieved 2026-07-27 KST: Claude Code Agents and Parallel Work, https://code.claude.com/docs/en/agents and https://code.claude.com/docs/en/agent-view
- [S7] Primary, POSIX.1-2024 page/no page date, retrieved 2026-07-27 KST: `rename`, https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html
- [S8] Primary, POSIX.1-2024 page/no page date, retrieved 2026-07-27 KST: `open`, https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html
- [S9] Primary, no date, retrieved 2026-07-27 KST: POSIX `readdir`, https://pubs.opengroup.org/onlinepubs/007904875/functions/readdir_r.html
- [S10] Primary, no date plus retired APFS guide updated 2018-06-04, retrieved 2026-07-27 KST: Apple APFS and disk-write docs, https://developer.apple.com/documentation/foundation/about-apple-file-system and https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/Features/Features.html
- [S11] Primary library docs, no date, retrieved 2026-07-27 KST: python-atomicwrites, https://python-atomicwrites.readthedocs.io/en/latest/
- [S12] Primary package docs, version 8.0.0 published “2 months ago” on page, retrieved 2026-07-27 KST: write-file-atomic, https://www.npmjs.com/package/write-file-atomic
- [S13] Primary, no date, retrieved 2026-07-27 KST: Git `gitignore` docs, https://git-scm.com/docs/gitignore
- [S14] Secondary/vendor docs, no date, retrieved 2026-07-27 KST: GitHub Ignoring Files, https://docs.github.com/en/get-started/getting-started-with-git/ignoring-files
- [S15] Primary, no date, retrieved 2026-07-27 KST: Terraform Init `.terraform` tutorial, https://developer.hashicorp.com/terraform/tutorials/cli/init
- [S16] Primary, no date, retrieved 2026-07-27 KST: Terraform style/workspaces/state docs, https://developer.hashicorp.com/terraform/language/style and https://developer.hashicorp.com/terraform/cli/workspaces
- [S17] Primary, no date, retrieved 2026-07-27 KST: pytest cache docs/source, https://docs.pytest.org/en/stable/how-to/cache.html and https://docs.pytest.org/en/stable/_modules/_pytest/cacheprovider.html
- [S18] Primary, no date, retrieved 2026-07-27 KST: Rails Active Record Migrations, https://guides.rubyonrails.org/active_record_migrations.html
- [S19] Primary, no date, retrieved 2026-07-27 KST: Flyway migrations docs source, https://github.com/flyway/flywaydb.org/blob/gh-pages/documentation/concepts/migrations.md
- [S20] Primary, no date, retrieved 2026-07-27 KST: Anthropic Prompt Caching, https://platform.claude.com/docs/en/build-with-claude/prompt-caching
- [S21] Primary, Linux man-pages 6.18 / 2026-era page, retrieved 2026-07-27 KST: `kill(2)`, https://www.man7.org/linux/man-pages/man2/kill.2.html
- [S22] Primary, no exact page date, retrieved 2026-07-27 KST: `daemon(7)`, https://www.man7.org/linux/man-pages/man7/daemon.7.html
- [S23] Primary, no exact page date, retrieved 2026-07-27 KST: Debian `start-stop-daemon(8)`, https://manpages.debian.org/unstable/dpkg/start-stop-daemon.8.en.html