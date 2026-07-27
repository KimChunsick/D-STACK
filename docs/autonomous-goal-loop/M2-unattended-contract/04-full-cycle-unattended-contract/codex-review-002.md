# Codex adversarial review — Round 002

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 23695 bytes (r1 12593). Allowlist unchanged:
the review-unit folder, `claude/skills/full-cycle/SKILL.md`.

**Relaunched after a harness kill.** The first attempt (`t04-r2`) published `exit=143` with zero
bytes; the retry fence confirmed nothing alive in the capture, and the same bundle was re-sent
under `t04-r2a`.

## GPT findings

[severity:high][technical correctness] Automatic recovery can reclaim a document from a live session without proving staleness, leaving the original session ungated while both continue.
Sites: `claude/skills/full-cycle/SKILL.md` — `autonomy.internal-recoveries`; confirmed: `claude/bin/dstack` `cmd_reclaim`, `claude/hooks/fullcycle-gate.sh` owner filtering.
Evidence: Foreign ownership triggers `reclaim`; `cmd_reclaim` explicitly has no liveness signal and replaces the owner, while the Stop hook skips records owned by another session.
Verification: After session B replaces session A’s owner with `mv -f`, A reaches the hook’s owner-mismatch `continue`, so its document is no longer enforced.
Suggested direction: Automatically reclaim only a provably orphaned handoff; treat unknown foreign ownership as a human stop unless liveness or explicit transfer evidence exists.

[severity:high][technical correctness] The fail-closed P6 shell block cannot establish its promised final state when executed as written: success continues into `unreg`, while failure exits before `reclaim` or `migrate`.
Sites: `claude/skills/full-cycle/SKILL.md` — P6 registration block.
Evidence: `set -e` precedes registration, but lifecycle-management commands are placed sequentially after `status` in the same fenced block.
Verification: Under zsh with substituted paths, the success probe executed `reg`, `reg`, `status`, `unreg`, `reclaim`, `migrate`; a failing first `reg` returned 3 and executed none of the recovery commands.
Suggested direction: Separate the fail-closed registration recipe from management examples, branch explicitly on recoverable failures, and finish by asserting both records remain owned by the current session.

[severity:medium][technical correctness] The autonomy transition table is contradictory: every nonzero external run is retried, but unavailable review models and unresolved registration are separately ordered to stop and are absent from the exclusive stop list.
Sites: `claude/skills/full-cycle/SKILL.md` — `autonomy.internal-recoveries`, `autonomy.stops`, P6 prose; confirmed: `claude/skills/codex-review/SKILL.md` model-availability rule.
Evidence: A pinned-model failure produces a nonzero run, selecting both “re-run under the next label” and “surface it and stop,” while `stops` claims no other human stops exist.
Verification: A missing or unavailable `gpt-5.6-sol` deterministically yields repeated failed attempts under one rule and immediate termination under the other, leaving the orchestrator without a unique transition.
Suggested direction: Give stop rules precedence, enumerate unavailable required dependencies and unresolved registration, and restrict automatic retry to diagnosed transient failures.

[severity:medium][software structure] The round-001 launch contradiction remains in the invoked review instructions even though the top-level contract was changed.
Sites: `claude/skills/full-cycle/SKILL.md` — `waits.external`; confirmed: `claude/skills/codex-review/SKILL.md` Step 2 and launch bullet.
Evidence: Full-cycle permits setup before the blocking terminal step, while codex-review still requires “nothing else in that call” and then places `mktemp`, a trap, and path assembly in that call.
Verification: Omitting setup leaves `SCRATCH` and `RD` unavailable; retaining the runnable setup violates the review skill’s literal invariant.
Suggested direction: Apply the “blocking terminal step is `dstack run`” invariant consistently in the invoked review skill.

[severity:medium][technical correctness] The codex-review wrapper’s cleanup-only signal handlers swallow `INT`, `TERM`, and `HUP`, so termination can be ignored while the background command continues.
Sites: `claude/skills/codex-review/SKILL.md` — Step 2 launch recipe.
Evidence: `trap 'rm -rf "$SCRATCH"' EXIT INT TERM HUP` performs cleanup but neither restores the signal nor exits.
Verification: On zsh 5.9 and bash 3.2.57, self-TERM produced `CSC` and exit 0; TERM sent while a foreground child ran likewise produced `CSC` and wrapper exit 0.
Suggested direction: Use separate signal handlers that disarm `EXIT`, clean once, and exit with the corresponding signal status.

Omitted-detail: 0 low

GPT verdict: reject — The current contract still permits silent gate loss and unsafe ownership theft, with contradictory recovery and launch behavior that prevents reliable unattended execution.

## Carried decisions
- **`reclaim` is a HUMAN STOP, not an internal recovery.** `cmd_reclaim` has no liveness signal, so
  it cannot distinguish a crashed session from a working one — it replaces the owner outright, and
  the Stop hook then SKIPS records owned by another session. An autonomous reclaim therefore
  silently un-gates a live session's work while both keep running. Automatic only when the handoff
  is provably orphaned; otherwise ask.
- **The P6 fence is a RUNNABLE BLOCK, and the other subcommands are a reference list.** Putting
  `set -e` above a list that ends in `unreg` made the success path deregister the document it had
  just registered. Split, and the runnable block now ASSERTS its promised end state with `status`
  rather than trusting `reg`'s exit code. Verified: normal path rc=0 "P6 registration confirmed";
  a missing record rc=1 "P6 BLOCKED".
- **A `stops` entry beats an `internal-recoveries` entry.** An unavailable pinned review model is
  both a nonzero run (retry) and a missing required dependency (stop); without precedence the
  orchestrator had no unique transition and would retry a missing model forever. Stops win, and
  automatic retry is restricted to a DIAGNOSED transient failure.
- The launch invariant and the signal-handler form must be identical in `full-cycle` and in the
  skills it invokes. Fixing the contract and leaving the invoked recipe contradicting it hands the
  orchestrator both rules.

Consensus: disagreed
