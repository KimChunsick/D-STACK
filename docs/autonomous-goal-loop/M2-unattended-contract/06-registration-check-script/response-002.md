# Maintainer response — Round 002 (batch pass 2, closure)

Not bundled. Four highs and two mediums plus two lows, all agreed, all fixed and re-verified against
a fixture repository. Two rounds, seven highs total on ~200 lines — the reviewer is earning its keep
on this file and the reason is worth stating: this script exists because the same check failed five
prose repairs, and moving it into code moved the defects rather than removing them. What changed is
that they are now reproducible.

## F008 [high] the fence toggle ignores fence length — AGREED, fixed, with a named residual

A ```` block legitimately contains ``` lines. The naive toggle flipped on each of them, and measured
on exactly that input it read NEITHER the fenced fake row NOR the real one below — worse than the
bug round 001 fixed. Closers must now match the opener's character, be at least as long, and carry
nothing else.

**The residual is the F024 tension and I am not going to pretend it away.** `check-parallel.sh`
still uses the naive toggle, so on a four-backtick block the two parsers now disagree — which is
what F024 said was worse than one parser being wrong. The reason to fix mine anyway is the
DIRECTION: the scheduler would read the fenced rows as declarations, this checker reads the real
ones, the two sets do not match, and it BLOCKS. One parser failing loudly beats two agreeing on
fiction. The identical fix belongs in `check-parallel.sh` and is a recorded follow-up for the unit
that owns it; until then the divergence is named in the script's own comments.

## F009 [high] granularity by substring — AGREED, fixed

`Review granularity: not task` selected task mode. That is the worst possible reading of the one
line whose job is to fix the depth everything else is checked at. Only the documented `per task` and
`per milestone` values are accepted now.

## F010 [high] producer failures erased before comparison — AGREED, fixed

Round 001 raised this class and I fixed the instances I could see; the reviewer found the ones I
could not. A pipeline reports only its LAST command's status, so `sed … | pad | sort > want` hides a
failing `sed`. A process substitution's status is not observable at all — measured,
`while read …; done < <(exit 7)` leaves the loop at rc 0. The chain is what makes it dangerous: an
erased producer yields an empty set, an empty set yields empty deltas, and empty deltas read as "no
differences found". A crash renders as a PASS. Every stage is now materialised into its own file
with its own status check.

## F011 [high] ownership classified in only one branch — AGREED, fixed

Also round 001's class, also incompletely applied: the closed-unit and wrong-depth branches tested
`! owned`, so a foreign-owned record slipped through both. Absent, ours, and another session's are
three different states with three different fixes, and now every branch says which one it found.

## F012 [medium] the "nothing else" guarantee had a blind spot by construction — AGREED, fixed

Round 001 raised this and I answered by NARROWING THE CLAIM to what the code did — one alternate
depth, files named `task.md`. The reviewer is right that this was the weaker move: `<goal>/<Mn>/note.md`
registered to this session was a gate over a document no phase governs, and nothing would ever see
it. Now enumerated from the registry and subtracted from the allowed set, which has no blind spot at
all. A real check beats an accurate disclaimer.

## F013 [medium] the milestone bijection was enforced at neither end — AGREED, fixed

`###M1oops` and `M1oops/task.md` both yielded id 1. A heading boundary is required after `M<n>`, and
the folder must be `M[0-9]+-<non-empty-slug>`.

## F014 / F015 [low] — AGREED, both fixed

The Goal directory is canonicalised to the repo-relative spelling the registry stores, so `docs/g`,
`./docs/g`, `docs/g/` and an absolute path all check the same thing instead of reporting every
document unregistered — four spellings verified identical. And the success line stopped claiming
"all units owned by this session" immediately after a branch that requires closed units to be
UNregistered; it reports scaffolded, open-and-registered, and what was checked.

Consensus: resolved
