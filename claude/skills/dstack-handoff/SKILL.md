---
name: dstack-handoff
description: Prepare or resume an explicit cross-main handoff of an existing D-STACK Goal between Claude and Codex, using the source session history and the CLI-owned packet. Use for requests to hand off the current work to the other main or resume a supplied RESUME.md. Do not use for ordinary adoption, quick tasks, project mode changes or automatic quota monitoring.
---

# dstack-handoff

Read the shared runtime.md installed in the actual provider's agent home. Its supplied-role
boundary applies first: a handoff summarizer returns strict JSON only and never runs this skill.
For an explicit user handoff request, this skill runs before the ordinary host mismatch check.
The saved main may still be Claude while the current host is Codex, or the reverse. That narrow
exception permits only handoff preparation/resume; no other main work starts until resume
succeeds. No CLI command can change the current conversation's engine.

## Prepare the handoff

Work from the exact existing run worktree. Use the requested destination and an explicit run
id when supplied; otherwise the CLI selects CURRENT. Read-only status inspection is allowed.
Do not adopt the run, change project mode, refresh the mode snapshot or start workers first.

```bash
dstack handoff --to codex --run <run-id> --dry-run
dstack handoff --to codex --run <run-id>
```

Use --to claude for the reverse direction. A dry run validates the selected source and shows
the destination provider, model, high effort, cwd and argv without writing or calling a model.
The actual command always calls the destination provider, independently of the source main
and saved sub. It uses a fresh read-only handoff summarizer, never the source model or a
fallback provider. It does not monitor quotas or switch a session in the background.

The CLI selects the saved main and owner_session, then the known transcript_path or an exact
session match in local Claude/Codex JSONL history. It never uses the newest unrelated session.
--session must equal the saved owner_session. For a custom home or unsupported archived store,
use the explicit source history path:

```bash
dstack handoff --to codex --run <run-id> --session <source-session-id> --history <source.jsonl>
```

Missing, mismatched, malformed or oversized history is a visible failure. Report that error;
do not substitute another session, handwrite a summary or decode hidden reasoning/encrypted
payloads. Keep any supported trailing-record or omitted-history warnings as evidence gaps.
The source session, native workers and captured commands must be quiescent while preparing.
If the source history, ledger or Git contents change, prepare a fresh packet.

Only the CLI writes .dstack/runs/<run-id>/handoffs/<handoff-id>/, including context.md,
packet.json, summary.json, RESUME.md and its ready hash. Do not edit or patch those files.
Successful summary validation seals the packet; it does not yet change the main or owner.
Return the printed packet id, exact worktree and RESUME.md path to the user.

## Resume in the new destination main

Open a new destination main session in the exact same worktree and read the CLI-produced
RESUME.md. The new session must have a distinct, nonempty identity supplied by the host
environment. Do not copy the old identity or invent one to pass the check.

Before using --source-stopped, obtain explicit acknowledgement that the source session and
all its native workers are stopped. Honor an acknowledgement already supplied in this session;
do not infer it from an idle screen, a model error or a prepared packet. If missing, ask only
for that acknowledgement. There must also be no unresolved exec capture.

```bash
dstack handoff resume <handoff-id> --host codex --source-stopped --run <run-id>
```

Use the actual destination host. Resume checks the original history, ledger and Git contents
against the snapshot, packet integrity, stopped-source acknowledgement and unused packet.
It changes only this run's main/owner metadata and CURRENT. Project defaults, the saved sub
and existing evidence remain intact. A failed or stale packet needs fresh preparation; there
is no force path. A process interrupted during takeover leaves a resuming marker and blocks
automatic retry. Inspect the receipt and actual metadata; never delete the guard blindly or
claim the takeover finished from a partial write.

After successful resume:

```bash
dstack mode show --host codex --run <run-id>
dstack status
dstack run verify
```

Continue through the shared dstack-develop and dstack-verify workflow. Read active changes,
failed attempts, blockers, next steps and source references before doing new work. Preserve
completed work without restarting it, but do not turn recorded implementation into accepted
requirements. Carry every evidence gap and pending check into the normal verification gates.
Normal dstack run adopt --refresh-mode remains an explicit refresh of the project selection;
it is not a replacement for this history-preserving cross-main handoff.

## Record what ran

Retain the CLI stdout/stderr/exit and the exec capture path for the preparation attempt. The
main records requirement evidence through dstack evidence add, then uses the normal request,
coverage, decisions, verification and gate checks. A renderer, stub provider or isolated
installer test is configuration evidence only. Record actual Claude/Codex execution separately
with the observed engine and result, or report skipped: <reason> for each unexecuted model run,
installation or gate. Never silently promote those skips to a pass.
