# Maintainer response — Round 001 (batch pass 1)

Not bundled. Every finding agreed. Four highs on ~180 lines of new code, and the round is right
about all four — moving the check out of prose moved the defects rather than removing them. What
changed is that they are reproducible now, and every fix below was demonstrated against a fixture
repository before and after.

## F001 [high] fence context parsed incorrectly — AGREED, fixed

The script tracked fences only after entering the `## Milestones & tasks` section.
`check-parallel.sh` tracks them globally from line one, and the difference is not a nicety. Fixture:
a GOAL.md whose research section quotes a fenced decomposition example. The old parser skipped the
opening backticks (not in the section yet), took the fenced `## Milestones & tasks` as the real
heading, read the fenced `Review granularity: **per milestone**` and both fake `T91`/`T92` rows,
then hit the CLOSING backticks and toggled the fence ON — so the three real task rows below were
discarded as fenced. It reported task-count 2, granularity milestone, and both numbers were fiction.

Rewritten to mirror `check-parallel.sh` line for line: `^[[:space:]]*` + backticks toggles before
any section test, task rows accepted only at column zero with the `-` marker, a repeated section
heading keeping the section open. Same fixture: 3 rows, granularity task.

## F002 [high] review-unit identity is lossy — AGREED, fixed

Three separate losses in one pass.

- **Milestone mode compared counts.** Fixture: declared M1 and M2, scaffolded M1 and M3. Two and
  two, so it passed. Now it compares identities in both directions and blocks with "declared but not
  scaffolded: 002 / scaffolded but not declared: 003".
- **Malformed paths were silently discarded.** A folder that carries no readable id is exactly the
  case a check like this exists for, and dropping it made it invisible. Reported now, with the shape
  it expected.
- **`uniq` collapsed duplicate prefixes.** Fixture: 03 scaffolded twice, 02 absent. Deduped, the
  sets matched and it passed. Both facts are now reported, and the duplicate message names the
  colliding paths.

## F003 [high] false results conflated with errors — AGREED, fixed

An unreadable unit doc fell through to "closed" and a foreign-owned registration to "absent". Both
readings are worse than useless: the documents most likely to be broken became the ones nothing
checked. Three outcomes now, with three messages. The foreign case says what it costs — the Stop
hook SKIPS records owned by another session, so a foreign-owned document looks registered and
enforces nothing, which is strictly worse than an absent record and needs a different fix. A doc
with no `## Gate status` checkboxes is its own error too, read with the hook's own section rule so
the two cannot disagree about which boxes count.

## F004 [high] unchecked transformations can produce a false pass — AGREED, fixed

An erased delta is indistinguishable from no delta. Every `find`, `sort`, `comm` and extraction
status is checked, and there is a count-in/count-out guard: if the parser found N rows and the id
extractor produced fewer than N ids, it dies rather than comparing a quietly shrunken set. That is
the specific shape where the check passes by producing nothing.

Exit 2 is also separated from exit 1 throughout, so "the check could not run" is never read as "the
check passed".

## F005 [medium] `sort -n` violates `comm`'s ordering requirement — AGREED, fixed

Measured rather than reasoned about, because the failure is silent. Declared {2,10,20} against
scaffolded {10}: `comm -23` over `sort -n` inputs returns `2 10 20`, calling a present id missing.
Zero-padded and sorted lexically under `LC_ALL=C` it returns `002 020`, which is the truth. `LC_ALL`
is pinned so `sort` and `comm` cannot disagree about collation under a different locale.

## F006 [medium] the alternate-depth guarantee is wider than the code — AGREED, claim narrowed

The code scans one other depth and only files named `task.md`. Rather than grow the scan, the file
now says that: a registered document at some third depth, or under another name, is outside what
this proves. A guarantee that overstates is the thing this whole unit exists to stop.

## F007 [high] found while fixing F001 — the caller registers the wrong level

Not raised by the round, recorded because it is the same defect one level up: P6's fence iterated a
literal task-depth glob regardless of declared granularity. `--depth` was added so the fence reads
the level from the same parse the check uses. Filed against unit 04 as F023, where the fence lives.

Consensus: disagreed
