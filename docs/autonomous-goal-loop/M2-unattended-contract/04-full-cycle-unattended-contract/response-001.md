# Maintainer response — Round 001

Not bundled into any review round.

## F001 [high] a failed registration was a warning — AGREED, fixed

The sharpest finding this Goal has produced, because it is a defect my change CREATED without
touching the line. `"$DS" reg … || echo "WARN: … UNGATED"` predates this task and was survivable
for one reason: a human was reading the transcript and would see the warning. `scheduling.autonomy`
removes that reader. So the same line now means an unregistered document, a Stop hook with no
record, every downstream gate enforcing nothing, and a run that finishes looking complete — the
exact "lie that it's done" the standing rules forbid, arrived at without anyone lying.

Fixed as a fail-closed P6 gate: `set -e`, both `reg` calls, then `status` to confirm before any P7
work. The failures `dstack reg` actually produces are recoverable and each is routed —
another session owning the doc → `reclaim`, a legacy `.fullcycle-active` → `migrate`, an empty
session id → fix the environment. Unresolvable is a human stop, not a warning.

## F002 [medium] the stop taxonomy did not match the reachable states — AGREED, fixed

Both halves are right. INVALID is not a stop; it has a complete internal transition (return to P5
and fix the declaration), and listing it beside "ask the user" told the orchestrator to do two
different things. And the states where the wake mechanism itself is gone were missing entirely,
which is the more serious half — those are exactly the states with no autonomous transition, so
omitting them meant the pipeline would stall silently in the one situation the whole Goal is about.

Now two lists. `internal-recoveries` holds INVALID, a `reg` conflict, and a nonzero external run.
`stops` holds the product/risk choice, the concrete HIGH at closure, an explicit user approval, and
the new one: `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` or a resumed session whose background task was
not restored. Say which, say the pipeline is manual until it is fixed, stop.

## F003 [medium][UX] the notification had no named mechanism — AGREED, fixed

Fair. "Send a push notification" is not an instruction, it is a wish. `autonomy.notify` now names
`PushNotification`, states that delivery is best effort (terminal notification always, mobile push
only with Remote Control connected, and it can report not-sent), and says a non-delivery is neither
retried nor a stop — the work docs are the durable record.

## F004 [medium] "nothing else in that call" was unsatisfiable — AGREED, fixed

Correct, and I wrote both violating recipes myself. `codex-review` Step 2 and `codex-research` Step 2
each do `mktemp -d`, install a trap, and assemble paths in the same background call. The invariant I
actually meant is about what happens AFTER the launch — the call does not return until the external
command finishes, so anything after it is work you are waiting on. Restated as: one background call
whose blocking terminal step is `dstack run`; setup before it is fine, dependent work after it is
not.

## F005 [medium] found by trying to follow my own rule — fixed

`notify` listed "review round sealed" as a branch point and, two lines later, called a per-round
notification noise. Following it for this Goal would have meant roughly twenty notifications across
five units. A sealed round is not a branch point; a closed milestone, a real block, and Goal
completion are.

## Verification

The schema is the thing this change edits, so parsing it is the check — and the first version of
this change broke it, twice, both times a bare `: ` inside a plain multi-line scalar:

```
before fixes: block 1  FAIL  mapping values are not allowed in this context at line 336 column 55
after fixes:  block 0  OK  [pipeline, version, skip, phases]
              block 1  OK  [scheduling]   autonomy=[rule, internal-recoveries, stops, bounds, notify]
                                          waits=[external, external-residuals, user-input]
              block 2  OK  [hook-contract]
bash tests/secret-guard.sh                              ✓ PASS
claude/skills/full-cycle/tests/check-parallel.test.sh     PASS
claude/skills/full-cycle/tests/skill-schema.test.sh       PASS  (FAILS with the break reintroduced)
```

`PushNotification` is present in this session's tool list, which is what "named mechanism" needed;
I did not verify its delivery behaviour, and the file states that behaviour as best-effort rather
than as a guarantee.

## Class-wide sweep (Step 0)

Class: *a rule that was safe only because a human was in the loop*. F001 is the instance; the sweep
is every other place this file degrades gracefully by telling a person something. Found and checked:
the `|| echo WARN` registration (fixed), `waits.user-input`'s honest admission that unregistering
opens a hole in the tripwire (still true, still correct — it is a deliberate manual escape hatch and
now sits under a named stop), and the `honest-scope` note that a `scope` PASS is not write
confinement (a statement of limits, not a human-in-the-loop dependency). Second class, *a rule that
contradicts its own rationale within the same paragraph*: F005, swept the new `autonomy` block and
the prose section against each other.
