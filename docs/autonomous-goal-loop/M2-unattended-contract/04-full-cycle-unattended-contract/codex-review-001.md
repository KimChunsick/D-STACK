# Codex adversarial review — Round 001

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 12593 bytes
Allowlist: the review-unit folder, `claude/skills/full-cycle/SKILL.md`.

## GPT findings

[severity:high][technical correctness] P6 can enter unattended execution without gate enforcement because failed registrations are converted into warnings, leaving the Goal or review unit invisible to the Stop hook.
Sites: `claude/skills/full-cycle/SKILL.md` — confirmed: P6 scaffold, unattended-gate prose, `scheduling.autonomy.stops`.
Evidence: Both registration commands use `"$DS" reg … || echo "WARN: … UNGATED"`, while the new prose guarantees that every gate still runs.
Verification: The warning returns success after `dstack` rejects an empty session, active owner, or legacy cutover; no registry record then exists for the hook to enforce.
Suggested direction: Make registration and status confirmation a fail-closed P6 gate, with conflicts mapped to an explicit recovery or human stop.

[severity:medium][technical correctness] The stop taxonomy contradicts the reachable transition graph: `INVALID` is labeled a stop despite having an internal P5 repair, while loss or disablement of background execution creates an omitted manual state.
Sites: `claude/skills/full-cycle/SKILL.md` — confirmed: `checker.verdicts.INVALID`, `waits.external-residuals`, `autonomy.stops`.
Evidence: `INVALID` says “return to P5 … fix,” but `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` makes the pipeline manual and session loss removes automatic pickup without either condition appearing in `stops`.
Verification: The first state has a complete autonomous transition; the latter states have no autonomous resume transition, so the orchestrator cannot consistently decide whether to repair, wait, or request intervention.
Suggested direction: Separate internal recoveries from human stops and either enumerate mechanism-unavailable states or explicitly narrow the unattended guarantee around them.

[severity:medium][UI & UX / DX] `autonomy.notify` promises push delivery without naming the callable mechanism or its delivery preconditions, so an unattended maintainer may receive nothing at a required branch point.
Sites: `claude/skills/full-cycle/SKILL.md` — confirmed: unattended prose and `autonomy.notify`.
Evidence: The document says notifications “go out” and commands “send a push notification,” but never names `PushNotification` or defines how a skipped delivery is handled.
Verification: Installed client 2.1.220 sends a terminal notification and mobile push only with Remote Control connected; it may return “not sent” when active, disabled, or without a destination.
Suggested direction: Name `PushNotification`, state its best-effort conditions, and record or handle a non-delivery result.

[severity:medium][software structure] The “nothing else in that call” invariant is incompatible with the referenced research and review launch recipes, making the prescribed background action structurally unsatisfiable.
Sites: primary: `claude/skills/full-cycle/SKILL.md` `waits.external`; confirmed: `codex-review` Step 2 and `codex-research` Step 2.
Evidence: The top-level rule permits only `dstack run`, while the recipes put `mktemp`, traps, path setup, validation, and directory creation in the same background Bash call.
Verification: Removing that setup leaves required scratch variables or cleanup absent; retaining it violates the literal top-level invariant.
Suggested direction: Define the invariant as one background tool invocation whose blocking terminal step is `dstack run`, allowing setup before it but no dependent post-launch work.

Omitted-detail: 0 low

GPT verdict: reject — The unattended contract still permits an ungated run and contains unresolved stop, notification, and launch-control contradictions with concrete failure paths.

## Carried decisions
- **P6 registration is FAIL-CLOSED**, and it is a gate. A `reg` that fails leaves the Stop hook with
  no record, so every downstream gate enforces nothing and the run finishes looking complete. That
  was survivable when a human read the transcript; under `autonomy` nobody does. `set -e`, then
  confirm with `status` before any P7 work.
- **`internal-recoveries` and `stops` are different lists.** An INVALID declaration, a `reg` refused
  because another session owns the doc, and a nonzero external run all have defined next moves and
  need no human. Conflating them with real stops left the orchestrator unable to decide whether to
  repair, wait, or ask.
- **The unattended guarantee has an edge and the edge is named.** If
  `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` is set, or a resumed session did not restore its
  background task, nothing will wake the session. There is no autonomous transition out; say which
  one it is and stop, rather than stalling silently.
- **The launch invariant is "one background call whose BLOCKING TERMINAL STEP is `dstack run`"**,
  not "nothing else in that call". Setup before it is required by both recipes; what is forbidden is
  dependent work after it, because the call does not return until the external command finishes.
- **`autonomy.notify` names `PushNotification` and calls it best effort.** Delivery depends on
  Remote Control being connected and can legitimately report not-sent; the work docs are the durable
  record, so a non-delivery is not retried and not a stop. A sealed review round is NOT a branch
  point — three to five rounds per unit means one notification per round is exactly the noise the
  rule forbids.

Consensus: disagreed
