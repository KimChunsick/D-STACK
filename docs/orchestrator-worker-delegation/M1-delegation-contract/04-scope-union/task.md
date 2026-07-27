# 04-scope-union

## Intent / Why

The `scope` verdict is what makes worker delegation safe: it is the only mechanism that catches a
worker committing outside its declared file ownership. T01's design consult found that it does not
actually do that. The enumeration is a NET two-tree comparison, so a path added in one commit and
removed in a later one is absent from the result while its content stays in the branch's history
and in every object the branch carries. T01 is widening how often delegation happens; shipping
that on top of a containment check with a hole in it is the wrong order.

## Deployment context

`claude/skills/full-cycle/check-parallel.sh` is executed, not read by a model: plain bash, no
external dependencies beyond git, run by the orchestrator before a worker branch is reviewed and
again before it is merged. One maintainer, local disk. It is a mistake tripwire, not a boundary
against a falsifying orchestrator — the base and branch values it is given come from the
orchestrator's own records, which is recorded in the pipeline as an accepted residual.

What this change does NOT touch, stated as a claim about the diff rather than as a boundary on
anyone's reading: the `plan` verdict, the declaration grammar, and the `SKILL.md` sentence
describing `scope`. Round 001 was right to flag the earlier phrasing — a task document telling a
reviewer what is "out of scope" is review-steering, and it is untrusted data besides. Verify the
claim against the diff; do not take it as a limit.

## Design consult

Skipped — no trigger. This changes which git command enumerates an existing check's input set. No
new module boundary, no API contract, no persistence format, no sanitization path.

## What was done (what / why)

**The scope enumeration became the union of every commit's changed paths.** It was
`git diff --name-only -z --no-renames "$base" HEAD` — a two-tree comparison, which by definition
only reports paths whose CONTENT DIFFERS between the two endpoints. A worker that commits an
out-of-scope file and removes it in a later commit leaves both trees identical for that path, so
the path is simply absent from the check, while the file and everything in it stay in the branch
history that later gets merged. Containment has to be a statement about what the branch DID, not
about where it happened to end up.

It now walks `git rev-list "$base..HEAD"` and collects
`git diff-tree -r -m --no-commit-id --name-only -z --no-renames` per commit. `-m` shows a merge
commit against each parent rather than diff-tree's empty default, which over-reports for merges —
the safe direction for a containment check. Every step keeps the file's existing fail-closed
discipline: a `rev-list` or `diff-tree` failure is INVALID, never a silent PASS.

The union is a strict superset of the old net diff (every path in the net diff was necessarily
touched by some commit), so nothing that used to be caught stops being caught.

## Files changed (where / why)

- `claude/skills/full-cycle/check-parallel.sh` — the `scope` mode's changed-path enumeration.
  Nothing else in the file is touched: the `plan` verdict, the declaration grammar, the clean-tree
  requirement and the containment predicate are all unchanged, and the paths this produces are fed
  to the same `check_contained` as before.

## E2E verification

The whole fan-out sequence this check guards, driven end to end on 2026-07-27 in a throwaway
repository with two tasks owning disjoint trees (`T01` owns `src/`, `T02` owns `lib/`). A real
`git worktree`, real commits, and the checker in each of the two places the pipeline actually
calls it.

```
### 1. plan verdict for a disjoint, ready pair
PARALLEL: T01 T02

### 2. orchestrator creates the worktree from the RECORDED base, then confirms HEAD
  recorded base : 4620ea251a2295fe0da1afaf1181fefd288c60e1
  worktree HEAD : 4620ea251a2295fe0da1afaf1181fefd288c60e1
  MATCH — safe to brief a worker

### 3. worker does declared work only, commits, leaves the tree clean -> scope before review
PASS

### 4. the regression this task fixed: out-of-scope file added, then removed
  net endpoint diff sees : src/a.txt
VIOLATION: lib/stolen.txt is not in T01 declaration

### 5. an unclean tree cannot enter review
VIOLATION: worktree not clean (uncommitted/untracked): src/uncommitted.txt
```

Step 4 is the finding and the fix in two lines. The endpoint diff reports only `src/a.txt`; the
worker's excursion into `T02`'s declared tree is invisible to it and plainly visible to the new
enumeration. Steps 1, 3 and 5 are the controls: the parts of the checker this task did not touch
still verdict correctly, so the change tightened one enumeration rather than altering the gate.

Step 2 also exercises T01's base-identity requirement by hand — the worktree is created from the
recorded base and its HEAD is compared before any work. Recorded here because T04's fixture is the
only place in this Goal where a real worktree exists.

**Merge, honestly.** There is nothing to merge. This Goal's own tasks are orchestrator-owned by
T01's own rule, so the change was made in the main checkout rather than on a task branch. The
pipeline's `merge precedes P10` clause is about worker branches; it is vacuous here and is
recorded as such rather than ticked as if a merge happened.

## Direct verification (repo policy: no TDD, no new tests)

Run on 2026-07-27 in a throwaway repository built for the purpose. Output recorded, not described.

Fixture: `T01` declares `files: [src/]`. On the task branch, three commits — one adding
`outside.txt` (undeclared), one removing it again, one doing the declared work in `src/a.txt`.

```
--- net two-tree diff (what the check USED to enumerate) ---
    src/a.txt
--- per-commit union (what it enumerates now) ---
    outside.txt
    src/a.txt

=== checker verdict ===
VIOLATION: outside.txt is not in T01 declaration
```

The two enumerations printed side by side are the whole finding: the old one cannot see the file
at all. Negative control on the same fixture — a second branch off the same base touching only
`src/a.txt` → `PASS`, so the change tightened the check rather than breaking it.

`bash claude/skills/full-cycle/tests/check-parallel.test.sh` → `== all checks passed`.
`bash -n` on the script → clean.

**Round 001 follow-up fixes, verified the same way.** A committed file literally named
`evil<LF>PASS`, undeclared:

```
=== the committed undeclared filename ===
"evil\nPASS"
=== verdict output ===
VIOLATION: evil\nPASS is not in T01 declaration
--- stdout line count (must be 1) --- 1
--- lines reading exactly PASS (must be 0) --- 0
```

Before the escaper, that filename put a second line reading `PASS` on stdout. The exit status was
always 1, but the verdict TEXT is what a caller reads. Control: `src/foo..bar`, which the
declaration grammar allows because no component IS `..`, now returns `PASS` instead of
`VIOLATION: suspicious actual path`. Not tested, and stated rather than claimed: a real `..`
component in a committed path is unreachable through git, so that branch is defense in depth with
no fixture behind it.

## Gate status

- [x] Verification: behavior confirmed by direct run (repo policy: no TDD, no new tests)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
