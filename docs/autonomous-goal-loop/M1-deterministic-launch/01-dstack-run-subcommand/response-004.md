# Maintainer response — Round 004

Not bundled into any review round.

## F011 [high] quiescence was a warning, not a gate — AGREED, fixed

Exactly right, and it undid the point of F007's repair. `run_group_settle "$child" || printf
WARNING` was followed unconditionally by `run_publish`, and an `exit` file is precisely what makes
a capture terminal — after which `rm-run` stops guarding it. So a group that outlived SIGKILL got a
terminal record *and* deletion permission for the directory it was still writing into.

Now: publication happens only inside the `run_group_gone` branch, on both the normal path and the
abort path. When the group will not die, nothing is published, a loud ERROR names the group, and
the capture stays nonterminal — which keeps `rm-run` refusing it. `run_done` replaces
`run_published` so the two finalisation outcomes (published / refused to publish) are one state,
and the EXIT handler cannot re-enter and undo the refusal.

Verified by fault injection — `run_group_gone` forced to report the group alive forever:

```
dstack: the command exited 0 but its process group 97561 survived SIGKILL — refusing to publish a
        terminal record while something may still be writing into …/q1; stop it, then remove the
        capture by hand
exit file present: no        ← capture stays nonterminal, so rm-run keeps guarding it
```

## F010 [high] EXIT cleanup loses its locals — REBUTTED with a direct reproduction of the named path

The mechanism is not what happens on the deployed interpreter, and the interpreter is not in doubt:
this file's shebang is `#!/bin/bash`, and on this machine both `/bin/bash` and the `bash` on PATH
are **GNU bash 3.2.57(1)-release (arm64-apple-darwin25)** — the same 3.2.57 the finding names. So
the recorded evidence and production run under one shell, and the round-003 methodology error
(testing something other than the real thing) is not repeating here.

Measured, twice, on that shell:

```
# 1. Does an EXIT trap see cmd_run's locals when `die` is called from a NESTED function?
#    (require_plain → die is exactly this shape.)
died: from a nested function
  trap: run_published=[0] d=[/tmp/x] child=[]        ← all readable, no unbound variable

# 2. The exact path the finding says is broken, fault-injected into the REAL script:
#    the post-fork child-record write forced to fail.
dstack: cannot record the launched pid at …/fault1/.launch/child
dstack: run fault1 aborted — launched process group torn down, recorded exit 143
  exit file=[143]   stray '/bin/sleep 46': 0
```

The second is the finding's own claim — "a child-record write failure after the fork exits without
signalling the launched group, recreating the orphan path" — run against the code as shipped. The
group was signalled, the status was published, nothing was orphaned. Bash runs an EXIT trap
triggered by `exit` from inside a function while that function's locals are still readable; the
unwinding the finding describes applies when the function has *returned* first, which no path here
does (the only normal return happens after the trap is disarmed).

**The suggestion is taken anyway.** Every read in `run_cleanup` is now defaulted (`${child-}`,
`${run_done-0}`, `${label-?}`, `${d-?}`), because a handler that *can* die on an unbound variable
is one refactor away from doing so and defaulting costs nothing. One real bug surfaced while making
that change: an unguarded `rm -rf "${d-}/.launch"` would resolve to `/.launch` if `d` were ever
unset, so the claim release is now guarded on `d` being non-empty. That is a genuine improvement
this finding produced, even though its stated mechanism does not hold.

## F012 [low] pgid recycling during settlement — ACCEPTED RESIDUAL, recorded in the code

Real, and recorded rather than closed — which is the disposition the finding itself offered. A pgid
carries no ownership token, so if the group empties and the id is recycled between the liveness
probe and the signal, the signal lands elsewhere. The window requires the group to be fully gone
first (while any descendant lives the id cannot be reused), so it is the probe-to-signal instant
only. Pinning an owned identity through settlement is more machinery than a single-user tool
warrants. The residual is written where the signalling happens.

## Class-wide sweep (Step 0)

Class: *a guard that reports but does not gate*. Swept every place this change decides something on
a predicate — `run_group_settle` at both call sites (was report-only, now gates), `run_publish`
failure (already fatal), the reserved-name check (already fatal), `rm-run`'s two liveness checks
(already fatal), and the `--stdin` checks (already fatal). F011 was the only report-only one.

## Carried decisions

- Publication is gated on confirmed group quiescence at BOTH call sites. A group that survives
  SIGKILL yields no terminal record, on purpose: nonterminal is what keeps `rm-run` guarding.
- `run_done` is the single finalisation state — published, or deliberately not published.
- The deployed interpreter is /bin/bash 3.2.57, and it runs an EXIT trap fired by `exit`-from-
  inside-a-function with that function's locals readable. Verified directly, including from a
  nested `die`. `run_cleanup` still defaults every read.
- pgid recycling during settlement is an accepted residual, recorded in the code.
- Rounds 001–003 carried decisions all still stand.
