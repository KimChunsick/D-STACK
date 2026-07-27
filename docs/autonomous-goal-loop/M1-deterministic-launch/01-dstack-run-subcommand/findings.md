# Finding ledger — 01-dstack-run-subcommand

Every finding ever raised for this review unit, with a stable id. The loop closes when a round
raises nothing that is both NEW to this ledger and CONCRETE (a real failure path, counterexample,
or reproducible risk). A restatement, or a variant of a class already recorded, is logged and does
not reopen.

Design-consult findings (pre-implementation, D00x) are included: they are part of this unit's
history and several were resolved by a design change rather than a patch.

| id | round | severity | class | summary | status |
|---|---|---|---|---|---|
| D001 | design | high | process lifetime | detached child is invisible to the harness; a killed waiter leaves nothing to notify | resolved by design change — detachment dropped |
| D002 | design | high | race | bounded grace for the pid file falsely reports a failed launch | unreachable after the design change — no pid file, no grace period |
| D003 | design | high | race | `kill -0` on a recycled pid; a dead wrapper proves nothing about its child | unreachable after the design change — bash `wait` on our own child |
| D004 | design | high | race | allocate-or-adopt is check-then-write | fixed — atomic `mkdir "$d/.launch"` claim, never removed |
| D005 | design | high | lifecycle | `rm-run`/`prune` can delete an actively running capture | fixed for `rm-run` (see F005 for `prune`) |
| D006 | design | medium | validation | terminal/pid record validation and producer-failure handling | fixed — status is bash's own `$?`, published atomically, failed publish dies |
| D007 | design | medium | security | TOCTOU on `--stdin` and on reserved output files | partially fixed — symlinks refused; residual window recorded in code (bash has no `O_NOFOLLOW`) |
| D008 | design | medium | DX | child status collides with dstack's documented exit codes | fixed — 0 on success, 6 on child failure, exact status in `exit` and on the DONE line |
| D009 | design | medium | verification | harness background tracking unverified past 9 minutes | closed by measurement — probe L, 25 minutes, woke the session |
| D010 | design | low | structure | does process execution belong in `dstack` | accepted — consult says keep it; the claim and terminal publication coordinate state dstack owns |
| F001 | 001 | high | process lifetime | terminating the supervisor leaves `codex exec` orphaned; `rm-run` then deletes its capture | fixed — own process group, trap-driven teardown, `rm-run` checks both pids |
| F002 | 001 | medium | validation | empty label collapses the capture path to the session root | fixed — `require_label` rejects `''` |
| F003 | 001 | medium | validation | adoption does not establish reserved-path invariants (`exit` as file or directory) | fixed — every reserved name must be absent at claim time |
| F004 | 001 | medium | the real Why | no long, output-silent `dstack run` through one background call had been exercised | rebutted with evidence — round 001 is that run: 10.3 min, 0 bytes seen by the harness, woke the session |
| F005 | 001 | low | documentation | the disposition claimed `prune` was fixed when only `rm-run` was | fixed — wording narrowed; `prune` behaviour recorded as a non-blocking follow-up |
| F013 | 005 | high | a guard that covers the wrong span | `run_done=1` was set BEFORE group settlement, so a signal during that 15s window made cleanup return without tearing anything down | fixed — reentrancy (trap disarm on entry) and finalisation (`run_done`) separated; verified by TERMing the supervisor mid-settlement: descendant dead, exit 7 published |
| F014 | 005 | medium | — | restatement of F010: `die`-driven EXIT loses `cmd_run`'s locals, so claim release is skipped and publication lacks the capture path | **disproved** — `run_cleanup` instrumented in the real script printed populated `d`/`label` on both paths; pre-fork released the claim, post-fork published exit 143 with no stray. Two independent instrumented runs now contradict this mechanism |
| F015 | 005 | low | reporting | a failed publish printed both "could not publish" and "recorded exit" | fixed — the success line only runs in the `if run_publish` branch |
| F010 | 004 | high | — | claimed bash unwinds `cmd_run`'s locals before the EXIT trap, so `run_cleanup` dies on `set -u` and pre/post-fork `die` paths strand or orphan | **rebutted** — on the deployed /bin/bash 3.2.57 (shebang target and PATH shell alike) the trap reads those locals, including from a nested `die`; fault-injecting the exact post-fork failure into the real script tore the group down, published exit 143, left no stray. Suggestion adopted anyway: every read in `run_cleanup` is defaulted, and the claim release is guarded on `$d` so it can never resolve to `/.launch` |
| F011 | 004 | high | a guard that reports but does not gate | `run_group_settle \|\| printf WARNING` then published unconditionally, making a capture terminal — and therefore deletable — while a group that survived SIGKILL was still writing into it | fixed — publication happens only inside the `run_group_gone` branch at both call sites; otherwise a loud ERROR and no terminal record, so `rm-run` keeps refusing |
| F012 | 004 | low | recycled identity | a pgid carries no ownership token, so settlement could signal an unrelated group if the id is recycled between probe and signal | accepted residual, recorded in the code — the window needs the group fully gone first, so it is the probe-to-signal instant only |
| F007 | 003 | high | a check that names the wrong entity | `run_abort` read the group LEADER's death as the group's; a TERM-resistant descendant stayed alive while the capture was published terminal and `rm-run` would delete it | fixed — `run_group_gone`/`run_group_settle` gate publication on group quiescence with bounded TERM→KILL escalation, on both paths; `rm-run` checks the group |
| F008 | 003 | medium | protective window does not span what it protects | cleanup was armed after the claim's own writes and covered only INT/TERM/HUP | fixed — one `run_cleanup` owner on EXIT + 8 signals, armed one statement after the claim, disarmed only after publication; `claim_release_and_die` deleted as a redundant second path |
| F009 | 003 | low | wrong belief written into the code | the post-wait liveness loop's recycled-pid rationale was factually wrong (bash caches a reaped status; the loop could spin on an unrelated pid) | fixed — loop removed entirely; the signal handler exits rather than returning to it |
| F006 | 002 | high | protective window does not span what it protects | F001's repair was open at both ends: trap installed after the fork, `die` on the pid-write failure, trap cleared before publication; and `rm-run` read a missing child record as safe | fixed — trap up before the fork and down only after publication, `run_abort` instead of `die`, `$!` as the fork-window tiebreaker, pre-fork failures release the claim, `rm-run` fail-closed on an unknown child |

## Non-blocking follow-ups (recorded, not carried into another round)

- **F005 / D005-b — `prune` does not consult launch state.** It selects by mtime at a threshold of
  eight complete days, against runs of 3–25 minutes, so a capture it selects cannot plausibly be
  live. Revisit only if that threshold ever narrows.
- **D007 — `--stdin` TOCTOU.** Bash cannot express `O_NOFOLLOW`. The callers are recipes in this
  repository naming a file the same session just wrote, inside a directory it created at mode 700.

## Blocking count per round

| round | new concrete blocking findings (high + concrete medium) |
|---|---|
| 001 | 4 (F001 high, F002, F003, F004 medium) |
| 002 | 1 (F006 high) |
| 003 | 2 (F007 high, F008 concrete medium) |
| 004 | 1 (F011 high; F010 rebutted, F012 low) |
| 005 | 1 (F013 high; F014 disproved, F015 low) |

**Closed at the round cap (5 for a per-task unit).** Nothing concrete remains open: F013 fixed,
F014 disproved, F015 fixed. The loop stopped on its cap, not on exhaustion — every one of the five
rounds found at least one genuine reproducible defect, all in the same subsystem. Residual carried
to the final report: F013's fix is verified by direct run but was not itself adversarially
reviewed.

Not strictly decreasing at 002→003, but that is one round, not the three consecutive rounds the
non-convergence test needs. Round cap for a per-task unit is 5, so round 004 runs under the
**wind-down rule**: close on a positive consensus as soon as no unresolved high and no unresolved
*concrete* medium remain, recording anything else as a non-blocking follow-up.

**Reading of the stream.** Every finding so far has been genuinely new rather than a restatement,
and each carried its own reproduction:

- F001 — there is no teardown at all.
- F006 — the teardown exists but does not span the interval it protects.
- F007 — the teardown spans the interval but signals the wrong *entity* (leader, not group).
- F008 — ownership starts one statement too late and covers too few signals.

All four are the same subsystem and, in hindsight, one root cause: **process lifetime was being
reasoned about instead of measured.** Each repair was verified against a child that cooperated
(`/bin/sleep`), so each passed while the uncooperative case stayed broken. Round 003's own carried
decision — evidence must exercise a child that does NOT honour the signal under test — is the
correction, and F007's fix was verified that way.

A fifth finding of the form "…and this narrower window is still open" needs its own demonstrated
failure path to count as new rather than a variant of F006/F008.
