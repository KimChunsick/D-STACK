## Carried decisions — Round 003
- **The P6 fence checks EVERY review unit, by EXACT LINE, and requires `(this session)`.** Three
  independent holes in the round-002 form, all demonstrated: it registered one unit where P6 needs
  all of them; `grep -qF` is a substring match, so `…/task.md.bak` satisfied a check for
  `…/task.md`; and a line reading `(session <other>)` passed while the Stop hook SKIPS foreign
  records. Each reported success over work that was not gated. Now a `DOCS` array plus
  `grep -qxF -- "  $d  (this session)"`, verified against all three counterexamples.
- **An unusable session id is a STOP.** `dstack reg` returns 1 for an empty or malformed session id
  and there is no autonomous repair; continuing means running ungated. Same for a registry that
  cannot be written, and for a `status` line that never shows the document as this session's.
- **One precedence table, and the prose defers to it.** P6's failure paragraph used to restate
  outcomes that `autonomy` also defines, which is how a state ends up with two answers. It now
  points at `autonomy` and states none of its own.
- **Signal handlers do not cancel `dstack`, and `waits.external` says so.** Both shells defer a
  pending trap while a foreground command runs, so a TERM lands only after the run returns —
  measured, `rc=143` after a full five-second child. `<run-dir>/exit` is the run's status; a
  completed round can be reported 143. Cancelling in flight means stopping the recorded process
  group, not signalling the wrapper.

Consensus: disagreed
