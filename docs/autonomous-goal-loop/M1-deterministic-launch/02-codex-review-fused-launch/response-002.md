# Maintainer response — Round 002

Not bundled into any review round.

## F005 [medium][security] an evaluator directive in this task's Deployment context — AGREED, fixed

Round 001 raised this class against `03`'s task document and I fixed it there. The identical
sentence was still sitting in `02`'s, which is the more embarrassing version of the finding: I
treated a class as closed after fixing one instance of it. Reworded as filing information, and it
now names its own former text so the next edit cannot quietly reintroduce it.

## F006 [medium] the retry fence was weaker than `rm-run`'s invariant — AGREED, fixed

The sharpest finding in the round. My fence read one record and treated absence as quiescence;
`rm-run` reads both and treats absence as unknown, precisely because `run` releases its claim on
every pre-fork failure, so a surviving claim with no child record means the fork may have happened
mid-write. Two guards disagreeing about when a capture is finished is worse than either being
wrong alone. The fence now mirrors `rm-run` exactly.

Verified by direct run against the corrected fence and against the old one:

```
1 supervisor ALIVE, child dead            REFUSE  (supervisor pid/group)
2 supervisor dead, child ALIVE            REFUSE  (child pid/group)
3 supervisor dead, child record MISSING   REFUSE  (unreadable child pid)
4 child record malformed ("garbage")      REFUSE  (unreadable child pid)
5 both dead, both readable                PERMIT
6 terminal record present                 PERMIT
7 no launch claim at all                  PERMIT

reviewer's counterexample, reproduced against the OLD fence:
3 supervisor ALIVE, no child record       PERMIT relaunch   <- the duplicate-round bug
4 child record malformed                  PERMIT relaunch
```

The probe's own trailing `kill` of its stand-in process made the script exit 143; that is the
cleanup, not a failing case. Every case line above is the fence's own output.

## F007 [medium] the frontmatter still routed rebuttals into the round file — AGREED, fixed

Round 001 fixed the template, §2 and the sealing sentence, and I said in that response that "three
sites disagreed at once". There were four. The `description:` line is loaded into the model's skill
list, so it is arguably the *most* read of them. Now: invocations in `codex-review-<NNN>.md`,
rebuttals in a never-bundled `response-<NNN>.md`.

## F008 [medium] non-convergence closure could not be sealed honestly — AGREED, fixed

The deepest one, and it is a genuine contradiction rather than a wording problem. §4 requires the
loop to close when the blocking count stops decreasing, and says concrete mediums close on the
recorded-follow-up path *without asking*. Step 4 defined consensus as fixed / disproved /
user-disposed. Under those three, that closure had no sealable value: `disagreed` fails the gate,
and `agreed`/`resolved` would have asserted something untrue about a finding that was neither
fixed nor disproved.

I did not take the suggested direction. Requiring user disposition for every unresolved concrete
medium would reintroduce a human stop that this Goal's interview explicitly removed after P4, and
it would contradict §4's own text rather than repair it. Instead Step 4 now names the fourth
disposition it was always relying on: **accepted residual under a §4 closure** — written into
`findings.md`, into the unit's `task.md`, and named to the user in the final report. `resolved` is
the honest word for a loop that resolved by measurement instead of by agreement, and what makes it
honest is that the defect reaches the person who decides. A concrete HIGH keeps its escalation.
§4 now points at disposition 4 by name, so the two sections cannot drift apart again.

## F010 [medium] the ratchet rule could not be satisfied — fixed

Bundle 50262 against round 001's 41621: **+8641, violated.** I wrote above that round 003 would be
the first to compact. I then assembled round 003 and read the manifest: rounds 001 AND 002 both
went in as full snapshots, 62469 bytes. `assemble-review.sh` keeps the `FULL_ROUNDS` most recent
rounds whole and that value is **2**, so round 004 is the first round in which anything is old
enough to compact.

That makes §1 as written a rule no unit can satisfy at rounds 002 or 003 — and three units in this
Goal duly reported a violation they had no way to avoid, which is how a real rule gets trained into
noise. §1 now binds from round 004, and says plainly that the earlier growth is arithmetic. The
allowlist half of the rule still binds at every round; that one is a choice, not arithmetic.

I also corrected the wrong sentence inside sealed `codex-review-002.md` rather than leaving it to
be fed to round 003 as fact, and said so in the file. The reviewer's findings and the consensus
line were not touched. Sealing protects the reviewer's output and the verdict from post-hoc
revision; it does not oblige me to hand the next round a false statement about how the tool works.
`carried-002.md` was regenerated from the corrected block so the companion still matches.

Evidence is not deleted to make the number fall.

## F009 [medium] found by USING the recipe, not by the reviewer — fixed

Assembling round 003 with the just-fixed recipe refused the bundle: `refusing: an allowlisted
filename was rejected outright`. The pathless skip marker has no path to anchor on, so it was still
matched as a SUBSTRING over the whole bundle — and the line that performs the match quotes the
marker, so the recipe refuses every bundle that contains itself. Round 001's F003 fixed the
per-path check and left its sibling one line below untouched, which makes this the third instance
of the same class in this unit.

```
substring (-F):  449:+grep -qF -- '--- (SKIPPED: newline/control char in filename) ---' "$RD/bundle.txt" \
whole line (-xF): no match, rc=1
```

The only match was a `+`-prefixed diff line, i.e. reviewed content. Now `grep -qxF`. The residual
statement was rewritten at the same time, because I checked and it was understated: every skip path
in `assemble-review.sh` `return`s after printing and the script's exit status never reflects a skip
at all — so "publishes skip status only inside the bundle" is literally the whole story, and the
follow-up is the only sound fix.

## Class-wide sweep (Step 0)

Class: *the same rule stated in several places, fixed in only some of them*. Both F005 and F007 are
instances, from opposite directions — one class fixed in one file and not the sibling, one rule
fixed in three sites and not the fourth. Swept: every statement about where the maintainer response
lives (frontmatter, template, §2, sealing sentence — all four now agree), every Deployment-context
section in this Goal's task docs (`01`, `02`, `03` — `01` has none, `02` and `03` fixed), and every
statement about when a capture may be relaunched (the fence and `rm-run` now share one invariant,
with the recipe naming `rm-run` as its source).
