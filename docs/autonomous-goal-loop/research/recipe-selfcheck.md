## Needed info

Anthropic docs say background Bash commands run asynchronously and return a background task ID; they can be listed/stopped with `/tasks`, and `TaskStop` stops a running background task by ID. [S1][S2]

For `claude -p`, docs say a background Bash task is terminated about five seconds after Claude returns the final result and stdin closes. [S3]

If the `claude -p` run itself is stopped with `SIGTERM`, docs say Claude Code aborts the turn, terminates the process tree of any running Bash command, runs `SessionEnd` hooks, and exits with code `143`. [S3] The changelog confirms the same `SIGTERM`/Bash/process-tree/exit-143 behavior for print/SDK mode. [S5]

The Python SDK docs say `stop_task(task_id)` is followed by a `TaskNotificationMessage` whose status is `"stopped"`; that documented message does not expose a numeric exit code. [S4]

## Opposing views

Do not generalize `exit 143` to every `TaskStop` of a background Bash task: the public `TaskStop` docs say “stops” by ID, and the SDK docs document a `"stopped"` status, not a numeric Bash exit status. [S2][S4]

Do not conflate “background Bash task” with “background session”: release notes separately say background-session teardown sends `SIGTERM` before `SIGKILL`, but that is documented for `claude rm`/`stop`/idle reap of sessions, not specifically for `TaskStop` on a Bash task. [S5]

## For the goal

The narrow answer is supportable from primary Anthropic docs: in headless `claude -p`, background Bash tasks are cleaned up after final output, and caller-visible stop-by-`SIGTERM` exits `143`. [S3][S5]

## Against the goal

If the goal is to prove the exact signal and numeric exit status for a background Bash task stopped via `/tasks` or `TaskStop`, Anthropic’s public docs I found are insufficient: they document stop/status semantics, not the task process’s signal or exit code. [S2][S4]

## Unverified

I could not verify from public Anthropic documentation whether `TaskStop` for a background Bash task uses `SIGTERM`, escalates to `SIGKILL`, or records/returns `143` in the task output. I also did not verify behavior by running Claude Code locally.

## Sources

[S1] Primary: https://code.claude.com/docs/en/interactive-mode, publication date: no date, retrieved: 2026-07-28.  
[S2] Primary: https://code.claude.com/docs/en/tools-reference, publication date: no date, retrieved: 2026-07-28.  
[S3] Primary: https://code.claude.com/docs/en/headless, publication date: no date, retrieved: 2026-07-28.  
[S4] Primary: https://code.claude.com/docs/en/agent-sdk/python, publication date: no date, retrieved: 2026-07-28.  
[S5] Primary: https://code.claude.com/docs/en/changelog, publication dates cited: 2026-07-17 and 2026-06-02, retrieved: 2026-07-28.