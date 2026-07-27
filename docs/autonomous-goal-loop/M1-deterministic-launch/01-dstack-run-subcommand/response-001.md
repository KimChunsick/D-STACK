# Maintainer response — Round 001

Not bundled into any review round (see `codex-review` §2: the reviewer learns what changed from
the diff, which is ground truth; prose about what changed is what compounds).

## F001 [high] supervisor termination leaves an orphaned child — AGREED, fixed

The reviewer is right and this was the worst kind of error in the change: `task.md` and the code
comment both ASSERTED "the child dies with the supervisor" as an accepted residual. That statement
was false. Signalling only the `dstack` process leaves `codex exec` running, orphaned and
invisible — exactly the failure that dropping the detached launcher was supposed to remove. The
reviewer reproduced it (`foreground_child_alive_after_supervisor_term=yes`).

Fixed as directed. `cmd_run` now runs the child under `set -m`, so it gets its own process group;
records the child pid in the launch claim; and traps `INT TERM HUP` to signal that whole group
(`kill -TERM "-$child"`, falling back to the bare pid if job control did not take effect). Because
a trapped signal interrupts `wait` *without* reaping, the wait is a loop that exits only once the
child is genuinely gone.

`SIGKILL` cannot be trapped, so an orphan remains possible. That residual is now covered rather
than asserted away: `rm-run` consults BOTH recorded pids, so a hard-killed supervisor's still-live
child keeps its capture from being deleted out from under it.

Class-wide sweep (Step 0): the same "a claim about process lifetime that was never executed"
pattern was checked across the change. `rm-run`'s guard was the only other site asserting
liveness, and it is the one repaired above. `prune` is addressed under F005.

Verified by direct run:
```
supervisor=74995 launched=75054 ; alive before TERM: yes
launched alive AFTER supervisor TERM: no        # the reviewer's reproduction no longer reproduces
DONE x-term exit=143 …   supervisor final status=6   exit file=[143]

# SIGKILL path:
orphan alive after supervisor KILL: yes
dstack: capture 'x-kill' has an orphaned launched process (pid 75119) still running and no
        terminal record — refusing to delete it; stop that process first
capture still present: yes
(after stopping the orphan)  removed capture: x-kill
```

## F002 [medium] empty label collapses the capture path — AGREED, fixed

Correct, and the reviewer's account of why it was missed is exact: `run-dir` never saw an empty
label because it defaults its argument, while `run` passes the caller's string straight through,
and none of `require_label`'s patterns matches the empty string. `require_label` now rejects `''`
first, with a comment naming that asymmetry so it is not reintroduced.

```
$ dstack run "" -- /bin/echo hi
dstack: run label must not be empty
```

## F003 [medium] adoption does not establish reserved-path invariants — AGREED, fixed

Both sub-cases confirmed. The check was `-L` only, so a pre-existing regular `exit` disabled
`rm-run`'s guard for the whole run, and an `exit` *directory* made `mv -f exit.tmp exit` succeed by
nesting — `DONE` reported while the promised status file did not exist.

Fixed more strictly than suggested: every reserved name (`out.txt`, `err.txt`, `exit`, `exit.tmp`,
`cmd`) must be **absent**, not merely non-symlink. An adopted directory holds only material the
caller assembled; those five names are ours to create, so their presence means the label was
already used.

```
$ dstack run x-exitfile -- /bin/echo hi     # dir contains a regular `exit`
dstack: '…/x-exitfile/exit' already exists — an adopted capture directory must not contain the
        names this command publishes; use the next label
$ dstack run x-exitdir  -- /bin/echo hi     # dir contains `exit/`
dstack: '…/x-exitdir/exit' already exists — …
```

## F004 [medium, the real Why] the long output-silent E2E was never exercised — REBUTTED with evidence

Accurate when the bundle was assembled, and no longer true: **round 001 itself is that E2E.** It
was assembled at 23:58:40 and launched at 23:59:10 as a single `dstack run` under one harness
background call, with no watcher armed anywhere. It ran **10.3 minutes**, during which the harness
observed **zero bytes** of output from the command — `cmd_run` redirects everything into the
capture (`out.txt` 4185 bytes, `err.txt` 179599 bytes, both invisible to the harness). Its
completion notification then re-invoked the session with no human input, carrying `exit code 0`,
and `DONE t01-r1 exit=0 dir=…` was the whole of the command's output.

That answers both halves of the finding: long, and output-silent. The reviewer could not see it
because the evidence is produced by the very round being reviewed.

Two corroborating details. The foreground Bash tool caps at a 10-minute timeout, so a 10.3-minute
round could not have completed as a foreground call at all — the background path is not an
optimization here, it is the only one that finishes. And probe L (25 minutes, four intervening turn
boundaries) bounds the duration question above the 15–25 minute range the finding names, even
though the reviewer is right that probe L was not itself output-silent.

The reviewer's documentation point stands and is recorded as a live dependency, not dismissed:
completion re-invocation is not a documented platform guarantee. It is a measured local behaviour
of client 2.1.220, now observed six times in this session. The code comment names it as the thing
to re-check if it ever regresses.

## F005 [low] the disposition overclaims `prune` — AGREED, wording corrected

Right: only `rm-run` was changed. `prune` still selects purely by mtime. The `task.md` disposition
row said "`rm-run`/`prune`" and now says `rm-run`, with the reason `prune` is left alone stated
where the claim is made: its threshold is eight complete days against runs of 3–25 minutes, so a
capture it selects cannot plausibly be live. Recorded as a non-blocking follow-up rather than a
change, per the reviewer's own assessment that ordinary exposure is unlikely.

## Carried decisions

- Not detaching is settled (design consult + F001's repair). The residual is SIGKILL-only, and it
  is covered by `rm-run` refusing to delete a capture with a live recorded pid — not by asserting
  the orphan cannot happen.
- Completion re-invocation on background-command exit is a MEASURED local behaviour of client
  2.1.220, not a documented guarantee. It is the change's load-bearing external dependency.
- `prune` deliberately does not consult launch state; the eight-day threshold is the argument.
  Non-blocking follow-up if that threshold ever narrows.
