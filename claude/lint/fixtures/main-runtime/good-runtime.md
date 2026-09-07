document: claude/runtime.md
Main boundary: main owns user conversation, dispatch, compact receipts and CLI state only.
Worker boundary: fresh bounded workers own investigation, implementation, tests, failure diagnosis, fixes and heavy evidence inspection.
Retry boundary: a failure, missing receipt or capacity/tool refusal remains pending or blocked; use a new worker with a bounded handoff, never main takeover or stale context reuse.
Receipt: location/HEAD; R outcomes; changed files/commit; commands/exits; artifact paths; blockers/skips. Raw logs stay in artifacts.
Wait protocol: keep active run/Plan/worker IDs; use the host's interruptible native completion wait and handle input before resuming the same wait.
Question: answer then resume the same wait with the same active IDs.
Addition: main records req/decision changes through dstack, approves authorized scope and adjusts only affected work.
Conflict: safe-stop affected workers, preserve busy-plan guards and confirm they stopped before replacement; if no supported CLI transition exists, leave pending/blocked.
Stop: stop affected workers and report their actual state; stopped is not done.
Unsupported: if interruptible wait/input is unavailable, disclose that limit and use supported completion events; do not promise concurrent input.
No duplicate launch, blanket restart, manual JSON reset or false completion to free input.
Host tools: Claude Agent completion events; Codex wait_agent (when available), with interruptible bounded waits using the actual tool schema.
Main checks receipt metadata only; workers execute repository tests and lint, diagnose failures and inspect large artifacts.
Independent review/research/audit use fresh saved-sub sessions via dstack mode exec; seal the independent Plan review before dstack plan done.
Validation limits: deterministic document/fixture checks are not live host UI or concurrent-input evidence and cannot prove arbitrary prose semantics.
GSD provenance: retained secondary attribution; original checkout/revision unavailable, no fresh primary-source inspection claimed.
GSD sources: docs/explanation/the-phase-loop.md and skills/gsd-execute-phase/SKILL.md; quotations are retained in dstack-develop §11.
