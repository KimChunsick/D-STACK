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

## Recover a legacy query overwrite before preparation

Ordinary target resolution and positional cases resolution are read-only: they never renew
heartbeats or change ownership, even for the current owner. Explicit run new/adopt and handoff
resume retain their ownership writes. Older versions could overwrite the
source owner during a query while leaving its transcript_path and saved main intact.
For that specific observed failure, use the actual current host/session and the exact saved
source transcript identity. Never override a session environment variable to impersonate it.

Obtain explicit acknowledgement that the source and all native workers are stopped, honoring
acknowledgement already given. From the exact run Git worktree, execute:

```bash
dstack handoff recover-owner --run <run-id> --host codex --session <source-session-id> --history <exact-source.jsonl> --source-stopped
```

Use --host claude for a Codex source. This narrow repair is part of handoff preparation and is
allowed before the ordinary host mismatch check; it does not authorize any other main work.
The actual caller must equal the overwritten current owner, differ from the source and run
on the declared host, which must differ from the saved main. Saved transcript_path is required;
its filename must exactly encode the source (Claude SOURCE.jsonl, Codex rollout-*-SOURCE.jsonl).
If that stored path exists, the supplied canonical path must match. If it is missing or moved,
the supplied file must have the same filename and pass full original provider/session/Git
worktree history validation. Do not guess the newest history or alter normal --session guards.
Incomplete trailing history, invalid evidence, active commands, any existing handoff attempt
and any previous recovery directory block this repair. Inspect failures; there is no force path.

The CLI exclusively creates owner-recovery/intent.json under the run, with original metadata
and its hash, proposed metadata and its hash, source path/hash/session, actual caller, timestamp
and stopped acknowledgement. It rechecks history and the full snapshot before one atomic
metadata replacement, then records completed. Only owner_session and canonical transcript_path
are restored; misleading legacy owner_pid and owner_ts are removed because the stopped source
has no verified current heartbeat. Mode, status, CURRENT, requests, plans and evidence remain.
A missing completed marker means uncertain recovery: inspect receipt and actual metadata;
never remove the guard or blindly retry. Even completed receipts remain immutable and prevent
repeating this exceptional repair. After success use ordinary prepare and then distinct-session
resume; recovery alone does not transfer the run to the destination host.

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
