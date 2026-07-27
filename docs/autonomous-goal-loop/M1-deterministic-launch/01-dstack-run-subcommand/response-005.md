# Maintainer response — Round 005 (closing)

Not bundled into any review round.

## F013 [high] `run_done` set before settlement — AGREED, my bug, fixed

Correct and entirely self-inflicted: round 004's fix set `run_done=1` *before* calling
`run_group_settle`, so a signal arriving during that bounded window — up to 15 seconds while the
group is being escalated TERM→KILL — found `run_done=1`, returned from `run_cleanup` immediately
without tearing anything down, and exited 6 with descendants alive. The reviewer's reproduction
(`supervisor_status=6`, `group_alive_after_supervisor_exit=yes`) is exact.

Fixed as directed: reentrancy protection and finalisation are now separate facts. `run_cleanup`
disarms its own traps on entry, which is what prevents re-entry; `run_done` is set only once the
run has been published or publication has been explicitly refused. The normal path no longer marks
anything done before settlement.

Verified — TERM delivered while the supervisor is inside settlement:

```
leader 7713 exited -> supervisor is now inside settlement
supervisor status=6
TERM-ignoring descendant still alive: 0   (want 0)
exit file=[7]
```

## F014 [medium] `die`-driven EXIT loses `cmd_run`'s locals — DISPROVED by instrumenting the real handler

This is round 004's F010 restated with a sharper claim, so it deserved a sharper answer than a
synthetic probe. I instrumented `run_cleanup` **in the real script** to print its view of the state
on entry, and ran both paths the finding names:

```
pre-fork die (adopted dir already holds `exit`):
  [PROBE] d=[…/k-pre] label=[k-pre] child=[] run_done=[0]
  dstack: run k-pre aborted before anything was launched — claim released, label is free to retry
  .launch left: no                      ← claim release did NOT skip

post-fork die (child-record write forced to fail):
  [PROBE] d=[…/k-post] label=[k-post] child=[7506] run_done=[0]
  dstack: run k-post aborted — launched process group torn down, recorded exit 143
  exit=[143]  stray=0                   ← publication had the capture path it "lacks"
```

Both of the finding's stated consequences — "pre-fork failures skip claim release" and "post-fork
failures lack the capture path needed for publication" — are contradicted by the code as shipped,
on the interpreter the finding names. `d` and `label` are populated in both cases.

Bash runs an EXIT trap fired by `exit`-from-inside-a-function while that function's locals are
still readable; the unwinding the finding describes happens when the function has *returned*
first, which no path here does — the only normal return is after the traps are disarmed.

Two rounds have now filed this mechanism and two independent instrumented runs contradict it. I am
recording it as disproved rather than fixing around it, because writing code to defend against
behaviour the interpreter does not exhibit would be adding machinery on a false premise. The
defensive defaults from round 004 stay (they cost nothing and caught a real `/.launch` bug), but
nothing further is built on this claim.

## F015 [low] an abort reported both failure and success — AGREED, fixed

Right: `run_publish "$st" || printf WARNING` was followed by an unconditional success line, so a
failed publish printed "could not publish" and "recorded exit" together. The success line now runs
only in the `if run_publish` branch; the failure branch says the capture stays nonterminal.

## Closure at the round cap

Five rounds, each of which found at least one genuine, reproducible defect in one subsystem: the
lifetime of the launched process. The cap closes the loop. Nothing concrete is left open —
F013 fixed, F014 disproved, F015 fixed — so this closes on the recorded-follow-up path rather than
by escalation.

**The residual, stated plainly:** F013's fix is verified by direct run but has not been through an
adversarial round of its own. The loop is being stopped by its cap, not by having run out of things
to find. Rounds 001–005 each found something, and honest extrapolation says a sixth might too. What
bounds the risk is that every finding has been in one place, the failure mode is now measured
rather than reasoned about, and the code fails closed at every point where it cannot prove the
launched work is gone: no terminal record without confirmed group quiescence, no deletion of a
capture with a live pid or group, no claim released once anything has been launched.

## Carried decisions

- Reentrancy protection (`run_cleanup` disarming its own traps) and finalisation (`run_done`) are
  separate. Never set `run_done` before settlement and publication have run.
- F014/F010's mechanism is DISPROVED on /bin/bash 3.2.57 by two instrumented runs of the real
  handler. Do not add machinery premised on it. The defensive defaults stay.
- All of rounds 001–004's carried decisions still stand.
- Closed at the round cap with the residual above recorded for the final report.
