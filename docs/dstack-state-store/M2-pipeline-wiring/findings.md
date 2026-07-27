# Finding ledger — M2 (pipeline wiring and prompt trim)

The termination signal for this unit's review loop. The `GPT verdict` line is advisory; what
closes the loop is this ledger going quiet — a round that raises no finding both NEW and CONCRETE
— or the non-convergence rule firing.

Blocking findings per round: **R1 5 · R2 4 · R3 6 · R4 4 · R5 3 · R6 5 · R7 4 · R8 2 · R9 3 · R10 3**

R7-R10 is 4, 2, 3, 3: not strictly decreasing across three consecutive rounds. Non-convergent by
measurement, so the loop closed at R10.

Bundle bytes per round: R5 110,913 · R6 115,229 · R7 136,496 · R8 155,283 · R9 170,782 ·
R10 184,373 — monotonically UP. The blocking count stayed flat while the reviewed surface grew
66%, which is the whole argument for the ratchet and the response-file extraction.

## Open (carried out of the closed loop)

| # | Sev | Class | Finding | Raised |
|---|---|---|---|---|
| F-01 | low | security | `dstack status` records `session/label` with no unit ownership, so capture cleanup is driven from a hand-written label list that also verifies itself | R10 |
| F-02 | medium | correctness | no worker fan-out flow is exercised end to end anywhere in this Goal; the committed recipe is honest and untested-by-use | R9-R10 |

Both are recorded with evidence in `task.md` under «Recorded follow-ups».

## Closed in the final round (R10)

- [high][security] snapshot reads had no parent containment — one `contained` helper now serves
  both read paths
- [high][correctness] the committed-mode invocation was commented out — written out runnable
- [medium][correctness] `unit-scope` named a checker mode that does not exist — claim deleted,
  fan-out restricted to single-task units
- [low][correctness] `run.sh` published non-atomically — temp + rename
- [low][correctness] the schema check reported `ok` after its own setup failed, and its bare-call
  scan was missing `rm-run` — both fixed, negative-controlled
- [low][structure] record drift about which files are declared where — corrected in place
