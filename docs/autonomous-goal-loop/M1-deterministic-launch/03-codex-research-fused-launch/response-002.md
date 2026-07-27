# Maintainer response — Round 002

Not bundled into any review round.

## F006 [medium] "CATCHABLE termination" was still wider than the trap set — AGREED, fixed

Round 001 narrowed this from "the round dies with the supervisor" to "any catchable termination",
and I recorded that as closed. It was not: `RUN_SIGNALS` is `INT TERM HUP QUIT PIPE ALRM USR1 USR2`,
and `ABRT`, `XCPU`, `XFSZ` are catchable and absent from it. So the sentence still promised more
than the launcher delivers, one narrowing later. It now names the actual set, names the three
signals it excludes, and keeps `SIGKILL` as the untrappable case.

I did not widen `RUN_SIGNALS`. That is `claude/bin/dstack`, which is T01's declaration, not this
task's — and §1's allowlist rule forbids pulling a file in to absorb a finding. Recorded as a
follow-up for its own review unit, named in the file.

## F007 [medium][security] `<goal>`/`<topic>` path traversal — AGREED, fixed

The strongest finding in the round and the one I would not have found: the recipe is a template
whose placeholders I substitute by hand, so I read them as constants. They are not — they are
interpolated into `mkdir -p`, into the `--stdin` path, and into `-o`'s absolute path, and
`TOPIC=../../AGENTS` puts the model's last message on top of a tracked repository file. Quoting was
doing nothing about it, and `dstack`'s label check runs after the damage.

Both are now validated against a plain-slug grammar before the first filesystem operation:

```
'autonomous-goal-loop'      accept
'background-task-lifetime'  accept
'../../../AGENTS'           REFUSE     <- the reviewer's counterexample
'..'                        REFUSE
'.'                         REFUSE
''                          REFUSE
'a/b'                       REFUSE
'.hidden'                   REFUSE
'x;rm'                      REFUSE
```

## F008 [low] the E2E record overstated its own evidence — AGREED, fixed

Two overstatements in a paragraph I wrote to fix an overstatement, which is the pattern this unit
keeps producing:

- "the exact block, **unedited**" — the `<goal>`/`<topic>` placeholders were substituted. The
  record now says exactly that.
- "**33 cited sources**" — I ran `grep -c 'https\?://'` over the whole document, which counts
  inline citations. The `## Sources` section has 13 entries: 12 unique URLs and one local
  installed-CLI artifact. Verified by counting the section rather than the document. Corrected in
  `task.md`, in `GOAL.md`, and in `response-001.md`, all three of which carried the wrong number.

The reviewer also noted that the capture proves the child invocation but not the wrapper's own
`set -u`, trap, or backgrounding. True — `cmd` records the launched command. Those are now labelled
as run-time observation rather than presented as recorded evidence.

## F009 [low] `-o` is not the only repository write — AGREED, fixed

`dstack run` writes `.dstack/runs/<sid>/<label>/{cmd,out.txt,err.txt,exit,.launch/…}` under the
repository on every invocation. "The one deliberate repository write" was a sentence about the
Codex sandbox that overreached into a claim about the whole invocation. Now: two deliberate
writers, and the sandbox constrains the model rather than the harness around it.

## F010 [low][security] the evaluator-directive fix was incomplete — AGREED, fixed

Correct, and it is the funnier version of round 001's finding: my repair sentence ended "stated as
filing information, not as a scope instruction to a reviewer", which is itself a sentence addressed
to the reviewer telling it how to read the document. Removed. The Deployment context now states
what the change is and what file it touched, and stops there.

## Class-wide sweep (Step 0)

Class: *a claim narrowed once and still wider than its evidence*. F006, F008 and F009 are all that
shape — each had already been "fixed" in round 001. Swept every remaining guarantee in the file
against the thing it describes: the trap set (checked against `RUN_SIGNALS` in `dstack`), the
verification claim (checked against what `cmd` actually stores), the write claim (checked against
what `dstack run` creates), the source count (counted from the section), and the nonzero-exit rule
(unchanged, still accurate). Also swept the sibling `codex-review/SKILL.md`, which carries the same
teardown sentence — it is inside an OPEN round 003 bundle and therefore frozen, so the identical
fix is queued for its round 004 rather than made mid-round.
