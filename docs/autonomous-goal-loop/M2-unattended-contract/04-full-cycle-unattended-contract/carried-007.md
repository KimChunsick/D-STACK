## Carried decisions — Round 007
- **Every checker outcome now has a transition, and the missing one was a dead end.** Under `set -e`
  a checker exit 1 halted P6 with no branch in `internal-recoveries` or `stops`. Added: a document
  THIS SESSION registered that must not be (closed, or wrong depth) is `unreg`-ed and the check
  re-run — a genuine recovery, not a disguised `reclaim`, because the record is this session's own
  and the gate it held was over a document no phase governs. A STRUCTURAL mismatch returns to P6 or
  P5 instead. Exit 2 is deliberately NOT a recovery: a check that did not run must never be treated
  as one that found nothing.
- **`find -exec` masks the failure of the command it runs.** Measured: `find . -exec false {} \;`
  exits 0. The P6 loop was therefore registering every later document after one `reg` had already
  failed, turning one ownership conflict into several. The fence now reads an explicit list and
  checks each `reg` on its own.
- **Registering before classifying is what made "safe to re-run" false.** A depth-wide loop claims
  undeclared folders and already-closed units before anything decides whether they belong.
  `--list` emits GOAL.md plus every declared, scaffolded, still-open unit, so the fence cannot
  create the state the checker is about to refuse.
- **The `<goal>` slug check is defence in depth against a mistake, not a boundary.** Verified: the
  substituted value `safe; printf INJECTED` executed the second command under both shells, and now
  refuses. The honest framing is `codex-research`'s — the orchestrator writes the whole command, so
  no quoting form is a boundary; if the value ever comes from outside the session, the recipe is the
  wrong shape.

Consensus: resolved
