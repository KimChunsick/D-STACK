# Maintainer response — Round 002

Not bundled into any review round.

## F006 [high] catchable-signal gaps at both ends of the teardown — AGREED, fixed

Correct on every point, including the one about my own evidence. Round 001's repair covered only
the interval it could see, and the test that "verified" it signalled *after* `.launch/child`
existed — so it structurally could not reach the window the finding names. Three distinct gaps:

1. **Trap installed after the fork.** A signal between the launch and the `trap` line took the
   default disposition: supervisor dead, child orphaned.
2. **A failed child-pid write called `die`,** which exits without tearing anything down — leaving
   the child running with nothing left to record it.
3. **Trap cleared before publication.** `trap - INT TERM HUP` ran ahead of the `exit` write, so a
   signal in between killed the supervisor with no terminal record at all.

Fixed as directed — cleanup ownership is established before the fork and held through publication:

- `trap 'run_abort' INT TERM HUP` now goes up **before** `set -m` and the launch.
- The child-pid write failure calls `run_abort`, never `die`.
- Publication happens **first**, `trap -` second.
- `run_abort` disarms re-entry, signals the child's process group, reaps it, publishes the real
  status, and exits 6.

**The instruction-level window, and why it is not left as "unknown".** Between `&` returning and
`child=$!` being assigned, `child` is empty. My first repair published nothing there and relied on
`rm-run` refusing the capture — which satisfied the finding but stranded every aborted setup until
the retention sweep, and the racing test produced exactly that. The shell sets **`$!` as part of
executing the background command itself**, so inside that window `$!` is already correct while
`child` is not. `run_abort` now falls back to it, which separates the two cases outright: a process
exists → tear it down and publish; none ever did → release the claim, because nothing ran.
(Reading an unset `$!` is fatal under `set -u`, so that read is bracketed by `set +u`/`set -u` —
a crash inside a signal handler would be the opposite of a report.)

**`rm-run` treats a missing child record as unknown**, as suggested. That is now cheap rather than
punitive, because the ordinary "nothing was launched" case no longer leaves a claim behind at all:
every failure between the claim and the fork goes through `claim_release_and_die`, which removes
`.launch` — the same precedent `run-dir` already sets when it cannot set a directory's mode.

Class-wide sweep (Step 0): the defect class is "a protective window that does not span the thing it
protects". Swept every ownership transition in `cmd_run` — claim→supervisor record, supervisor
record→reserved-name checks, capture-file creation→cmd record, fork→pid record, wait→publish,
publish→disarm. The three above were the open ones; the rest either precede the fork (now
claim-releasing) or are covered by the trap.

### Verification — the window the previous test could not reach

TERM straddling the fork, 20 samples with the delay stepped so the signal lands at different
points after the claim:

```
stray '/bin/sleep 41' processes                          : 0   (want 0)
aborted with a published terminal record (child existed) : 20
aborted with the claim released (nothing forked)         : 0
STUCK (claim held, no terminal record)                   : 0   (want 0)
```

An earlier 16-sample run with a much shorter delay produced 0 strays as well, but every sample
landed before the fork — which is why it is reported here as covering the pre-fork case only, not
as evidence for this finding. Saying otherwise would repeat exactly the mistake round 002 caught.

Regression after the change:

```
normal completion            → DONE n-ok exit=0, status 0
failing child (exit 7)       → dstack status 6, exit file [7]
TERM after the pid record    → child alive after: no, exit file [143]
pre-fork failure             → refusal message, and NO .launch left behind (label reusable)
rm-run vs a live claim       → refuses, naming the process
bash -n / secret-guard       → OK / ✓ PASS
```

## Carried decisions

- Cleanup ownership spans fork→publication. Do not narrow it at either end again; both ends were
  found open once already.
- `$!` is the tiebreaker for the fork→assignment window. It is what makes "nothing was launched"
  distinguishable from "a child exists but is unrecorded", so a claim is released in the first case
  and never in the second.
- Every pre-fork failure releases the claim; every post-fork path keeps it. That asymmetry is what
  lets `rm-run` be fail-closed on unknown state without stranding routine failures.
- Round 001's carried decisions all still stand (see `carried-001.md`).
