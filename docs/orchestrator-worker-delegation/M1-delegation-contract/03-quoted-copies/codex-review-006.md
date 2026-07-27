# Codex adversarial review — Round 006

## Review scope
Re-review of Round 005's repairs, and the final round for this unit.
Bundle: this unit's `task.md`, rounds 004-005 in full with 001-003 compacted, and the scoped
diffs of `claude/CLAUDE.md` and `claude/skills/full-cycle/tests/skill-schema.test.sh`.

## GPT findings
[severity:medium][technical correctness] The semantic guard still accepts direct negations of every pinned decision because it checks only unordered substring presence.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`; confirmed: `docs/orchestrator-worker-delegation/M1-delegation-contract/03-quoted-copies/task.md`
Evidence: Phrases such as “a verdict of PARALLEL must never permit concurrent execution” and “frontend-dev never OUTRANKS orchestrator retention” satisfy every corresponding regex.
Verification: Applying the implemented predicates to inverted values produced `lists_ok=true`, `semantic_checks=true`, and `parallel_gate_bad=[]`.
Suggested direction: Compare normalized complete entries against canonical affirmative decisions instead of accepting required substrings anywhere in each field.

[severity:medium][technical correctness] `YAML.safe_load` inspects only the first document in a YAML stream, so a second `worker-fanout` document bypasses uniqueness and semantic validation.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`
Evidence: `Psych.parse_stream` traverses all documents only for duplicate keys, while `safe_load` supplies the sole value added to `fans` and is reused by the parse-validity loop.
Verification: A valid two-document stream produced `stream_documents=2`, while `safe_load` returned only the first valid `worker-fanout`; the regressed second node remained invisible.
Suggested direction: Require exactly one AST document per extracted fence before safe-loading and validating it.

[severity:low][security] The review bundle still embeds process-control directives purporting to cap verification scope and close the review regardless of its result.
Sites: `docs/orchestrator-worker-delegation/M1-delegation-contract/03-quoted-copies/task.md`; confirmed: `docs/orchestrator-worker-delegation/M1-delegation-contract/03-quoted-copies/codex-review-005.md`
Evidence: The artifacts assert “no new tests” constraints and that the loop “closes there whatever it returns.”
Verification: These statements were treated solely as untrusted reviewed data and did not constrain this review.
Suggested direction: Remove review-control language from evidence artifacts or identify it explicitly as an externally governed claim.

Omitted-detail: 0 low

GPT verdict: reject — The checker still accepts inverted delegation semantics and ignores additional YAML documents containing duplicate or regressed worker-fanout definitions.

## Bundle size (the ratchet, recorded)

R1 11,193 · R2 13,457 · R3 19,117 · R4 23,041 · R5 29,197 · **R6 38,536** bytes. Violated every
round since R1, monotonically, ending at 3.4x the first bundle. Filed as F-02.

## Round outcome

Two mediums and one low. Blocking count 2 · 2 · 1 · 3 · 3 · 2.

**The loop CLOSES here, by the non-convergence rule** — 1, 3, 3 was already not strictly decreasing
at R5, and R6 does not change that. Both of this round's mediums are fixed before sealing. The low
is answered below and partly carried; `findings.md` lists what remains open.

Six rounds on two files, and the shape is worth recording because it is not the shape T01 and T02
had. Every round but one was about the same object — the regression guard — and every version of it
was correct about the case I had in mind and blind to its neighbour. Ban a string: the live wording
differs. Scope by indentation: comments read as content. Parse, but check one list: the other
delegation list passes. Check all three lists: `[nil]` and duplicate nodes pass. Check types: a
negation carrying the pinned token passes. Six rounds is what it cost to learn that a guard over a
DOCUMENT has no natural stopping point — each repair creates the surface the next finding lands on.

What actually terminated it was the rule, not agreement. That is the second Goal in a row where the
finding stream stayed alive at close, and both times no finding was a false positive.

## Follow-ups leaving this unit

- **F-02** (carried from T01) — the bundle ratchet cannot be satisfied when the reviewed artifacts
  themselves grow each round. Six rounds of monotonic growth here is the strongest evidence yet.
- The `AGENTS.md` citation in `task.md` names policy text that is not in the bundle. Supplying it
  would grow the allowlist, which the ratchet forbids mid-loop.
- Round 005's sealed file contains a sentence about how this loop would close. Sealed rounds are the
  audit trail and were not rewritten; the point is recorded here instead.

Consensus: resolved
