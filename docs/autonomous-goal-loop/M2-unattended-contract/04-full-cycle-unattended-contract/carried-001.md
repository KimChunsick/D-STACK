## Carried decisions — Round 001
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
