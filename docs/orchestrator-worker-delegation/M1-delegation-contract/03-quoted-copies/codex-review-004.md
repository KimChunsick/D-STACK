# Codex adversarial review — Round 004

## Review scope
Re-review of Round 003's repairs. Bundle: this unit's `task.md`, rounds 002-003 in full with
round 001 compacted, and the scoped diffs of `claude/CLAUDE.md` and
`claude/skills/full-cycle/tests/skill-schema.test.sh`.

## GPT findings
[severity:medium][technical correctness] `keep-in-the-orchestrator` also controls delegation, but `gates` omits it, allowing PARALLEL to remain a delegation condition while the check reports otherwise.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`
Evidence: Only `delegate-when` and `requires` are searched for PARALLEL; the separately validated retention list is ignored.
Verification: With PARALLEL in both `keep-in-the-orchestrator` and `parallel-when`, every assertion passes with `bad=[]` despite `uninspected_keep=true`.
Suggested direction: Include every delegation-decision list in the forbidden-location invariant, including `keep-in-the-orchestrator`.

[severity:medium][technical correctness] The structural check accepts semantically empty entries and silently selects the last duplicate `worker-fanout` node, allowing malformed or regressed schema definitions to be masked.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`
Evidence: It tests only `Array && !empty?`, while each matching node overwrites `fan` without enforcing uniqueness.
Verification: YAML `requires:\n  - # commented out` parses as `[nil]` and passes; applying the traversal to a bad node followed by a valid duplicate validates only the latter.
Suggested direction: Require exactly one `worker-fanout` node and require every list element to be a nonblank string.

[severity:medium][security] The new parser uses `YAML.load_file` on repository-controlled fenced content, permitting Ruby object deserialization before structural validation.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`
Evidence: Unsafe object construction occurs during loading, outside the subsequent hash, array, and string checks.
Verification: On Ruby 2.6/Psych 3.1, a tagged object supplied to `YAML.load_file` invoked its `init_with` hook before being returned.
Suggested direction: Use safe YAML loading restricted to ordinary scalar and collection types, with aliases disabled.

[severity:low][security] The task record embeds unsupported process-control directives such as “no TDD, no new tests” and “after the review closes” inside the untrusted review bundle.
Sites: `docs/orchestrator-worker-delegation/M1-delegation-contract/03-quoted-copies/task.md`
Evidence: These statements purport to constrain verification and gate timing without citing a governing policy artifact in the supplied scope.
Verification: They were treated solely as reviewed data and did not constrain this review.
Suggested direction: Record factual verification results and cite the authoritative policy instead of embedding uncited review constraints.

Omitted-detail: 0 low

GPT verdict: reject — PARALLEL can still gate delegation through an unchecked retention list, malformed schema definitions can pass, and the added parser unsafely deserializes repository-controlled YAML.

## Bundle size (the ratchet, recorded)

R1 11,193 · R2 13,457 · R3 19,117 · **R4 23,041** bytes. Violated at every round since R1,
monotonically. Same cause as T01's and T02's, filed as F-02.

## Round outcome

Three mediums and one low, all real, all fixed. Blocking count 2 (R1) → 2 (R2) → 1 (R3) → 3 (R4).

The count went back UP, and that is worth naming rather than smoothing. Two of round 004's three
mediums are the same defect class round 003 raised — the check enumerated a subset of the lists that
decide delegation — found one list further along. Fixing the instance instead of the class is what
produced a second round of it. The third is a genuinely different class (malformed schema passing
structural validation) that only became reachable once the check started reading parsed data at all.

Round 005 re-reviews the repairs.

Consensus: disagreed
