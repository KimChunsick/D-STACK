# Finding ledger — 04-full-cycle-unattended-contract

The loop closes when a round raises nothing both NEW to this ledger and CONCRETE.

| id | round | severity | class | summary | status |
|---|---|---|---|---|---|
| F001 | 001 | **high** | a safety net that only worked while a human watched | a failed `dstack reg` printed a warning and continued; under the new unattended rule that leaves the Stop hook with no record and every downstream gate enforcing nothing | fixed — P6 registration is fail-closed (`set -e`) and confirmed with `status` before P7; recoverable causes routed to `reclaim`/`migrate` |
| F002 | 001 | medium | a taxonomy that does not match the reachable states | `stops` mixed internal recoveries (INVALID) with real human stops, and omitted the states where the wake mechanism itself is gone | fixed — `internal-recoveries` split out; mechanism-unavailable added as an explicit stop with its reason |
| F003 | 001 | medium (UX/DX) | a promise with no named mechanism | `notify` said notifications "go out" without naming the tool or its delivery preconditions | fixed — names `PushNotification`, states best-effort and what a not-sent means; a sealed round demoted from branch point to noise |
| F004 | 001 | medium | an invariant no caller can satisfy | "nothing else in that call" is violated by both launch recipes, which need `mktemp`, a trap, and path setup | fixed — the invariant is one background call whose BLOCKING TERMINAL STEP is `dstack run`; setup before is fine, dependent work after is not |
| F005 | in-use, at 001 | medium | a rule that contradicts its own rationale | `notify` listed "review round sealed" as a branch point while the same sentence called a per-round notification noise; found by trying to follow it | fixed with F003 |

| F006 | 002 | **high** | a fix that created the next defect | `reclaim` was listed as the automatic answer to foreign ownership, but it has no liveness signal — an autonomous reclaim silently un-gates a LIVE session while both keep working | fixed — `reclaim` is a human stop unless the handoff is provably orphaned; `migrate` stays an internal recovery because it refuses lossy conversions |
| F007 | 002 | **high** | a fix that created the next defect | `set -e` was added above a REFERENCE LIST of subcommands, so the success path ran `unreg` and deregistered the document it had just registered | fixed — runnable block split from the reference list, and it now asserts its end state via `status`; probed rc=0 confirmed / rc=1 on a missing record |
| F008 | 002 | medium | a transition table with no unique answer | an unavailable pinned review model is both a nonzero run (retry) and a missing dependency (stop) | fixed — `stops` wins over `internal-recoveries`; missing required dependencies are an explicit stop; retry restricted to diagnosed transient failures |
| F009 | 002 | medium | a rule fixed in the contract and not in what it invokes | `codex-review` still said "nothing else in that call" after `full-cycle` was corrected | fixed in `codex-review` (its own unit, round 004) — identical invariant in both |
| F010 | 002 | medium | a rule fixed in the contract and not in what it invokes | `codex-review`'s cleanup-only signal handlers swallowed INT/TERM/HUP | fixed in `codex-review` (its own unit, round 004) — identical handler form in both, measured |

| F011 | 003 | **high** | the same defect, one repair later | the fail-closed P6 recipe proved neither the complete registration set nor exact current-session ownership | fixed at the time by deriving the set with `find`; superseded by F019's move into code |
| F012 | 003 | medium | a handler that returns a status and changes nothing | the repaired signal handlers never forward termination to the foreground `dstack`, so a signalled wrapper reports failure over a live run | resolved by stating it rather than pretending otherwise — both shells defer the trap, so the wrapper CANNOT cancel the round; `<run-dir>/exit` is the status |
| F013 | 003 | medium | a taxonomy with missing and conflicting transitions | registration failure, unavailable model and cap closure did not each select a unique next state | fixed — `stops` wins over `internal-recoveries`, non-migratable `reg` failure added as its own stop |
| F014 | 003 | low | the record contradicting what it records | `task.md` kept stale claims about the instruction and its parsed schema | fixed — the task doc was rewritten against the parsed schema |
| F015 | 004 | **high** | an assertion that is its own proof | the P6 fence proved only its self-declared `DOCS` array; `DOCS=(GOAL u1 u1)` printed "3 documents" with a required unit absent | fixed at the time by derivation; superseded by F019 |
| F016 | 004 | **high** | a second authority contradicting the first | P6 still presented foreign ownership as recoverable via `reclaim`, against the autonomy stop | fixed — P6 names no failure outcomes of its own; and in round 006 the stop's own "provably orphaned" carve-out was removed as unreachable |
| F017 | 004 | medium | competing authorities for one transition | Step 2a re-ran every nonzero result while model unavailability said stop; P9 escalated "blockers" while §4 escalates only concrete highs | fixed for retry in round 004; the P9/§4 half persisted and is F022 |
| F018 | 004 | medium | an orphan path the recipes did not implement | `waits.external-residuals` acknowledged SIGPROF orphaning while both recipes still cleaned up unconditionally on EXIT | fixed — cleanup gated on `<run-dir>/exit`; refined again in round 006 so a signalled exit leaves the gated trap ARMED |
| F019 | 005 | **high** | five repairs, five new defects, same thirty lines | P6 still hand-set `GRAN`, compared cardinality not identities, and never cross-checked the milestone case | fixed by moving the check into code — `check-registration.sh` (T06), after the maintainer chose code over a sixth prose repair |
| F020 | 005 | medium | a loop returning 1 on its SUCCESS path | `grep && { exit 1; }` as the last statement of a `while` body left status 1 when grep did not match, so a trailing `|| exit 1` aborted the fence silently | fixed with F019 |
| F021 | 005 | medium | a pipeline reporting the wrong command's status | `find | sort` reports SORT's status, so a `find` that fails after emitting one path is accepted as a complete list | fixed — `find` alone, then `sort -o` in place, both statuses checked |
| F022 | 005 | medium | the P9/§4 half of F017 | P9 escalated at the cap for all "blockers" — high AND medium — while `autonomy` and §4 close concrete mediums without a human | fixed in round 006 — P9 defers cap closure entirely to §4 |

| F023 | 006 | **high** | a recipe implementing the opposite of the table above it | P6's registration loop iterated a literal `<Mn>/<NN-task>/task.md`, registering task-depth documents even for a Goal that declared milestone granularity | fixed — the loop calls `check-registration.sh --depth`, which returns 3 or 2 from the same GOAL.md parse the check itself uses |
| F024 | 006 | **high** | two parsers disagreeing about what a declaration is | the registration checker did not parse the same source as the scheduler, so fenced examples could override both granularity and task identities | fixed — fences tracked globally from line one, task rows at column zero, mirroring `check-parallel.sh`; reproduced on a fixture where the old parser read the fenced fakes and none of the real rows |
| F025 | 006 | medium | a variable read before it exists | `RUNDIR="$RD"` executed four lines before `RD` was defined, in a call where `$RD` from the assembly step no longer existed at all | fixed in `codex-review` Step 2 (where the recipe lives) — `LABEL` first, `RD` derived from it, trap armed after |
| F026 | 006 | medium | F022, still open at the start of this round | concrete-MEDIUM cap closure had two transitions | fixed with F022 |
| F027 | 006 | medium | a fail-loud claim with unchecked dependencies | the exit-2 guarantee was false when `comm`/`sed`/`uniq` failed — an erased delta reads as a pass | fixed in `check-registration.sh` (T06) — every transformation's status checked, plus a count-in/count-out guard |

| F028 | 007 | **high** | a fail-closed halt with no branch | wrong-depth and closed registrations make the checker exit 1 under `set -e`, matching neither `internal-recoveries` nor `stops` — a dead end in an unattended run | fixed — `unreg` the offending same-session record and re-run (a real recovery, not a disguised `reclaim`, since the record is ours); structural mismatches return to P6 or P5; exit 2 is explicitly NOT a recovery |
| F029 | 007 | medium | a loop that hides the failure it should stop on | `find -exec cmd \;` does not propagate cmd's status — measured, `find . -exec false {} \;` exits 0 — so one failed `reg` was invisible while every later document kept being claimed | fixed — the fence reads `--list` and checks each `reg` on its own |
| F030 | 007 | medium (security) | a placeholder interpolated into shell source | `G=docs/<goal>` with no slug constraint; the substituted value `safe; printf INJECTED` executed under both shells | fixed — the same slug `case` `codex-research` uses, with the same honest framing: defence in depth against a mistake, not a boundary |
| F031 | 007 | (found while fixing F029) | mutate-then-classify | the fence registered everything at the depth before anything decided whether it belonged, which is also why "safe to re-run" was false | fixed — `--list` emits only declared, scaffolded, still-open units |
| F032 | 007 | (from unit 05, deferred here) | one state, two rules, inside the authority | `autonomy.stops` forbids autonomous `reclaim` while the milestone-handoff prose prescribes it after `/clear` | fixed — the handoff lists what it intends to reclaim and ASKS; presence, not provability, is what makes that case different |
| F033 | 007 | (from unit 05, deferred here) | the copy fixed, the original not | `waits.external` still said "the call does not return" of a `run_in_background` Bash call | fixed — it names the STEP as the blocking thing and says the tool call returns immediately |

## Non-blocking follow-ups (recorded, not carried into another round)

None from this round.

## Blocking count per round

§4's counter is the number of concrete blocking findings still OPEN at the END of the round.

| round | raised (new, concrete, blocking) | OPEN at end of round |
|---|---|---|
| 001 | 4 (F001–F004) + 1 in-use (F005) | 0 |
| 002 | 5 (F006–F010, two of them high) | 0 |
| 003 | 3 (F011–F013) | 0 |
| 004 | 4 (F015–F018, two of them high) | 0 |
| 005 | 4 (F019–F022, one high) | 1 (F022 carried) |
| 006 | 4 new (F023, F024, F025, F027) + F026 = F022 recurring | 0 |
| 007 | 1 high + 2 medium (F028–F030) + 3 folded in from unit 05 and from fixing | 0 — **§4 cap closure** |

**Rounds 003–005 were backfilled in round 006.** The ledger stopped at F010 while five rounds ran,
which is a §3 violation on its face: the termination test is "nothing NEW to this ledger", and a
ledger that stops recording makes every later finding look new. Nothing was suppressed — the
findings are in the round files and were fixed — but the ledger is what §3 reads, so it is now
complete. Recording that it lapsed matters more than a tidy table.
