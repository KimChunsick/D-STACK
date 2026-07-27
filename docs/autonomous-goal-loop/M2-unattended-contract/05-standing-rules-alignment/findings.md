# Finding ledger — 05-standing-rules-alignment

The loop closes when a round raises nothing both NEW to this ledger and CONCRETE.

| id | round | severity | class | summary | status |
|---|---|---|---|---|---|
| F001 | 001 | medium | a summary read as an authority | the unattended stop list presents itself as exhaustive but omits the non-migratable `dstack reg` failure, so a run can continue past a stop the authority names | fixed — the missing entry added, and the list now says it is a summary and points at `scheduling.autonomy` before anything concludes something is not a stop |
| F002 | 001 | medium | the same divergence, other direction | the summary forbids autonomous `reclaim` outright while the authority carved out a "provably orphaned" handoff | fixed by narrowing the AUTHORITY — `reg` returns 0 for a document this session already owns, so the carve-out named a state it never reaches, and "or the user says so" is not autonomy at all |
| F003 | 001 | low | blocking attributed to the wrong lifecycle | "the call does not return until the command finishes" is false of the Bash tool call, which returns immediately; it is the background task that stays alive | fixed — the rule is stated of the `dstack run` STEP, with the distinction spelled out, because getting this wrong is what invites a hand-rolled watcher |
| F004 | 001 | low (the real Why) | a gate row claiming more than the evidence | the E2E section says behaviour was not verified while the gate row says behaviour was confirmed by direct run | fixed — the row and the section now say the same thing, and what the recorded commands actually check is named |

| F005 | 002 | medium | one state, two rules — now INSIDE the authority | `autonomy.stops` says there is no autonomous `reclaim`; the milestone-boundary handoff prose says to `/clear` and then `reclaim` the records the id rotation orphaned | **DEFERRED — frozen file.** `full-cycle/SKILL.md` is inside unit 04's open round-007 bundle. Resolution agreed and carried to unit 04: after `/clear` the operator is present by construction, so the handoff reclaim is an explicit user-input confirmation, never autonomous |
| F006 | 002 | low | the copy fixed, the original not | round 001 corrected "the call does not return until the command finishes" here; `waits.external` in the skill still carries it | **DEFERRED — frozen file**, carried to unit 04 with F005 |
| F007 | 002 | low | honest limits that were not complete | the standing file omitted OS reaping and the `SIGKILL`/`SIGPROF` orphan case — so a capture with no terminal record read as a plain failure and the documented move was to relaunch over a live `codex exec` | fixed — both residuals added, with the liveness check named; verified with line wrapping folded out after a naive phrase grep reported two false absences |
| F008 | 002 | low (security) | the repair became the next instance | F004's fix wrote "behaviour itself is out of scope here" into the gate row — an evaluator directive inside data the reviewer is told to distrust | fixed — the section states what the commands establish and what they do not, and instructs nothing. The reviewer proved it operationally: distrusting the disclaimer is what surfaced F007 |

## Non-blocking follow-ups (recorded, not carried into another round)

**Carried to unit 04 (`full-cycle/SKILL.md`), not to this unit's next round.** F005 and F006 are
findings against a file this unit does not own and cannot touch while it sits in an open bundle.
They are recorded here because this round raised them and in unit 04's ledger because that is where
they get fixed. Neither is closed by being deferred.

## Blocking count per round

§4's counter is the number of concrete blocking findings still OPEN at the END of the round.

| round | raised (new, concrete, blocking) | OPEN at end of round |
|---|---|---|
| 001 | 2 (F001, F002) + 2 low | 0 |
| 002 | 1 (F005) + 3 low | 1 (F005, deferred to unit 04 under the freeze-rule) |

F005 stays counted as OPEN. Deferring a finding to the unit that owns the file is a routing
decision, not a closure, and counting it as closed here would let a real contradiction disappear
between two ledgers.

This unit's first round is the consolidated batch pass the maintainer authorised mid-Goal, so it
carries no earlier history. Both mediums are one defect seen from two sides: a summary and its
authority disagreeing about the same rule. Closing it by editing only the summary would have left
the two files still capable of diverging, so the authority moved too.
