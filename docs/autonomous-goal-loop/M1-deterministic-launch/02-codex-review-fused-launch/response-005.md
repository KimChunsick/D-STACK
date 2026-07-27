# Maintainer response — Round 005

Not bundled into any review round. This is the §4 round cap for a per-task unit and the loop closes
here. Measurements are also in `## Carried decisions` of `codex-review-005.md`.

## F022 [medium] the response file is neither sealed nor bundled — RAISED A THIRD TIME, disposition
unchanged

Rounds 003, 004 and 005 have each raised the disagreement with
`codex/skills/adversarial-review/SKILL.md`. Nothing in this round's version is new: the same two
clauses, the same suggested direction. My answer is on the record in `response-003.md` and has not
changed — this file governs the pipeline's closure semantics, the Codex-side contract needs the same
two edits, that file is outside this unit's declaration, and the ratchet rule forbids pulling it in
to absorb a finding. §3 says a restatement is recorded and does not reopen. Recorded, with the
follow-up naming the file.

I will say the uncomfortable part plainly: the reviewer is correct that the two documents disagree,
and it will keep being correct until the Codex-side file is edited. That is a real outstanding
inconsistency in the repository, not a resolved one, and it is named as such in `findings.md` rather
than dressed up as agreement.

## F023 [medium] disposition 4, and the §3 regression exemption — SPLIT: half rejected, half AGREED
and it was the important half

The disposition-4 half is F008/F012/F018 again and my answer stands.

The other half I had not seen, and it is the finding of the round. §3 said a variant of an
already-recorded class **in code a fix just introduced** does not reopen. Read literally, that is a
licence to ship the defect your own repair created, on the grounds that you had already written the
class down. It also contradicts, in the same section, the rule two paragraphs above it — "discovery
time never changes a finding's blocking status".

And the reviewer did not argue it abstractly; it produced the instance in the same round. The `awk
-v` backslash bug below is a variant of a recorded class (the skip gate's matcher), in code my
round-004 fix had just introduced, and under the old §3 I could have declined to reopen it while it
silently let a skipped file into a review. Removed. A regression introduced by a fix always reopens;
the exemption now covers only a restatement about code that has not moved.

## F024 [medium] `awk -v` is not byte-literal — AGREED, fixed

Third defect in this one check, and the third time the answer was "the matcher is not what I thought
it was". `-v` decodes backslash escapes before the program sees the value:

```
              awk -v        ENVIRON
path\to       MISSED        REFUSE
path\new      MISSED        REFUSE
plain/path    REFUSE        REFUSE
real bundle   PASS          PASS
```

Now `P_MARKER="--- $f (" awk '… index($0,ENVIRON["P_MARKER"])==1 …'`. I took the reviewer's suggested
direction verbatim; `ENVIRON` is the byte-preserving channel and there was no reason to invent
another.

## F025 [medium][DX] the handlers do not cancel `dstack` — AGREED as a LIMIT, stated rather than
implemented

Correct, and it is the same measurement T03's round 005 produced. Both shells defer a pending trap
while a foreground command runs, so the handler fires only after `dstack run` returns — five-second
child, `rc=143` after five seconds.

I did not implement forwarding. What the wrapper would have to do is background `dstack run`, track
its pid, and signal it — which rebuilds inside a recipe the supervision `dstack run` already does,
and the last three rounds are a long argument against adding mechanism I have not measured. Instead
the file now states three things it previously implied wrongly: the handlers cancel nothing;
`<run-dir>/exit` is the round's status because a signalled wrapper can report 143 over a completed
round; and the handlers must not clean up, since the same deferral has them deleting a live child's
scratch directory. To stop a round in flight, stop the recorded process group.

## F026 [low] the printed probe is misquoted — AGREED, fixed

Identical to T03's F021 and I fixed it there and not here, which is the "fixed in one sibling"
class again. `"… kill -<sig> $$ …"` has its `$$` expanded by the invoking shell. Now single-quoted
with the signal name as an argument; the corrected form reproduces the table
(`TERM rc=143 [T]`, `XCPU rc=152 [T]`, `PROF rc=155 []`).

## Closure (§4 round cap)

Five rounds. **Open concrete findings at close: 0** — F024, F025, F026 and the §3 half of F023 are
fixed; F022 and the disposition half of F023 are restatements with a recorded disposition and a
named follow-up.

Raised per round ran 3, 4, 5, 3, 5. It did not decay, and the ledger says why: the skip gate took
four attempts, the launch invariant two, and three separate rounds landed on a rule that was fixed
in this file but not in the sibling it invokes. Both structural causes are now closed — the sibling
alignment by making `waits.external` the single source, and the fix-introduces-the-next-defect
pattern by removing §3's exemption for exactly that case.

Follow-ups recorded against files outside this declaration: `codex/skills/adversarial-review/SKILL.md`
(two clauses), `assemble-review.sh` (a skip channel the payload cannot write), `claude/bin/dstack`
(`SIGPROF`, and the fork-to-pid-record window).

Sealed `Consensus: resolved`.

## Class-wide sweep (Step 0)

Class: *the matcher is not what I thought it was*. Four instances now in one check — bare substring,
anchored regex, two-grep substring, `awk -v` escape decoding. The sweep this round was to stop
guessing and run each candidate against a hostile input before writing it down, which is how the
`ENVIRON` form was chosen rather than argued for. Second class, *a rule fixed here and not in the
sibling*: swept every shared rule between this file, `codex-research` and `full-cycle` — launch
invariant, signal-handler form, capture-is-the-status, the printed probe, and the teardown table.
All five now agree.
