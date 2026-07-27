# Codex adversarial review — Round 003

## Review scope
Re-review of Round 002's repairs. Bundle: this unit's `task.md`, rounds 001-002, and the
scoped diffs of `claude/CLAUDE.md` and `claude/skills/full-cycle/tests/skill-schema.test.sh`.

## GPT findings
[severity:medium][technical correctness] The parsed guard still permits `PARALLEL` under `delegate-when`, making it a delegation precondition while falsely reporting otherwise; `honest-scope` also remains unscoped global text.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh:116-163`
Evidence: The checker only rejects `PARALLEL` from `fan["requires"]`; it merely requires `delegate-when` to be nonempty, while `honest-scope` is checked only through `has`.
Verification: Applying the exact predicates to nonempty lists with `PARALLEL` in both `delegate-when` and `parallel-when` produced six PASS results, including “PARALLEL is not a delegation precondition.”
Suggested direction: Validate the task-shape entries under `worker-fanout.delegate-when`, reject `PARALLEL` from every delegation-gating list, and resolve `honest-scope` from the parsed `worker-fanout` node.

Omitted-detail: 0 low

GPT verdict: reject — The core regression guard still accepts a contract in which PARALLEL gates delegation, leaving the intended invariant unenforced.

## Bundle size (the ratchet, recorded)

R1 11,193 · R2 13,457 · **R3 19,117** bytes. Violated at R2 and R3, monotonically. Same cause as
T01's and T02's, filed as F-02: rounds join the bundle and the task record grows with the provenance
each round demands. Not fixed in the rounds that broke it.

## Round outcome

One medium, real, fixed. Blocking count 2 (R1) → 2 (R2) → 1 (R3).

Fourth consecutive round on the same guard, and the pattern is now the finding. Each version was
correct about the case its author had in mind and blind to the neighbouring one: ban a string, and
the live wording differs; scope by indentation, and comments read as content; parse properly but
check one list, and the identical line under the other delegation-gating list passes while the
check prints that PARALLEL is not a delegation precondition. Every one of those was a control that
exercised the case I already knew about.

Round 004 re-reviews the repairs.

Consensus: disagreed
