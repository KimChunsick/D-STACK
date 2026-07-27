# Codex adversarial review — Round 003

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 33441 bytes (r1 12593, r2 23695). Allowlist
unchanged: the review-unit folder, `claude/skills/full-cycle/SKILL.md`.

## GPT findings

[severity:high][technical correctness] The fail-closed P6 recipe proves neither the complete registration set nor exact current-session ownership, allowing review units to remain silently ungated.
Sites: `claude/skills/full-cycle/SKILL.md` — P6 schema and registration fence; confirmed: `claude/bin/dstack` status formatting, `claude/hooks/fullcycle-gate.sh` owner filtering.
Evidence: P6 requires every review unit but registers one `U`; substring checks also accept longer paths and status lines marked `(session foreign)`, which the hook subsequently skips.
Verification: Under zsh, the recipe printed “P6 registration confirmed” with one of two expected units, with only longer foreign paths, and with exact foreign-owned paths.
Suggested direction: Add an exact, owner-aware `dstack` verification operation accepting the Goal and complete review-unit set, then make that single operation the P6 fence.

[severity:medium][technical correctness] The repaired signal handlers return the right status but never forward termination to foreground `dstack`, so cancellation can remain deferred until the external run finishes.
Sites: `claude/skills/codex-review/SKILL.md` — Step 2 launch recipe; confirmed: `claude/skills/codex-research/SKILL.md` Step 2 and `claude/skills/full-cycle/SKILL.md` `waits.external`.
Evidence: Each handler cleans and exits but never signals the foreground command; `dstack` cannot execute its process-group teardown unless it receives the signal.
Verification: TERM sent to wrappers running a three-second foreground child under bash and zsh returned 143 only after the full three seconds elapsed.
Suggested direction: Make wrapper termination explicitly reach `dstack`, or transfer terminal-process ownership with an execution shape whose cleanup remains correct.

[severity:medium][technical correctness] The exclusive autonomy taxonomy still has missing and conflicting transitions for registration failure, unavailable models, and concrete-medium closure.
Sites: `claude/skills/full-cycle/SKILL.md` — `autonomy`, P6 failure prose, and P9 round-budget prose; confirmed: `claude/skills/codex-review/SKILL.md` Steps 2a and 4.
Evidence: Unresolved empty-session registration is absent from `stops`; unavailable models select both stop and unconditional nonzero rerun; concrete mediums at the cap select both automatic recorded closure and user escalation.
Verification: Actual `dstack reg` returned 1 for empty and malformed session IDs; tracing the other two documented states reaches both contradictory branches deterministically.
Suggested direction: Define one exhaustive precedence table and have P6 and the invoked review skill defer to it without restating conflicting transitions.

[severity:low][software structure] The task record preserves stale claims that contradict the current instruction and its parsed schema.
Evidence: It still says the launch contains “nothing else,” and its recorded autonomy keys omit `internal-recoveries`.
Verification: Direct Ruby YAML parsing returned `["rule", "internal-recoveries", "stops", "bounds", "notify"]`, while the task’s recorded output lists only four keys.

Omitted-detail: 0 low

GPT verdict: reject — The P6 fence can still report success while required documents are ungated, and the signal and transition rules retain reproducible unattended-execution failures.

## Carried decisions
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
