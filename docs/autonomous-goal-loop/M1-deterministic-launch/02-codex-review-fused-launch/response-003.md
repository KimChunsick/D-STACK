# Maintainer response — Round 003

Not bundled into any review round. The measurements below are also in `## Carried decisions` of
`codex-review-003.md`, because a disproof the next round cannot see gets re-raised — which is
finding F011 of this very round.

## F011 [medium] a disproof that lives only in the response is invisible — AGREED, fixed

The finding is right and it is about a hole I made in round 001. §2 moved the maintainer response
out of the corpus to stop prose compounding, and that is still correct for a FIXED finding, where
the diff is the evidence. It is wrong for a DISPROVED one: the measurement exists only in a file the
reviewer is never sent, so the finding comes back and gets argued twice. §2 now says a disproof and
its number go into `## Carried decisions` — a bounded channel that compacts — while the argument
stays in the response.

## F012 [medium] the fourth disposition contradicts the Codex-side contract — AGREED it is a real
contradiction; NOT resolved the suggested way

I checked `codex/skills/adversarial-review/SKILL.md` rather than taking the citation on trust, and
it says both things the finding claims: "One invocation/rebuttal exchange is one immutable English
file", and consensus as fixed / disproved / user-disposed. So the two files genuinely disagree, on
round-file shape AND on dispositions.

I did not take the suggested direction of requiring user disposition for every remaining concrete
medium. That would reintroduce the human stop this Goal's interview removed after P4 and would
contradict §4 rather than repair it. What I did instead: `codex-review/SKILL.md` now states
explicitly that for this pipeline's closure semantics IT governs, that the Codex-side contract needs
the same two edits, and that the edit is a follow-up for its own review unit because that file is
not in this declaration. A reviewer filing the disagreement again is reporting something true.

## F013 [medium] the teardown guarantee — AGREED it was wrong, but NOT for the stated reason

This is the one where I checked the reviewer's own verification and it did not reproduce. The
finding says `/bin/bash -c 'trap "printf EXIT-TRAP" EXIT; kill -XCPU $$'` exits 152 *without*
running the EXIT trap. Measured here, three runs each:

```
/bin/bash 3.2.57   TERM rc=143 [T]  ABRT rc=134 [T]  XCPU rc=152 [T]  XFSZ rc=153 [T]  VTALRM rc=154 [T]  PROF rc=155 []
/bin/zsh  5.9      TERM rc=143 [ ]  ABRT rc=134 [ ]  XCPU rc=152 [ ]  XFSZ rc=153 [ ]  VTALRM rc=154 [ ]  PROF rc=155 []
```

bash runs the EXIT trap on fatal signals; zsh never does. `dstack` has a `/bin/bash` shebang, so
`run_cleanup` DOES run for XCPU, XFSZ, VTALRM and ABRT despite their absence from `RUN_SIGNALS`.
The real gaps are exactly two: `SIGKILL` and `SIGPROF`.

So the sentence was wrong — as was my round-002-era "any catchable termination", and as was the
round-003 claim. All three are replaced by the table, in the file. Covering `PROF` is a `dstack`
change and is a follow-up. The shell difference also lands somewhere concrete: the recipe fence runs
under zsh, where an EXIT-only trap is silent, which is F016 below.

## F014 [medium] the ratchet was exempted in one place and enforced in another — AGREED, fixed

§1 grew a round-004 qualification in round 002 and I did not carry it to the closing procedure,
which still said "rebuild the bundle no larger than the last one" after every rejecting round. Both
now carry the same qualification. This is the F007 class again — one rule, several sites, fixed in
some.

## F015 [medium][security] the evaluator directive, third costume — AGREED, fixed

Round 001 found "Out of scope: …". Round 002 found the same sentence still present. Round 003 finds
that my *rewrite* — which named the assembler and said to treat that as filing information — is
itself the directive, because telling the evaluator how to classify something is the thing. The
Deployment context is now a changed-files line and nothing more.

## F016 [low] the scratch trap does not survive a signal — AGREED, fixed, with a correction

Agreed on the conclusion; the stated verification is half wrong, in the direction that matters
here. Both shells exit 143 on self-TERM, but bash *does* run the EXIT trap and zsh does not. The
recipes run under zsh, so the leak is real. `trap 'rm -rf "$SCRATCH"' EXIT INT TERM HUP` — verified
firing in both shells (`CLEANUP-RAN`, rc=0).

## §4's counter was ambiguous, and this unit is the reason I noticed

Not a reviewer finding. Applying §4 to this unit's own numbers — 3, 4, 5 new blocking findings —
says the loop is non-convergent and must close, with five concrete mediums open, while every one of
them is a real defect I can fix in an hour. That reading is wrong, and it is wrong because the
counter never said *which* count. §4 now says: the number still OPEN at the end of the round, after
that round's fixes, and the test applies from round 004. Counting what a round *raised* closes
loops that are going well, because a document under active repair generates new findings at a rate
unrelated to how close it is to done.

Under the corrected counter this unit's open count is 3 → 4 → 0: everything from round 003 is fixed
above.

## Class-wide sweep (Step 0)

Class: *a claim I adopted from a review without measuring it*. F013 and F016 are both that, in the
same round, and the fix in each case was to run the thing. Swept every remaining empirical claim in
the file: the bash/zsh trap behaviour (measured, table above), `FULL_ROUNDS=2` and when compaction
starts (read from the assembler, confirmed against a real manifest), `assemble-review.sh`'s exit
status on a skip (read: it `return`s, status never reflects a skip), and the skip-marker matching
(reproduced both ways). Second class, *one rule stated in several places*: F014 is another instance,
so I re-swept the ratchet, the response-file rule, and the closure rule across every site.
