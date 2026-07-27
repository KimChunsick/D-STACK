# 04-review-io

## Intent / Why
Two separate leaks in the review loop. The reviewer's full stdout is echoed into the main
context every round, so an 18-round task carries 18 full reviews it never needed. And the loop
has no real bound: the round-4 wind-down cuts on severity, but what actually stretches a loop
is the reviewer finding new ground in code that did not change. Move the round's input and
output to `.dstack/runs/`, read only the verdict and finding headings back, and bound the loop
with a scope rule plus a six-round escalation to the user.

## Executable surface added by this task's own review rounds

Recorded because an earlier draft of this section claimed the task added no API. It did, twice,
in response to its own findings: `dstack rm-run <label>...` (Round 5 — the capture cleanup was an
unfixable check-then-delete race in any calling shell) and the `REVIEW_BASE`/`REVIEW_HEAD`
contract in `assemble-review.sh` (Rounds 7-8 — the gate could not bind to the committed identity
it claims to review, and failed open on any diff error). `assemble-review.sh` was ADDED to this
task's declared `files` in GOAL.md once Round 8 made it clear the review gate's own enforcement
point was being edited every round; `claude/bin/dstack` stays declared by M1's T02 and is named
here because this task's findings are what grew it.

## Design consult
Skipped — no trigger. This edits an instruction document; it defines no new module boundary and
no persistence format.

## What was done (what / why)
Three changes to `codex-review`, all aimed at what a long review loop actually costs.

**The bundle and output moved to `.dstack/runs/`, and stopped being deleted on exit.** They were
`mktemp` files behind an `EXIT` trap, so a round that died took its own evidence with it — and a
round 6 is exactly when you want to see what round 5 was fed. `dstack run-dir` creates the
directory mode 700, because gitignored is not private: a bundle holds full code diffs, and
backups, sync folders, and snapshots all see those bytes. `dstack prune` bounds retention.

**`cat "$OUT"` is gone.** It pulled the reviewer's entire output into the main context every
round; an 18-round task carried 18 full reviews it never needed to re-read. The output is what
you write into the round file, not what you need in order to decide the next move. Replaced with
reading the verdict line and the finding headings, then opening only the findings being acted on.

**The loop is bounded, but never by downgrading a real defect.** The first version of this rule
did exactly that — it aged out any late finding on unchanged code unless it was high severity,
which means a concrete, reproducible medium found in round 4 would have been recorded and
shipped. Round 1 of this milestone's review caught it, and it was rewritten: **discovery time
never changes a finding's blocking status.** Lateness may only affect items with no demonstrated
failure path, which is a restatement of the existing severity wind-down. The bound is the
six-round budget, which stops and puts the choice to the user in Korean (accept the open items as
recorded risk / split the unit / continue) — a person deciding with the defect in front of them
is the only sanctioned way a concrete blocker becomes shippable.

Also told the invocation to carry repo policy, so a repository that records no tests by policy
says so and the reviewer judges direct-run evidence rather than filing "no tests" as a finding.

## Files changed (where / why)
- `claude/skills/codex-review/SKILL.md` — Step 1 bundle location, Step 2 background invocation
  and the no-`cat` reading recipe, Step 4 unchanged-code rule and round budget.
